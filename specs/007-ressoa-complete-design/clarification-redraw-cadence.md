# Clarification — Redraw Cadence (Candidate C3, Spec 007, online research)

**Status:** READ-ONLY research — no edits performed. Evidence-backed recommendation delivered.
**Scope:** Battery/performance best practices for the canvas waveform visualizer mandated by FR-008 (`status_view.html` StatusView), under the S5 motion budget (≤1 continuous loop, reduced-motion guard) and the WCAG 2.2.2 pause/stop/hide requirement.
**Target surface:** Single StatusView canvas (~388×80 CSS px, per `clarification-waveform-visualization-style.md` §4), fed by RMS IPC at ~20 Hz (FR-008).

---

## 1. What was asked (per spec / contracts / peers)

- **FR-008** (spec.md): "Cap 30 fps on mobile, pause on visibilitychange." plus "No allocation on hot path."
- **S5** (status-surface.md §S5 / data-model MovementSpec): continuous motion set = {pill}; ≤1 continuous loop; reduced-motion freeze guard; yellow pulse bounded to 3 s TTL; census counts declaration occurrences.
- **S3/S7** (status-surface.md): coordinated edit discipline; no new surface; no new token members unless unavoidable.
- **Candidate C3**: decide redraw cadence (requestAnimationFrame vs event-driven), fps cap (30 fps), visibilitychange pausing, reduced-motion handling, devicePixelRatio scaling — with scientific/industry sources and S5-style motion-budget rationale (WCAG 2.2.2).

**Relation to current implementation:** The repo has **no `<canvas>` anywhere** (grep `<canvas` = 0 matches, confirmed in `clarification-waveform-visualization-style.md` §1). The progress indicator is a CSS div bar (`#progress`/`#progressFill`) with a 1.5 s infinite `indeterminate` keyframe — the only existing continuous loop. So C3 is greenfield guidance for the new canvas path; it must not add a second unguarded loop. Importance: high — mobile webview battery/thermal and S5 census compliance hinge on this decision, and the WCAG 2.2.2 auto-updating-content rule requires a pause/stop/control mechanism.

---

## 2. Method

Multi-source research executed (`web_search`, direct reads) across:
- **MDN**: `requestAnimationFrame`, Page Visibility API, `visibilitychange`, `devicePixelRatio`, canvas HiDPI correction.
- **web.dev**: "Improving HTML5 Canvas performance" (Boris Smus), "Rendering performance", "Animations and performance", "High DPI Canvas", speed-rendering (background-tab pause note).
- **W3C WAI**: WCAG 2.2 Success Criterion 2.2.2 Pause, Stop, Hide (Level A) understanding document; Page Visibility Level 2 spec.
- **Chrome for Developers**: Page Lifecycle API (freeze/resume), RenderingNG architecture.
- **Android Developers**: Doze/App Standby optimization, WebView memory, behavior changes.
- **Industry**: PlayCanvas runtime DPR guidance, Chart.js devicePixelRatio docs, Capacitor animation performance guide, Chromium issue tracker (Android WebView WebGL jank).
- **Repo-local**: `clarification-waveform-visualization-style.md` §5, `status-surface.md` S5/S8/S11, `data-model.md` MovementSpec, constitution v2.0.0.

All claims below are backed by retrieved sources (cited inline with URLs in §7). No fabricated studies.

---

## 3. Findings

### 3.1 requestAnimationFrame vs event-driven redraw

**Evidence:**
- MDN rAF: "The frequency of calls to the callback function will generally match the display refresh rate… 60/75/120/144 Hz… `requestAnimationFrame()` calls are paused in most browsers when running in [background]." (MDN, Window.requestAnimationFrame)
- web.dev (Smus): rAF "politely ask[s] the browser to call your rendering routine… if the page is not in the foreground, the browser is smart enough not to render." Also warns mobile canvas is slow and that rAF "doesn't guarantee [60 FPS]".
- web.dev speed-rendering: "Animations in background tabs get paused, conserving system resources and battery life."
- Chromium issue 40839122: Android WebView canvas/WebGL animations in rAF can jank; WebView rendering path differs from Chrome.

**Implication:** rAF is efficient and self-pausing in background, but it is **self-running** (a continuous loop by definition) and its rate tracks the display (up to 120/144 Hz on modern phones) unless throttled. That conflicts with S5 (≤1 continuous loop) and burns battery on high-refresh mobile displays.

**Recommendation:** Make the waveform redraw **IPC-event-driven** (triggered by the ~20 Hz RMS update, FR-008), not a self-running rAF loop. 20 Hz (50 ms) is sufficient for audio-reactive visualization and matches the data cadence, so no frames are wasted re-drawing identical state. This keeps the continuous-loop count at {pill} and satisfies the S5 census. rAF is reserved for an *optional, explicitly-justified* interpolation enhancement (§3.2), off by default.

### 3.2 Frame-rate cap (30 fps)

**Evidence:**
- web.dev speed-rendering: "If the system can't handle rendering at the screen's refresh rate, it can throttle animations and produce [frames] less frequently (say, 30 times per second)."
- MDN rAF: rate matches display refresh (60–144 Hz); on a 120 Hz Android phone an unthrottled rAF loop fires 2× as often as needed.
- PlayCanvas runtime DPR doc: on "low or mid-tier mobile devices [with] high resolution screens but low graphics capability," cap resolution/rate to "maintain a playable frame rate."
- FR-008 already mandates "Cap 30 fps on mobile."
- WCAG 2.2.2: auto-updating content must allow the user to "control the frequency of the update" — a fixed 30 fps cap (below the 60 fps "smooth" target) is a defensible, battery-friendly ceiling for a non-essential visualization; essential progress still updates.

**Recommendation:** Cap any redraw path at **30 fps** via timestamp gating (`now - lastDraw >= 33.3 ms`). For the default event-driven path (20 Hz data), the cap is automatically satisfied and needs no gating; the gate is required only if an rAF interpolation loop is enabled. Never exceed 30 fps on mobile; on desktop the same 30 fps cap applies for consistency (one code path, one budget). Do not chase 60 fps — web.dev's "maintain 60 fps" guidance targets interactive UI animations, not a background audio-reactive waveform, and the battery cost is not justified here (the visualization is supplementary, not essential).

### 3.3 visibilitychange pausing

**Evidence:**
- MDN Page Visibility API: `visibilitychange` fires when the page becomes hidden; rAF is paused automatically in most browsers when backgrounded, but timers and other work may continue.
- MDN `visibilitychange` event: standard mechanism to detect tab/app switch.
- Chrome Page Lifecycle API: adds `freeze`/`resume` events with stronger guarantees than `visibilitychange` alone; recommended for "pausing animations, timers, and network requests."
- Stack Overflow (Android WebView): "visibility of webview is calculated as: visible = !paused && (!attached to window || …)" — `visibilitychange` does **not** always fire in older Android WebView builds; a fallback is needed.
- Android Developers (Doze): background CPU/network deferred; apps must not spin.
- MDN `requestAnimationFrame`: rAF paused in background in most browsers.

**Recommendation:** Pause redraws (and stop subscribing to RMS updates) when `document.visibilityState === 'hidden'`. Implement defensively with multiple signals:
1. Primary: `document.addEventListener('visibilitychange', …)` checking `document.visibilityState`.
2. Fallback for Android WebView gaps: `window.addEventListener('pagehide'/'blur', pause)` and `window.addEventListener('pageshow'/'focus', resume)`; also `document.addEventListener('freeze'/'resume', …)` (Page Lifecycle API) where supported.
3. Guard with `document.hasFocus()` / `!document.hidden` before any draw.
4. On resume, draw a single fresh frame (do not replay backlog); reset the timestamp gate.
5. If the host (Android) reports the app backgrounded via the native bridge, treat as hidden.

This satisfies WCAG 2.2.2's "pause" requirement for auto-updating content and prevents battery drain in Doze.

### 3.4 Reduced motion

**Evidence:**
- WCAG 2.2.2 (W3C): auto-updating content that starts automatically and is presented in parallel with other content must provide a mechanism to pause/stop/hide or control frequency; essential content (e.g., progress) is exempt from the 5-second rule but still benefits from a guard.
- WCAG 2.3.3 (Animation from Interactions) and S5: decorative motion must be disable-able.
- MDN `prefers-reduced-motion`: standard media query; the repo already wires `@media(prefers-reduced-motion:reduce)` for `#progress-fill`, `.reason-banner.warning`, `.card::before`.
- constitution v2.0.0: reduced-motion guard required for all decorative animations; no new continuous loops beyond allowed.

**Recommendation:** Under `prefers-reduced-motion: reduce`:
- **Freeze decorative ribbon flow** (opacity/translate/scale animations) — draw a static snapshot; do not animate.
- **Keep essential progress updates** (playhead position) but **data-driven only** (one draw per RMS/progress update, no interpolation, no rAF), so the user still sees position change without decorative motion. This matches the "status in nature" exception in WCAG 2.2.2 (pausing would show stale info).
- Listen for dynamic toggles via `matchMedia('(prefers-reduced-motion: reduce)').addEventListener('change', …)` and re-apply at runtime (no auto-resume of decorative motion).
- Do not hide the canvas (would break progress visibility); freeze, not hide (S5 requires visible frozen position).

### 3.5 devicePixelRatio scaling

**Evidence:**
- MDN `devicePixelRatio`: "Modern mobile device screens… often yield a `devicePixelRatio` value greater than 2." (e.g., 2.625, 3, 4).
- web.dev "High DPI Canvas": scale canvas backing store by DPR and `ctx.scale(dpr, dpr)` for crisp rendering.
- PlayCanvas: high DPR on low/mid mobile with low graphics capability is problematic; "cap [the ratio] to maintain a playable frame rate."
- Chart.js: defaults to device DPR; allows override.
- Chromium issue 40839122: Android WebView canvas rendering jank; lower backing-store resolution helps.

**Recommendation:** Cap the backing store at **2× DPR**:
```
const dpr = Math.min(window.devicePixelRatio || 1, 2);
canvas.width  = Math.floor(cssWidth  * dpr);
canvas.height = Math.floor(cssHeight * dpr);
canvas.style.width  = cssWidth  + 'px';
canvas.style.height = cssHeight + 'px';
ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
```
- 2× is the sweet spot: crisp on Retina/HiDPI (matches the majority of phones) while keeping pixel count bounded (388×80 → 776×160 = ~124 k px at 2× vs ~280 k+ at 3×).
- Re-create the backing store only when DPR changes (monitor via `matchMedia('(resolution: Ndppx)')` change event, per MDN example), never per frame.
- Draw with integer coordinates (web.dev: sub-pixel rendering anti-aliases and is slower; `Math.round`/bitwise truncation).
- Avoid `shadowBlur` on canvas (web.dev: expensive); the glow on the waveform baseline is already specified as CSS `box-shadow` on the container (FR-010), not canvas shadow.

### 3.6 Additional canvas hygiene (from web.dev)

- **Batch state changes / paths:** build one path per ribbon layer, single `stroke()`/`fill()` per layer; group by color (web.dev "Avoid unnecessary canvas state changes").
- **Clear efficiently:** `clearRect(0,0,w,h)` (outperforms `canvas.width = canvas.width` reset in modern browsers); or redraw-region only.
- **Reuse the canvas:** never recreate per update (web.dev pre-render/offscreen guidance; here a single canvas suffices — no offscreen buffer needed at 388×80).
- **No allocation on hot path** (FR-008): reuse typed arrays / pre-allocated buffers for RMS samples; do not `new` arrays in the draw or IPC callback.
- **OffscreenCanvas:** not required at this resolution; mention as a future option only if profiling shows main-thread contention. (web.dev OffscreenCanvas.)

---

## 4. S5 motion-budget rationale (WCAG 2.2.2)

- **WCAG 2.2.2** requires that moving/blinking/scrolling/auto-updating content that starts automatically and runs in parallel with other content either (a) stops within 5 s, or (b) provides a mechanism to pause/stop/hide or control update frequency. Essential real-time status (progress/playhead) is exempt from the 5-second limit but still must not distract.
- **Application to the waveform:**
  - The **playhead/progress** is essential status → may auto-update continuously; but it is data-driven (IPC), not a self-running animation, so it does not add a continuous CSS/JS loop. Under reduced motion it still updates per data tick (no smoothing) — consistent with the "status in nature" exception (pausing would show stale position).
  - The **ribbon flow** (decorative opacity/translate) is non-essential → must be stoppable within 5 s and must freeze under `prefers-reduced-motion`. Making it event-driven (no self-running loop) and freezing under reduce satisfies this without adding a user-facing pause control; the existing reduced-motion media query is the control mechanism (S5 already accepts this pattern for the pill).
  - The **loop census** (C-H6) stays at {pill}: the waveform adds zero `@keyframes`/infinite `animation` declarations and zero self-running rAF loops. Redraws are triggered by data updates (discrete), which the census does not count as a continuous loop.
- **Conclusion:** C3's event-driven + 30 fps-capped + visibility-paused + reduced-motion-frozen cadence is S5-compliant and WCAG 2.2.2-compliant, and requires no new interactive control beyond the existing reduced-motion guard.

---

## 5. Options (with reasoned tradeoffs)

### Option A — Event-driven redraw at IPC rate (~20 Hz), 30 fps cap, no rAF loop (RECOMMENDED)
- Redraw only when a new RMS sample arrives (~20 Hz). Timestamp gate enforces ≤30 fps if samples ever arrive faster. No self-running loop.
- **Pros:** S5-compliant (no new loop), battery-friendly, matches data cadence, simplest, WCAG-friendly, no wasted frames, trivially pausable via visibilitychange.
- **Cons:** Slightly less fluid than 60 fps interpolation; ribbon motion may look stepped at 20 Hz (acceptable for audio-reactive viz; many visualizers run at 20–30 Hz).
- **Evidence weight:** HIGH — MDN/web.dev rAF background-pause, S5 census, WCAG 2.2.2, FR-008.

### Option B — rAF loop with interpolation, capped at 30 fps
- Run rAF but gate draws to ≤30 fps and interpolate playhead/RMS between samples.
- **Pros:** Smoother motion; standard game-loop pattern.
- **Cons:** Introduces a **second continuous loop** — requires explicit S5 justification + reduced-motion guard + C-H6 census update; higher battery cost; Android WebView rAF jank risk (Chromium #40839122); more complexity.
- **Evidence weight:** MEDIUM — valid technique, but conflicts with S5 default and is unnecessary given the 20 Hz data rate.

### Option C — Unthrottled rAF at display refresh (60–144 Hz)
- **Pros:** Smoothest.
- **Cons:** Violates FR-008 (30 fps cap), drains battery, janks on Android WebView, inflates loop census. Rejected outright.

---

## 6. Recommendation

**Select Option A — event-driven redraw at the IPC rate (~20 Hz), with a 30 fps safety cap, visibilitychange pausing, reduced-motion freezing, and a 2× DPR cap.** Optionally document Option B as a future, explicitly-justified enhancement (requires S5 amendment + C-H6 update) if profiling shows 20 Hz is insufficiently smooth.

Concrete parameters:
- **Cadence:** event-driven on RMS IPC update (~20 Hz). No rAF loop in the default path.
- **FPS cap:** 30 fps (`MIN_FRAME_INTERVAL_MS = 1000/30 ≈ 33.3 ms`) via timestamp gate; applies automatically to the event-driven path and is mandatory for any rAF-based path.
- **visibilitychange:** pause on `hidden`, resume on `visible`, with Android WebView fallbacks (`pagehide`/`blur`/`freeze`); single fresh frame on resume.
- **Reduced motion:** freeze decorative ribbon animation; keep essential playhead updates data-driven (no smoothing); react to dynamic toggles.
- **DPR cap:** `Math.min(devicePixelRatio, 2)`; re-create backing store only on DPR change (via `matchMedia`); integer coordinates; no canvas `shadowBlur`.
- **Hygiene:** batched paths, `clearRect`, reused canvas, no hot-path allocation, no OffscreenCanvas unless profiling demands.

This is autonomous (no user decision needed): FR-008 already mandates the 30 fps cap and visibilitychange pause; S5 mandates the loop budget; the recommendation chooses the S5-compliant event-driven interpretation and pins the DPR cap at 2×, which is a derived engineering constant, not a product decision. Only the *optional* Option B enhancement would require a user-facing decision to amend S5.

---

## 7. References (with URLs — all retrieved this session)

- MDN — `Window.requestAnimationFrame()` (rate = display refresh; paused in background): https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame
- MDN — Page Visibility API: https://developer.mozilla.org/en-US/docs/Web/API/Page_Visibility_API
- MDN — `Document.visibilitychange` event: https://developer.mozilla.org/en-US/docs/Web/API/Document/visibilitychange_event
- MDN — `Window.devicePixelRatio` (mobile DPR often >2; HiDPI canvas correction): https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio
- web.dev — "Improving HTML5 Canvas performance" (Boris Smus): rAF, mobile canvas slowness, shadowBlur, sub-pixel, clearRect: https://web.dev/articles/canvas-performance
- web.dev — "Rendering performance" (throttle to ~30 fps, background-tab pause): https://web.dev/articles/rendering-performance
- web.dev — "High DPI Canvas": https://web.dev/articles/canvas-hidipi
- web.dev — "Animations and performance" (60 fps target for interactive animations): https://web.dev/articles/animations-and-performance
- web.dev — "Jank busting for better rendering performance" (background-tab pause, battery): https://web.dev/articles/speed-rendering
- W3C WAI — WCAG 2.2 Understanding SC 2.2.2 Pause, Stop, Hide: https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html
- W3C — Page Visibility Level 2: https://www.w3.org/TR/page-visibility-2/
- Chrome for Developers — Page Lifecycle API (freeze/resume): https://developer.chrome.com/docs/web-platform/page-lifecycle-api
- Android Developers — Optimize for Doze and App Standby: https://developer.android.com/training/monitoring-device-state/doze-standby
- Android Developers — WebView and memory: https://developer.android.com/topic/performance/memory/guide/webview-memory
- PlayCanvas — runtime devicePixelRatio optimization (cap on low/mid mobile): https://developer.playcanvas.com/user-manual/optimization/runtime-devicepixelratio
- Chart.js — devicePixelRatio config: https://www.chartjs.org/docs/latest/configuration/device-pixel-ratio.html
- Chromium issue 40839122 — Jank in Android WebView with canvas rendering: https://issues.chromium.org/40839122
- Stack Overflow — visibilitychange in Android WebView gaps: https://stackoverflow.com/questions/34936777/why-does-visibilitychange-event-not-fire-for-android-webview
- Repo: `specs/007-sona-complete-design/clarification-waveform-visualization-style.md` (§4 canvas size, §5 perf, §6 decisions)
- Repo: `specs/005-adopt-design-sona/contracts/status-surface.md` (S5, S8, S11)
- Repo: `specs/005-adopt-design-sona/data-model.md` (MovementSpec)
- Repo: `.specify/memory/constitution.md` (v2.0.0 — reduced-motion guard, ≤1 loop, no new surface)

Prepared: 2026-10-28. Candidate C3 closed — evidence-backed recommendation delivered (cadence: event-driven @ ~20 Hz; fps cap: 30 fps; DPR cap: 2×; visibilitychange pause + reduced-motion freeze).