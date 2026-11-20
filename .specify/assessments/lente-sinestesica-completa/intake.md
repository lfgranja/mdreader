# Idea Intake: Implementar a proposta COMPLETA de design SONA

- **Slug**: lente-sinestesica-completa
- **Created**: 2025-10-27
- **Source**: `DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png` (image, 768x1376)
- **Type**: new-capability

## Idea (as captured)

"[Image #1, 768x1376] Implementar agora a proposta COMPLETA de design SONA, com TODOS os elementos de UI/UX propostos"

The proposal image (`DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png`) depicts the full SONA Lente Sinestésica design: dark UI with decorative wave graphics, "Concha Harmônica R" logo, title card (e.g. "Rust Async Architecture / Chapter 3 · Pinning & Futures"), wave visualization, and bottom control bar with transport controls and "1.25×" rate indicator. The user wants this complete proposal implemented (all UI/UX elements).

Comparison (by inspection of image + current device screenshot `device_final.png` + feature 005 scope):
- Implemented (feature 005, reduced Lente): dark `#0B0F1A` background, header logo, "Seção: —" with glyph, transport controls, progress indicator, static failure banner; no decorative waves, no title card, no full visualization, positional-only rate.
- Proposal (complete): adds decorative waves, title/section card with content, wave visualization, full control layout/styling, expanded UI elements.

## Restated

Implement the complete SONA Lente Sinestésica design from the proposal (`ui-proposta-2-lente-sinestesica.png`), replacing/expanding the current reduced token-restyle (feature 005) with all visual/UX elements shown in the full mockup.

## Origin & Context

- **Raised by**: [NEEDS CLARIFICATION]
- **Trigger**: Comparison of device screenshot with proposal image during review; user request to implement the complete proposal.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: Is this replacing feature 005's reduced Lente or extending it?]
- [NEEDS CLARIFICATION: Which elements are in scope (waves/visualization/title card/controls/layout)?]
- [NEEDS CLARIFICATION: Does it require new data sources (document title/section, audio visualization)?]
- [NEEDS CLARIFICATION: Should it change to a 2-column/expanded layout or stay single-column?]
- [NEEDS CLARIFICATION: Does it affect notification surface or only in-app StatusView?]
- [NEEDS CLARIFICATION: Are there performance constraints (visuals/animations)?]
- [NEEDS CLARIFICATION: Should it be behind a flag or replace current UI?]
- [NEEDS CLARIFICATION: What is the acceptance criteria / device trials for the complete design?]
- [NEEDS CLARIFICATION: Does it conflict with feature 005 contracts (tokens closed set, S1 surfaces, etc.)?]

## Artifacts referenced
- Proposal image: `DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png`
- Current device screenshot: `/tmp/device_final.png`
- Current implementation: `src-tauri/src/status_view.html`, `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt`
- Prior feature: `specs/005-adopt-design-sona/` (reduced Lente scope)
