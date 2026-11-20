# Idea Research: Implementar a proposta COMPLETA de design SONA

- **Slug**: lente-sinestesica-completa
- **Created**: 2025-10-27
- **Evidence confidence (overall)**: medium

## Users & Demand

- No direct user feedback captured in intake — [ASSUMPTION] (confidence: low). Triggered by comparison of proposal image with current device screenshot during review; likely internal design preference rather than external user signal.

## Prior Art

- **Feature 005 (adopt-design-sona, Option A)** — Implemented reduced Lente (token restyle + stamps only): `specs/005-adopt-design-sona/spec.md` FR-001..FR-012, `contracts/status-surface.md` S1–S7, `contracts/surface-audit.md`. Clarified reduced scope (FR-008 reworded): no RMS-reactive strip, no layout restructuring, no 2-field grid, no new surfaces. Status: merged/reduced implemented; current device shows reduced UI. — [source: specs/005-adopt-design-sona/spec.md, plan.md, contracts/, device screenshot /tmp/device_final.png] (confidence: high)
- **DESIGN-SONA full design** — Proposal image `DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png` (768x1376) shows full Lente with decorative waves, title card, visualizer, expanded controls. — [source: DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png] (confidence: high)
- **Feature 001 (android-media-shell)** — Audio pipeline, MediaSession, notification; accent-only notification design. — [source: specs/001-android-media-shell/] (confidence: high)

## Market & Context

- Full Lente provides richer visual experience (waves, title card) vs minimal reduced Lente. Cost of doing nothing: current UI is functional but less rich; users may want more visual feedback. — [ASSUMPTION] (confidence: low)
- Reduced Lente was explicitly chosen to avoid RMS instrumentation, new frontend toolchain, navigation to excluded screens — adding full Lente would likely require those capabilities. — [source: specs/005-adopt-design-sona/spec.md Q90, C6, D1] (confidence: high)

## Data & Constraints

- **Surface count**: Must remain exactly 2 surfaces (notification + StatusView) per contracts; Lente completa must still be on same StatusView (restructure not new surface). — [source: contracts/status-surface.md S1, specs/005-adopt-design-sona/spec.md Q96] (confidence: high)
- **Tokens closed set**: `--sona-void`, `--sona-stage`, `--sona-voice`, `--sona-etwahl`, `--sona-hush`, `--sona-harp`, `--sona-celeste`, `--sona-boundary`, `--signal-caution`, `--signal-failure` (10 members); adding decorative elements may need new tokens or violate closed set. — [source: contracts/status-surface.md S3, specs/005-adopt-design-sona/data-model.md] (confidence: high)
- **Layout**: Reduced Lente stays single-column (no 2-field grid); full proposal implies more complex layout. — [source: specs/005-adopt-design-sona/spec.md C6, clarify-c3-two-field-grid.md] (confidence: high)
- **No RMS/reactive**: Full proposal shows wave visualization but current design says no RMS-reactive strip; would need instrumentation or static decoration. — [source: specs/005-adopt-design-sona/spec.md Q90, D1] (confidence: high)
- **Android constraints**: Tauri 2.12.1 pinned; Service/MediaSession owned by Kotlin, no playback logic there. — [source: .specify/memory/constitution.md, AGENTS.md] (confidence: high)

## Evidence Against the Idea

- **Scope creep vs reduced Lente**: Feature 005 explicitly reduced scope (no RMS, no layout change, no toolchain); full Lente is a larger feature with more surface area. — [source: specs/005-adopt-design-sona/spec.md Q90, C6] (confidence: high)
- **New assets/graphics**: Proposal needs decorative wave SVG/graphics, possibly animations beyond pill movement; must follow design constraints (no scripts, no foreignObject in SVG; colors via CSS vars). — [source: DESIGN-SONA/components.md, specs/005-adopt-design-sona/contracts/status-surface.md S5] (confidence: high)
- **Data dependencies**: Title card "Rust Async Architecture" suggests metadata; current uses `document_title` / filename fallback; may need different data. — [source: src-tauri/src/status_view.html line ~389, specs/005-adopt-design-sona/spec.md F3] (confidence: high)
- **Token budget**: Closed token set; adding celeste/aria colors needs coordination (celeste was marked prohibited in operational UI per some clarifications, but used in proposal). — [source: specs/005-adopt-design-sona/spec.md Q82, contracts/status-surface.md S3] (confidence: high)
- **Device trials**: Full Lente would require updated SC/verification; T030 device trials for reduced Lente deferred but structure exists. — [source: specs/005-adopt-design-sona/quickstart.md §3, Phase 9 T030] (confidence: high)

## Gaps & Open Questions

- [NEEDS CLARIFICATION: Is the full Lente meant to replace the reduced Lente entirely, or be added as a separate mode?]
- [NEEDS CLARIFICATION: Are decorative waves static SVGs or animated? If animated, must respect reduced-motion (S5) and ≤1 continuous loop constraint.]
- [NEEDS CLARIFICATION: What data provides the title card content? Is it always available?]
- [NEEDS CLARIFICATION: Does the proposal require adding new tokens (e.g., wave colors) or using existing closed set?]
- [NEEDS CLARIFICATION: Should the layout change to match proposal (2-column) or adapt to single-column?]
- [NEEDS CLARIFICATION: Is this in scope for feature 005 or a new feature (006+)?]
- [NEEDS CLARIFICATION: Performance impact of decorative SVG/waves?]
- [NEEDS CLARIFICATION: Need to update contracts (surface-audit A1–A4, status-surface S1–S7) if layout/tokens change?]
- [NEEDS CLARIFICATION: Device trials for full Lente — different criteria than reduced Lente?]

## Sources

- `DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png` (host: local file, policy: local filesystem — not fetched over network; included as design reference)
- `specs/005-adopt-design-sona/spec.md` (reduced Lente scope, FR-008 clarifications)
- `specs/005-adopt-design-sona/contracts/status-surface.md` (S1–S7)
- `specs/005-adopt-design-sona/contracts/surface-audit.md` (A1–A4)
- `src-tauri/src/status_view.html` (current implementation)
- `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (notification)
- `/tmp/device_final.png` (current device screenshot)
- `.specify/memory/constitution.md` (governance)
