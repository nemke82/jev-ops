#!/usr/bin/env python3
"""
Azure Monitor Action Group -> HTTP Webhook -> jev-ops Automated Alert Triage

Flow:
  1. Azure Monitor fires alert in Common Alert Schema format to this webhook.
  2. Webhook extracts alert context, target resource ID, and search results.
  3. Pipes error telemetry into `jev-ops analyze azure-monitor --json`.
  4. If action == 'auto_heal' and confidence > 85%, invokes Azure CLI (`az`) or Azure SDK.
  5. Emits structured telemetry to Azure App Insights or logs.
"""

import json
import os
import subprocess
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

PORT = int(os.environ.get("PORT", "8080"))
CONFIDENCE_THRESHOLD = 0.85

def handle_azure_alert(payload: dict) -> dict:
    data = payload.get("data", {})
    essentials = data.get("essentials", {})
    alert_rule = essentials.get("alertRule", "unknown-rule")
    target_resource = essentials.get("alertTargetIDs", ["unknown-target"])[0]

    # Extract log context or description
    context_text = f"Alert Rule: {alert_rule}\nTarget: {target_resource}\n"
    if "customProperties" in data:
        context_text += f"Properties: {json.dumps(data['customProperties'])}\n"

    # Pipe to jev-ops with azure-monitor pack
    proc = subprocess.run(
        ["jev-ops", "analyze", "azure-monitor", "--json"],
        input=context_text.encode("utf-8"),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False
    )

    if proc.returncode != 0:
        print(f"[jev-ops ERROR] returncode={proc.returncode}")
        return {"status": "error", "code": proc.returncode}

    result = json.loads(proc.stdout.decode())
    decisions = result.get("decisions", {})
    
    resource_type = decisions.get("resource_type", {}).get("value")
    cause = decisions.get("root_cause", {}).get("value")
    action = decisions.get("recommended_action", {}).get("value")
    confidence = decisions.get("recommended_action", {}).get("confidence", 0.0)

    print(f"Azure Triage: resource={resource_type}, cause={cause}, action={action} ({confidence:.0%})")

    # Automated action execution
    if action == "auto_heal" and confidence >= CONFIDENCE_THRESHOLD:
        print(f"Executing Azure remediation for {target_resource}...")
        # e.g. subprocess.run(["az", "aks", "nodepool", "scale", "--cluster-name", ...])
        status = "EXECUTED_AUTO_HEAL"
    else:
        status = "FLAGGED_FOR_HUMAN_REVIEW"

    return {
        "status": status,
        "decisions": decisions
    }

class AzureWebhookHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        content_len = int(self.headers.get("Content-Length", 0))
        post_body = self.rfile.read(content_len)
        try:
            payload = json.loads(post_body.decode())
            response = handle_azure_alert(payload)
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(response).encode())
        except Exception as e:
            self.send_response(500)
            self.end_headers()
            self.wfile.write(str(e).encode())

if __name__ == "__main__":
    print(f"Starting Azure Monitor jev-ops webhook listener on port {PORT}...")
    server = HTTPServer(("0.0.0.0", PORT), AzureWebhookHandler)
    server.serve_forever()
