# Feature Specification: Directory UI Controls

**Feature Branch**: `feature/023-directory-ui-controls`
**Created**: 2026-11-18
**Status**: Draft (assess verdict `go`)
**Input**: Open follow-ups T229/T230/T234 of `specs/010-directory-reading/tasks.md` + 80/20 decision in STATE.md.

## Problem / Evidence Summary

The bridge exposes and tests `pick_directory`, `next_chapter` and `previous_chapter` (`src-tauri/src/lib.rs:4495-4497,7491-7502`), but the canonical status view has only an optional JS handler for a button that does not exist (`status_view.html:1244-1247`), no `directory-selected` listener, and no chapter navigation controls. Five tracked HTML copies are byte-identical today.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Open a directory (P1)

As a reader, when I tap the directory button, the system opens the SAF tree picker via the existing `pick_directory` action.

**Independent Test**: Structural test asserts `#btnPickDirectory` markup exists with an accessible label and both click and keydown wiring to `sendAction('pick_directory')`.

### User Story 2 — Chapter metadata arrives (P1)

As a reader, when a directory is selected, the surface shows the chapter title/count without a manual refresh.

**Independent Test**: Structural test asserts a `window.__TAURI__.event.listen('tauri://channel/directory-selected', ...)` handler that calls `refreshStatusView()`.

### User Story 3 — Switch chapters in-app (P1)

As a reader in a directory, when I tap previous/next chapter, the existing bridge actions run; in a single document the controls are absent.

**Independent Test**: Structural test asserts `#btnPrevChapter`/`#btnNextChapter` markup, actions, and visibility toggle driven by `chapterIndex`/`chapterCount`.

## Functional Requirements

- **FR-DUI-001**: The canonical view MUST contain a `#btnPickDirectory` button with click and keyboard activation dispatching `pick_directory`.
- **FR-DUI-002**: The view MUST listen to `tauri://channel/directory-selected` and refresh chapter title/count state.
- **FR-DUI-003**: The view MUST contain `#btnPrevChapter`/`#btnNextChapter` buttons dispatching `previous_chapter`/`next_chapter`, visible only when chapter metadata is present.
- **FR-DUI-004**: Chapter visibility MUST use the `hidden` attribute, not a literal `display:none` corpse.
- **FR-DUI-005**: All five tracked HTML bundles MUST remain byte-identical after the change.
- **FR-DUI-006**: No Rust playback authority, design token, pinned aria label, `data-elements="10"` or `role="status"` count may change.
- **FR-DUI-007**: Structural host tests MUST cover FR-DUI-001..004 before implementation (TDD RED).

## Key Entities

- `chapterIndex` / `chapterCount` from `get_status_view` (already published).
- Bridge actions `pick_directory`, `previous_chapter`, `next_chapter` (already implemented).
- `directory-selected` event payload from `shell_jni.rs:1456`.

## Edge Cases

- Non-directory reading: chapter metadata null → controls hidden.
- Native button keyboard activation (Enter/Space) must not double-fire — `preventDefault` on keydown.
- Host/test environment without `window.__TAURI__` — listener registration must be guarded.

## Success Criteria

- **SC-DUI-001**: Structural tests (new target) pass and host gates 1–8 remain green.
- **SC-DUI-002**: Five bundles byte-identical (sha256).
- **SC-DUI-003**: Spec 010 follow-ups T229/T230/T234 marked done; T232/T233 unchanged.

## Assumptions

- No device required; host structural checks are authoritative for this UI wiring change; device visual verification is [NV].

## Out of Scope

- T232/T233; new bridge actions; Rust authority changes; new design tokens; new status roles.
