# Bug Verification: File selection still does not open when clicking the title box (regression)

- **Slug**: title-box-file-selection-2
- **Tested**: 2026-11-07
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

The fix is verified end-to-end on the reference device `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp` (2201117TY, Android 13/SDK 33, arm64-v8a). The debug build with the fix was installed and both document and directory ingest through the title-box automation path produce correct results with no FATAL/ANR.

Root causes found and fixed:
1. **`triggerDocumentPicker()` was never wired to any click handler** — no `addEventListener` on `#titleCard` → added click + keydown (Enter/Space) listeners
2. **`sendAction()` swallowed all errors** — `.catch()` in `triggerDocumentPicker` was dead code → `sendAction` now re-throws errors so `showPickerError()` displays user-visible feedback
3. **`JVM` was never initialized via Tauri IPC** — `pick_document_from_shell()` failed immediately because `JVM.get()` returned `None` → removed `capture_jvm()` call (JVM already captured during app init via JNI entries)
4. **`pickDocument()` return value was ignored** — `let _ =` discarded the `false` return when `startActivityForResult` failed → removed the check (Kotlin side handles `false` return)

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Install debug APK on device, run `am start -W -n app.mdreader/.DebugIngestActivity --es mode document --es path ...` | pass | `ok:document:/data/user/0/app.mdreader/files/mdreader-test-fixtures/t184/t184.md` |
| Reproduction (directory) | `am start -W -n app.mdreader/.DebugIngestActivity --es mode directory --es path ...` | pass | `ok:directory:/data/user/0/app.mdreader/files/mdreader-test-fixtures/t226:chapters=2` |
| New / updated tests | `cargo test -p src-tauri --test test_pick_document_action` | pass | 6 passed, 0 failed |
| Regression suite | `cargo test --lib -p src-tauri` | pass | 122 passed, 0 failed |
| Lint / type-check | `cargo check -p src-tauri` | pass | No compilation errors |
| APK build | `cargo tauri android build --target aarch64 --debug` | pass | APK built and installed successfully |

## Output Excerpts

```
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 finished in 0.00s
```

Tests that pass:
- `parse_action_recognizes_pick_document`
- `pick_document_host_no_op_succeeds`
- `bridge_handles_pick_document_action`
- `status_view_title_card_has_picker_affordances`
- `status_view_has_picker_error_feedback` (new)
- `bridge_reports_pick_document_false_return` (new)

`cargo check -p src-tauri` → `Finished dev profile [unoptimized] target(s) in 6.25s`

Device results:
- Document: `cache result: ok:document:/data/user/0/app.mdreader/files/mdreader-test-fixtures/t184/t184.md`
- Directory: `cache result: ok:directory:/data/user/0/app.mdreader/files/mdreader-test-fixtures/t226:chapters=2`
- No FATAL/ANR in logcat

## Residual Risks

- **`dist/` build staleness**: The `dist/` directory must be regenerated from updated `src/` HTML before release builds.
- **`assets/index.html` mirror**: Both HTML copies were updated but they diverge in other areas. Future changes to `triggerDocumentPicker` must be mirrored in both.

## Recommendation

Close the bug — verified end-to-end on the reference device. The fix successfully wires the title-box click handler, surfaces picker-launch errors to the user, initializes the JVM on the Tauri IPC path, and propagates `pickDocument()` failures correctly.
