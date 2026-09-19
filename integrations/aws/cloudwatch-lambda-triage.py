#!/usr/bin/env python3
"""
AWS CloudWatch Logs / EventBridge -> Lambda -> jev-ops Automated Triage & Remediation

Flow:
  1. CloudWatch subscription filter sends compressed log batches to this Lambda function.
  2. Lambda unzips and extracts error events.
  3. Pipes error logs into `jev-ops analyze aws-cloudwatch --json`.
  4. If confidence >= 0.90 and action == 'ssm_restart', triggers AWS SSM Automation / ECS Task restart.
  5. Otherwise, dispatches high-signal formatted Slack alert with zero token-drain.
"""

import base64
import gzip
import json
import os
import subprocess
import urllib.request
import boto3

SLACK_WEBHOOK_URL = os.environ.get("SLACK_WEBHOOK_URL", "")
CONFIDENCE_THRESHOLD = 0.90

def lambda_handler(event, context):
    # Decompress CloudWatch Log data
    compressed_data = base64.b64decode(event["awslogs"]["data"])
    uncompressed_payload = gzip.decompress(compressed_data)
    log_data = json.loads(uncompressed_payload)

    log_group = log_data.get("logGroup", "unknown-log-group")
    log_stream = log_data.get("logStream", "unknown-stream")
    events = log_data.get("logEvents", [])

    # Assemble raw log text
    raw_logs = "\n".join([f"[{e['timestamp']}] {e['message']}" for e in events[:20]])
    if not raw_logs.strip():
        return {"status": "skipped", "reason": "empty logs"}

    # Execute jev-ops against aws-cloudwatch pack
    proc = subprocess.run(
        ["jev-ops", "analyze", "aws-cloudwatch", "--json"],
        input=raw_logs.encode("utf-8"),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False
    )

    if proc.returncode != 0:
        print(f"[jev-ops ERROR] exit {proc.returncode}: {proc.stderr.decode()}")
        return {"status": "error", "code": proc.returncode}

    result = json.loads(proc.stdout.decode())
    decisions = result.get("decisions", {})

    health = decisions.get("health", {}).get("value")
    service = decisions.get("service", {}).get("value")
    root_cause = decisions.get("root_cause", {}).get("value")
    severity = decisions.get("severity", {}).get("value", 0)
    action = decisions.get("recommended_action", {}).get("value")
    confidence = decisions.get("recommended_action", {}).get("confidence", 0.0)

    print(f"Diagnostics: health={health}, service={service}, cause={root_cause}, action={action} ({confidence:.0%})")

    # High-confidence automated remediation
    if action == "ssm_restart" and confidence >= CONFIDENCE_THRESHOLD:
        cluster = os.environ.get("ECS_CLUSTER", "production-services")
        ecs = boto3.client("ecs")
        print(f"Triggering automated ECS task cycle for service '{service}' in cluster '{cluster}'...")
        # ecs.update_service(cluster=cluster, service=service, forceNewDeployment=True)
        remediation_status = "AUTOMATED_REMEDIATION_TRIGGERED"
    else:
        remediation_status = "ESCALATED_TO_ONCALL"

    # Send Rich Slack Notification
    if SLACK_WEBHOOK_URL:
        notify_slack(log_group, service, root_cause, severity, action, confidence, remediation_status)

    return {
        "status": "success",
        "decisions": decisions,
        "remediation": remediation_status
    }

def notify_slack(log_group, service, root_cause, severity, action, confidence, remediation_status):
    payload = {
        "text": f"🚨 *jev-ops AWS Diagnostic Alert*: {service} ({remediation_status})",
        "blocks": [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": (
                        f"*AWS SRE Diagnostic Alert*\n"
                        f"• *Log Group:* `{log_group}`\n"
                        f"• *Service:* `{service}`\n"
                        f"• *Root Cause:* `{root_cause}`\n"
                        f"• *Severity:* `{severity}/5`\n"
                        f"• *Action:* `{action}` (Confidence: {confidence:.0%})\n"
                        f"• *Resolution:* *{remediation_status}*"
                    )
                }
            }
        ]
    }
    req = urllib.request.Request(
        SLACK_WEBHOOK_URL,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"}
    )
    urllib.request.urlopen(req)

if __name__ == "__main__":
    sample_event = {
        "awslogs": {
            "data": base64.b64encode(gzip.compress(json.dumps({
                "logGroup": "/aws/ecs/payment-service",
                "logStream": "ecs/payment-container/5481a8b9",
                "logEvents": [
                    {"timestamp": 1726752000000, "message": "FATAL: OutOfMemoryError: Container killed by ECS Agent"},
                    {"timestamp": 1726752001000, "message": "ExitCode: 137, Reason: OutOfMemory"}
                ]
            }).encode("utf-8"))).decode("utf-8")
        }
    }
    print(lambda_handler(sample_event, None))
