// Preset diagnostic scenarios for the interactive live simulator
const SCENARIOS = {
  ext4: {
    pack: "linux",
    input: `[  512.102345] EXT4-fs error (device nvme0n1p2): ext4_lookup:1845: inode #2104928: comm nginx: deleted inode referenced: 2104929
[  512.102390] Aborting journal on device nvme0n1p2-8.
[  512.102420] EXT4-fs (nvme0n1p2): Remounting filesystem read-only
[  512.102500] EXT4-fs error (device nvme0n1p2) in ext4_reserve_inode_write:6120: Journal has aborted
[  512.102550] Buffer I/O error on dev nvme0n1p2, logical block 0, lost sync page write`,
    decisions: [
      { name: "Health", value: "unhealthy", class: "danger", confidence: 96, pct: 96 },
      { name: "Category", value: "filesystem", class: "danger", confidence: 98, pct: 98 },
      { name: "Severity", value: "5 / 5", class: "danger", confidence: 91, pct: 91 },
      { name: "Attention", value: "yes", class: "danger", confidence: 99, pct: 99 }
    ]
  },
  oom: {
    pack: "linux",
    input: `[  120.456123] Out of memory: Killed process 4129 (mysqld) total-vm:4194304kB, anon-rss:3145728kB, file-rss:0kB
[  120.456201] oom_reaper: reaped process 4129 (mysqld), now anon-rss:0kB
[  120.456312] memory: usage 33554432kB, limit 33554432kB, failcnt 9123
[  120.456480] invoked oom-killer: gfp_mask=0x1100cca(GFP_HIGHUSER_MOVABLE), order=0`,
    decisions: [
      { name: "Health", value: "unhealthy", class: "danger", confidence: 96, pct: 96 },
      { name: "Category", value: "memory", class: "warning", confidence: 97, pct: 97 },
      { name: "Severity", value: "4 / 5", class: "warning", confidence: 94, pct: 94 },
      { name: "Attention", value: "yes", class: "danger", confidence: 98, pct: 98 }
    ]
  },
  k8s: {
    pack: "kubernetes",
    input: `NAME                              READY   STATUS             RESTARTS      AGE
checkout-service-7df98f98f-4kmw9   0/1     CrashLoopBackOff   8 (90s ago)   14m

Events:
  Warning  BackOff    82s (x35 over 13m)   kubelet            Back-off restarting failed container checkout in pod checkout-service`,
    decisions: [
      { name: "Health", value: "unhealthy", class: "danger", confidence: 95, pct: 95 },
      { name: "Root Cause", value: "crashloop", class: "danger", confidence: 96, pct: 96 },
      { name: "Severity", value: "4 / 5", class: "warning", confidence: 92, pct: 92 },
      { name: "Attention", value: "yes", class: "danger", confidence: 98, pct: 98 }
    ]
  },
  ssh: {
    pack: "linux",
    input: `Sep 19 11:21:04 server sshd[28412]: Failed password for invalid user admin from 198.51.100.24 port 43120 ssh2
Sep 19 11:21:06 server sshd[28415]: Failed password for invalid user root from 198.51.100.24 port 43122 ssh2
Sep 19 11:21:15 server sshd[28430]: Maximum authentication attempts exceeded for invalid user admin [preauth]`,
    decisions: [
      { name: "Health", value: "degraded", class: "warning", confidence: 89, pct: 89 },
      { name: "Category", value: "security", class: "cyan", confidence: 95, pct: 95 },
      { name: "Severity", value: "3 / 5", class: "warning", confidence: 88, pct: 88 },
      { name: "Attention", value: "yes", class: "danger", confidence: 92, pct: 92 }
    ]
  },
  aws: {
    pack: "aws-cloudwatch",
    input: `2026-09-19T10:14:02.120Z [ecs-agent] Task payment-worker (arn:aws:ecs:us-east-1:123456789012:task/cluster/8fa41) stopped.
2026-09-19T10:14:02.122Z [ecs-agent] StopReason: Essential container in task exited
2026-09-19T10:14:02.125Z [container:payment-worker] ExitCode: 137, Reason: OutOfMemoryError
2026-09-19T10:14:02.130Z [alb/app-lb/502] HTTP/1.1 502 Bad Gateway - Target.FailedHealthChecks - response_processing_time: -1`,
    decisions: [
      { name: "Health", value: "unhealthy", class: "danger", confidence: 96, pct: 96 },
      { name: "Service", value: "ecs", class: "danger", confidence: 98, pct: 98 },
      { name: "Root Cause", value: "oom_killed", class: "danger", confidence: 97, pct: 97 },
      { name: "Action", value: "ssm_restart", class: "warning", confidence: 94, pct: 94 }
    ]
  },
  azure: {
    pack: "azure-monitor",
    input: `AzureMonitorAlert: Severity=Sev1, MonitorCondition=Fired
AlertRule: AppGateway-UnhealthyHostCount-Prod
TargetResource: /subscriptions/sub-id/resourceGroups/rg-prod/providers/Microsoft.Network/applicationGateways/appgw-prod
LogSummary: HTTP 502 Bad Gateway detected on backend pool 'aks-ingress-backend'.
Probe Error: Connection refused on port 8080. Consecutive failed probes: 5.`,
    decisions: [
      { name: "Health", value: "degraded", class: "warning", confidence: 91, pct: 91 },
      { name: "Resource", value: "app_gateway", class: "warning", confidence: 95, pct: 95 },
      { name: "Root Cause", value: "probe_failure", class: "danger", confidence: 93, pct: 93 },
      { name: "Action", value: "auto_heal", class: "success", confidence: 89, pct: 89 }
    ]
  },
  gcp: {
    pack: "gcp-cloud-ops",
    input: `2026-09-19T10:15:30.412Z [CRITICAL] resource.type="cloud_run_revision", service_name="checkout-api"
textPayload: "Memory limit of 512 MiB exceeded with 536 MiB used. Consider increasing the memory limit."
jsonPayload: { "container_id": "crun-98a1", "exit_code": 137, "status": "TERMINATED" }
severity: "ERROR"`,
    decisions: [
      { name: "Health", value: "unhealthy", class: "danger", confidence: 96, pct: 96 },
      { name: "Service", value: "cloud_run", class: "warning", confidence: 97, pct: 97 },
      { name: "Root Cause", value: "container_exited_137", class: "danger", confidence: 96, pct: 96 },
      { name: "Action", value: "restart_revision", class: "cyan", confidence: 92, pct: 92 }
    ]
  },
  healthy: {
    pack: "linux",
    input: `● nginx.service - A high performance web server
     Active: active (running) since Sat 2026-09-19 10:00:00 UTC; 4h ago
Sep 19 10:00:00 srv01 systemd[1]: Started Service nginx.service.
Sep 19 10:00:00 srv01 systemd[1]: All systems operational.`,
    decisions: [
      { name: "Health", value: "healthy", class: "success", confidence: 98, pct: 98 },
      { name: "Category", value: "normal", class: "success", confidence: 95, pct: 95 },
      { name: "Severity", value: "0 / 5", class: "success", confidence: 95, pct: 95 },
      { name: "Attention", value: "no", class: "success", confidence: 99, pct: 99 }
    ]
  }
};

const PACK_YAMLS = {
  linux: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "linux"
  version: "0.1.0"
  description: "General Linux diagnostic classification"
  author: "jev-ops"

spec:
  input:
    type: "text"
    max_bytes: 1048576

  decisions:
    health:
      type: "choice"
      values: [healthy, degraded, unhealthy, unknown]
    category:
      type: "choice"
      values: [normal, cpu, memory, filesystem, network, application, security, hardware, unknown]
    severity:
      type: "score"
      min: 0
      max: 5
    needs_attention:
      type: "boolean"`,

  kubernetes: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "kubernetes"
  version: "0.1.0"
  description: "Kubernetes pod, event, and cluster diagnostics"
  author: "jev-ops"

spec:
  input:
    type: "text"
    max_bytes: 2097152

  decisions:
    health:
      type: "choice"
      values: [healthy, degraded, unhealthy, unknown]
    root_cause:
      type: "choice"
      values: [scheduling, image_pull, crashloop, resources, networking, storage, application, unknown]
    severity:
      type: "score"
      min: 0
      max: 5
    needs_attention:
      type: "boolean"`,

  docker: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "docker"
  version: "0.1.0"
  description: "Docker container health, crash loops, and resource limits"
  author: "jev-ops"

spec:
  input:
    type: "text"

  decisions:
    health:
      type: "choice"
      values: [healthy, degraded, unhealthy, unknown]
    failure_mode:
      type: "choice"
      values: [normal, oom_killed, crash_loop, unhealthy_probe, exit_nonzero, port_conflict, unknown]
    severity:
      type: "score"
      min: 0
      max: 5
    needs_attention:
      type: "boolean"`,

  nginx: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "nginx"
  version: "0.1.0"
  description: "Nginx web server error logs, upstream gateways, and SSL issues"
  author: "jev-ops"

spec:
  input:
    type: "text"

  decisions:
    health:
      type: "choice"
      values: [healthy, degraded, unhealthy, unknown]
    issue_type:
      type: "choice"
      values: [normal, upstream_502_504, upstream_timeout, ssl_certificate, rate_limited, worker_connections, unknown]
    severity:
      type: "score"
      min: 0
      max: 5
    needs_attention:
      type: "boolean"`,

  mysql: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "mysql"
  version: "0.1.0"
  description: "MySQL / MariaDB performance bottlenecks, connection saturation, and deadlocks"
  author: "jev-ops"

spec:
  input:
    type: "text"

  decisions:
    health:
      type: "choice"
      values: [healthy, degraded, unhealthy, unknown]
    bottleneck:
      type: "choice"
      values: [normal, max_connections, deadlock, slow_queries, replication_lag, disk_full, unknown]
    severity:
      type: "score"
      min: 0
      max: 5
    needs_attention:
      type: "boolean"`,

  checkmk: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "checkmk"
  version: "0.1.0"
  description: "CheckMK monitoring alert noise reduction and intelligent action router"
  author: "jev-ops"

spec:
  input:
    type: "text"

  decisions:
    action:
      type: "choice"
      values: [ignore, watch, notify, auto_repair, escalate]
    root_domain:
      type: "choice"
      values: [normal, disk, memory, cpu, network, service_down, flapping, unknown]
    urgency:
      type: "score"
      min: 0
      max: 5
    page_oncall:
      type: "boolean"`,

  "ci-canary": `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "ci-canary"
  version: "0.1.0"
  description: "CI/CD canary deployment triage and automated rollback decision engine"
  author: "jev-ops"

spec:
  input:
    type: "text"

  decisions:
    rollout_action:
      type: "choice"
      values: [promote, hold_and_watch, rollback, manual_override]
    error_spike:
      type: "choice"
      values: [none, minor_tolerable, severe_5xx, latency_degradation, unknown]
    risk_score:
      type: "score"
      min: 0
      max: 5
    rollback_now:
      type: "boolean"`,

  custom: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "queue-service"
  version: "0.1.0"
  description: "Proprietary queue worker diagnostic pack"
  author: "Acme SRE"

spec:
  input:
    type: "text"

  decisions:
    state:
      type: "choice"
      values: [healthy, delayed, stuck, crashed, unknown]
    severity:
      type: "score"
      min: 0
      max: 5
    intervention_required:
      type: "boolean"`,

  aws: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "aws-cloudwatch"
  version: "0.1.0"
  description: "AWS CloudWatch Logs, ECS Fargate, ALB, Lambda, and RDS incident triage"
  author: "jev-ops"

spec:
  input:
    type: "text"
    max_bytes: 1048576

  decisions:
    health:
      type: "choice"
      values: [healthy, degraded, unhealthy, unknown]
    service:
      type: "choice"
      values: [ecs, lambda, alb, rds, api_gateway, dynamodb, s3, unknown]
    root_cause:
      type: "choice"
      values: [normal, oom_killed, timeout, connection_pool_exhausted, target_unhealthy, upstream_5xx, iam_denied, throttling, unknown]
    severity:
      type: "score"
      min: 0
      max: 5
    recommended_action:
      type: "choice"
      values: [no_action, ssm_restart, scale_out, flush_connection_pool, rollback_deployment, escalate_oncall]`,

  azure: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "azure-monitor"
  version: "0.1.0"
  description: "Azure Monitor alerts, AKS node pressure, App Gateway probes, and CosmosDB triage"
  author: "jev-ops"

spec:
  input:
    type: "text"
    max_bytes: 1048576

  decisions:
    health:
      type: "choice"
      values: [healthy, degraded, unhealthy, unknown]
    resource_type:
      type: "choice"
      values: [aks, app_gateway, cosmosdb, azure_functions, virtual_machine, storage_account, unknown]
    root_cause:
      type: "choice"
      values: [normal, crashloop, probe_failure, rate_limited_429, memory_pressure, certificate_expired, connection_refused, unknown]
    severity:
      type: "score"
      min: 0
      max: 5
    recommended_action:
      type: "choice"
      values: [no_action, auto_heal, scale_node_pool, throttle_backoff, renew_cert, escalate_oncall]`,

  gcp: `api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "gcp-cloud-ops"
  version: "0.1.0"
  description: "Google Cloud Logging, Cloud Run container faults, GKE evictions, and BigQuery triage"
  author: "jev-ops"

spec:
  input:
    type: "text"
    max_bytes: 1048576

  decisions:
    health:
      type: "choice"
      values: [healthy, degraded, unhealthy, unknown]
    service:
      type: "choice"
      values: [cloud_run, gke, cloud_sql, bigquery, pubsub, compute_engine, unknown]
    root_cause:
      type: "choice"
      values: [normal, container_exited_137, preemptible_eviction, connection_timeout, quota_exhausted, dead_letter_surge, out_of_memory, unknown]
    severity:
      type: "score"
      min: 0
      max: 5
    recommended_action:
      type: "choice"
      values: [no_action, restart_revision, reschedule_pod, increase_quota, dead_letter_purge, escalate_oncall]`
};

let currentScenario = "ext4";
let currentView = "cards";

const INTEGRATION_SNIPPETS = {
  checkmk: `#!/usr/bin/env bash
# /omd/sites/prod/local/share/check_mk/notifications/jev-ops-filter.sh
set -euo pipefail

# 1. Pipe CheckMK alert context directly to jev-ops
DECISION=$(echo "$NOTIFY_SERVICEOUTPUT" | jev-ops analyze checkmk --json)

ACTION=$(echo "$DECISION" | jq -r '.decisions.action.value')
CONFIDENCE=$(echo "$DECISION" | jq -r '.decisions.action.confidence')

# 2. Confidence-gated policy: Suppress noise if confidence > 85%
if [[ "$ACTION" == "IGNORE" && $(echo "$CONFIDENCE > 0.85" | bc -l) -eq 1 ]]; then
    echo "[jev-ops] Suppressing transient noise alert." >&2
    exit 0
elif [[ "$ACTION" == "AUTO_REPAIR" && $(echo "$CONFIDENCE > 0.95" | bc -l) -eq 1 ]]; then
    /opt/scripts/safe-remediation.sh
    exit 0
fi

# 3. Escalate real incidents to Slack / PagerDuty
/usr/bin/send_pagerduty_notification.sh`,

  ansible: `---
# playbooks/autonomous-triage.yml
- name: SRE Autonomous Diagnostics and Gated Remediation
  hosts: fleet
  tasks:
    - name: Fetch last 50 journal errors
      ansible.builtin.shell: "journalctl -p err -n 50 --since '-10m'"
      register: journal_data

    - name: Evaluate through jev-ops System One decision layer
      ansible.builtin.shell: "jev-ops analyze linux --json"
      args:
        stdin: "{{ journal_data.stdout }}"
      register: jev_out

    - name: Parse typed decisions
      ansible.builtin.set_fact:
        jev: "{{ jev_out.stdout | from_json }}"

    # SAFETY GATE: Auto-restart only if model confidence >= 0.95
    - name: Safe service restart
      ansible.builtin.service:
        name: nginx
        state: restarted
      when:
        - jev.decisions.needs_attention.value == true
        - (jev.decisions.category.confidence | float) >= 0.95`,

  k8s: `#!/usr/bin/env bash
# k8s-triage-daemon.sh — Live cluster event stream triage
kubectl get events -A --watch-only | while read -r event; do
    if echo "$event" | grep -qE "BackOff|OOMKilled|Unhealthy"; then
        # Evaluate pod event in real-time
        DECISION=$(echo "$event" | jev-ops analyze kubernetes --json)
        
        CAUSE=$(echo "$DECISION" | jq -r '.decisions.root_cause.value')
        SEV=$(echo "$DECISION" | jq -r '.decisions.severity.value')
        
        echo "[jev-ops] Event Alert: cause=\${CAUSE} severity=\${SEV}/5"
        if [[ "$CAUSE" == "crashloop" && "$SEV" -ge 4 ]]; then
            # Escalate or initiate automated diagnostic dump
            curl -X POST "$SLACK_WEBHOOK" -d "{\\"text\\":\\"🚨 CrashLoopBackOff: $event\\"}"
        fi
    fi
done`,

  gha: `name: Canary Rollout Gate
on: [deployment_status]

jobs:
  canary-check:
    runs-on: ubuntu-latest
    steps:
      - name: Evaluate Canary Metrics with jev-ops
        id: gate
        run: |
          DECISION=$(curl -s "https://metrics.internal/canary" | jev-ops analyze ci-canary --json)
          ROLLBACK=$(echo "$DECISION" | jq -r '.decisions.rollback_now.value')
          echo "rollback=$ROLLBACK" >> $GITHUB_OUTPUT

      - name: Safety Gate Check
        if: steps.gate.outputs.rollback == 'true'
        run: |
          echo "❌ Canary metrics degraded! Initiating automated rollback..."
          argocd app rollback payment-service`,

  openclaw: `# OpenClaw / Agentic AI Router integration
from openclaw import Agent, Tool
import subprocess, json

def route_incident_tool(incident_log: str) -> str:
    """Uses jev-ops System One layer to decide which specialist agent to run"""
    proc = subprocess.run(
        ["jev-ops", "analyze", "linux", "--json"],
        input=incident_log.encode(),
        stdout=subprocess.PIPE
    )
    result = json.loads(proc.stdout)
    category = result["decisions"]["category"]["value"]
    
    # Fast deterministic routing without burning Claude / GPT tokens:
    if category == "database":
        return "mysql_diagnostic_agent"
    elif category == "network":
        return "network_mesh_agent"
    return "general_sre_agent"`,

  aws: `# AWS Lambda / CloudWatch Logs Automated Remediation
import base64, gzip, json, subprocess, boto3

def lambda_handler(event, context):
    payload = gzip.decompress(base64.b64decode(event["awslogs"]["data"]))
    logs = "\\n".join([e["message"] for e in json.loads(payload)["logEvents"][:20]])
    
    # Analyze via jev-ops aws-cloudwatch pack
    proc = subprocess.run(
        ["jev-ops", "analyze", "aws-cloudwatch", "--json"],
        input=logs.encode(), stdout=subprocess.PIPE
    )
    result = json.loads(proc.stdout)
    action = result["decisions"]["recommended_action"]["value"]
    conf = result["decisions"]["recommended_action"]["confidence"]

    # Automated remediation if confidence >= 90%
    if action == "ssm_restart" and conf >= 0.90:
        ecs = boto3.client("ecs")
        ecs.update_service(cluster="prod", service="payment-worker", forceNewDeployment=True)
        return {"remediated": True, "action": "ecs_cycled"}
    return {"remediated": False, "escalated": True}`,

  azure: `# Azure Monitor Action Group HTTP Webhook Handler
import json, subprocess
from http.server import BaseHTTPRequestHandler, HTTPServer

def triage_azure_alert(alert_payload):
    target = alert_payload["data"]["essentials"]["alertTargetIDs"][0]
    proc = subprocess.run(
        ["jev-ops", "analyze", "azure-monitor", "--json"],
        input=json.dumps(alert_payload).encode(), stdout=subprocess.PIPE
    )
    res = json.loads(proc.stdout)
    action = res["decisions"]["recommended_action"]["value"]
    
    if action == "auto_heal" and res["decisions"]["recommended_action"]["confidence"] >= 0.85:
        # Trigger automated Azure CLI probe resolution
        subprocess.run(["az", "aks", "nodepool", "scale", "--cluster-name", "prod-aks", "--count", "5"])
        return "AUTO_HEAL_TRIGGERED"
    return "ESCALATED_TO_SRE"`,

  gcp: `#!/usr/bin/env bash
# GCP Cloud Logging -> Pub/Sub -> Cloud Run Auto-Healer
set -euo pipefail

RAW_LOGS=$(gcloud logging read "resource.type=cloud_run_revision AND severity>=WARNING" --limit=20 --format="value(textPayload)")

# Triage logs through GCP cloud pack
RESULT=$(echo "$RAW_LOGS" | jev-ops analyze gcp-cloud-ops --json)
ACTION=$(echo "$RESULT" | jq -r '.decisions.recommended_action.value')
CONFIDENCE=$(echo "$RESULT" | jq -r '(.decisions.recommended_action.confidence * 100) | floor')

if [[ "$ACTION" == "restart_revision" && "$CONFIDENCE" -ge 85 ]]; then
    echo "[jev-ops GCP] Safe Auto-Heal: Boosting Cloud Run memory ceiling to 1Gi..."
    gcloud run services update checkout-api --memory 1Gi --quiet
    exit 0
fi`,

  terraform: `#!/usr/bin/env bash
# Terraform / OpenTofu Plan Safety Gate
set -euo pipefail

# Parse plan changes and pipe into jev-ops
RESULT=$(terraform show -json tfplan.binary | jev-ops analyze aws-cloudwatch --json 2>/dev/null || echo '{}')
SEV=$(echo "$RESULT" | jq -r '.decisions.severity.value // 0')

if [ "$SEV" -ge 4 ]; then
    echo "❌ [BLOCK] High blast-radius destruction detected (Severity $SEV/5)!"
    echo "Aborting CI/CD automated apply. Manual dual-approval required."
    exit 1
fi
echo "✅ [PASS] Plan verified safe for automated apply."`
};

function setupIntegrationTabs() {
  const buttons = document.querySelectorAll(".integ-tab-btn");
  const codeBlock = document.getElementById("integCodeBlock");
  if (!buttons.length || !codeBlock) return;

  buttons.forEach(btn => {
    btn.addEventListener("click", () => {
      buttons.forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      const integ = btn.dataset.integ;
      if (INTEGRATION_SNIPPETS[integ]) {
        codeBlock.textContent = INTEGRATION_SNIPPETS[integ];
      }
    });
  });
}

document.addEventListener("DOMContentLoaded", () => {
  setupThemeToggle();
  setupScenarioPicker();
  setupViewTabs();
  setupPackTabs();
  setupIntegrationTabs();
  setupCopyButtons();
  runAnalysis();
});

function setupScenarioPicker() {
  const buttons = document.querySelectorAll(".scenario-btn");
  const textarea = document.getElementById("simInput");

  buttons.forEach(btn => {
    btn.addEventListener("click", () => {
      buttons.forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      currentScenario = btn.dataset.scenario;
      textarea.value = SCENARIOS[currentScenario].input;
      runAnalysis();
    });
  });

  const runBtn = document.getElementById("simRunBtn");
  if (runBtn) {
    runBtn.addEventListener("click", () => {
      runAnalysis();
    });
  }
}

function setupViewTabs() {
  const tabs = document.querySelectorAll(".btn-tab");
  tabs.forEach(tab => {
    tab.addEventListener("click", () => {
      tabs.forEach(t => t.classList.remove("active"));
      tab.classList.add("active");
      currentView = tab.dataset.view;
      renderOutput();
    });
  });
}

function runAnalysis() {
  const outputContainer = document.getElementById("outputView");
  outputContainer.style.opacity = "0.5";
  
  setTimeout(() => {
    outputContainer.style.opacity = "1";
    renderOutput();
  }, 150);
}

function renderOutput() {
  const outputContainer = document.getElementById("outputView");
  const scenario = SCENARIOS[currentScenario] || SCENARIOS.ext4;

  if (currentView === "cards") {
    let html = `<div class="decision-cards">`;
    scenario.decisions.forEach(d => {
      html += `
        <div class="decision-card">
          <div class="decision-meta">
            <span class="decision-name">${d.name}</span>
            <span class="decision-value ${d.class}">${d.value}</span>
          </div>
          <div class="decision-meter">
            <span class="meter-pct">${d.confidence}%</span>
            <div class="meter-bar-track">
              <div class="meter-bar-fill" style="width: ${d.pct}%"></div>
            </div>
          </div>
        </div>
      `;
    });
    html += `</div>`;
    outputContainer.innerHTML = html;
  } else if (currentView === "human") {
    let text = `jev-ops analysis\n\n`;
    text += `Pack:       ${scenario.pack} 0.1.0\n`;
    text += `Provider:   typesafe-jev (jev-latest)\n`;
    text += `Input:      ${scenario.input.length} B\n\n`;
    scenario.decisions.forEach(d => {
      const padName = (d.name + ":").padEnd(12, " ");
      const padVal = d.value.padEnd(14, " ");
      text += `${padName}${padVal} ${d.confidence}%\n`;
    });
    outputContainer.innerHTML = `<pre><code>${escapeHtml(text)}</code></pre>`;
  } else if (currentView === "json") {
    const jsonObj = {
      schema_version: "1",
      pack: { name: scenario.pack, version: "0.1.0" },
      provider: "typesafe-jev",
      input: { bytes: scenario.input.length, truncated: false },
      decisions: {}
    };
    scenario.decisions.forEach(d => {
      const key = d.name.toLowerCase().replace(" ", "_");
      let val = d.value;
      let type = "choice";
      if (val === "yes") { val = true; type = "boolean"; }
      else if (val === "no") { val = false; type = "boolean"; }
      else if (val.includes("/ 5")) { val = parseInt(val.charAt(0)); type = "score"; }

      jsonObj.decisions[key] = {
        type: type,
        value: val,
        confidence: d.confidence / 100.0
      };
    });
    outputContainer.innerHTML = `<pre><code>${escapeHtml(JSON.stringify(jsonObj, null, 2))}</code></pre>`;
  }
}

function setupPackTabs() {
  const buttons = document.querySelectorAll(".pack-tab-btn");
  const codeBlock = document.getElementById("packCodeBlock");

  buttons.forEach(btn => {
    btn.addEventListener("click", () => {
      buttons.forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      const pack = btn.dataset.pack;
      if (PACK_YAMLS[pack]) {
        codeBlock.textContent = PACK_YAMLS[pack];
      }
    });
  });
}

function setupCopyButtons() {
  document.querySelectorAll(".copy-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      const targetId = btn.dataset.target;
      const target = document.getElementById(targetId);
      if (target) {
        navigator.clipboard.writeText(target.innerText || target.textContent).then(() => {
          const original = btn.textContent;
          btn.textContent = "Copied!";
          setTimeout(() => { btn.textContent = original; }, 2000);
        });
      }
    });
  });
}

function escapeHtml(str) {
  return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function setupThemeToggle() {
  const toggleBtn = document.getElementById("themeToggle");
  const themeIcon = document.getElementById("themeIcon");
  if (!toggleBtn) return;

  // Retrieve saved theme or default to light theme
  const savedTheme = localStorage.getItem("jev-ops-theme") || "light";
  document.documentElement.setAttribute("data-theme", savedTheme);
  updateThemeIcon(savedTheme, themeIcon);

  toggleBtn.addEventListener("click", () => {
    const current = document.documentElement.getAttribute("data-theme") || "light";
    const next = current === "dark" ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", next);
    localStorage.setItem("jev-ops-theme", next);
    updateThemeIcon(next, themeIcon);
  });
}

function updateThemeIcon(theme, iconEl) {
  if (!iconEl) return;
  // If dark, show sun to switch to light; if light, show moon to switch to dark
  iconEl.textContent = theme === "dark" ? "☀️" : "🌙";
}

