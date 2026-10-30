# Clarification — Candidate C5: Waveform Accessibility (Spec 007, research)

**Status:** READ-ONLY research — no edits performed. Proposes findings and a recommendation only.

**Source evidence base:**
- `src-tauri/src/status_view.html` (surface of record; canvas absent today; progress = `#progress` div)
- `specs/005-adopt-design-sona/contracts/status-surface.md` S4 (aria contract), S5 (movement/freeze), S8 (decorative wave aria-hidden)
- `specs/007-sona-complete-design/spec.md` FR-008 (waveform), SC-002 (no a11y regressions)
- `DESIGN-SONA/tokens.md`, constitution v2.0.0 (value-blindness amended for speed-control area only)
- MDN, W3C WAI-ARIA, WCAG 2.2, ARIA APG, Paul J. Adam canvas a11y demo

---

## 1. Question

How should the new canvas-based reactive waveform (FR-008) be exposed to assistive technology, given:
- canvas is a bitmap with no semantic content (MDN: "Canvas content is not exposed to accessibility tools");
- the surface already has a semantic `#progress` (`role="progressbar"`) plus a visually-hidden `role="status"` live region (S4);
- value-blindness forbids painted numeric rate except in the speed-control area (FR-003);
- reduced-motion must freeze, not hide (S5).

Candidate sub-questions:
(a) `role="img"` + `aria-label` on the canvas vs. `aria-hidden="true"` + parallel semantic progressbar?
(b) How to satisfy WCAG 1.1.1 (non-text content) and 4.1.2 (name/role/value) for a dynamic canvas?
(c) Reduced motion on canvas: freeze vs. hide?

## 2. How it relates to the current implementation

- **No `<canvas>` exists today** (grep confirmed zero matches). The progress visualization is a CSS div bar `#progress`/`#progressFill` with `role="progressbar"`, `aria-valuenow/min/max/valuetext` (status_view.html:278), plus a visually-hidden `<div class="sr-only" role="status" aria-live="polite" id="liveRegion">` (line 279) and a `role="status"` reason banner (line 281).
- S4 already mandates: progress = `role="progressbar"` + `aria-valuenow`/`aria-valuetext`; rate ladder value exposed only through the visually-hidden live region; reason banner single `role="status"`.
- S8 already mandates decorative wave container `aria-hidden="true"` (C-H5).
- Therefore the canvas waveform is a **new** element that must be slotted into the existing aria contract without displacing the progressbar/live-region semantics.

## 3. Evidence

### 3.1 Canvas is not semantic; AT cannot read drawn content
- MDN `<canvas>`: "The `<canvas>` element on its own is just a bitmap and does not provide information about any drawn objects. Canvas content is not exposed to accessibility tools as semantic HTML is. In general, you should avoid using canvas in an accessible website or app." (https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/canvas)
- MDN "Applying styles and colors" (Canvas tutorial): "Canvas content is not accessible to screen readers. If the canvas is purely decorative, include `role='presentation'` on the `<canvas>` element." (https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Applying_styles_and_colors)
- W3C HTML canvas wiki: canvas maps to no MSAA role; ARIA roles must be added via attributes; fallback content is the documented accessibility mechanism. (https://www.w3.org/html/wg/wiki/AddedElementCanvas)

### 3.2 `role="img"` requires an accessible name and makes descendants presentational
- WAI-ARIA 1.2 `img` role: "In order for elements with a role of `img` to be perceivable, authors MUST provide a label using `aria-label` or `aria-labelledby`." (https://www.w3.org/TR/wai-aria-1.2)
- MDN `img` role: descendants of an `img` are automatically given `role="presentation"` — "most screen readers will consider the element with `role='img'` set on it to be like a black box, and not access the individual elements inside it." (https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/img_role)
- Paul J. Adam canvas demo: `role="img"` + `aria-label` gives the canvas an accessible name surfaced in the image quick-nav list. (https://pauljadam.com/demos/canvas.html)
- Consequence: putting `role="img"` + `aria-label="Waveform"` on the canvas would (a) announce a static label that does not track the dynamic progress value, and (b) if any semantic child (e.g., an overlaid playhead button) were nested, it would become presentational. It does not convey progress value.

### 3.3 WCAG 1.1.1 Non-text Content (Level A)
- All non-text content presented to the user has a text alternative serving the equivalent purpose, except decorative content (which may be implemented so AT ignores it). (https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html)
- The waveform is a visualization of audio/progress. Its *essential* information (playback position/progress) is already (or must be) available through the semantic progressbar + live region. The ribbon shapes themselves are decorative emphasis. Hence the canvas qualifies for the "decorative" exemption → `aria-hidden="true"` (or `role="presentation"`) is permitted and correct, provided the parallel progressbar carries the equivalent.

### 3.4 WCAG 4.1.2 Name, Role, Value (Level A)
- For every UI component, name and role must be programmatically determinable; states, properties, values must be settable/notifyable. (https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html)
- A canvas alone cannot satisfy this for a dynamic value (no value property on a bitmap). The semantic `#progress` (`role="progressbar"` with `aria-valuenow/min/max/valuetext`) is the component that satisfies 4.1.2. Therefore the progressbar MUST remain a separate, non-canvas element and MUST be kept in sync with playback.

### 3.5 Exposing dynamic progress to screen readers
- MDN progressbar role: update `aria-valuenow` when the visual indicator updates; `aria-valuemin`/`aria-valuemax` define the range; omit `aria-valuenow` only when indeterminate. (https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/progressbar_role)
- WAI-ARIA 1.3: authors SHOULD update `aria-valuenow` when the visual progress indicator is updated. (https://www.w3.org/TR/wai-aria-1.3/)
- ARIA25 (W3C technique): `progressbar` is a hybrid widget/live-region but screen readers do **not** reliably auto-announce value changes; use a supplementary `aria-live` region (or update `aria-valuetext`) to convey status. (https://www.w3.org/WAI/WCAG21/Techniques/aria/ARIA25)
- ARIA APG range-related properties: keep `aria-valuenow` within `[aria-valuemin, aria-valuemax]`. (https://www.w3.org/WAI/ARIA/apg/practices/range-related-properties)
- Conclusion: keep the existing three-part pattern — (1) visible/hidden `#progress` progressbar with updated `aria-valuenow`, (2) visually-hidden `role="status"` `aria-live="polite"` live region for announcements, (3) reason banner single `role="status"`. The canvas must not replace or duplicate these.

### 3.6 Reduced motion: freeze, do not hide
- MDN `prefers-reduced-motion`: `reduce` indicates the user wants minimized motion. (https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion)
- WCAG C39 (CSS technique): use `prefers-reduced-motion` to prevent motion. (https://www.w3.org/WAI/WCAG21/Techniques/css/C39)
- WCAG 2.2.2 Pause, Stop, Hide (Level A): auto-starting motion lasting >5s that is not essential must be pausable/stoppable/hidable. (https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html)
- S5 contract (binding): freeze at a visible on-track position; invisible/opacity-zero is a FAIL; reduced-motion guard mandatory.
- Apple HIG / reduced-motion guidance: reduce motion, do not remove content. (https://developer.apple.com/design/human-interface-guidelines/accessibility)
- For canvas: guard the `requestAnimationFrame` loop with `matchMedia('(prefers-reduced-motion: reduce)')`; when reduce, do **not** hide the canvas (hiding would remove the visual + could imply progress info loss and breaks S5 "visible position" spirit), instead draw a **static snapshot** once per RMS update (or a static placeholder) and stop the rAF loop. This mirrors the S8 wave freeze (`animation:none !important; transform:none !important`) and the S5 pill freeze.

## 4. Recommendation (autonomous)

**Adopt `aria-hidden="true"` on the waveform canvas + retain the parallel semantic progressbar + live region. Do NOT use `role="img"` + `aria-label` on the canvas.**

Rationale chain:
1. Canvas is a bitmap; AT cannot read it (MDN). Any label would be a static string that cannot track dynamic progress, and would be announced as "image" noise.
2. `role="img"` makes descendants presentational (MDN img role), so it would obstruct any overlaid semantic child and still not convey value.
3. The essential, dynamic information (position/progress) is a UI component → must satisfy 4.1.2 via the existing `#progress` progressbar with live `aria-valuenow`; canvas is decorative emphasis → qualifies for the 1.1.1 decorative exemption → `aria-hidden="true"` is conformant.
4. Dynamic announcements handled by the existing visually-hidden `role="status"` `aria-live="polite"` live region (ARIA25).
5. Reduced motion: freeze the canvas rAF loop and draw a static snapshot; do not hide. Consistent with S5/S8.

### Concrete contract implications (for later implementation, per S11)
- Add to S4 aria contract: "waveform canvas carries `aria-hidden='true'` (decorative; progress exposed via parallel `#progress` progressbar + live region). No `role='img'`/`aria-label` on the canvas."
- S5 freeze clause extended to canvas: `matchMedia` guard; static draw under reduce; no hide.
- Surface inventory (data-model §SurfaceInventory) gains one element: `<canvas id="waveform" aria-hidden="true">` (total becomes 10 — requires §VII 3-file coordinated edit + justification, since S4 says exactly 9 elements; this is an unavoidable addition justified by FR-008).
- Keep `#progress` progressbar in DOM (do not remove) even though progress is visually integrated into the waveform; keep it visually-hidden or as a fallback per clarification-waveform doc §(4).
- Graceful degradation (FR-008): if RMS unavailable, render static SVG/CSS placeholder, still `aria-hidden="true"`; progressbar still updates from playback position.

### What remains a user decision
None blocking. The only judgment call — whether the canvas should be `aria-hidden` vs. `role="presentation"` — is functionally equivalent for AT (both remove from tree) and `aria-hidden="true"` is what S8 already uses for the decorative wave, so reuse for consistency. Recommend `aria-hidden="true"`.

One open (non-blocking) item for the implementer: confirm the screen-reader announcement cadence (throttle `aria-valuenow` updates to ~1/s or rely on the live region) to avoid verbosity. ARIA25 leaves this to author judgment; recommend updating `aria-valuenow` on every progress tick but announcing via the live region only on meaningful changes (e.g., every 10%). This is an implementation detail, not a spec decision.

## 5. Sources
1. MDN `<canvas>` element — https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/canvas
2. MDN Canvas tutorial "Applying styles and colors" — https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Applying_styles_and_colors
3. MDN ARIA `img` role — https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/img_role
4. WAI-ARIA 1.2 `img` role (requires accessible name) — https://www.w3.org/TR/wai-aria-1.2
5. WAI-ARIA 1.3 progressbar (`aria-valuenow` SHOULD be updated) — https://www.w3.org/TR/wai-aria-1.3/
6. MDN ARIA `progressbar` role — https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/progressbar_role
7. W3C ARIA25 technique (live region to convey progressbar status) — https://www.w3.org/WAI/WCAG21/Techniques/aria/ARIA25
8. W3C APG range-related properties — https://www.w3.org/WAI/ARIA/apg/practices/range-related-properties
9. Paul J. Adam canvas accessibility demo — https://pauljadam.com/demos/canvas.html
10. WCAG 2.1 Understanding 1.1.1 Non-text Content — https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html
11. WCAG 2.1 Understanding 4.1.2 Name, Role, Value — https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html
12. WCAG C39 (prefers-reduced-motion) — https://www.w3.org/WAI/WCAG21/Techniques/css/C39
13. WCAG 2.2 Understanding 2.2.2 Pause, Stop, Hide — https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html
14. MDN `prefers-reduced-motion` — https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion
15. Apple HIG Accessibility (reduce motion) — https://developer.apple.com/design/human-interface-guidelines/accessibility
16. W3C HTML WG wiki "AddedElementCanvas" — https://www.w3.org/html/wg/wiki/AddedElementCanvas
17. Chart.js accessibility (canvas ARIA precedent) — https://www.chartjs.org/docs/latest/general/accessibility.html

---

**No contract, spec, plan, or constitution file was modified.** This document proposes findings and decision points only; any element addition requires the S11 3-file coordinated edit.