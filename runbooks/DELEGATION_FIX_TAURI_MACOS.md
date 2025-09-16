# Task: Fix LIMINAL V1 MVP macOS Runtime Issue

## Quick Context
The LIMINAL V1 MVP is complete and working, but crashes on macOS due to a Tauri v1 Objective-C bridge issue. We need to fix this so the application can run and demonstrate its multi-agent coordination capabilities.

## Your Mission
Fix the macOS runtime panic in the LIMINAL V1 MVP application so it can launch and execute the Phase 2 scenario demonstration.

## Technical Details
See the detailed technical runbook at: `runbooks/tasks/fix_macos_runtime_issue.md`

## Current State
- ✅ All functionality implemented (Phase 1 & 2 complete)
- ✅ Code compiles without errors
- ❌ Runtime crash on macOS (panic in Objective-C exception handler)

## Recommended Approach
Upgrade from Tauri v1 to Tauri v2, which has resolved these macOS compatibility issues.

## Success Criteria
When you're done, running `npm run tauri dev` in the `liminal-v1/` directory should:
1. Open the LIMINAL V1 MVP window without crashing
2. Show "Agent A: Idle" and "Agent B: Idle" status
3. Execute the scenario when "Start Scenario" button is clicked
4. Display real-time status updates and messages in the UI

## Deliverable
Create a pull request with your fix that includes:
- Updated dependencies (Tauri v2 if that approach is taken)
- Any necessary code changes
- Confirmation that the application runs successfully on macOS

The application demonstrates a working prototype of coordinated multi-agent interaction through message routing and territory management - we just need it to actually run!