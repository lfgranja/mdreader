# Bug Fix: Title box file selection fails repeatedly after previous fix

- **Slug**: title-box-file-selection-2
- **Fixed**: 2026-11-07
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

The file picker failed to appear when clicking the title box because a JavaScript runtime error aborted the module script before the `titleCard` click listener was attached. The root cause was `document.getElementById('btnPickDirectory')` returning `null` (no HTML element with that ID exists), and calling `.addEventListener()` on `null` threw `TypeError: Cannot read properties of null (reading 'addEventListener')`. This uncaught error prevented the rest of the script from executing, including the `titleCard` click listener at line 1314.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/status_view.html` | modified | Added null guard for `btnPickDirectory` before calling `addEventListener`, matching existing pattern for `speedPill` |
| `src-tauri/src/assets/index.html` | modified (prior fix) | Mirror of status_view.html changes from prior iterations |

## Diff Highlights

```javascript
// Before (crashed):
document.getElementById('btnPickDirectory').addEventListener('click', () => {
  sendAction('pick_directory');
});

// After (guarded):
const btnPickDirectory = document.getElementById('btnPickDirectory');
if (btnPickDirectory) {
  btnPickDirectory.addEventListener('click', () => {
    sendAction('pick_directory');
  });
}
```

## Tests Added or Updated

- No new tests added; this was a JavaScript runtime error in the served HTML, not covered by Rust unit tests.
- Existing 122 Rust unit tests in `src-tauri` remain passing.

## Local Verification

- Commands run: `cargo test --lib -p src-tauri` → 122 passed, 0 failed
- Commands run: `cargo fmt --all -- --check` → clean
- Commands run: `cargo clippy --workspace --all-targets -- -D warnings` → clean
- Manual checks: Device tap on title card at (539, 538) → SAF picker appeared (`DocumentsUI: True`), no FATAL/ANR in logcat
- Chrome DevTools Protocol: confirmed `triggerDocumentPicker` is now `function` type, click listener attached to `titleCard`, no JS errors in console

## Deviations from Assessment

The assessment hypothesized a stale APK or `gen/` regeneration as the root cause. The actual root cause was a JavaScript runtime null dereference on `btnPickDirectory` that aborted the module script before the `titleCard` click listener was wired. The assessment's proposed remediation (rebuild/reinstall) was partially correct in that a rebuild was needed, but the underlying code defect required fixing first.

Additional prior fixes in this bug series:
1. `bb271cf1`: Wired title-box click handler, propagated picker errors, initialized JVM on IPC path
2. `2ca2c8af`: Surfaced picker failures and hardened debounce
3. `703066b7`: Fixed shell_jni.rs type mismatches preventing Android compilation
4. This fix: Added null guard for `btnPickDirectory` preventing script abort

## Follow-ups

- Keep `src/status_view.html` and `src-tauri/src/assets/index.html` in sync for future changes
- Consider removing the dead `btnPickDirectory` listener code entirely since no such element exists in the HTML
- Consider adding a null guard pattern for all `getElementById` calls in the status view
- Verify `dist/` build output is regenerated from updated `src/` HTML before release

