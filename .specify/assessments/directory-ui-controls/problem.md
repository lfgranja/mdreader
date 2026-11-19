# Problem Definition: Directory UI Controls

- **Slug**: directory-ui-controls
- **Created**: 2026-11-18
- **Inputs used**: intake.md | research.md

## Problem Statement

Directory-reading users cannot open a folder or switch chapters from the in-app status view: the bridge exposes `pick_directory`, `next_chapter` and `previous_chapter` (with host tests), but the canonical view has no button markup, no `directory-selected` listener and no chapter navigation controls.

## Affected Users & Stakeholders

- **Users**: readers of multi-chapter Markdown directories — need in-app chapter switching.
- **Stakeholders**: maintainer — owns surface contracts/a11y invariants (C-H4, SC-001).

## Goals

- Add a dedicated directory picker button with click and keyboard activation.
- Refresh chapter title/count when the shell publishes `directory-selected`.
- Add previous/next chapter buttons visible only in directory mode.

## Non-Goals

- No playback/transport logic in the frontend; no new bridge actions; no Rust authority changes.
- No new design tokens; no changes to pinned aria labels, `data-elements="10"`, or status-role counts.
- No T232/T233 (pre-synthesis, resume offer dialog) — separate follow-ups.

## Success Metrics

- Structural host tests pass for button markup, event listener and handler wiring.
- All five HTML bundles remain byte-identical.
- Host gates 1–8 and CI green.

## Cost of Inaction

The P1 directory-reading feature remains unusable from the surface for ordinary users.

## Open Questions

- None blocking.
