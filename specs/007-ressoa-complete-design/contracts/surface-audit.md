# Surface Audit Amendments — SONA Complete Design (007)

**Feature**: `007-sona-complete-design` | **Base**: `specs/005-adopt-design-sona/contracts/surface-audit.md` | **Date**: 2026-10-28

Amends specific checks; all other C-H rules remain in force.

## C-H1 — hex census (amended)

- Jade `#5AD6A8` and celeste `#B9A6F5` are **no longer "banned anywhere"**; they are allowed **only in sanctioned scopes**:
  - Celeste `#B9A6F5`: waveform-ribbon violet layers + decorative-wave violet stops.
  - Jade `#5AD6A8`: waveform-ribbon middle layers + brand-layer assets.
- Paint of celeste/jade **outside** these scopes = violation (including numeric rate area, title card, notification, glow).
- The chroma census allowlist must be **scope-aware** (file + element context), not a flat hex list, to enforce sanctioned scopes.
- Jade erratum `#5FD0A0` remains banned everywhere.

## C-H4 — element inventory (amended: 9 → 10)

- Add `#waveform` canvas and repeat control to the inventory.
- `#progress` remains in inventory as sr-only aria container (not a corpse — visually hidden, not `display:none`).
- No new `display:none` corpses.

## C-H5 — aria inventory (amended)

- Canvas carries `aria-hidden="true"` (decorative).
- `#progress` retains `role="progressbar"` + `aria-valuenow/min/max/valuetext`, updated by JS (currently static — fix required).
- `#liveRegion` (`role="status"`, `aria-live="polite"`) populated on meaningful changes.
- No `role="img"` on canvas.
- No painted numeric rate except speed-control area.

## C-H6 — movement census (amended)

- No new `@keyframes` for waveform (canvas is not CSS animation).
- Continuous loop count remains ≤1 (`indeterminate` pill only).
- Reduced-motion guard extended to canvas (freeze → static snapshot).
- Decorative wave `wave-subtle` remains one-shot (≤400 ms).

## C-H8 — served identity (unchanged)

Served bundle HTML = `src-tauri/src/status_view.html` (byte-identical).

## New: C-H10 — RMS channel & canvas (007)

- RMS `AtomicF32` present in player crate callback path.
- Dedicated throttled channel (~20 Hz) published.
- Canvas renders on IPC receipt (no rAF loop).
- 30 fps cap enforced via timestamp gate.
- Pause on `visibilitychange`; freeze under reduced motion; DPR cap 2×.
- Static placeholder fallback when RMS unavailable.

## New: C-H11 — token sanctioned-scope census (007)

- Automated check (or code review) verifies celeste/jade hex values appear only in sanctioned scopes (waveform ribbon draw code + decorative wave CSS/SVG).
- Rate area, title card, notification, glow elements must not contain celeste/jade.

## B. Concrete measurement & governance values (amended per Candidate A–D research)

### B1. C-H1 — scope-aware hex census (Candidate D)
- Scope tuple: `(file-glob, context, token-hex)`; allowlist-based.
- Ledger declarations (`--sona-celeste: #B9A6F5` in `:root`) are carve-outs.
- Matching case-insensitive on hex; 3-digit hex expanded.
- Comments (`/* */`, `//`, `<!-- -->`) ignored; string literals ignored only with `/* census:doc */`, otherwise treated as paint (fail-closed).
- Implement as Rust test in `tests/contract_census.rs` (scope-aware variant) + verify.sh gate step, fail-closed.

### B2. C-H4 — inventory count & dead-CSS scan (Candidate D)
- `data-elements="10"` in `src-tauri/src/status_view.html`.
- Test names: `census_inventory_count_is_ten`, `inventory_count_unchanged` → `"10"`.
- Dead-CSS scan: (1) `display:\s*none` count == 0; (2) every CSS selector resolves to inventory allowlist or sanctioned component class; (3) `#progress` visually hidden via `.sr-only`, never `display:none`.
- Repeat control: `<button id="repeat" type="button">`, `aria-label="Repetir seção"`, focus ring `--sona-etwahl`, earcon `EarconKind::SkipBack`, haptic id 4.

### B3. C-H5 — aria inventory (Candidate B)
- Canvas `aria-hidden="true"` (decorative); no `role="img"`.
- `#progress` sr-only `role="progressbar"` with `aria-valuenow/min/max/valuetext`, updated by `renderProgress()`.
- `#liveRegion` (`role="status"`, `aria-live="polite"`) populated on meaningful changes (≥5% delta or state transition).
- Indeterminate: no playhead knob; omit `aria-valuenow`; announce "carregando" once.
- Contrast: text ≥4.5:1 (normal)/≥3:1 (large); UI non-text ≥3:1.
- Focus ring: ≥2 CSS px, ≥3:1 change-of-contrast, 1–2 px offset, `outline`-based.
- Keyboard: native button, tab order repeat→prev→play→next→speed, Enter/Space, no traps; canvas not focusable.
- Forced-colors: system colors, outlined canvas, no color-only signaling.

### B4. C-H6 — movement census (Candidate A)
- No new `@keyframes` for waveform (canvas is not CSS animation).
- Continuous loop count ≤1 (`indeterminate` pill only).
- Decorative wave `wave-subtle` one-shot ≤400 ms.
- Reduced-motion guard extended to canvas (freeze → static snapshot).
- Glow alpha band 0.15–0.25 (0.20 default), never above 0.30; glow retained static under reduced motion.

### B5. C-H10 — RMS channel & canvas (Candidate C)
- RMS `AtomicF32` in player crate callback (`consume_into`), no allocation on hot path.
- Dedicated throttled channel (~20 Hz), last-wins coalescing, skip while hidden.
- Canvas renders on IPC receipt (no rAF loop); 30 fps cap via 33.33 ms gate (`performance.now()`, inclusive at threshold).
- Visibility: `document.visibilityState === 'hidden'` primary; fallbacks `pagehide`, `blur`, `freeze`/`resume`; resume = draw one fresh frame + reset gate.
- DPR: `min(devicePixelRatio, 2)` with fallback to 1 for non-finite/≤0; re-create backing store only on change (debounce 100 ms).
- Static placeholder: inline SVG/CSS bars, 388×80 CSS px, 30% opacity teal on `--sona-stage`, no animation, `aria-hidden="true"`; trigger = RMS unavailable OR canvas unsupported.
- Hot path: 0 heap allocations per redraw (DevTools Allocation instrumentation over 100 frames).
- Cleanup on unmount: remove listeners, cancel frames, `ctx.clearRect`, `canvas.width = canvas.height = 0`, null buffers, unsubscribe IPC.
- 10/10 trial: 10 × 60 s, 5 s warm-up excluded; pass = max ≤30 fps AND zero intervals <33.33 ms in all 10.
- Zero-redraw instrumentation: `redrawCount`/`redrawsWhileHidden` counters; assert 0 increments over 10 s while hidden.
- Device class: mid-tier Android WebView (ref device per spec 003); desktop proxy = Chrome DevTools 4× CPU throttle.

### B6. C-H11 — token sanctioned-scope census (Candidate D)
- Allowlist-based: scope tuple `(file-glob, context, token-hex)`.
- Celeste `#B9A6F5`: waveform-ribbon violet layers + decorative-wave violet stops.
- Jade `#5AD6A8`: waveform-ribbon middle layers + brand-layer assets + decorative-wave teal→jade stops.
- Rate/title/notification/glow remain banned for both.
- Ledger declarations (`--sona-celeste: #B9A6F5` in `:root`) are carve-outs.
- Matching case-insensitive; 3-digit hex expanded; comments ignored; string literals ignored only with `/* census:doc */`.

### B7. C-H8 — served-bundle byte-identity (Candidate D)
- SHA-256 of `src-tauri/src/status_view.html` vs served bundle; assert equality + file size; fail-closed gate step; print both hashes on mismatch.

### B8. Governance values (Candidate D)
- Alpha ops allowlist: `rgba()`/`rgb()` only for glassmorphism/translucency; base color ∈ verbatim SonaToken hex set; alpha ∈ {0.04, 0.08, 0.12, 0.20, 0.32, 0.48}, 0.04 ≤ α ≤ 0.48; `backdrop-filter: blur(24px)` only with solid fallback; drift CI bans all others.
- Promotion gates (8): design justification in `DESIGN-SONA/tokens.md` + commit; sanctioned scopes enumerated; coordinated edit (ordering below); census update (C-H1/C-H11); version bump; RED tests updated; C-H1 + C-H4 green; byte-identity green; rollback plan attached.
- Coordinated-edit ordering: (1) contract amendment → (2) data-model → (3) spec → (4) plan/tasks → (5) RED tests → (6) implementation → (7) census/verify gates → (8) version bump + changelog → (9) commit. All-or-nothing.
- Semver: MAJOR = breaking token/surface/principle; MINOR = additive token/element/scope/amendment; PATCH = wording/test/doc. 007 → MINOR design-system bump (0.1.0→0.2.0); constitution stays 2.0.0.
- Rollback: all-or-nothing; single atomic commit/PR; on any gate failure revert entire change set; token stays reserved.
- Traceability matrix: columns `FR | Contract amendment | Data-model entity | Test file | C-H rule`; rows FR-001..FR-011.
- Unchanged base sections appendix: S1, S2, S5, S6, S7, S12 (all other 005 rules remain in force).
