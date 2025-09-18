# LIMINAL Router Foundation

LIMINAL is a desktop-first multi-agent workspace built with Vite + React + Tauri. The current foundation focuses on the router, territory (lease) manager, PTY pipeline, and observability surface.

## Current Capabilities

- **Unified Message Router** – Priority queues (Info → DirectorOverride) with token buckets and aging boosts build directly from `liminal.config.yaml` defaults.
- **Territory Manager** – Lease acquisition, deferral, overrides, and escalations, with metrics surfaced for active leases and pending queues.
- **PTY Bridge** – Spawn demo PTY agents and capture structured `FORGE_EVENT` outputs into the metrics timeline.
- **Metrics Dashboard** – Real-time snapshot plus `health_alert` streaming that flags queue depth, rate-limit, and escalation breaches from the config file.
- **Synthetic Load** – Built-in commands to stress router quota and lease contention so testers can validate KPIs quickly.

## Getting Started

```bash
cd liminal-v1
npm install
npm run tauri dev
```

The Tauri shell exposes controls for:
- `Start Scenario` – scripted lease + message exchange.
- `Start PTY Scenario` – launches PTY demo agents.
- `Simulate Router Load` – floods the router with mixed priority traffic.
- `Simulate Lease Contention` – queues multiple agents on a shared resource.

The metrics dashboard auto-streams updates (`metrics_snapshot`) and logs any `health_alert` events. Devtools (`Cmd+Option+I` on macOS) display the raw event payloads if deeper inspection is needed.

## Regression Scenario

Run the workflow in `runbooks/tasks/router_foundation_regression.md` after major changes to confirm:
- Token quotas throttle mixed-priority traffic.
- Lease contention produces deferrals, overrides, and escalations.
- PTY structured events appear in the metrics UI without destabilising the stream.

## Testing

Backend tests live under `liminal-v1/src-tauri/tests/` and can be exercised with:

```bash
cd liminal-v1/src-tauri
cargo test
```

Frontend linting is not yet configured; `npm run lint` prints a placeholder reminder until ESLint rules are introduced.
