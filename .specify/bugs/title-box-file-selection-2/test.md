# Bug Verification: Title box file selection fails repeatedly after previous fix

- **Slug**: title-box-file-selection-2
- **Tested**: 2026-11-07
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

The bug is resolved. The JavaScript runtime error that aborted the module script before the `titleCard` click listener was attached has been fixed. Device verification confirmed the SAF picker now opens when tapping the title box. All 122 Rust unit tests pass, and lint/format checks are clean.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Device tap on title card at (539, 538) | pass | SAF picker opened (`DocumentsUI: True`), no FATAL/ANR |
| Chrome DevTools Protocol | Eval `triggerDocumentPicker` type | pass | Returns `function` (was `undefined` before fix) |
| Chrome DevTools Protocol | Eval `els` type | pass | Returns `object` (was `undefined` before fix) |
| Chrome DevTools Protocol | Eval titleCard click listener | pass | `addEventListener` now executes (was aborted before) |
| New / updated tests | `cargo test --lib -p src-tauri` | pass | 122 passed, 0 failed |
| Regression suite | `cargo test --lib -p src-tauri` | pass | All tests pass |
| Lint / type-check | `cargo fmt --all -- --check` | pass | Clean |
| Lint / type-check | `cargo clippy --workspace --all-targets -- -D warnings` | pass | Clean |

## Output Excerpts

```
Device tap test:
Title card center: (539, 538)
Tapping title card...
UI AFTER TAP:
DocumentsUI: True
reasonBanner: (not found)

Host tests:
test result: ok. 122 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.37s

Clippy:
Finished `dev` profile [unoptimized] target(s) in 27.61s
```

## Residual Risks

- `src-tauri/src/assets/index.html` is still out of sync with `src-tauri/src/status_view.html` (does not contain `btnPickDirectory` at all). This is a pre-existing condition not introduced by this fix.
- The dead `btnPickDirectory` listener code remains in `status_view.html` guarded by a null check. It could be removed entirely since no such element exists in the HTML.
- No automated (Robolectric) test exists for the `ShellHost.pickDocument()` null-activity path.
- The `dist/` build output should be regenerated from updated `src/` HTML before release.

## Recommendation

Close the bug — verified end-to-end on the reference device. The SAF picker now opens when tapping the title box. All host checks pass. No regressions detected.

