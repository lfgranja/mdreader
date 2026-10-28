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

- **Independent Test**: Open app, verify all proposal visual elements present: wave ribbons (teal→jade→violet), glassmorphism card, glow/neon effects, decorative wave, title card with metadata/filename fallback, empty-state glyph. (Numeric rate display and repeat control are validated under US2.)

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
- **FR-007**: Update contracts if tokens/layout change; maintain surface count = 2 (surfaces) and inventory count = 10 (elements); expand token set (jade, celeste) with §VII justification.
- **FR-008**: Waveform visualizer reactive via true RMS: AtomicF32 in audio callback (`consume_into`), throttled IPC channel (~20 Hz, last-wins coalescing), canvas smooth ribbons (5 layers, cubic-bezier control points at 1/3 & 2/3 width, per-layer opacity 0.15/0.20/0.25/0.30/0.35, mirrored on center axis; color→layer: teal→jade→violet; white 1 px center line at 0.90 opacity + 6 px circular playhead). Canvas `aria-hidden="true"`; progress via parallel sr-only `#progress` progressbar + live region. IPC-event-driven redraw (no rAF loop); 30 fps cap via 33.33 ms gate (`performance.now()`); pause on `visibilityState === 'hidden'` (resume with one fresh frame); freeze (static snapshot) under reduced motion and on dynamic toggle; DPR backing store capped at `min(devicePixelRatio,2)` with fallback to 1; static SVG/CSS placeholder (388×80 CSS px, 30% opacity teal on `--sona-stage`, no animation) if RMS/canvas unavailable. No allocation on hot path (0 heap allocations per redraw, verified via DevTools). Graceful degradation: static placeholder or `#progress` bar.
- **FR-009**: Glassmorphism via `backdrop-filter: blur(24px) saturate(120%)` + translucent overlay `rgba(20,27,45,0.65)` (verbatim `--sona-stage` base at 65% alpha; Doppelrand borders: outer `1px rgba(255,255,255,0.06)`, inner `1px rgba(143,216,232,0.15)`). Fallback to solid `--sona-stage` if unsupported. Reduced-motion guard applies. (S3 alpha-variant rule per FR-005.)
- **FR-010**: Glow/neon via `box-shadow` on header mark (`0 0 20px rgba(143,216,232,0.20)`) and waveform baseline (`0 0 10px rgba(232,196,106,0.20)`). Alpha band 0.15–0.25 (0.20 default); never above 0.30. Subtle, 2 elements max. Reduced-motion guard applies (glow retained static; any pulsing disabled).
- **FR-011**: Repeat button restarts the current section from the beginning. Reuses existing skip/section logic. Earcon: reuse "skip back" earcon. New interactive element added to inventory (update C-H4). Keyboard: native `<button>`, tab order repeat→prev→play→next→speed, Enter/Space activation, no traps; focus ring `--sona-etwahl` ≥2 CSS px.

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
- Token set expanded from 10 to 12+ members (jade #5AD6A8, celeste #B9A6F5 promoted to painted) with sanctioned scopes: jade in waveform-ribbon middle layers + brand-layer assets; celeste in waveform-ribbon violet layers + decorative-wave violet stops. Rate stays `--sona-harp` (gold). Requires coordinated §VII edit to contracts.
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
- Waveform canvas fallback: if canvas/RMS unavailable, hide canvas and show `#progress` bar; never leave empty region.
- Indeterminate progress: no playhead knob drawn; omit `aria-valuenow`; announce "carregando" once via live region.
- Canvas redraw is IPC-event-driven (no rAF loop); under reduced motion draw static snapshot; pause on `visibilitychange` (resume with one fresh frame); DPR backing store capped at 2×.
- Aria update gap: `renderProgress()` must set `#progress` `aria-valuenow`/`aria-valuetext` (currently static defaults) and populate `#liveRegion` on meaningful changes (≥5% delta or state transition).
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
### Session 2026-10-28 (third pass — autonomous resolutions)

- Q: Progress integration into waveform (integrated playhead vs separate progress bar) → A: Integrated playhead-in-waveform with parallel hidden progressbar — canvas is `aria-hidden="true"` decorative visualization; keep `#progress` as visually-hidden (not `display:none`) `role="progressbar"` carrying `aria-valuenow/min/max/valuetext`, updated by `renderProgress()`; playhead (white center axis + circular knob) drawn from progress percentage; no knob in indeterminate state; fallback shows bar if canvas unavailable. Preserves C-H5/SC-002; no new surface.
- Q: Waveform redraw cadence and S5 loop budget → A: IPC-event-driven redraw — redraw canvas only on throttled RMS IPC updates (~20 Hz), NO self-running `requestAnimationFrame` loop, keeping continuous-motion set = {indeterminate pill} (S5). Cap canvas to 30 fps via 33.3 ms timestamp gate; pause on `visibilitychange` (resume with one fresh frame); freeze (static snapshot) under `prefers-reduced-motion: reduce` and on dynamic toggle; DPR backing store capped at 2×; static SVG/CSS placeholder if RMS unavailable. No new `@keyframes` for waveform.
- Q: Repeat button earcon → A: Reuse existing skip-back earcon (Allegro-trás, 880→660 Hz, 40 ms) — repeat restarts current section from its beginning (backward repositioning to section boundary); no new `EarconKind`/haptic id; closed set unchanged; avoids 004 grammar amendment and blind-test re-validation.
- Q: Waveform accessibility role → A: `aria-hidden="true"` on the canvas + parallel hidden progressbar (Option B) — canvas has no native semantics and cannot track dynamic progress; progress value exposed via `#progress` `role="progressbar"` + `aria-live="polite"` live region (ARIA25); `role="img"` on canvas rejected. JS must also fix latent gap: update `aria-valuenow`/`aria-valuetext` in `renderProgress()` (currently static) and populate live region on meaningful changes (≥5% delta or state transition).

### Session 2026-10-28 (fourth pass — C1 resolution)

- Q: Should the reactive waveform canvas ribbons paint jade (`#5AD6A8`) and celeste (`#B9A6F5`) to match the proposal's teal→jade→violet gradient, or stay reserved/brand-only (static SVG assets + canvas restricted to sanctioned paints)? → A: Option A — paint jade + celeste in the canvas ribbons (proposal-faithful). Amend C-H1/S3 so celeste paint is sanctioned only in waveform-ribbon violet layers + decorative-wave violet stops, and jade only in waveform-ribbon middle layers + brand-layer assets; banned elsewhere on the operational surface. Rate stays `--sona-harp` (gold); glassmorphism uses translucent overlays of existing tokens (no new paint); glow stays on logo + waveform baseline (2 elements max); notification stays accent-only. Requires coordinated §VII edit to `status-surface.md` S3, `data-model.md` §SonaToken, `surface-audit.md` C-H1, and chroma census scoping.

