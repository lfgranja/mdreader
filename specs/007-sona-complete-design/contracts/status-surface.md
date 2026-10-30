# Contract Amendments — SONA Complete Design (007)

**Feature**: `007-sona-complete-design` | **Base**: `specs/005-adopt-design-sona/contracts/status-surface.md` | **Date**: 2026-10-28

This document records the §VII coordinated edits required by 007. It does not replace the 005 contract; it amends specific sections. All other 005 rules remain in force.

## A1. S3 — Token binding rules (amended)

- **Celeste (`#B9A6F5`) promoted from reserved to painted**, but ONLY in sanctioned scopes: waveform-ribbon violet layers + decorative-wave violet stops. Paint elsewhere on the operational surface = violation.
- **Jade (`#5AD6A8`) promoted from brand-only to painted**, but ONLY in sanctioned scopes: waveform-ribbon middle layers + brand-layer assets. Paint elsewhere on the operational surface = violation.
- **Rate display uses `--sona-harp` (gold)** — celeste is NOT used for the numeric rate (avoids the 005 rate collision).
- **Glassmorphism** uses `rgba()`/`rgb()` alpha variants for translucency; base colors must be verbatim SonaToken members (amended S3).
- **Glow/neon** via `box-shadow` on 2 elements max: header mark (`0 0 20px var(--sona-etwahl)`) + waveform baseline (`0 0 10px var(--sona-harp)`).
- Notification/MediaStyle remains accent-only `#F2F0E6` (no jade/celeste).

## A2. S4 — Element inventory (amended: 9 → 10)

Add to the closed enumeration:
- **Waveform canvas** (`<canvas id="waveform" aria-hidden="true">`) — smooth ribbons + integrated playhead; decorative visualization.
- **Repeat control** — new interactive element in transport grid; reuses skip/section logic.

Dead removals / structural changes: `#progress` becomes visually hidden (sr-only) aria container, not removed (C-H4 corpse rule).

## A3. S8 — Decorative wave pseudo-element (amended)

`.card::before` wave ribbons may use the full teal→jade→violet triad (teal `--sona-etwahl`, jade `#5AD6A8`, violet `#B9A6F5`) as CSS/SVG gradient stops — sanctioned scope for jade/celeste in decorative layer. Decorative container carries `aria-hidden="true"`.

## A4. S9 — Title card region (amended)

Title card uses glassmorphism: `backdrop-filter: blur(24px)` + translucent overlay (verbatim token + opacity property). Fallback to solid `--sona-stage` if unsupported. Reduced-motion guard applies. Title binding unchanged (`metadata_title.unwrap_or(filename)`).

## A5. S10 — Refined controls (amended)

- Transport grid now includes repeat (5 controls: repeat, prev, play, next, speed).
- Rate displays numeric value (`1.25×`) painted in speed-control area next to positional dots (value-blind amended for this area only).
- Waveform canvas integrated with progress (playhead); `#progress` retained as sr-only aria container.
- Earcon for repeat reuses skip-back (Allegro-trás); no new earcon variant.

## A6. S11 — Change discipline (unchanged)

All 007 additions follow the 3-file coordinated edit (contract + data-model + source + commit justification + C-H1/C-H4 verification). Token expansion recorded in `DESIGN-SONA/tokens.md` + commit message + semver bump.

## A7. New: RMS instrumentation contract

- Rust (`crates/player`): `AtomicF32` RMS store updated in `SampleQueue::consume_into` (no allocation on hot path).
- IPC: dedicated `tauri::ipc::Channel`, throttled ~20 Hz; skip publishes when unchanged/backgrounded.
- Frontend: canvas redraw on IPC receipt (event-driven, no rAF loop); 30 fps cap; pause on `visibilitychange`; freeze under reduced motion; DPR cap 2×; static placeholder fallback.

## A8. New: Waveform accessibility contract

- Canvas `aria-hidden="true"` (decorative).
- `#progress` retained as sr-only `role="progressbar"` with `aria-valuenow/min/max/valuetext`, updated by `renderProgress()`.
- `#liveRegion` (`role="status"`, `aria-live="polite"`) populated on meaningful changes (≥5% delta or state transition).
- Indeterminate: no playhead knob; omit `aria-valuenow`; announce "carregando" once.
- No `role="img"` on canvas.

## B. Concrete values (amended per Candidate A–D research)

### B1. Glassmorphism (Candidate A)
- `backdrop-filter: blur(24px) saturate(120%)`.
- Translucent overlay: `rgba(20, 27, 45, 0.65)` (verbatim `--sona-stage` base at 65% alpha).
- Doppelrand borders: outer `1px rgba(255,255,255,0.06)`, inner `1px rgba(143,216,232,0.15)`.
- Fallback (unsupported): solid `background: var(--sona-stage)`, no blur, borders retained.

### B2. Glow/neon (Candidate A)
- Header mark: `box-shadow: 0 0 20px rgba(143, 216, 232, 0.20)`.
- Waveform baseline: `box-shadow: 0 0 10px rgba(232, 196, 106, 0.20)`.
- Alpha band: 0.15–0.25 (0.20 default); never above 0.30.
- Reduced motion: glow retained static; any pulsing disabled.

### B3. Decorative wave ribbon (Candidate A)
- Single layer; base opacity 0.80; animation peak opacity 0.90 at 50% keyframe.
- Animation: one-shot, `400 ms ease-in-out`, no loop; `animation-fill-mode: both`.
- Reduced motion: `animation: none; opacity: 0.80` (static snapshot).
- (Optional 3-layer variant: opacities 0.40/0.60/0.80 back→front; requires explicit aesthetic decision.)

### B4. Waveform ribbons (Candidate A)
- Layer count: 5 (within sanctioned 4–6 band).
- Bezier control points: CP1 at `(x + w/3, y + amp·k)`, CP2 at `(x + 2w/3, y − amp·k)`; `k` ∈ {0.6, 0.75, 0.9, 1.0, 0.85} for layers 1→5.
- Per-layer opacity (back→front): 0.15, 0.20, 0.25, 0.30, 0.35.
- Color→layer mapping: L1 teal `--sona-etwahl`, L2 jade `#5AD6A8`, L3 jade→violet blend, L4 celeste `#B9A6F5`, L5 violet (highest opacity).
- Center line (progress axis): 1 px solid white `#FFFFFF` at 0.90 opacity along y = centerY.
- Playhead: circular knob, 6 px radius, white, drawn on top.
- Reduced motion: single static snapshot (last RMS frame), all 5 layers at specified opacities; no per-frame redraw.

### B5. Accessibility (Candidate B)
- Contrast: text ≥4.5:1 (normal) / ≥3:1 (large); UI non-text ≥3:1 (WCAG 1.4.3/1.4.11).
- Focus ring: ≥2 CSS px thickness, ≥3:1 change-of-contrast, 1–2 px offset, `outline`-based (forced-colors safe).
- Keyboard: native `<button>`; tab order repeat→prev→play→next→speed; Enter/Space activation; no traps; canvas not focusable.
- Forced-colors: system colors, outlined canvas, no color-only signaling.

### B6. Performance (Candidate C)
- 30 fps cap via 33.33 ms gate (`performance.now()`; draw allowed when `now - lastDrawTs >= 33.333333`, inclusive).
- Device class: mid-tier Android WebView (reference device per spec 003); desktop proxy = Chrome DevTools 4× CPU throttle.
- IPC cadence: ~20 Hz (50 ms), last-wins coalescing; drop all updates while hidden; never queue a backlog.
- Visibility trigger: `document.visibilityState === 'hidden'` (primary); fallbacks `pagehide`, `blur`, `freeze`/`resume`; resume = draw one fresh frame + reset gate.
- DPR: `dpr = (Number.isFinite(window.devicePixelRatio) && window.devicePixelRatio > 0) ? Math.min(window.devicePixelRatio, 2) : 1`; re-create backing store only on change (debounce 100 ms), never per frame.
- Static placeholder: inline SVG/CSS bars, 388×80 CSS px, 30% opacity teal on `--sona-stage`, no animation, `aria-hidden="true"`; trigger = RMS unavailable OR canvas unsupported.
- Hot path: 0 heap allocations per redraw (verified via DevTools Allocation instrumentation over 100 frames).
- Cleanup on unmount: remove all listeners, cancel pending frames, `ctx.clearRect`, `canvas.width = canvas.height = 0`, null RMS buffers, unsubscribe IPC.
- 10/10 trial protocol: 10 trials × 60 s, 5 s warm-up excluded; pass = max ≤30 fps AND zero intervals <33.33 ms in all 10.
- Zero-redraw instrumentation: `redrawCount`/`redrawsWhileHidden` counters; assert 0 increments over 10 s while hidden.

### B7. Governance (Candidate D)
- Scope-aware census: scope tuple `(file-glob, context, token-hex)`; allowlist-based; ledger declarations (`--sona-celeste: #B9A6F5` in `:root`) are carve-outs; case-insensitive hex; comments and string literals (unless `/* census:doc */`) ignored; 3-digit hex expanded.
- Alpha ops allowlist: `rgba()`/`rgb()` only for glassmorphism/translucency; base color ∈ verbatim SonaToken hex set; alpha ∈ {0.04, 0.08, 0.12, 0.20, 0.32, 0.48} with 0.04 ≤ α ≤ 0.48; `backdrop-filter: blur(24px)` only with solid fallback; drift CI bans all other rgba/rgb.
- Promotion gates (8): design justification in `DESIGN-SONA/tokens.md` + commit; sanctioned scopes enumerated; coordinated edit per B8 ordering; census update (C-H1/C-H11); version bump; RED tests updated; C-H1 + C-H4 green; byte-identity green; rollback plan attached.
- Inventory count: `data-elements="10"`; test names updated (`…_is_ten`).
- Repeat control: `<button id="repeat" type="button">` (implicit role), `aria-label="Repetir seção"`, focus ring `--sona-etwahl`, earcon `EarconKind::SkipBack`, haptic id 4.
- Dead-CSS scan: (1) `display:\s*none` count == 0; (2) every CSS selector resolves to inventory allowlist or sanctioned component class; (3) `#progress` visually hidden via `.sr-only`, never `display:none`.
- Coordinated-edit ordering: (1) contract amendment → (2) data-model → (3) spec → (4) plan/tasks → (5) RED tests → (6) implementation → (7) census/verify gates → (8) version bump + changelog → (9) commit. All-or-nothing.
- Sanctioned locations (explicit): celeste `#B9A6F5` in waveform-ribbon violet layers + decorative-wave violet stops; jade `#5AD6A8` in waveform-ribbon middle layers + brand-layer assets + decorative-wave teal→jade stops. Rate/title/notification/glow remain banned.
- Unchanged base sections appendix: S1, S2, S5, S6, S7, S12 (all other 005 rules remain in force; amendment overrides only named sections).
- C-H8 byte-identity: SHA-256 of `status_view.html` vs served bundle; assert equality + file size; fail-closed gate step.
- Semver: MAJOR = breaking token/surface/principle change; MINOR = additive token/element/scope/amendment; PATCH = wording/test/doc. 007 → MINOR design-system bump (0.1.0→0.2.0); constitution stays 2.0.0 (no principle redefinition). Update constitution version + date + sync-impact report, `DESIGN-SONA/VERSION`, `CHANGELOG.md`, commit message.
- Rollback: all-or-nothing; single atomic commit/PR; on any gate failure revert entire change set; token stays reserved; `tokens.md`/`CHANGELOG.md` revert; version bump reverted.
- Traceability matrix: columns `FR | Contract amendment | Data-model entity | Test file | C-H rule`; rows FR-001..FR-011; kept in sync via coordinated-edit checklist.
