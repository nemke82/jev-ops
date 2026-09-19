# jev-ops

> **Extensible AI-powered diagnostics for DevOps & SRE** — analyze logs and infrastructure signals using pluggable diagnostic packs and typed probabilistic decisions powered by [Jev](https://docs.typesafe.ai/introduction).

[![CI](https://github.com/nemke82/jev-ops/actions/workflows/ci.yml/badge.svg)](https://github.com/nemke82/jev-ops/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v2026.09.19-blue.svg)](https://github.com/nemke82/jev-ops/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Overview

`jev-ops` is an extensible CLI and diagnostic framework designed for DevOps, SRE, and platform engineers. It ingests unstructured operational state (system logs, journal entries, Kubernetes events, database telemetry, command outputs) and evaluates them against declarative diagnostic packs to return structured, typed decisions.

Unlike conversational LLMs that output verbose prose requiring fragile regex parsing, `jev-ops` is built on the **TypeSafe System One** model philosophy: **unstructured state in, typed probabilistic decisions out**.

```
Linux / Kubernetes / Logs / Monitoring
                 ↓
              jev-ops
                 ↓
           [Diagnostic Pack]
                 ↓
      [TypeSafe Jev Inference]
                 ↓
 ┌──────────┬──────────┬──────────┬──────────┐
 │  IGNORE  │  NOTIFY  │  REPAIR  │ ESCALATE │
 └──────────┴──────────┴──────────┴──────────┘
                 ↓
        Bash / SRE Automation
```

> [!NOTE]
> **TypeSafe Jev Live Integration**:
> `jev-ops` supports both the deterministic offline `mock` provider and live inference via the **TypeSafe System One API** (`POST https://api.typesafe.ai/v1/systemone`).
>
> Simply export your API key:
> ```bash
> export TYPESAFE_API_KEY="ts_live_..."
> ```
> `jev-ops` automatically detects your key and connects to the `jev-latest` model. Without a key, `analyze` exits with code `5` rather than guessing; pass `--provider mock` explicitly for offline, keyword-based output (useful for demos and tests, never for automation).

---

## Core Philosophy: Core Knows Nothing, Packs Teach Everything

The Rust core binary contains **zero technology-specific diagnostic logic**. It knows nothing about Linux, Docker, Kubernetes, MySQL, or Magento. 

Domain vocabulary, problem categories, rubrics, and diagnostic instructions are defined entirely inside **externally loadable diagnostic packs** (`pack.yaml`). Anyone can add support for proprietary internal services without modifying or recompiling `jev-ops`.

### Pack = Data, Not Code (Security First)
In `jev-ops`, diagnostic packs are strictly declarative YAML manifests.
- **NO** shell execution
- **NO** subprocess spawning (`exec`, `system`)
- **NO** network requests or socket access
- **NO** filesystem mutations

---

## TypeSafe System One Primitives

In accordance with [TypeSafe AI's System One architecture](https://docs.typesafe.ai/introduction), `jev-ops` evaluates atomic, well-scoped questions against system state:

| Primitive | Purpose | Returns |
| :--- | :--- | :--- |
| **`choice`** | Classify into discrete domain categories | Selected option, confidence score (0.0–1.0), and probabilities distribution |
| **`score`** | Score system state on a rubric of 2–10 described levels (e.g. 0 to 5) | Numerical score in the pack's range, confidence, and probabilities distribution |
| **`boolean`** (Noul) | Binary condition check ("Is attention required?") | `yes` / `no`, confidence score (0.0–1.0) |

---

## Installation

### Standalone Binaries (Direct Install)

Install pure standalone binaries directly with `curl`:

```bash
# Linux (x86_64 glibc)
sudo curl -fsSL https://github.com/nemke82/jev-ops/releases/download/v2026.09.19/jev-ops-linux-x86_64 -o /usr/local/bin/jev-ops
sudo chmod +x /usr/local/bin/jev-ops

# Linux (static musl / Alpine / containers)
sudo curl -fsSL https://github.com/nemke82/jev-ops/releases/download/v2026.09.19/jev-ops-linux-x86_64-musl -o /usr/local/bin/jev-ops
sudo chmod +x /usr/local/bin/jev-ops

# Linux (ARM64 / AWS Graviton)
sudo curl -fsSL https://github.com/nemke82/jev-ops/releases/download/v2026.09.19/jev-ops-linux-arm64 -o /usr/local/bin/jev-ops
sudo chmod +x /usr/local/bin/jev-ops

# macOS (Apple Silicon M1/M2/M3/M4)
sudo curl -fsSL https://github.com/nemke82/jev-ops/releases/download/v2026.09.19/jev-ops-macos-arm64 -o /usr/local/bin/jev-ops
sudo chmod +x /usr/local/bin/jev-ops

# macOS (Intel)
sudo curl -fsSL https://github.com/nemke82/jev-ops/releases/download/v2026.09.19/jev-ops-macos-x86_64 -o /usr/local/bin/jev-ops
sudo chmod +x /usr/local/bin/jev-ops

# Windows (PowerShell)
curl.exe -fsSL https://github.com/nemke82/jev-ops/releases/download/v2026.09.19/jev-ops-windows-x86_64.exe -o jev-ops.exe
```

Standalone binaries and SHA256 checksums for all platforms are available on [GitHub Releases](https://github.com/nemke82/jev-ops/releases/tag/v2026.09.19).

### From Source

```bash
git clone https://github.com/nemke82/jev-ops.git
cd jev-ops
cargo build --release
sudo cp target/release/jev-ops /usr/local/bin/
```

---

## Usage & Workflows

### 1. Modern Linux & Cloud Pipelines (STDIN)

Pipe diagnostic outputs directly into `jev-ops`:

```bash
# Analyze kernel dmesg errors with the linux pack
dmesg -T | grep -E "EXT4|error" | jev-ops analyze linux

# Analyze AWS CloudWatch log streams from ECS Fargate
aws logs tail /aws/ecs/payment-service --since 10m | jev-ops analyze aws-cloudwatch

# Analyze GCP Cloud Run errors from Google Cloud Logging
gcloud logging read "resource.type=cloud_run_revision AND severity>=WARNING" --limit=25 | jev-ops analyze gcp-cloud-ops

# Analyze Azure Monitor alert context
az monitor alert show --id "$ALERT_ID" | jev-ops analyze azure-monitor

# Analyze Kubernetes events or failing pods
kubectl get pods -A | jev-ops analyze kubernetes
```

### 2. Machine-Readable JSON Output (`--json`)

Perfect for scripts, monitoring hooks, or automation agents:

```bash
cat tests/fixtures/linux/ext4-error.txt | jev-ops analyze linux --provider mock --json
```

Output:
```json
{
  "schema_version": "1",
  "pack": {
    "name": "linux",
    "version": "0.1.0"
  },
  "provider": "mock",
  "input": {
    "bytes": 512,
    "truncated": false
  },
  "decisions": {
    "health": {
      "type": "choice",
      "value": "unhealthy",
      "confidence": 0.96
    },
    "category": {
      "type": "choice",
      "value": "filesystem",
      "confidence": 0.98
    },
    "severity": {
      "type": "score",
      "value": 5,
      "confidence": 0.91
    },
    "needs_attention": {
      "type": "boolean",
      "value": true,
      "confidence": 0.99
    }
  }
}
```

Extract decisions directly with `jq`:
```bash
$ cat error.log | jev-ops analyze linux --json | jq '.decisions.severity.value'
5
```

### 3. Human-Readable Terminal Summary

```bash
$ cat tests/fixtures/linux/ext4-error.txt | jev-ops analyze linux --provider mock
jev-ops analysis

Pack:       linux 0.1.0   
Provider:   mock          
Input:      512 B         

Health:     unhealthy      96%
Category:   filesystem     98%
Severity:   5/5            91%
Attention:  yes            99%
```

### 4. Confidence-Gated Alerting (`--min-confidence`)

Filter or flag decisions when the model reports uncertainty:

```bash
cat dmesg.log | jev-ops analyze linux --min-confidence 0.95
```

The threshold must be between `0.0` and `1.0`. Human output marks weak decisions with `[LOW CONFIDENCE]`; with `--json`, the output adds `min_confidence` and a `low_confidence` array naming the decisions below it.

### 5. Connecting to Live TypeSafe Jev Flagship Model

Run live inference with TypeSafe's `jev-latest` model:

```bash
# Via environment variable
export TYPESAFE_API_KEY="ts_live_your_api_key"
cat error.log | jev-ops analyze linux

# Or explicitly via CLI flag
cat error.log | jev-ops analyze linux --provider typesafe --api-key "ts_live_..."

# Custom API endpoint / proxy / model override
export TYPESAFE_API_URL="https://api.typesafe.ai/v1"
export TYPESAFE_MODEL="jev-latest"
```

---

## DevOps Industry Automation Integrations

`jev-ops` integrates cleanly with standard monitoring, CI/CD, orchestration, and security tools. Production-ready recipes are located in [`integrations/`](integrations/):

### 1. CheckMK Alert De-noising & Triage
Pipe service checks and growth metrics into `checkmk` pack to suppress transient spikes and auto-remediate:
```bash
# In /omd/sites/<site>/local/share/check_mk/notifications/jev-ops-filter.sh
DECISION=$(echo "$NOTIFY_SERVICEOUTPUT" | jev-ops analyze checkmk --json)
ACTION=$(echo "$DECISION" | jq -r '.decisions.action.value')
CONFIDENCE=$(echo "$DECISION" | jq -r '.decisions.action.confidence')

if [[ "$ACTION" == "ignore" ]] && jq -e '.decisions.action.confidence > 0.85' <<<"$DECISION" >/dev/null; then
    exit 0 # Suppress noise
fi
```
See [integrations/checkmk/checkmk-notification.sh](integrations/checkmk/checkmk-notification.sh).

### 2. Ansible Gated Auto-Remediation
Use typed confidence to gate destructive actions in Ansible playbooks:
```yaml
- name: Safe service restart
  ansible.builtin.service:
    name: nginx
    state: restarted
  when:
    - jev.decisions.needs_attention.value
    - jev.decisions.category.value == 'application'
    - (jev.decisions.category.confidence | float) >= 0.95
```
See [integrations/ansible/auto-remediate.yml](integrations/ansible/auto-remediate.yml).

### 3. Kubernetes Event Stream Triage
Monitor pod warnings, CrashLoops, and OOMKilled events in real time:
```bash
kubectl get events -A --watch-only | ./integrations/k8s/k8s-event-watch.sh
```

### 4. CI/CD Canary Deployment Gate
Halt rollouts and trigger automated rollbacks in GitHub Actions or ArgoCD:
```yaml
- name: Evaluate Canary Health
  run: |
    DECISION=$(curl -s "https://metrics/canary" | jev-ops analyze ci-canary --json)
    if [ "$(echo $DECISION | jq -r .decisions.rollback_now.value)" = "true" ]; then
      argocd app rollback payment-service
      exit 1
    fi
```
See [integrations/github-actions/canary-gate.yml](integrations/github-actions/canary-gate.yml).

### 5. AWS CloudWatch & Lambda Auto-Remediation
Automate ECS task cycling and dispatch rich Slack triage when containers crash:
```python
# In AWS Lambda subscription filter handler
proc = subprocess.run(["jev-ops", "analyze", "aws-cloudwatch", "--json"], input=log_bytes, stdout=subprocess.PIPE)
result = json.loads(proc.stdout)
if result["decisions"]["recommended_action"]["value"] == "ssm_restart" and result["decisions"]["recommended_action"]["confidence"] >= 0.90:
    ecs.update_service(cluster="prod", service="checkout", forceNewDeployment=True)
```
See [integrations/aws/cloudwatch-lambda-triage.py](integrations/aws/cloudwatch-lambda-triage.py).

### 6. Azure Monitor Webhook Auto-Heal
Triage Azure Common Alert Schema webhooks and auto-remediate AKS and Application Gateway faults:
```python
# In Azure HTTP Webhook receiver
proc = subprocess.run(["jev-ops", "analyze", "azure-monitor", "--json"], input=alert_json, stdout=subprocess.PIPE)
res = json.loads(proc.stdout)
if res["decisions"]["recommended_action"]["value"] == "auto_heal":
    subprocess.run(["az", "aks", "nodepool", "scale", "--count", "5", ...])
```
See [integrations/azure/azure-monitor-webhook.py](integrations/azure/azure-monitor-webhook.py).

### 7. GCP Cloud Logging & Cloud Run Auto-Heal
Stream Cloud Run OOM events and automatically double memory ceilings:
```bash
RAW_LOGS=$(gcloud logging read "resource.type=cloud_run_revision" --limit=25)
RESULT=$(echo "$RAW_LOGS" | jev-ops analyze gcp-cloud-ops --json)
# If OOM confirmed by model:
gcloud run services update checkout-api --memory 1Gi --quiet
```
See [integrations/gcp/cloud-run-triage.sh](integrations/gcp/cloud-run-triage.sh).

### 8. Terraform Plan Blast-Radius Gate
Summarize `terraform show -json` output and judge it with the `terraform-plan` pack, blocking destructive drops or IAM and network changes before merge. The gate fails closed: if the plan cannot be parsed or analyzed, it blocks.
```bash
terraform show -json tfplan.binary | ./integrations/terraform/plan-triage.sh
```
See [integrations/terraform/plan-triage.sh](integrations/terraform/plan-triage.sh).

---

## Bundled Diagnostic Packs

`jev-ops` ships with 12 standard packs covering cloud platforms, infrastructure layers, and CI/CD:

| Pack | Focus Area | Key Decisions Evaluated |
| :--- | :--- | :--- |
| **`aws-cloudwatch`** | AWS ECS, ALB, Lambda, RDS | `health`, `service`, `root_cause`, `severity`, `recommended_action` |
| **`azure-monitor`** | Azure AKS, App Gateway, CosmosDB | `health`, `resource_type`, `root_cause`, `severity`, `recommended_action` |
| **`gcp-cloud-ops`** | GCP Cloud Run, GKE, BigQuery | `health`, `service`, `root_cause`, `severity`, `recommended_action` |
| **`linux`** | Kernel & system logs | `health`, `category` (memory/fs/network/sec), `severity`, `needs_attention` |
| **`kubernetes`** | Pods & cluster events | `health`, `root_cause` (crashloop/oom/scheduling), `severity`, `needs_attention` |
| **`docker`** | Containers & daemons | `health`, `failure_mode` (oom_killed/crash_loop/unhealthy_probe), `severity` |
| **`nginx`** | Web proxies & ingresses | `health`, `issue_type` (502_504/timeout/ssl/rate_limit), `severity` |
| **`mysql`** | Databases & replicas | `health`, `bottleneck` (connections/deadlock/slow_queries/replication), `severity` |
| **`checkmk`** | Alert de-noising | `action` (ignore/watch/notify/auto_repair/escalate), `root_domain`, `urgency` |
| **`ci-canary`** | Deployment verification | `rollout_action` (promote/watch/rollback), `error_spike`, `risk_score` |
| **`terraform-plan`** | IaC change review | `change_risk`, `blast_radius`, `destroys_stateful_resource` |
| **`queue-service`** | Custom reference pack | Demonstrates domain customization without Rust code |

---

## Managing Diagnostic Packs

### List Discovered Packs

```bash
$ jev-ops packs list
AVAILABLE DIAGNOSTIC PACKS:
NAME               VERSION    DESCRIPTION                              PATH
────────────────────────────────────────────────────────────────────────────────
example            0.1.0      Generic example diagnostic pack demon... ./packs/example/pack.yaml
kubernetes         0.1.0      Kubernetes pod, event, and cluster di... ./packs/kubernetes/pack.yaml
linux              0.1.0      General Linux diagnostic classificati... ./packs/linux/pack.yaml
```

### Inspect Pack Manifest & Decisions

```bash
$ jev-ops packs show kubernetes
Diagnostic Pack: kubernetes
Version:         0.1.0
Author:          jev-ops
Path:            ./packs/kubernetes/pack.yaml
Description:     Kubernetes pod, event, and cluster diagnostics

Spec Input:
  Type:          text
  Max Bytes:     2097152

Decisions (4 defined):
  - health: choice (options: healthy, degraded, unhealthy, unknown)
  - root_cause: choice (options: scheduling, image_pull, crashloop, resources, networking, storage, application, unknown)
  - needs_attention: boolean (yes/no)
  - severity: score (range: 0 to 5)
      0: Normal: no errors or anomalies in the signals
      1: Informational: notable events with no impact on service
      2: Minor: isolated errors or warnings with no user-visible impact
      3: Degraded: partial impairment, elevated errors or latency for some users or requests
      4: Serious: a major component is failing or most requests are affected; urgent action needed
      5: Critical: outage, data loss or corruption risk, or active security compromise

Instructions:
Analyze Kubernetes diagnostic output including pod statuses, events, describe outputs, and logs.
Classify whether the workload is healthy, degraded, or unhealthy.
Identify the most probable root cause from the domain options.
```

### Scaffold a New Pack

Create a custom diagnostic pack in seconds:

```bash
jev-ops packs new my-service --dir ./packs
```

### Validate a Pack Manifest

```bash
jev-ops packs validate ./packs/my-service/pack.yaml
```

---

## Anatomy of a Diagnostic Pack

Packs are defined in YAML (`pack.yaml`):

```yaml
api_version: "jev-ops/v1"
kind: "DiagnosticPack"

metadata:
  name: "queue-service"
  version: "0.1.0"
  description: "Diagnostic pack for queue workers and latency spikes"
  author: "SRE Team"

spec:
  input:
    type: "text"
    max_bytes: 1048576   # 1 MiB limit

  decisions:
    status:
      type: "choice"
      values:
        - healthy
        - backlogged
        - stuck
        - crashed
        - unknown

    severity:
      type: "score"
      min: 0
      max: 5
      levels:              # optional: one description per level, min to max
        - "No backlog; messages processed as they arrive"
        - "Small backlog that clears on its own"
        - "Growing backlog; latency noticeable but within SLO"
        - "Backlog breaching SLO for some consumers"
        - "Most consumers stalled; SLO breached broadly"
        - "Processing halted or messages being lost"

    page_oncall:
      type: "boolean"

  instructions: |
    Analyze queue worker heartbeat logs and message processing lag.
    Determine whether processing is healthy or stuck, and whether the oncall
    engineer must be paged immediately.
```

Pack rules worth knowing:

- Unknown keys are rejected, so a typo such as `max_byte` fails validation instead of being silently ignored.
- A `score` may span at most 10 levels (`max - min + 1 <= 10`), matching the TypeSafe API. If `levels` is given, it needs exactly one entry per level.
- `instructions` are sent to TypeSafe as `analysis_guidance` alongside the input, so rubric details written there reach the model.

---

## Pack Discovery Order

`jev-ops` resolves packs using deterministic search precedence:

1. CLI flag: `--pack-dir <PATH>`
2. Local working directory: `./packs/`
3. User configuration directory: `~/.config/jev-ops/packs/`
4. User local data directory: `~/.local/share/jev-ops/packs/`
5. System-wide share directory: `/usr/share/jev-ops/packs/`

---

## Shell Auto-Completions

Generate native shell completions for Bash, Zsh, Fish, or PowerShell:

```bash
# Bash
source <(jev-ops completion bash)

# Zsh
jev-ops completion zsh > "${fpath[1]}/_jev-ops"

# Fish
jev-ops completion fish > ~/.config/fish/completions/jev-ops.fish
```

---

## Exit Codes

`jev-ops` returns predictable standard process exit codes:

| Code | Status | Description |
| :--- | :--- | :--- |
| `0` | **Success** | Diagnostic analysis completed successfully |
| `1` | **General Error** | Internal runtime or I/O error |
| `2` | **Invalid CLI Usage** | Unknown argument, missing flag, or bad syntax |
| `3` | **Invalid Pack** | Pack not found, syntax error, or schema validation failure |
| `4` | **Invalid Input** | Empty input, input exceeding max bytes, binary garbage, or malformed UTF-8 |
| `5` | **Provider Failure** | Inference provider failure or timeout, or no API key when no `--provider` is given |
| `6` | **Invalid Provider Response** | Provider returned decisions that violate pack schema constraints |

---

## Roadmap

- **v2026.09.19 (Current Release)**:
  - Core extensible CLI and streaming pipeline with standard exit codes (`0`–`6`).
  - Direct integration with TypeSafe System One flagship model (`api.typesafe.ai/v1`) and offline mock fallback.
  - 11 production diagnostic packs covering AWS CloudWatch, Azure Monitor, GCP Cloud Ops, Linux, Kubernetes, Docker, Nginx, MySQL, CheckMK, Canary, and Example.
  - Multi-cloud automation recipes for AWS Lambda SSM, Azure Monitor webhooks, GCP Cloud Run auto-healing, and Terraform plan safety gates.
  - Automated multi-platform GitHub Releases (Linux x86_64/arm64/musl, macOS Darwin Apple Silicon/Intel, Windows) and live GitHub Pages showcase.

- **v2026.10 (Cloud Streaming & OpenTelemetry)**:
  - Native OpenTelemetry (OTel) log record and span ingestion.
  - CloudWatch, Azure Monitor, and GCP Cloud Logging stream helpers (`jev-ops stream aws --log-group <name>`).
  - JSON Schema generation for pack authors (`jev-ops packs schema`).
  - Interactive shell auto-completion for dynamically installed packs.

- **v2026.11 (Pack Ecosystem & Distribution)**:
  - Git-based pack manager (`jev-ops packs install github.com/<org>/<pack>`).
  - Semantic pack version pinning and compatibility verification.
  - Incident replay and regression testing harness (`jev-ops replay incident.log --pack <name>`).

- **v2026.12 (Multi-Signal Correlation & Rich SRE Webhooks)**:
  - Cross-pack cascading correlation: evaluate simultaneous network, database, and container signals into a unified root-cause decision.
  - Built-in Slack, PagerDuty, and Microsoft Teams webhook formatters with rich Block Kit triage cards.
  - Configurable policy engine for automated confidence gates (`--min-confidence 0.90`).

- **v2027.01+ (Kubernetes Operator & Air-Gapped Inference)**:
  - Kubernetes Auto-Remediation Operator (`jev-ops-operator`) with CRD-driven diagnostic routing.
  - Local edge model execution fallback for air-gapped and disconnected environments.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
