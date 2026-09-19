#!/usr/bin/env bash
# ==============================================================================
# Terraform / OpenTofu Plan Safety Gate powered by jev-ops
#
# Usage:
#   terraform plan -out=tfplan.binary
#   terraform show -json tfplan.binary | ./plan-triage.sh
#
# How it works:
#   1. Summarizes the plan JSON to one "<actions> <address>" line per changed resource,
#      which keeps large plans well under the pack's input limit.
#   2. Pipes the summary to `jev-ops analyze terraform-plan` to judge blast radius.
#   3. Exits 0 if safe. Exits 1 (blocking the pipeline) on a risky plan OR on any
#      failure to evaluate it: this gate fails closed.
# ==============================================================================

set -euo pipefail

block() {
    echo "❌ [BLOCK] $1"
    echo "Requires senior SRE dual-approval before 'terraform apply'."
    exit 1
}

PLAN_INPUT=$(cat -)

if [ -z "$PLAN_INPUT" ]; then
    echo "ERROR: No Terraform plan supplied on STDIN" >&2
    exit 1
fi

if ! SUMMARY=$(jq -r '
        .resource_changes // []
        | map(select(.change.actions != ["no-op"] and .change.actions != ["read"]))
        | .[]
        | "\(.change.actions | join("+")) \(.address)"' <<<"$PLAN_INPUT"); then
    block "Could not parse plan JSON (expected 'terraform show -json' output)."
fi

if [ -z "$SUMMARY" ]; then
    echo "✅ [PASS] Plan contains no resource changes."
    exit 0
fi

echo "[jev-ops Terraform] Analyzing $(wc -l <<<"$SUMMARY") resource changes for blast radius..."

if ! DIAG_RESULT=$(jev-ops analyze terraform-plan --json <<<"$SUMMARY"); then
    block "jev-ops could not evaluate the plan; refusing to pass it unreviewed."
fi

BLAST_RADIUS=$(jq -r '.decisions.blast_radius.value' <<<"$DIAG_RESULT")
DESTROYS_STATE=$(jq -r '.decisions.destroys_stateful_resource.value' <<<"$DIAG_RESULT")
CHANGE_RISK=$(jq -r '.decisions.change_risk.value' <<<"$DIAG_RESULT")

if [ "$DESTROYS_STATE" = "true" ]; then
    block "jev-ops detected destruction of a stateful resource (risk: ${CHANGE_RISK})."
elif [ "$BLAST_RADIUS" -ge 4 ]; then
    block "jev-ops detected critical blast radius (${BLAST_RADIUS}/5, risk: ${CHANGE_RISK})."
fi

echo "✅ [PASS] Plan within automated deployment limits (blast radius ${BLAST_RADIUS}/5, risk: ${CHANGE_RISK})."
exit 0
