# Release (R7): 006 Lente Sinestésica Completa

- **Date**: 2026-11-20 (artifact backfilled from `specs/006-lente-sinestesica-completa/`)
- **SemVer**: v0.5.0 (minor feature slice folded into the v0.1.0-rc release line; no standalone app bump)
- **Validation**: R4 PASS for documentary checks (`.specify/validations/lente-sinestesica-completa/validate.md`); device trial [NV]

## Summary

Expands the reduced Lente Sinestésica (spec 005) to the complete proposal on the single StatusView: decorative wave pseudo-element, metadata-driven title card with filename/empty-state fallback, refined controls, guarded subtle CSS animation and strict reduced-motion freeze — while keeping token census, rate-blindness, notification purity and surface count = 2.

## Changelog (IDs)

- **Added**: FR-001 (complete visuals on single StatusView); FR-002 (title card `metadata_title.unwrap_or(filename)`, empty glyph `—` + `.sr-only "Sem título"`); FR-003 (rate stays value-blind, positional only); FR-004 (reduced-motion guard, ≤1 continuous loop S5); FR-005 (static SVG/CSS preferred, animations only guarded, no scripts/`foreignObject`); FR-006 (notification accent-only); FR-007 (contracts updated for token/layout changes, surface count = 2).
- **Added**: SC-001..006 — objective presence/absence + C-H1 census (jade/celeste banned), C-H3 rate-blind + notification purity, no a11y regression, reduced-motion respected, C-H4/C-H5 inventory valid, device trial of objective criteria + freeze + title card.
- **Added**: coordinated §VII edits — `contracts/status-surface.md`, `contracts/surface-audit.md`, `data-model.md` §RessoaToken / `TitleCard` / `LenteCompleta`, with justification records; RED tests `contract_census.rs`, `reduced_motion_guard.rs`, `surface_inventory.rs`.
- **Changed**: `.card::before` decorative wave (aria-hidden), `#documentTitle`/`.subtitle` title card, `.controls`/`.speed-dots` layout, five byte-identical HTML bundles.
- **Removed / excluded**: numeric rate `1.25×`, extra interactive controls, green/violet wave paints outside the token set (proposal elements explicitly unreconciled); no new surfaces; no RMS instrumentation.
- **Bugs**: none open; C-H4 dead-`display:none` corpses pre-checked and kept absent (T004/T009).
- **Tasks**: T001..T055 (55/55 checked).

## Rollback

Revert the commit(s) restoring the reduced 005 surface; restore the previous contracts and `data-model.md` §RessoaToken from git, and the five HTML bundles to their prior byte-identical state. No remote or signed artifact was published; no device configuration changed.

## Residual risk / [NV]

- Host gates 1–8, census/dead-element/aria test re-run and bundle parity not executed at backfill — gate owner runs them.
- SC-006 device visual/reduced-motion trial [NV] — environment-dependent.
- Decorative CSS failure degrades to `animation:none` with layout preserved (clarification Q9); this failure mode is specified but not dynamically verified at backfill.
