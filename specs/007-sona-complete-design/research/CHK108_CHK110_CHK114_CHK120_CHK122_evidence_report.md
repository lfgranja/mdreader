# Evidence-Backed Report: CHK108 / CHK110 / CHK114 / CHK120 / CHK122 — Accessibility Quantification (Candidate B)

**Task:** `CandidateB_AccessibilityQuantification` (subagent)
**Project:** `mdreader` — Feature `007-sona-complete-design`
**Sources:** W3C WCAG 2.1/2.2, MDN, WAI-ARIA APG, Microsoft forced-colors docs, project contracts 005/006/007; English + Portuguese context.

---

## 1. Executive Summary

| Checklist item | Gap | Recommended resolution | Autonomy |
|---|---|---|---|
| **CHK108** keyboard nav for repeat/transport | No tab order/activation spec | Tab order repeat→prev→play→next→speed; native buttons; Enter/Space; no traps | **Autonomous** |
| **CHK110** contrast quantified | No ratios/color pairs | Text ≥4.5:1 (normal) / ≥3:1 (large); UI non-text ≥3:1; measured pairs all pass | **Autonomous** |
| **CHK114** focus-ring visibility | Not quantified | ≥2 CSS px thickness; ≥3:1 change-of-contrast; 2px offset; shape not color-only | **Autonomous** |
| **CHK120** keyboard testable | No acceptance criteria | Enumerated test list (tab order, Enter/Space, no trap, focus visible) | **Autonomous** |
| **CHK122** forced-colors/high-contrast | Only minimal guard from 006 | Minimal guard expanded: decorative static/outlined, layout preserved, system colors, canvas fallback | **Autonomous** (aligns with 006 CHK044 Option A) |

No item requires a user decision; all can be resolved with concrete measurable values below.

---

## 2. Evidence & Sources

### A. WCAG contrast (CHK110)
- WCAG 2.1/2.2 SC 1.4.3 Contrast (Minimum) (AA): text ≥4.5:1; large text (≥18pt or ≥14pt bold) ≥3:1; incidental/exempt text (inactive, decorative) exempt. (W3C Understanding 1.4.3)
- WCAG 2.1 SC 1.4.11 Non-text Contrast (AA): UI components and graphical objects that convey information / identify controls ≥3:1 against adjacent colors. Disabled/inactive components exempt. (W3C Understanding 1.4.11)
- Current palette measured (hex pairs, WCAG relative luminance):

| Foreground | Background | Ratio | Threshold | Verdict |
|---|---|---|---|---|
| `--sona-voice` #F2F0E6 | `--sona-void` #0B0F1A | 16.75:1 | 4.5:1 (text) | PASS |
| `--sona-voice` #F2F0E6 | `--sona-stage` #141B2D | 15.01:1 | 4.5:1 | PASS |
| `--sona-hush` #A9B8CE | `--sona-stage` #141B2D | 8.52:1 | 4.5:1 | PASS |
| `--sona-hush` #A9B8CE | `--sona-void` #0B0F1A | 9.51:1 | 4.5:1 | PASS |
| `--sona-etwahl` #8FD8E8 | `--sona-stage` #141B2D | 10.74:1 | 3:1 (UI) | PASS |
| `--sona-etwahl` #8FD8E8 | `--sona-void` #0B0F1A | 11.99:1 | 3:1 | PASS |
| `--sona-harp` #E8C46A | `--sona-stage` #141B2D | 10.24:1 | 3:1 (rate) | PASS |
| `--sona-celeste` #B9A6F5 | `--sona-stage` #141B2D | 8.03:1 | 3:1 | PASS |
| `--sona-jade` #5AD6A8 | `--sona-stage` #141B2D | 9.49:1 | 3:1 | PASS |
| `--signal-caution` #FFD400 | `--sona-stage` #141B2D | 11.98:1 | 3:1 | PASS |
| `--signal-failure` #FF4D63 | `--sona-stage` #141B2D | 5.30:1 | 3:1 | PASS |
| `--sona-boundary` #55688F | `--sona-stage` #141B2D | 3.07:1 | 3:1 (boundary) | PASS (margin 0.07) |
| Focus ring `--sona-etwahl` #8FD8E8 | `--sona-stage` #141B2D | 10.74:1 | 3:1 (focus) | PASS |

Recommendation: keep `--sona-boundary` only for decorative/non-essential dividers; never for text or control affordances. All meaningful text/UI pairs already exceed thresholds; no token changes required.

### B. Focus appearance (CHK114)
- WCAG 2.4.7 Focus Visible (AA): keyboard focus indicator must be visible.
- WCAG 2.2 SC 2.4.13 Focus Appearance (AAA): indicator area ≥ perimeter of 2 CSS px of the unfocused component; change-of-contrast ≥3:1 between focused/unfocused same pixels (or ≥3:1 against adjacent colors). (W3C Understanding 2.4.13)
- W3C technique C40/C41: 2 CSS px thick band is the minimum; two-color indicator ensures contrast on any background.
- Current CSS: `button:focus-visible { outline: 2px solid var(--sona-etwahl); outline-offset: 2px; }` — meets 2px thickness and ≥3:1 contrast. Recommend codifying as requirement: minimum 2 CSS px, offset 1–2 px, contrast ≥3:1, and indicator must not rely on color alone (shape/outline present). Use `outline` (not only `box-shadow`) so forced-colors can recolor it. For the new repeat control and speed control, apply the same rule.

### C. Keyboard navigation (CHK108)
- WAI-ARIA APG Button Pattern: Enter and Space activate. (W3C APG Button)
- APG Developing a Keyboard Interface: Tab moves between components; within a component use arrow keys only if composite; selection follows focus unless manual activation. For a flat transport toolbar of distinct buttons, Tab/Shift+Tab between buttons is correct; no roving tabindex required.
- No keyboard traps: Tab from last control exits to next focusable; Shift+Tab from first exits backward.
- Speed control: native button; Enter/Space cycles speed (or opens menu); if a menu, Escape closes and returns focus to trigger. Recommend cycle-on-activation to avoid modal trap (simpler, no escape needed), but if a menu is used, Escape must close and restore focus.
- Repeat: Enter/Space restarts current section; reuse skip-back earcon.
- Waveform canvas is `aria-hidden="true"` and not focusable; progress exposed via `#progress`.
- No new focus targets beyond the 5 transport controls (repeat, prev, play, next, speed); canvas and decorative waves excluded.

### D. Forced-colors / high-contrast (CHK122)
- MDN `@media (forced-colors)`: `active` means forced colors mode active; system palette replaces author colors. `forced-color-adjust: auto` (default) lets UA recolor; `none` opts out.
- Microsoft Edge blog / Fluent UI: detect via `@media (forced-colors: active)`; use system color keywords (`CanvasText`, `Canvas`, `Highlight`, `ButtonText`) when needed; `forced-color-adjust: none` only for elements where color is essential (e.g., charts), with manual high-contrast palette.
- 006 CHK044 Option A (minimal guard): decorative static/outlined, layout preserved, text via CurrentColor, no color-only signaling.
- Recommendation (expanded but still minimal):
  - `@media (forced-colors: active)`: decorative wave container and canvas remain in layout; animation disabled; use `border: 1px solid CanvasText` (or `outline`) on decorative container so it stays visible when gradients are stripped; no `opacity: 0`.
  - Text and control labels use system colors (default forced-color-adjust: auto); do not opt out for text.
  - Focus rings remain `outline` (UA recolors to Highlight) — do not use box-shadow-only rings.
  - Canvas waveform: under forced-colors, render a static high-contrast placeholder (monochrome) or hide the canvas and show `#progress` bar as visible fallback (aria-hidden removed from fallback? Keep progressbar sr-only but visible bar). Since canvas is decorative and aria-hidden, visual fidelity is secondary; recommend freeze + `forced-color-adjust: none` on canvas with a 1px border so it does not disappear. Simpler: `@media (forced-colors: active) { #waveform { border: 1px solid CanvasText; } }`.
  - No color-only signaling: failure/caution states must include text/icon (already reason banner text); progress uses position + label.
  - Device trial verifies under `forced-colors` if supported (SC-006 style).

---

## 3. Recommended Concrete Spec Amendments

### CHK108 — Keyboard Navigation (spec amendment)
> Transport controls (repeat, prev, play/pause, next, speed) MUST be native `<button>` elements in DOM order matching visual left-to-right order. Tab order MUST be repeat → prev → play → next → speed → (exit to next focusable outside the grid). Shift+Tab reverses. Enter and Space MUST activate each. No keyboard traps: Tab from speed leaves the grid; Shift+Tab from repeat leaves backward. Waveform canvas and decorative waves are not focusable (`aria-hidden="true"`, no `tabindex`). Speed control activation cycles through the rate ladder (or opens a menu; if a menu, Escape closes and restores focus to the trigger). Repeat activation restarts the current section and reuses the skip-back earcon. No additional focus targets beyond the five controls.

### CHK110 — Contrast (spec amendment)
> Text MUST meet WCAG 1.4.3: ≥4.5:1 for normal text, ≥3:1 for large text (≥18pt or ≥14pt bold). UI components and meaningful graphics MUST meet WCAG 1.4.11: ≥3:1 against adjacent colors. Disabled/inactive controls are exempt but MUST NOT be the only means of conveying state. Decorative elements are exempt. All sanctioned color pairs in the token set (§A1) MUST be verified ≥ thresholds; `--sona-boundary` (#55688F) MUST NOT be used for text or control affordances (3.07:1, margin 0.07). Measured ratios table (§2.A) is the acceptance baseline.

### CHK114 — Focus-Ring Visibility (spec amendment)
> Every focusable control MUST display a visible focus indicator with: thickness ≥2 CSS px; contrast ≥3:1 between focused and unfocused states (or against adjacent background) per WCAG 2.4.13; offset 1–2 px; rendered via `outline` (not color/box-shadow alone) so forced-colors can recolor; must not rely on color alone (shape preserved). Current rule `outline: 2px solid var(--sona-etwahl); outline-offset: 2px;` is the minimum baseline and applies to repeat + speed controls.

### CHK120 — Keyboard Testable (acceptance criteria)
> Keyboard requirements are verifiable by:
> 1. Tab order equals [repeat, prev, play, next, speed] in DOM/visual order.
> 2. Each control activates on Enter and on Space (native button behavior).
> 3. Tab from speed moves focus to the next focusable element outside the transport grid; Shift+Tab from repeat moves to the previous.
> 4. No focus trap: focus cannot become stuck within the grid.
> 5. Each control shows a visible focus indicator (≥2px, ≥3:1) when focused via keyboard.
> 6. Speed control activation changes the rate (or opens/closes menu with Escape restore).
> 7. Repeat activation restarts the current section.
> 8. Waveform canvas receives no focus (not in tab order).
> 9. Screen-reader labels present (aria-label or text) for all five controls.

### CHK122 — Forced-Colors (spec amendment)
> Under `@media (forced-colors: active)`: decorative waves and canvas remain in layout (no `display:none`, no opacity tricks); motion frozen; decorative container gains a 1px border (`CanvasText`) to remain visible when gradients are stripped; text/control labels use system colors (default forced-color-adjust: auto); focus rings remain `outline` (UA recolors); canvas gains 1px border and static render; failure/caution states retain text (no color-only signaling); device trial verifies if supported. No new interactive elements.

---

## 4. Autonomy Verdict

- **CHK108**: Resolvable autonomously — concrete tab order and activation keys proposed.
- **CHK110**: Resolvable autonomously — ratios and color-pair table provided; no token changes needed.
- **CHK114**: Resolvable autonomously — thickness/contrast/offset quantified.
- **CHK120**: Resolvable autonomously — enumerated testable criteria.
- **CHK122**: Resolvable autonomously — minimal guard expanded; aligns with 006 CHK044 Option A; no user decision needed.

No sub-item requires a user decision.

---

## 5. Sources

1. W3C Understanding SC 1.4.3 Contrast (Minimum): https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum
2. W3C Understanding SC 1.4.11 Non-text Contrast: https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast
3. W3C Understanding SC 2.4.7 Focus Visible: https://www.w3.org/WAI/WCAG22/Understanding/focus-visible
4. W3C Understanding SC 2.4.13 Focus Appearance (AAA): https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance
5. W3C Technique C40/C41 focus indicator: https://www.w3.org/WAI/WCAG22/Techniques/css/C40
6. WAI-ARIA APG Button Pattern: https://www.w3.org/WAI/ARIA/apg/patterns/button
7. WAI-ARIA APG Developing a Keyboard Interface: https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface
8. MDN forced-colors media feature: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/forced-colors
9. MDN forced-color-adjust: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/forced-color-adjust
10. Microsoft Edge — Styling for Windows high contrast: https://blogs.windows.com/msedgedev/2020/09/17/styling-for-windows-high-contrast-with-new-standards-for-forced-colors
11. Fluent UI high contrast: https://learn.microsoft.com/en-us/fluent-ui/web-components/design-system/high-contrast
12. Project: specs/006-lente-sinestesica-completa/clarifications.md (CHK044), specs/007.../contracts/status-surface.md (A5/A8), src-tauri/src/status_view.html.
13. Measured ratios computed via WCAG relative luminance formula (Python) on the token palette in status_view.html.