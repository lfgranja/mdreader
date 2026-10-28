# Feature Specification: SONA Complete Design — Real and Exact as Proposed

**Feature Branch**: `007-sona-complete-design`

**Created**: 2026-10-28

**Status**: Draft

**Input**: User description: "DESIGN SONA REAL E COMPLETO, EXATAMENTE IGUAL AO PROPOSTO" (design proposal: `DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png`)

**Constitution**: v2.0.0 (amended 2026-10-28 — design-token expansion + supplementary visual elements permitted)

## Overview

Implement the complete SONA design exactly as shown in the proposal image (`DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png`): all visual/UI elements including decorative wave graphics (teal-to-violet), glassmorphism title card, reactive waveform visualizer, numeric rate display (`1.25×`), extra controls (repeat), and glow/neon effects — on the single StatusView surface, respecting the amended constitution (v2.0.0) which now permits supplementary visual elements and expanded token set.

This feature builds on 006 (Lente Sinestésica Completa) and removes the exclusions that were in place due to the pre-amendment constitution constraints.

## User Scenarios & Testing

### User Story 1 - Ver o design SONA completo conforme a proposta (Priority: P1)

Como usuário em uso (condução/olhar periférico), quero ver o design completo conforme a proposta: cabeçalho com ondas teals-violeta, cartão de título glassmorphism, visualizador de waveform reativo, controles completos (repeat, prev, play, next, speed), efeitos glow/neon, com tokens SONA corretos (incluindo jade e celeste).

**Why this priority**: Entrega visual completa da proposta original, sem exclusões.

**Independent Test**: Open app, verify all proposal elements present: wave ribbons, glassmorphism card, waveform visualizer, numeric rate `1.25×`, repeat button, glow effects.

**Acceptance Scenarios**:

1. **Given** app on StatusView with document loaded, **When** viewed, **Then** header shows Concha Harmônica R with decorative wave ribbons (teal-to-violet), title card is glassmorphism (frosted translucent), waveform visualizer renders with reactive animation.
2. **Given** reduced motion preference, **When** viewed, **Then** decorative animations are paused/frozen per S5; no new continuous loops beyond allowed.
3. **Given** no title/section available, **When** viewed, **Then** glyph `—` with `aria-label="Sem título"` used.

### User Story 2 - Controles completos e waveform reativo (Priority: P1)

Como usuário em movimento, quero controles completos (repeat, prev, play, next, speed com valor numérico) e visualizador de waveform reativo ao áudio, com feedback auditivo/háptico existentes.

**Why this priority**: Funcionalidade visual completa da proposta.

**Independent Test**: Verify all 5 controls present and functional; waveform reacts to playback state.

**Acceptance Scenarios**:

1. **Given** complete visual applied, **When** transport/rate changed, **Then** earcon confirmation works; rate displays numeric value (`1.25×`) AND positional indicator.
2. **Given** warning/failure states, **When** triggered, **Then** banner behavior unchanged (static failure, warning with TTL); notification remains accent-only (SC-003).

### User Story 3 - Tokens expandidos e contratos atualizados (Priority: P1)

Como sistema, deve manter invariante: notificação permanece accent-only, closed sets preservados (token set expandido com justificativa), superfície única.

**Acceptance Scenarios**:

1. **Given** complete UI applied, **When** notification checked, **Then** only `--sona-voice` accent, no body/rate/voice/mark (SC-003).
2. **Given** surface inventory, **When** audited, **Then** no dead `display:none` corpses; accessibility preserved; new elements (waveform, repeat) in inventory.

## Requirements

### Functional Requirements

- **FR-001**: Apply complete SONA design to single StatusView: decorative wave ribbons (teal-to-violet, glassmorphism), title card (frosted translucent), waveform visualizer (reactive), refined controls (repeat, prev, play, next, speed with numeric rate), glow/neon effects — respecting amended constitution (v2.0.0).
- **FR-002**: Title card must use upstream authority `metadata_title.unwrap_or(filename)`; empty state glyph `—` with `aria-label="Sem título"`.
- **FR-003**: Rate displays numeric value (`1.25×`) painted in the speed-control area next to positional dots; value-blind constraint amended to allow painted numeric rate in speed-control area only. `aria-valuetext` retains positional descriptor.
- **FR-004**: Respect reduced-motion: provide reduced-motion guard for all decorative animations; maintain S5 (≤1 continuous loop, or justify additional loops).
- **FR-005**: Static assets preferred (SVG/CSS); CSS animations allowed with reduced-motion guard; no scripts/foreignObject. Glassmorphism may use `rgba()`/`rgb()` alpha variants for translucency, with base colors as verbatim SonaToken members (S3 amended).
- **FR-006**: Notification remains accent-only (no body/rate/voice/mark) — unchanged from 006.
- **FR-007**: Update contracts if tokens/layout change; maintain surface count = 2; expand token set (jade, celeste) with §VII justification.
- **FR-008**: Waveform visualizer must be reactive to playback state via true RMS: AtomicF32 in audio callback (`consume_into`), throttled IPC channel (~20 Hz), canvas-based smooth ribbon visualization (4-6 translucent bezier curves in teal→jade→violet, mirrored around center axis, white progress line + circular playhead). Graceful degradation: static waveform placeholder if RMS unavailable. No allocation on hot path. Cap 30 fps on mobile, pause on visibilitychange.
- **FR-009**: Glassmorphism via `backdrop-filter: blur(24px)` + translucent overlay (verbatim token + opacity property). Fallback to solid `--sona-stage` if unsupported. Reduced-motion guard applies.
- **FR-010**: Glow/neon effects via `box-shadow` on header mark (`0 0 20px var(--sona-etwahl)`) and waveform baseline (`0 0 10px var(--sona-harp)`). Subtle, 2 elements max. Reduced-motion guard applies.
- **FR-011**: Repeat button restarts the current section from the beginning. Reuses existing skip/section logic. Earcon: reuse "skip back" earcon. New interactive element added to inventory (update C-H4).

### Key Entities

- **LenteCompleta**: Visual configuration for complete design (waves, title card, waveform, layout, glassmorphism, glow) on StatusView.
- **TitleCard**: Metadata-driven card with fallback to filename; empty state handling; glassmorphism styling.
- **WaveformVisualizer**: Reactive audio visualization (RMS-based, canvas smooth ribbons, requires engine instrumentation).
- **RepeatControl**: Restarts current section. Reuses skip/section logic. New interactive element (inventory update required).
- **SonaToken (expanded)**: 12+ members (original 10 + jade #5AD6A8 + celeste #B9A6F5 promoted to painted).

## Success Criteria

### Measurable Outcomes

- **SC-001**: All proposal elements present: wave ribbons, glassmorphism card, waveform visualizer, numeric rate `1.25×`, repeat button, glow effects, refined controls.
- **SC-002**: No accessibility regressions (progressbar aria intact, empty slot aria-label, focus ring visible, no alert misuse).
- **SC-003**: Notification purity unchanged (accent-only).
- **SC-004**: Reduced-motion respected; animations frozen under `prefers-reduced-motion: reduce`.
- **SC-005**: Surface inventory valid; no dead `display:none` corpses; aria inventory correct; new elements (waveform, repeat) documented.
- **SC-006**: Device trial verifies all visual elements match proposal.

## Assumptions

- Constitution v2.0.0 (amended 2026-10-28) permits: design-token expansion (jade, celeste), supplementary visual elements (waveform, glassmorphism, glow), extra controls.
- Token set expanded from 10 to 12+ members (jade #5AD6A8, celeste #B9A6F5 promoted to painted).
- Waveform visualizer requires RMS instrumentation in the Rust engine (new capability): AtomicF32 in audio callback, throttled IPC channel (~20 Hz), canvas rendering. Performance: cap 30 fps on mobile, pause on visibilitychange, freeze under reduced-motion.
- Glassmorphism requires `backdrop-filter` support (available in modern webviews).
- Notification remains accent-only (SC-003 unchanged).
- Surface count remains 2 (StatusView + notification) — new elements are within StatusView, not new surfaces.

## Edge Cases

- Decorative wave CSS failure: `animation:none`, layout preserved, no broken-image/alert.
- Long filenames / empty title: `overflow:hidden; text-overflow:ellipsis; white-space:nowrap; flex:min-width:0`; empty-state glyph `—` with `.sr-only` "Sem título".
- Reduced-motion dynamic toggle: freeze all decorative/progress/banner animations at load and after OS-level toggle.
- Waveform failure: if RMS data unavailable, show static waveform placeholder or hide gracefully.
- Glassmorphism fallback: if `backdrop-filter` unsupported, fall back to solid `--sona-stage` background.
## Clarifications

### Session 2026-10-28

- Q: RMS instrumentation strategy for waveform visualizer → A: True RMS — implement AtomicF32 in audio callback + throttled IPC channel (~20 Hz) + canvas/SVG bar visualization. Truly reactive, matches proposal intent. Graceful degradation: static waveform placeholder if RMS unavailable.
- Q: Glassmorphism vs. contract S3 (no rgb()/derived variants) → A: Amend S3 to allow alpha variants specifically for glassmorphism/translucency effects, with the constraint that base colors must be verbatim SonaToken members. Simpler CSS, better visual fidelity.
- Q: Numeric rate display location → A: Paint numeric rate in speed-control area — add a text element next to the dots showing `1.25×`. Amend contracts S4/S10 to allow painted numeric rate in the speed-control area only. Update tests to permit the sanctioned element.
- Q: Wave ribbon implementation technique → A: CSS-animated SVG — use existing `waves-top.svg` as base, add CSS keyframes for subtle flowing animation (translate/opacity, ≤400ms, no infinite loop beyond S5 limit). Guard with `@media(reduce)`. Moderate effort, good fidelity, S3-compliant.
- Q: Glow/neon scope and intensity → A: Glow on logo + waveform baseline only — `box-shadow: 0 0 20px var(--sona-etwahl)` on header mark and `box-shadow: 0 0 10px var(--sona-harp)` on waveform baseline. Subtle, proposal-aligned. 2 elements max.


### Session 2026-10-28 (second pass)

- Q: Waveform visualization style — bars vs. smooth ribbons → A: Smooth ribbons (match proposal) — canvas-based with bezier curves, 4-6 translucent layers in teal/purple, mirrored around center axis, white progress line + circular playhead. Most faithful to proposal. Requires canvas rendering + RMS data. Moderate complexity.
- Q: Repeat button behavior → A: Repeat current section — pressing repeat restarts the current section from the beginning. Uses existing skip/section logic. Earcon: reuse "skip back" earcon or add a new one. Moderate complexity.
