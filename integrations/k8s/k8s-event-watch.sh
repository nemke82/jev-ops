#!/usr/bin/env bash
# ==============================================================================
# k8s-event-watch.sh — Real-time Kubernetes Event Stream Triage with jev-ops
# ==============================================================================
set -euo pipefail

NAMESPACE="${1:-default}"

echo "[jev-ops] Starting Kubernetes event triage stream for namespace '${NAMESPACE}'..." >&2

kubectl get events -n "${NAMESPACE}" --watch-only --output-watch-events | while read -r line; do
    # Filter for Warning / BackOff / OOM events
    if echo "$line" | grep -qE "Warning|BackOff|OOMKilled|Failed"; then
        echo "[jev-ops] Triggering analysis on event: $line" >&2
        
        # Analyze event against kubernetes pack; under `set -e` a failed analysis
        # (e.g. HTTP 429) would otherwise terminate the whole watch loop.
        if ! DECISION=$(printf '%s\n' "$line" | jev-ops analyze kubernetes --json); then
            echo "[jev-ops] Analysis failed for this event; continuing to watch." >&2
            continue
        fi
        
        ROOT_CAUSE=$(echo "$DECISION" | jq -r '.decisions.root_cause.value')
        SEVERITY=$(echo "$DECISION" | jq -r '.decisions.severity.value')
        CONFIDENCE=$(echo "$DECISION" | jq -r '.decisions.root_cause.confidence')
        
        echo "[jev-ops] K8s Triage: Cause=${ROOT_CAUSE}, Severity=${SEVERITY}/5, Conf=${CONFIDENCE}" >&2
        
        # Branch automation based on typed decisions
        if [[ "$ROOT_CAUSE" == "crashloop" && "$SEVERITY" -ge 4 ]]; then
            echo "[jev-ops] Flagging pod in CrashLoopBackOff for automated investigation..." >&2
        elif [[ "$ROOT_CAUSE" == "resources" ]]; then
            echo "[jev-ops] Pod killed by OOM. Notifying cluster administrator to bump memory limits." >&2
        fi
    fi
done
