# Router Foundation Regression Scenario

This runbook validates the current LIMINAL router, territory manager, PTY pipeline, and metrics surface. It combines mixed-priority traffic, lease contention, and PTY structured events while confirming health alerts driven by `liminal.config.yaml`.

## Prerequisites

- Dependencies installed (`cd liminal-v1 && npm install` already run).
- Local dev build ready: `npm run tauri dev` (from `liminal-v1/`).
- Metrics dashboard open in the desktop shell (the LIMINAL MVP window).
- Optional: open the Tauri devtools console (`Cmd+Option+I` on macOS) to watch `health_alert` payloads.

## Scenario Overview

1. **Mixed-Priority Load vs. Token Buckets** — exercise the router by injecting traffic of varying priorities until token quotas are hit.
2. **Lease Contention & Escalation** — contending agents drive deferrals and escalations, validating lease metrics.
3. **PTY Structured Events** — spawn PTY-backed agents and ensure their structured event stream populates metrics and alerts.

Each section below lists the exact commands/buttons to use and the expected telemetry you should confirm in the metrics dashboard and logs.

---

## 1. Mixed-Priority Traffic & Token Limits

1. In the LIMINAL window, click **“Simulate Router Load”**.
2. Observe the **Router** panel:
   - `Queue Depth` table shows non-zero entries (typically `Coordinate`, `Blocking`, and `Critical`).
   - `Rate-limited messages` counter increments.
3. Inspect the **Token Buckets** table:
   - Rows for `synthetic_sender_*` appear with decreasing `Tokens` and growing `Hits`.
   - `Last Refill` timestamps update every second.
4. Watch the console (devtools or terminal running Tauri):
   - `health_alert` entries with severity `warning` appear when queue depth or rate-limit thresholds from `liminal.config.yaml` are exceeded.
5. Confirm the dashboard’s status ribbon chips show **Streaming updates** and that the message log records `Triggered synthetic router load`.

**Expected alerts**
- `warning` or `critical` `health_alert` events referencing `queueDepth` and `ratePerMinute` context fields.

---

## 2. Lease Contention & Escalation Flow

1. Click **“Simulate Lease Contention”**.
2. In the **Leases** card verify:
   - `Active leases` rises (at least 1).
   - `Total pending` climbs to `≥ 2`.
   - `Pending by resource` lists the synthetic resource ID with queue depth.
   - `Escalations` counter increments when the policy threshold is hit.
3. Observe the log:
   - New lines such as `Triggered synthetic lease contention`.
   - Follow-up `health_alert` entries with messages about escalations or deadlock frequency if thresholds are breached.
4. Within ~5 seconds, the queue drains as synthetic agents release the lease. Confirm that `Pending` counts fall back toward zero.

**Optional deeper check**
- Re-run the command and monitor the console for `TerritoryEvent::Escalated` traces emitted by the backend (visible in the Tauri process logs).

---

## 3. PTY Structured Event Capture

1. Click **“Start PTY Scenario”** to spawn two PTY-backed demo agents.
2. In the **PTY Events** card verify:
   - `Total events` increments beyond zero.
   - `Events by name` lists `unknown` or event names parsed from the PTY stream.
   - `Last event` shows the latest agent ID and timestamp.
3. The **Message Log** should contain entries from the PTY agents and the status panel reports the PTY lifecycle.
4. Because PTY events feed the metrics stream, confirm no unexpected `health_alert` messages appear (there should be none for PTY noise unless thresholds are already breached).

---

## Wrap-Up Checklist

- [ ] Router metrics reflect rate-limited traffic and priority distribution.
- [ ] Token bucket table shows depleted tokens with refills over time.
- [ ] Lease panel reports deferrals and escalations, then recovers once contention clears.
- [ ] PTY event metrics register structured outputs from spawned agents.
- [ ] `health_alert` stream surfaces queue/rate-limit warnings during load.
- [ ] All synthetic load buttons can be triggered repeatedly without crashing the Tauri shell.

If any expectation fails, capture the console output and current `metrics_snapshot` (via `get_metrics_snapshot` in the devtools console) before filing an issue in the LIMINAL tracker.
