# Custom Pack Example: `queue-service`

This example demonstrates how anyone can extend `jev-ops` to diagnose proprietary or domain-specific applications **without writing or recompiling any Rust code**.

## Pack Structure

```
custom-pack/
├── pack.yaml
└── README.md
```

## How to Test This Pack

Validate the pack manifest:
```bash
jev-ops packs validate ./pack.yaml
```

Run an analysis using this custom pack:
```bash
cat << 'EOF' | jev-ops analyze queue-service --pack-dir .
[2026-09-19 12:00:00] queue-worker-01: Heartbeat OK, processed 4120 jobs.
[2026-09-19 12:05:00] queue-worker-02: Heartbeat OK, processed 3980 jobs.
EOF
```

With JSON output:
```bash
cat queue.log | jev-ops analyze queue-service --pack-dir . --json
```

## TypeSafe System One Principles for Custom Packs

1. **Atomic Questions**: Decompose complex questions into separate, well-scoped determinations (e.g. `state`, `severity`, and `intervention_required`).
2. **Domain Vocabulary**: Define custom choice options (`delayed`, `stuck`, `crashed`) that make sense for your specific technology.
3. **No Execution**: Packs are strictly declarative data. They never invoke shell scripts or execute commands.
