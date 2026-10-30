# Clarification — Waveform Visualization Style (Spec 007, deep dive)

**Status:** READ-ONLY research — no edits performed. Proposes findings only.

**Source evidence base:**
- `src-tauri/src/status_view.html` (current surface; lines cited below)
- `specs/007-sona-complete-design/spec.md` §FR-008 (RMS waveform requirement)
- `DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png` (proposal image)
- `specs/005-adopt-design-sona/contracts/status-surface.md` S3/S5/S7
- `specs/005-adopt-design-sona/contracts/surface-audit.md` (C-H4 inventory, movement census)
- `src-tauri/Cargo.toml` (tauri =2.12.1, features [])

---

## (1) Current visualization elements in `status_view.html`

- **No `<canvas>` anywhere in the repo** (grep `<canvas` returned zero matches across all files).
- **No inline `<svg>` in `status_view.html`**. SVGs are consumed as static assets:
  - `DESIGN-SONA/assets/waves/waves-top.svg` referenced as CSS `background-image` on `.card::before` (lines 66–81, 73) — decorative top ribbon, not a data viz.
  - `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg` as `<img id="headerMark">` (line 262).
- **Progress visualization = CSS div bar**: `#progress` (line 278) track 6px tall; `#progressFill` (lines 182–188) width set via JS `renderProgress()` (lines 428–445), `transition: width .3s ease` (line 187). Indeterminate state uses a CSS `linear-gradient` + `@keyframes indeterminate` (lines 433–437, 780–788) with `transform: translateX`.
- **Animations in use**:
  - `wave-subtle` keyframes (lines 82–86), 400ms ease-in-out, applied to `.card::before` (line 78) — decorative, not infinite.
  - `slideIn` (line 248), 0.2s, on `.reason-banner.warning`.
  - `indeterminate` (lines 783–786), 1.5s infinite, progress fill only.
  - `transition` on buttons/speed-dots (lines 120, 168).
- **Reduced-motion guard** present in scaffold tokens (line 257): `#progress-fill`, `.reason-banner.warning`, `.card::before` animations disabled.
- **No waveform, visualizer, bar, or RMS rendering exists**. The scaffold inventory declares `data-elements="9"` (line 258).

## (2) Available rendering techniques in the Tauri webview

Tauri 2.12.1 (`src-tauri/Cargo.toml` line 21, `features = []`) embeds the platform WebView:
- **Windows**: WebView2 (Edge/Chromium) — Canvas 2D, SVG, WebGL, CSS animations, `requestAnimationFrame`.
- **macOS/iOS**: WKWebView — same standard set.
- **Linux**: WebKitGTK — Canvas 2D, SVG, CSS; WebGL available but less reliable.
- **Android**: System WebView (Chromium-based) — Canvas 2D, SVG, CSS animations, `requestAnimationFrame`, WebGL (hardware-dependent).

**Conclusion:** Canvas 2D, inline SVG, and CSS are all feasible. WebGL is technically available but adds risk on Android System WebView and is unnecessary for a bar waveform. The spec FR-005 prefers "Static assets (SVG/CSS)" and FR-008 mandates "canvas/SVG bar visualization" for RMS data — so **Canvas 2D is the sanctioned reactive technique**, SVG/CSS for decorative/static layers. No `foreignObject` (FR-005). No allocation on hot path (FR-008).

## (3) What the proposal shows (`ui-proposta-2-lente-sinestesica.png`)

Vision analysis of the proposal image:
- **Waveform visualizer = layered smooth-curve ribbons**, not bars, not a line graph, not circular.
- Mirrored vertically around a horizontal center axis (upper + lower lobes).
- Envelope peaks in the center (~x=380) and tapers at both edges — center-weighted amplitude shape.
- ~4–6 overlapping translucent ribbons in mint/teal-green + lavender/purple, with soft outer glow, blending where overlapped.
- Spans nearly full width (x≈105–660), positioned below the title card and above the playback controls (y≈690–1000).
- **Progress is integrated INTO the waveform**: a thin white horizontal center line acts as the axis; a white circular knob (~50% across) is the playhead/scrub handle. No separate progress bar.
- No enclosing card; waveform floats on the dark background. Title sits in a separate frosted-glass card above.
- Other visuals: decorative teal/purple top ribbons, glowing cyan spiral logo, frosted glass title card, translucent glass playback pill (repeat/prev/play/next/1.25×), soft glow beneath pill.

**Mismatch with spec FR-008 wording:** FR-008 says "canvas/SVG **bar** visualization", but the proposal shows **smooth filled ribbons/curves**, not discrete bars. This is the key clarification gap — the implementation style (bars vs. smoothed ribbons) must be resolved. Options:
  - A) Strict bars (FR-008 literal) — simpler, cheaper, but diverges from proposal's smooth ribbons.
  - B) Smoothed filled curves (proposal-faithful) — matches image, needs Canvas 2D `quadraticCurveTo`/`bezierCurveTo` + mirroring; higher per-frame cost.
  - C) Hybrid: coarse bars (e.g., 32–48 bins) with rounded caps, approximating ribbons at lower cost.

## (4) Integration with existing `.card` layout

Current `.card` (lines 55–65): `max-width: 420px`, `padding: 2rem`, `position: relative`, `overflow: hidden`, dark gradient background, box-shadow. Contains: header mark + title, subtitle, controls grid, speed control, `#progress`, label, reason banner, footer.

Integration options:
- **Replace `#progress` div with a canvas element** (`<canvas id="waveform" aria-hidden="true">`) sized to card width × ~80px, keeping `role="progressbar"` on a visually-hidden or parallel container to preserve aria (SC-002). The white center line + playhead knob become canvas-drawn or overlaid DOM elements.
- **Keep progress bar as fallback**: render waveform canvas above `#progress`; when RMS unavailable, hide canvas and show `#progress` (FR-008 graceful degradation: "static waveform placeholder if RMS unavailable").
- **Card overflow**: `.card` has `overflow: hidden`, so canvas must fit within padding; no negative margins. Canvas width = card content width (~388px at 420px max). Height ~60–80px to match proposal envelope.
- **Z-index/layering**: `.card::before` wave ribbon sits `z-index: 0`; waveform canvas should sit at z-index 1 above background but below controls (z-index default auto). Controls are later in DOM so paint above.
- **Glow**: FR-010 allows `box-shadow: 0 0 10px var(--sona-harp)` on waveform baseline (2 elements max with header mark).
- **Reduced motion**: canvas animation must freeze (not hide) under `prefers-reduced-motion: reduce` — draw a static snapshot; guard the rAF loop.
- **Accessibility**: preserve `#progress` aria (`role="progressbar"`, `aria-valuenow/min/max/valuetext`) — either keep a visually-hidden progress element or update aria on the canvas wrapper. SC-002 requires no aria regressions.

## (5) Performance implications for mobile webview

- **RMS data path (FR-008):** AtomicF32 in audio callback (`consume_into`), throttled IPC ~20 Hz. 20 Hz = 50ms between updates — comfortable for Canvas 2D; avoids per-sample IPC. "No allocation on hot path" applies to Rust side.
- **Rendering cadence:** Driving canvas redraws at 20 Hz (matching IPC) rather than 60 Hz rAF saves CPU/battery. For smooth ribbon animation, interpolate between samples or run rAF at 30–60 fps reading the latest RMS buffer; cap to 30 fps on battery-constrained devices (Android Doze). Under reduced motion, draw once per update.
- **Canvas size:** ~388×80 px = ~31k pixels — trivial for Canvas 2D on all target webviews. Even 4–6 overlapping ribbons with bezier curves are cheap at this resolution.
- **Android System WebView specifics:** Avoid WebGL (driver variance); Canvas 2D is stable. Avoid `backdrop-filter` stacking with canvas (compositing cost) — FR-009 glassmorphism on title card only, not overlapping waveform. Use `devicePixelRatio` scaling carefully (cap at 2× to avoid memory blowup on high-DPI Android).
- **Memory:** Single canvas, no offscreen buffers needed. Reuse the same canvas; never recreate per update.
- **Battery/thermal:** Prefer 30 fps cap on mobile; pause rAF when tab/backgrounded (`visibilitychange`); freeze on reduced motion. The existing indeterminate animation (1.5s infinite) is the only continuous loop currently — adding a waveform loop would be a second continuous loop, which S5/Option C disallows unless justified (≤1 continuous loop total). **Resolution:** make the waveform redraw driven by IPC updates (event-driven, not a self-running rAF loop) so it does not count as an additional continuous animation loop; or justify a second loop per S5 with reduced-motion guard.
- **Graceful degradation:** If RMS IPC unavailable, render a static waveform placeholder (SVG/CSS) — no canvas animation, no performance cost.

## Recommended clarification decisions (for review)

1. **Style:** Resolve bars-vs-ribbons. Recommend Option B (smoothed filled ribbons) to match proposal, implemented in Canvas 2D with mirrored upper/lower lobes and 4–6 layers; amend FR-008 wording from "bar" to "ribbon/curve" (or keep "bar" as bins with rounded caps = Option C). **Decision needed.**
2. **Progress integration:** Adopt proposal's integrated playhead-in-waveform; keep a hidden `#progress` for aria (SC-002) rather than removing it.
3. **Loop budget:** Make waveform redraw IPC-event-driven (not a self-running rAF loop) to stay within S5's ≤1 continuous loop; document in S5 census.
4. **Frame rate:** Cap at 30 fps on mobile; pause on `visibilitychange`; freeze (draw static) under reduced motion.
5. **DPR:** Cap canvas backing store at 2× devicePixelRatio.
6. **Fallback:** Static SVG/CSS placeholder when RMS unavailable (FR-008).

**No contract, spec, plan, or constitution file was modified.** This document proposes findings and decision points only.
