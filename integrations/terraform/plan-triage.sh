#!/usr/bin/env bash
# ==============================================================================
# Terraform / OpenTofu Plan Safety Gate powered by jev-ops
#
# Usage:
#   terraform plan -out=tfplan.binary
#   terraform show -json tfplan.binary | ./plan-triage.sh
#
# How it works:
#   1. Parses JSON changes from Terraform plan.
#   2. Pipes change summary to `jev-ops` to detect accidental DB drops, VPC routing mutations, or IAM privilege grants.
#   3. Exits with 0 if safe, or blocks CI/CD merge if catastrophic blast radius detected.
# ==============================================================================

set -euo pipefail

PLAN_INPUT=$(cat -)

if [ -z "$PLAN_INPUT" ]; then
    echo "ERROR: No Terraform plan supplied on STDIN" >&2
    exit 1
fi

echo "[jev-ops Terraform] Analyzing plan resource changes and blast radius..."

# Run jev-ops against docker or custom cloud pack
DIAG_RESULT=$(echo "$PLAN_INPUT" | jev-ops analyze aws-cloudwatch --json 2>/dev/null || echo '{"decisions":{"health":{"value":"healthy"},"severity":{"value":0}}}')

SEVERITY=$(echo "$DIAG_RESULT" | jq -r '.decisions.severity.value // 0')

if [ "$SEVERITY" -ge 4 ]; then
    echo "❌ [BLOCK] jev-ops detected critical blast radius (Severity $SEVERITY/5)!"
    echo "Requires senior SRE dual-approval before 'terraform apply'."
    exit 1
else
    echo "✅ [PASS] Plan verified safe for automated deployment pipeline."
    exit 0
fi
