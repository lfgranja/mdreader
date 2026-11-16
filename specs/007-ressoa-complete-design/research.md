# Research: SONA Complete Design (007)

**Date**: 2026-10-28 | **Feature**: [spec.md](spec.md)

Consolidates findings from clarification sessions (2026-10-28 ×4) and deep-dive docs in this directory. All `NEEDS CLARIFICATION` items from the spec are resolved.

## R1. RMS instrumentation strategy

- **Decision**: True RMS — `AtomicF32` in the CPAL audio callback (`SampleQueue::consume_into`), throttled IPC channel (~20 Hz), canvas smooth-ribbon visualization. Graceful degradation to static placeholder if RMS unavailable.
- **Rationale**: Truly reactive; matches proposal intent; no allocation on hot path (stack accumulator + lock-free store). AtomicF32 is the only safe cross-thread RMS handoff from a real-time callback.
- **Alternatives considered**: envelope/fake RMS (rejected — not reactive); per-sample IPC (rejected — IPC flood); RMS derived in frontend (rejected — no audio access in webview).

## R2. Glassmorphism vs. contract S3

- **Decision**: Amend S3 to allow `rgba()`/`rgb()` alpha variants specifically for glassmorphism/translucency, with base colors as verbatim SonaToken members.
- **Rationale**: Simpler CSS, better visual fidelity, `backdrop-filter: blur(24px)` + translucent overlay. Fallback to solid `--sona-stage` if unsupported.
- **Alternatives considered**: opacity on whole element (rejected — affects text); pre-blended hex (rejected — violates verbatim rule).

## R3. Numeric rate display location

- **Decision**: Paint numeric rate in speed-control area — text element next to dots showing `1.25×`. Amend S4/S10 to allow painted numeric rate in speed-control area only. `aria-valuetext` retains positional descriptor.
- **Rationale**: Eyes-free value-blindness preserved on the progress surface; numeric value aids sighted verification in the control area.
- **Alternatives considered**: hidden-only (rejected — proposal shows painted rate); painted everywhere (rejected — value-blind violation).

## R4. Wave ribbon implementation technique

- **Decision**: CSS-animated SVG — existing `waves-top.svg` as base, CSS keyframes (translate/opacity ≤400 ms, no infinite loop beyond S5), guarded by `@media(reduce)`.
- **Rationale**: Moderate effort, good fidelity, S3-compliant, static-asset preferred.
- **Alternatives considered**: animated SVG (rejected — FR-005 no animated SVG); pure CSS shapes (rejected — lower fidelity).

## R5. Glow/neon scope

- **Decision**: Glow on logo + waveform baseline only — `box-shadow: 0 0 20px var(--sona-etwahl)` on header mark, `0 0 10px var(--sona-harp)` on waveform baseline. 2 elements max. Reduced-motion guard applies.
- **Rationale**: Subtle, proposal-aligned, within budget.

## R6. Waveform visualization style

- **Decision**: Smooth ribbons (match proposal) — canvas-based bezier curves, 4-6 translucent layers teal→jade→violet, mirrored around center axis, white progress line + circular playhead.
- **Rationale**: Most faithful to proposal; requires canvas + RMS data.
- **Alternatives considered**: bars (simpler, diverges from proposal); hybrid rounded bars (lower cost, less faithful).

## R7. Repeat button behavior

- **Decision**: Repeat current section — restarts current section from beginning; reuses skip/section logic; earcon = reuse skip-back (Allegro-trás).
- **Rationale**: Moderate complexity; consistent with existing section logic.

## R8. Progress integration into waveform

- **Decision**: Integrated playhead-in-waveform with parallel hidden progressbar. Canvas is `aria-hidden="true"`; `#progress` stays visually hidden (not `display:none`) `role="progressbar"` carrying `aria-valuenow/min/max/valuetext`, updated by `renderProgress()`. Playhead (white center axis + circular knob) drawn from progress percentage; no knob in indeterminate state; fallback shows bar if canvas unavailable.
- **Rationale**: Preserves C-H5/SC-002; no new surface; canvas has no native semantics.
- **Alternatives considered**: canvas as progressbar (poor AT support); remove progressbar (violates C-H5).

## R9. Waveform redraw cadence & S5 loop budget

- **Decision**: IPC-event-driven redraw (no `requestAnimationFrame` loop), keeping continuous-motion set = {indeterminate pill}. Cap 30 fps via 33.3 ms gate; pause on `visibilitychange` (resume with one fresh frame); freeze (static snapshot) under reduced motion and on dynamic toggle; DPR backing store capped at 2×; static placeholder if RMS unavailable.
- **Rationale**: Battery/thermal; preserves S5 ≤1 loop; no new `@keyframes`.
- **Alternatives considered**: rAF loop with interpolation (second continuous loop — requires S5 amendment; rejected).

## R10. Repeat earcon

- **Decision**: Reuse existing skip-back earcon (Allegro-trás, 880→660 Hz, 40 ms). No new `EarconKind`/haptic id; closed set unchanged.
- **Rationale**: Repeat is backward repositioning to section boundary; avoids 004 grammar amendment and blind-test re-validation.
- **Alternatives considered**: new dedicated earcon (high cost: Rust + Android haptic table + contract amendment + device data).

## R11. Waveform accessibility role

- **Decision**: `aria-hidden="true"` on canvas + parallel hidden progressbar (Option B). `role="img"` on canvas rejected. Fix latent gap: `renderProgress()` must set `aria-valuenow`/`aria-valuetext` (currently static) and populate `#liveRegion` on meaningful changes (≥5% delta or state transition).
- **Rationale**: Canvas has no native semantics; ARIA25 live region for announcements; avoids duplicate announcements.
- **Alternatives considered**: `role="img"` + `aria-label` (static, can't track dynamic progress).

## R12. Celeste/jade paint scope (C1, user-confirmed Option A)

- **Decision**: Paint jade + celeste in canvas ribbons (proposal-faithful). Amend C-H1/S3: celeste sanctioned only in waveform-ribbon violet layers + decorative-wave violet stops; jade only in waveform-ribbon middle layers + brand-layer assets; banned elsewhere on operational surface. Rate stays `--sona-harp` (gold). Requires coordinated §VII edit to `status-surface.md` S3, `data-model.md` §SonaToken, `surface-audit.md` C-H1, chroma census scoping.
- **Rationale**: User priority "real and exact as proposed"; constitution v2.0.0 permits promotion with justification; sanctioned scopes narrow and derivable.
- **Alternatives considered**: keep reserved via static SVG (role-discipline, fidelity loss); hybrid (two visual languages).

## Consolidated unknowns resolved

All `NEEDS CLARIFICATION` from Technical Context resolved: RMS strategy, glassmorphism contract, numeric rate location, wave ribbon technique, glow scope, waveform style, repeat behavior, progress integration, redraw cadence, repeat earcon, a11y role, token paint scope. No remaining unknowns.
