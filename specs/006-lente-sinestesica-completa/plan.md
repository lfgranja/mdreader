# Implementation Plan: Lente Sinestésica Completa

**Branch**: `006-lente-sinestesica-completa` | **Date**: 2025-10-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-lente-sinestesica-completa/spec.md`

## Summary

Expand the reduced Lente Sinestésica (feature 005) to the complete design shown in the proposal (`DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png`): add decorative wave graphics, title/section card using document metadata (with filename fallback), wave visualization, and refined control layout on the single StatusView surface. Respect closed token set where possible, reduced-motion (S5), static assets preferred with optional subtle CSS animations guarded by reduced-motion, no RMS instrumentation, no new surfaces, notification remains accent-only.

## Technical Context

**Language/Version**: Rust 1.85+ (Edition 2024), Kotlin (Android), HTML/CSS/JS (StatusView)  
**Primary Dependencies**: Tauri 2.12.1, cpal 0.18.2, redb, blake3, thiserror, tracing, clap  
**Storage**: redb (bookmarks/settings), app-private storage (diag JSONL)  
**Testing**: cargo nextest, unit/integration tests, surface-audit checks  
**Target Platform**: Android (API 29+), Linux dev  
**Project Type**: Cargo workspace modular monolith + Tauri app  
**Performance Goals**: Eyes-free use, minimal animations, respects reduced-motion  
**Constraints**: Closed sets (tokens/variants), no `_ =>`, notification accent-only, single StatusView surface, static assets preferred, CSS animations only with reduced-motion guard (≤1 continuous loop), no RMS/reactive strip  
**Scale/Scope**: StatusView restyle/expansion only; contracts may need coordinated updates

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **§I–XI**: Follow all constraints (Rust rigor, closed sets, supply chain, Android manifest rules, audio handoff, notification rules, diag rules)
- **§VII Coordinated edit**: Any contract/token changes require coordinated updates
- **S1**: Exactly 2 surfaces (notification + StatusView)
- **S3**: Closed token set (prefer reuse; additions need justification)
- **S5**: Movement ≤1 continuous loop, reduced-motion guard, yellow pulse ≤3s, no pulse on failure, banner ≤200ms
- **S6**: Notification accent-only
- **FR-008 equivalent**: Keep single surface, no new screens

**Gate Status**: PASS (with constraints per assess decision)

## Project Structure

### Documentation (this feature)
```
specs/006-lente-sinestesica-completa/
├── plan.md              # This file
├── spec.md              # Feature specification
└── checklists/
    └── requirements.md  # Quality checklist
```

### Source Code (relevant)
```
src-tauri/src/
├── status_view.html     # StatusView UI (primary)
├── status_view.rs       # StatusView backend
└── ...

android-shell/src/main/kotlin/app/mdreader/
└── MediaMirror.kt       # Notification (must remain accent-only)

specs/005-adopt-design-sona/contracts/
├── status-surface.md
├── surface-audit.md
└── earcon-mapping.md
```

**Structure Decision**: Modify existing StatusView (`status_view.html`) and related assets/styles; update contracts if needed; no new surfaces.

## Phase 0: Outline & Research

### Unknowns to resolve
- Title card data source: use `metadata_title.unwrap_or(filename)` as specified; confirm availability in UI context
- Decorative waves: implement as static SVG or CSS with reduced-motion guard (clarified: allow subtle CSS animations with guard)
- Token additions: prefer reuse of existing closed set; only add if unavoidable with justification
- Layout adaptation: fit within single-column without breaking existing controls

### Research tasks
1. Review proposal image elements and map to existing tokens/stamps
2. Check how `metadata_title`/filename flow to StatusView
3. Review S5 constraints for any animations
4. Assess contract impact (surface-audit, status-surface)

**Output**: research.md (if needed) — can be minimal given assess research exists

## Phase 1: Design & Contracts

### Data Model
- **TitleCard**: { title: String (metadata or filename), hasTitle: bool, ariaLabel: Option<String> } — driven by upstream authority
- **LenteCompleta**: visual state flags (waves enabled, animations guarded, reduced-motion)

### Contracts
- May need updates to `contracts/status-surface.md` (S1–S7) and `contracts/surface-audit.md` (A1–A4) if layout/tokens change — coordinated edit per §VII
- Notification contract unchanged (S6)

### Quickstart
- Lint / deny / format gates (constitution §I / repo-rules): `cargo fmt --all -- --check`; `cargo deny check advisories licenses bans sources`; `cargo clippy --workspace -- -D warnings`; `cargo nextest run --workspace`.
- Test: `cargo nextest run --workspace` (or relevant tests)
- Android build: `cargo tauri android build --target aarch64` (if needed)
- Device verify: install APK and check visual against proposal intent

## Implementation Strategy

- Header asset: `DESIGN-SONA/assets/headers/concha-harmonica-r.svg` (named for SC-001 verification; not decorative wave `waves-top.svg`).
- Phase execution clarification: milestone chart shows sequential gates (1→2→3→4→5→6); actual execution permits US2 / US3 to start in parallel after Phase 2 gate completes (no cross-story file overlap except contracts/data-model already done in Phase 2).
2. **Title card**: Add title card element using existing metadata/filename; empty state `—` with `aria-label="Sem título"`
3. **Controls/layout**: Refine control bar to match proposal style while keeping functionality
4. **Styling**: Use existing tokens; avoid jade/celeste; rate value-blind
5. **A11y**: Preserve progressbar aria, no alert misuse, reduced-motion support
6. **Contracts**: Update if necessary with justification
7. **Tests/verification**: Run surface-audit checks, verify no regressions

## Risks & Mitigations

- **Token creep**: Prefer reuse; if adding tokens, coordinate contract updates
- **Animation overuse**: Enforce reduced-motion guard, ≤1 continuous loop
- **Layout breakage**: Test with different content lengths, keep single-column
- **Notification regression**: Explicit checks (SC-003)

## Complexity Tracking
No constitution violations anticipated if constraints followed.
