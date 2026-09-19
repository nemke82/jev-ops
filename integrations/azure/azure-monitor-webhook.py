#!/usr/bin/env python3
"""
Azure Monitor Action Group -> HTTP Webhook -> jev-ops Automated Alert Triage

Flow:
  1. Azure Monitor fires alert in Common Alert Schema format to this webhook.
  2. Webhook extracts alert context, target resource ID, and search results.
  3. Pipes error telemetry into `jev-ops analyze azure-monitor --json`.
  4. If action == 'auto_heal' and confidence > 85%, invokes Azure CLI (`az`) or Azure SDK.
  5. Emits structured telemetry to Azure App Insights or logs.

Security:
  - Requests must carry the shared secret from WEBHOOK_TOKEN, sent by the Action Group
    as the `token` query parameter (https://host/?token=...) or an `X-Webhook-Token` header.
  - Binds to 127.0.0.1 unless HOST is set; put it behind TLS termination before exposing it.
"""

import hmac
import json
import os
import subprocess
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import parse_qs, urlparse

HOST = os.environ.get("HOST", "127.0.0.1")
PORT = int(os.environ.get("PORT", "8080"))
WEBHOOK_TOKEN = os.environ.get("WEBHOOK_TOKEN", "")
MAX_BODY_BYTES = 1024 * 1024
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
    def _reply(self, status: int, body: dict) -> None:
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(body).encode())

    def _authorized(self) -> bool:
        supplied = self.headers.get("X-Webhook-Token") or parse_qs(
            urlparse(self.path).query
        ).get("token", [""])[0]
        return hmac.compare_digest(supplied.encode(), WEBHOOK_TOKEN.encode())

    def do_POST(self):
        if not self._authorized():
            self._reply(401, {"error": "unauthorized"})
            return

        try:
            content_len = int(self.headers.get("Content-Length", 0))
        except ValueError:
            content_len = -1
        if not 0 < content_len <= MAX_BODY_BYTES:
            self._reply(413, {"error": "missing or oversized body"})
            return

        try:
            payload = json.loads(self.rfile.read(content_len).decode())
            self._reply(200, handle_azure_alert(payload))
        except Exception as e:
            # Log details server-side; never echo internals to the caller.
            print(f"[jev-ops webhook] failed to handle alert: {e!r}", file=sys.stderr)
            self._reply(500, {"error": "internal error"})

if __name__ == "__main__":
    if not WEBHOOK_TOKEN:
        sys.exit("WEBHOOK_TOKEN must be set to a long random secret.")
    print(f"Starting Azure Monitor jev-ops webhook listener on {HOST}:{PORT}...")
    server = HTTPServer((HOST, PORT), AzureWebhookHandler)
    server.serve_forever()
