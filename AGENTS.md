# Repository Guidelines

## Project Structure & Module Organization
- `liminal-v1/`: App source (Vite + React + Tauri v2). Frontend in `src/`; desktop backend in `src-tauri/`.
- `docs-canonical/`: System docs (concepts, reference, tutorials).
- `config/`: Runtime config (e.g., `liminal.config.yaml`).
- `runbooks/`: Operational notes and issue playbooks.
- Tests: Rust integration tests in `liminal-v1/src-tauri/tests/` (e.g., `integration_test.rs`).
- Assets: `liminal-v1/src-tauri/icons/` and `liminal-v1/app-icon.png`.

## Build, Test, and Development Commands
- Install deps: `cd liminal-v1 && npm i`.
- Web dev: `npm run dev` (Vite server).
- Desktop dev: `npm run tauri dev` (launch app with Rust backend).
- Build: `npm run build` (web bundle to `liminal-v1/dist/`), `npm run tauri build` (desktop binaries).
- Rust: `cd liminal-v1/src-tauri && cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`. Optional: `cargo watch -x check`.
- Lint: If ESLint is configured, run `npm run lint`; otherwise match existing style.

## Coding Style & Naming Conventions
- No code comments unless explicitly requested. Prefer editing existing files over adding new ones.
- TypeScript/React: 2‑space indent; strict TS. Components `PascalCase`, hooks `use*`, files in `liminal-v1/src/`.
- Rust: `snake_case` for funcs/modules, `CamelCase` types; keep modules cohesive. Always run `cargo fmt`; use `cargo clippy` where available.

## Testing Guidelines
- Rust: place integration tests in `src-tauri/tests/` (`*_test.rs`); run `cargo test`.
- Frontend: no runner configured; if adding tests, use Vitest and place in `liminal-v1/src/__tests__/` with `*.test.tsx`.
- Target core flows (router, territory, agent interactions) and UI state transitions.

## Architecture & Invariants
- Router: all agent comms via central router only; no direct agent-to-agent. Prioritized queue; critical messages require fast ack.
- Agent lifecycle: spawn PTY with limits, register with router (timeout), lease territory, process via router, release on shutdown; force-kill on hang.
- Tauri bridge: commands for RPC, events for updates; keep TS types in lockstep with Rust structs (prefer codegen if introduced); use serde `rename_all = "camelCase"` for parity.
- Performance targets: routing <10ms; spawn <500ms; UI 60fps; state sync <50ms; memory/agent <50MB.
- Security boundaries: PTY sandbox; lease system; message validation; sanitize output surfaced in UI.

## Commit & Pull Request Guidelines
- Commit frequently; keep each commit focused on a single logical change.
- Use format `component: action` with optional scope tags (e.g., `src-tauri:`, `router:`).
- Always author commits as WAR-0 and omit AI/specialist attribution. Configure once per repo:
  ```bash
  git config user.name "WAR-0"
  git config user.email "war0@liminal.dev"
  git config commit.gpgsign false
  ```
- Draft clear messages explaining intent so history stays readable when multiple agents contribute.
- PRs include purpose/summary, linked issues, run/test steps, screenshots (UI), and capability/config notes (`tauri.conf.json`, `src-tauri/capabilities/*.json`).

## LIMINAL Workflow Overview
The project operates in repeating Epochs coordinated between the Human Director and the Director Agent. Every Epoch follows the eight-step LIMINAL Workflow, which keeps planning, execution, and review tightly coupled:

1. **Goal & Plan** – Human Director and Director Agent agree on the Epoch objective.
2. **Runbook Creation** – Director Agent drafts a Markdown runbook containing the ordered Turns required to hit the goal.
3. **Delegation (Start of Turn)** – Human Director copies the next Turn prompt from the runbook and assigns it to a specialist agent.
4. **Execution** – The specialist agent completes the assigned task.
5. **Response (End of Turn)** – Specialist returns outputs plus a concise summary.
6. **Loop** – Human Director repeats steps 3–5 until the runbook is finished, in sequence or in parallel as the runbook specifies.
7. **Summary & Review** – Human Director reports results back to the Director Agent.
8. **Repeat** – A new Epoch begins with fresh goals and planning.

When drafting instructions, writing runbooks, or coordinating agents, keep this workflow in mind so prompts and artifacts align naturally with the process.
