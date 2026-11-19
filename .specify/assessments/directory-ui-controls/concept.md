# Concept: Directory UI Controls

- **Slug**: directory-ui-controls
- **Created**: 2026-11-18
- **Recommended option**: Option A

## Options

### Option A — Wire existing actions into the canonical view
- **Sketch**: Add `#btnPickDirectory` and a `#chapterNav` group with `#btnPrevChapter`/`#btnNextChapter` to the status view; listen for `tauri://channel/directory-selected` to refresh state; toggle chapter-nav visibility from the existing `chapterIndex`/`chapterCount` fields. Keep the same markup in all five bundles.
- **Appetite**: small
- **Trade-offs**: Wins full in-app directory control with no Rust changes. Sacrifices nothing functional; risk is a11y-count regression, mitigated by structural tests.
- **Rabbit holes**: `display:none` corpses (use `hidden`), double keyboard activation on native buttons (preventDefault in keydown), bundle drift (copy after edit).

### Option B — Add a native Android menu instead
- **Sketch**: Expose controls only through MediaSession/Kotlin UI.
- **Appetite**: medium
- **Trade-offs**: Diverges from the in-app surface contract and needs device validation; rejected.

### Option C — Do nothing
- **Sketch**: Keep bridge-only actions.
- **Appetite**: none
- **Trade-offs**: Zero risk, zero user value; P1 feature remains unreachable.

## Recommendation

Option A. It matches the approved 80/20 choice, is small, host-verifiable and reversible.

## Out of Scope (recommended option)

- T232/T233, new bridge actions, Rust authority changes, new tokens, pinned aria-label changes.

## Assumptions to Validate

- `hidden` attribute satisfies the C-H4 corpse check (verified: check looks only for literal `display:none`).
- `get_status_view` already publishes `chapter_index`/`chapter_count` (verified: `status_view.html:1168-1169`).
