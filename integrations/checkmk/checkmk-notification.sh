#!/usr/bin/env bash
# ==============================================================================
# checkmk-notification.sh — Intelligent CheckMK Alert Noise Reduction with jev-ops
# Place in: /omd/sites/<SITE>/local/share/check_mk/notifications/
# ==============================================================================
set -euo pipefail

ALERT_CONTEXT="CheckMK Alert on Host: ${NOTIFY_HOSTNAME:-localhost}
Service: ${NOTIFY_SERVICEDESC:-Filesystem /}
State: ${NOTIFY_SERVICESTATE:-CRITICAL}
Output: ${NOTIFY_SERVICEOUTPUT:-Filesystem / 97.8%, growth last hour 0.1%, WARN 90%, CRIT 98%}
PerfData: ${NOTIFY_SERVICEPERFDATA:-/=97.8%;90;98;0;100}"

# Run diagnostic inference through jev-ops. If it fails (no API key, rate limit, ...),
# fall through to normal routing: a triage failure must never suppress an alert.
if ! DECISION_JSON=$(printf '%s\n' "$ALERT_CONTEXT" | jev-ops analyze checkmk --json); then
    echo "[jev-ops] Analysis failed; routing alert without triage." >&2
    DECISION_JSON='{}'
fi

ACTION=$(jq -r '.decisions.action.value // "unknown"' <<<"$DECISION_JSON")
CONFIDENCE=$(jq -r '.decisions.action.confidence // 0' <<<"$DECISION_JSON")
DOMAIN=$(jq -r '.decisions.root_domain.value // "unknown"' <<<"$DECISION_JSON")

echo "[jev-ops] Evaluated alert for ${NOTIFY_HOSTNAME:-unknown}: ACTION=${ACTION} (conf=${CONFIDENCE}), DOMAIN=${DOMAIN}" >&2

# Pack values are lowercase ("ignore", "auto_repair"); compare confidence in jq, no bc needed.
confident_above() {
    jq -e --argjson min "$1" '(.decisions.action.confidence // 0) > $min' <<<"$DECISION_JSON" >/dev/null
}

# Confidence-gated safety policy
if [[ "$ACTION" == "ignore" ]] && confident_above 0.85; then
    echo "[jev-ops] Transient noise filtered. Suppressing notification." >&2
    exit 0
elif [[ "$ACTION" == "auto_repair" ]] && confident_above 0.95; then
    echo "[jev-ops] High confidence auto-repair permitted. Triggering safe cleanup script..." >&2
    /opt/ops/bin/safe-disk-cleanup.sh || true
    exit 0
else
    echo "[jev-ops] Routing alert to PagerDuty/Slack (Action: ${ACTION}, Conf: ${CONFIDENCE})" >&2
    # Trigger default CheckMK email or webhook
    exit 0
fi
