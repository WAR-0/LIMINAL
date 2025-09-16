# Fix macOS Tauri Runtime Panic Issue

## Problem
The LIMINAL V1 MVP application crashes on macOS with a panic in the Objective-C exception handler when trying to launch. The app compiles successfully but fails at runtime with:

```
thread 'main' panicked at library/core/src/panicking.rs:218:5:
panic in a function that cannot unwind
```

The panic occurs in `objc_exception::try_no_ret::try_objc_execute_closure` when Tauri tries to initialize the application window.

## Current State
- ✅ Phase 1 & 2 complete: Core LIMINAL functionality implemented
- ✅ Code compiles without errors (only dead code warnings)
- ✅ React UI with event listeners ready
- ❌ Runtime crash on macOS due to Objective-C bridge issue

## Solution Options

### Option 1: Upgrade to Tauri v2 (Recommended)
Tauri v2 has better macOS compatibility and fixes many v1 issues with the Objective-C bridge.

**Steps:**
1. Update `liminal-v1/src-tauri/Cargo.toml`:
   - Change `tauri = { version = "1", ...}` to `tauri = { version = "2", ...}`
   - Update `tauri-build` to v2
2. Update `liminal-v1/package.json`:
   - Update `@tauri-apps/cli` to v2
   - Update `@tauri-apps/api` to v2
3. Migrate configuration from v1 to v2 format
4. Test the application

### Option 2: Add macOS-specific workaround
Add exception handling wrapper for macOS in `main.rs`:

```rust
#[cfg(target_os = "macos")]
fn setup_macos_exception_handler() {
    // Add Objective-C exception handling setup
}
```

### Option 3: Disable problematic features
Temporarily disable features that might be causing the issue:
- Remove icon configuration
- Simplify window configuration
- Use basic Tauri setup without custom configurations

## Testing Requirements
After implementing the fix:
1. Run `npm run tauri dev` - should open without crashing
2. Click "Start Scenario" button - should execute the hardcoded scenario
3. Verify agent status updates appear in real-time
4. Verify messages appear in the message log
5. Confirm "Scenario Finished" message appears

## Files to Modify
- `liminal-v1/src-tauri/Cargo.toml` - Tauri version
- `liminal-v1/package.json` - Tauri CLI/API versions
- `liminal-v1/src-tauri/tauri.conf.json` - Configuration updates
- `liminal-v1/src-tauri/src/main.rs` - Possible workaround code

## Success Criteria
The application should:
- Launch without crashing on macOS
- Display the LIMINAL V1 MVP window
- Execute the Phase 2 scenario when button clicked
- Show real-time updates in the UI

## Additional Context
The crash log shows the issue is in the Tauri/Tao event loop initialization on macOS. This is a known issue with Tauri v1 on recent macOS versions. The application works on Linux/Windows, so the code logic is correct - only the macOS runtime bridge has issues.