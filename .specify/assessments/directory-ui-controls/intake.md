# Idea Intake: Directory UI Controls

- **Slug**: directory-ui-controls
- **Created**: 2026-11-18
- **Source**: Repo path `specs/010-directory-reading/tasks.md` (open follow-ups T229/T230/T234) + 80/20 decision in `.specify/memory/STATE.md`
- **Type**: improvement

## Idea (as captured)

> "- [ ] T229 Add directory picker button (btnPickDirectory) to status_view.html and wire click/keydown handlers (FR-001, FR-010). The selected canonical view has only an optional JS handler; no button markup is present." — `specs/010-directory-reading/tasks.md:140`
> "- [ ] T230 Add directory-selected event listener in status_view.html to update chapter title and counter (FR-010). The selected canonical view has no such listener." — `:141`
> "- [ ] T234 Add chapter navigation controls to status_view.html (next/prev chapter buttons) (FR-011). Bridge actions exist; the selected canonical view has no chapter navigation controls." — `:145`

## Restated

Expose the already-implemented directory and chapter-bridge actions in the in-app status view: a dedicated directory picker button, a `directory-selected` event listener that refreshes chapter title/count, and previous/next chapter buttons shown only in directory mode.

## Origin & Context

- **Raised by**: Block 5 80/20 decision, 2026-11-18 (follow-ups of spec 010).
- **Trigger**: `origin/dev:src-tauri/src/lib.rs:4495-4497,7491-7502` exposes and tests the actions, but the canonical view has no reachable controls.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: how to hide chapter controls without tripping C-H4 `display:none` corpse check — resolved in research: use `hidden` attribute toggled in JS]
