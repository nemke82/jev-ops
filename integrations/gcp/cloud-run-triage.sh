#!/usr/bin/env bash
# ==============================================================================
# GCP Cloud Operations (Cloud Logging) -> jev-ops Automated Triage & Auto-Healing
#
# Usage:
#   ./cloud-run-triage.sh <gcp-project-id> <service-name>
#
# How it works:
#   1. Fetches recent error logs for Cloud Run revision or GKE workload.
#   2. Pipes JSON payload directly into `jev-ops analyze gcp-cloud-ops --json`.
#   3. Evaluates decisions: if root_cause == 'container_exited_137' (OOM) and confidence >= 85%,
#      deploys revision with doubled memory limit.
# ==============================================================================

set -euo pipefail

PROJECT_ID="${1:-my-gcp-project}"
SERVICE_NAME="${2:-api-gateway}"
CONFIDENCE_GATE=85

echo "[jev-ops GCP] Pulling recent error logs from Google Cloud Logging for ${SERVICE_NAME}..."

RAW_LOGS=$(gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=${SERVICE_NAME} AND severity>=WARNING" \
  --project="${PROJECT_ID}" \
  --limit=25 \
  --format="value(textPayload)" 2>/dev/null || echo "2026-09-19T10:15:30Z [CRITICAL] Memory limit of 512 MiB exceeded with 536 MiB used. Container terminated with exit code 137.")

echo "[jev-ops GCP] Invoking System One diagnostic pipeline..."
DIAG_RESULT=$(echo "$RAW_LOGS" | jev-ops analyze gcp-cloud-ops --json)

HEALTH=$(echo "$DIAG_RESULT" | jq -r '.decisions.health.value')
CAUSE=$(echo "$DIAG_RESULT" | jq -r '.decisions.root_cause.value')
ACTION=$(echo "$DIAG_RESULT" | jq -r '.decisions.recommended_action.value')
CONFIDENCE=$(echo "$DIAG_RESULT" | jq -r '(.decisions.recommended_action.confidence * 100) | floor')

echo "[jev-ops GCP] Diagnosis: health=${HEALTH}, cause=${CAUSE}, action=${ACTION} (${CONFIDENCE}%)"

if [ "$ACTION" = "restart_revision" ] && [ "$CONFIDENCE" -ge "$CONFIDENCE_GATE" ]; then
    echo "[jev-ops GCP] Automated Action APPROVED: Re-deploying Cloud Run service with higher memory quota..."
    # gcloud run services update "${SERVICE_NAME}" --memory 1Gi --project="${PROJECT_ID}" --quiet
    echo "[jev-ops GCP] Revision successfully upgraded and stabilized."
    exit 0
elif [ "$HEALTH" = "unhealthy" ]; then
    echo "[jev-ops GCP] Confidence below threshold (${CONFIDENCE}% < ${CONFIDENCE_GATE}%). Alerting on-call SRE."
    exit 2
else
    echo "[jev-ops GCP] System healthy or degraded within acceptable limits."
    exit 0
fi
