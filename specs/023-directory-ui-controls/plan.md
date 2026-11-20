# Implementation Plan: Directory UI Controls

**Branch**: `feature/023-directory-ui-controls` | **Date**: 2026-11-18 | **Spec**: [`spec.md`](./spec.md)

## Summary

Add directory picker, `directory-selected` listener and chapter navigation controls to the canonical status view, wire them to the existing bridge actions, mirror the HTML into all five tracked bundles, and cover the wiring with structural host tests.

## Technical Context

- **Language/Version**: HTML/CSS/ES module (Tauri WebView); Rust only for structural tests
- **Primary Dependencies**: `window.__TAURI__.event.listen`, existing `invoke('run_mobile_plugin', {action})`
- **Testing**: new `src-tauri/tests/directory_ui_controls.rs` (structural), `cargo nextest`, `scripts/verify.sh`
- **Target Platform**: Android WebView + host test
- **Constraints**: Rust authority unchanged; closed design token set; no literal `display:none`; pinned aria labels and counts unchanged; five bundles byte-identical

## Constitution Check

| Gate / Principle | Status | Notes |
|---|---|---|
| Rust sole playback authority | PASS | UI dispatches existing actions only |
| Mirrors-only frontend | PASS | No state computed in JS beyond chapter visibility |
| Closed sets / tokens | PASS | Reuse existing tokens; no new status roles |
| TDD / host gates | PASS | RED structural test first |
| Reversible | PASS | HTML-only + tests |

## Project Structure

```
src-tauri/src/status_view.html                 # canonical view (source of truth)
src-tauri/src/assets/index.html                # mirrors
src-tauri/gen/android/app/src/main/assets/index.html
dist/index.html
src-tauri/dist/index.html
src-tauri/tests/directory_ui_controls.rs       # new structural test target
specs/023-directory-ui-controls/*              # SDD artifacts
specs/010-directory-reading/tasks.md           # T229/T230/T234 marked done
```

## Implementation Order

1. Write RED structural test target (FR-DUI-001..004) and record failing output.
2. Edit canonical view: markup, CSS (reuse tokens), `els` entries, handlers, listener, visibility toggle.
3. Copy canonical view to the other four bundles; verify sha256 equality.
4. Update `specs/010-directory-reading/tasks.md` follow-up status.
5. Run targeted tests, then gates; open PR.

## Rollback

Revert the commit; restore the previous five HTML bundles from git. No remote config change.

## Complexity Tracking

| Risk | Why needed | Rejected alternative |
|---|---|---|
| `hidden` attribute vs C-H4 corpse check | Chapter controls must be absent in single-doc mode | Literal `display:none` (audit failure) |
