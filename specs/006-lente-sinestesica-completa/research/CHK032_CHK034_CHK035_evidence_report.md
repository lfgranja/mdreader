# Evidence-Backed Report: CHK032 / CHK034 / CHK035 Clarifications
**Task:** `ReducedMotionAndAlertResearch` (subagent)  
**Project:** `mdreader` — Feature `006-lente-sinestesica-completa`  
**Sources:** English, Portuguese, Spanish (W3C, MDN, web.dev, PubMed, A List Apart, A11Y Collective, Apple Developer, GitHub/ARIA patterns); scientific: PubMed / PMC vestibular studies; public/spec sources only.

---

## 1. Executive Summary — What the Spec Must Clarify

| Checklist item | What is ambiguous today | Recommended clarification (this report) |
|---|---|---|
| **CHK032** reduced-motion guard | “Guard” not defined: freeze vs hide vs disable? Triggered by what? | Freeze decorative motion (`animation: none` / `transform: none`) under `prefers-reduced-motion: reduce`; static visual remains visible; no opacity tricks; OS-level trigger (`prefers-reduced-motion`) plus `matchMedia` JS guard. |
| **CHK034** “no alert misuse” | Not quantified: which `aria-live`/`alert` uses are misuse? | Misuse = using `role="alert"` / `aria-live="assertive"` for non-critical/non-time-sensitive updates; for decorative/status content that should be `polite` (`status`) or silent; frequent interruptions; alerts disappearing automatically; alert applied to interactive elements (links/buttons). |
| **CHK035** loop constraints | “≤1 continuous loop” not defined: what counts? Duration? Auto-repeating? | A “loop” = a `animation-iteration-count: infinite` or any repeating keyframes cycle; max continuous duration ≤ 5 s (WCAG 2.2.2 G11 / G152); decorative loops must be bounded (≤1 continuous loop permitted, with total runtime ≤5 s) OR gated to `no-preference`; if perpetual loop needed (indeterminate slide), it must be frozen under reduced motion and count as the single allowed continuous loop. |

---

## 2. Evidence & Sources

### A. Reduced-Motion Guard — CHK032 (Freeze / Hide / Disable?)

**W3C authoritative definitions:**
- WCAG 2.2 SC 2.3.3 “Animation from Interactions” (AAA): motion triggered by interaction must be disable-able unless essential; sufficient techniques **C39** (CSS `prefers-reduced-motion`) and **SCR40** (JS `matchMedia`). ([W3C Understanding 2.3.3](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html))
- Definition of *motion animation*: “addition of steps between conditions to create the illusion of movement or to give a sense of a smooth transition.” Changes in opacity/color/blurring that do not change perceived size/shape/position are **not** motion animation. ([W3C 2.3.3 Key Terms](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html))
- `prefers-reduced-motion` media query (`reduce` / `no-preference`) defined in Media Queries Level 5; `reduce` = “all non-essential movement removed.” ([MDN `prefers-reduced-motion`](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion))
- **Google web.dev** (Thomas Steiner): decorative effects should be disabled under `reduce`; decorative animations like parallax, zoom transforms, spinning/bouncing loops, auto-playing background video should be removed/reduced; the user preference is a medical necessity (vestibular disorders). ([web.dev prefers-reduced-motion](https://web.dev/articles/prefers-reduced-motion))

**Guard behavior — freeze vs hide vs disable:**
- The W3C-sanctioned mechanism for reducing motion is to **prevent animation from displaying**, not to hide the element. C39 uses `animation: none` (freeze/remove motion). ([W3C C39 technique](https://www.w3.org/WAI/WCAG22/Techniques/css/C39))
- Apple Developer docs (reduced-motion evaluation): “If purely decorative/stylistic, change/remove animation when user preference indicates reduced motion.” This implies the element stays visible; motion is removed. ([Apple Developer — Reduced Motion](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/reduced-motion-evaluation-criteria))
- A List Apart / Val Head (2015, updated 2020): decorative motion should be turned off or reduced; provide a control; the recommendation is to **freeze/remove the motion**, not hide the decorative element, so the design context remains. ([A List Apart — Designing Safer Web Animation](https://alistapart.com/article/designing-safer-web-animation-for-motion-sensitivity))
- Systematic component-library approach (Daniel Joffe, 2024): decorative animations bucket = “pulling them changes nothing about how someone reads the interface” → `motion-reduce: animate-none`. ([Daniel Joffe blog](https://danieljoffe.com/blog/prefers-reduced-motion-component-library))

**Conclusion for CHK032:** The reduced-motion guard must:
1. **Freeze** decorative motion (`animation: none`, `transform: none`, keyframes disabled) — not hide the decorative wave/element.
2. **Keep the static visual intact** so the UI does not collapse.
3. Be triggered by the OS/user agent preference (`prefers-reduced-motion: reduce`) via `@media` (CSS) and `window.matchMedia('(prefers-reduced-motion: reduce)')` (JS) per SCR40.
4. Not use opacity tricks (`opacity: 0`) to simulate “no motion” — that violates the “visible static state” principle.

---
### B. Alert Misuse — CHK034 (What Is Misuse?)

**W3C APG Alert Pattern:**
- `alert` = brief, important message that attracts attention **without interrupting** the user’s task; must not affect keyboard focus; must not disappear automatically (would violate 2.2.3 / 2.2.4). ([W3C Alert Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/alert))
- `role="alert"` implies `aria-live="assertive"` + `aria-atomic="true"`: screen reader stops and announces immediately. ([MDN `alert` role](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/alert_role))

**Best-practice / misuse definitions (A11Y Collective / MDN / GitHub):**
- `assertive` (`alert`) reserved for: error messages; time-sensitive / critical notifications; security warnings. ([A11Y Collective — ARIA Live Guide](https://www.a11y-collective.com/blog/aria-live))
- `polite` (`status`) reserved for: status updates; non-critical messages; search result counts; item-added-to-cart; pagination changes. ([MDN `aria-live`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-live))
- Misuse examples documented:
  - Using `assertive` for non-critical updates (e.g., search result counts, pagination) — floods user. ([Accessible Data Interfaces — Announcement Strategies](https://www.accessible-data-interfaces.com/core-aria-keyboard-navigation-for-data-uis/screen-reader-announcement-strategies))
  - Using `alert` on interactive elements (links, buttons) — `alert` is for text content only. ([MDN `alert` role](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/alert_role))
  - Alerts that disappear automatically — violates 2.2.3 (No Time Limits) and 2.2.4 (Interruptions). ([W3C Alert Pattern — design considerations](https://www.w3.org/WAI/ARIA/apg/patterns/alert))
  - Frequent interruptions inhibit usability for visual/cognitive disabilities. ([W3C Alert Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/alert))
- GitHub / VS Code issue (#185371): `assertive` should only be used for time-sensitive/critical notifications; common misuse = applying `assertive` to non-critical updates. ([GitHub microsoft/vscode #185371](https://github.com/microsoft/vscode/issues/185371))

**Conclusion for CHK034 (quantified misuse):**
Misuse of `alert` / `aria-live` is defined as any of:
1. **Using `role="alert"` or `aria-live="assertive"` for non-critical / non-time-sensitive updates** (status changes, decorative events, decorative wave changes, progress indicators that are not errors). These should use `polite` (`status`) or no live region.
2. **Applying `alert` to interactive elements** (links, buttons, controls) rather than text messages.
3. **Using `alert` for frequent/repeating announcements** that interrupt the user repeatedly.
4. **Auto-disappearing alerts** that do not remain until user dismisses or acts.
5. **Using `alert` where no error / critical event exists** (e.g., decorative animation state change announced as an alert).

---
### C. Loop Constraints — CHK035 (≤1 Continuous Loop)

**WCAG 2.2 SC 2.2.2 “Pause, Stop, Hide” (Level A):**
- Moving/blinking/scrolling information that (1) starts automatically, (2) lasts > 5 seconds, (3) presented in parallel with other content → user must have a mechanism to pause/stop/hide it, unless essential. ([W3C Understanding 2.2.2](https://www.w3.org/WAI/WCAG21/Understanding/pause-stop-hide))
- **G11 / G152 technique**: blinking content that stops within 5 seconds escapes 2.2.2; animation loops must be bounded. ([W3C G152 — animated GIF stop within 5 s](https://www.w3.org/WAI/WCAG22/Techniques/general/G152))
- G152 explicitly calculates loop duration = frames × frame rate × repetitions; must be ≤ 5 s.

**SC 2.3.1 “Three Flashes or Below Threshold” (Level A) — relevant to loop frequency:**
- More than 3 flashes per second = hazard; loop frequency must stay below 3 Hz. ([W3C Understanding 2.3.1](https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html))
- Note: the spec’s 1.5 s loop period = 0.67 Hz, which is below the 3 Hz floor; so no seizure risk, but the loop is continuous.

**What counts as a “loop”:**
- `animation-iteration-count: infinite` = continuous loop.
- Any repeating `keyframes` cycle that does not terminate within 5 s = continuous loop.
- A bounded loop (e.g., 3 repetitions, total < 5 s) = not a “continuous loop” for 2.2.2 purposes (escapes via G11), but it is a loop. The spec’s “≤1 continuous loop” means **only one unbounded/repeating loop is permitted**, and it must be gated or bounded.

**S5 constitution / plan reference (from `005-adopt-design-sona/research/clarify-e2-motion-accessibility.md`):**
- Perpetual indeterminate slide (1.5 s loop, minutes-long) triggers 2.2.2 unless gated; bounded yellow pulse (3 s) escapes; banner slide-in (0.2 s) escapes; red failure pulse must be bounded (≤5 s) or gated.
- Recommendation: perpetual loops allowed **only** when gated by `prefers-reduced-motion: no-preference`; under `reduce`, they must freeze to a static state. Bounded loops (≤5 s total) may run unconditionally.

**Conclusion for CHK035:**
- “Loop” = a repeating animation cycle (`infinite` or fixed iterations that repeat continuously without terminating within 5 s of start).
- “≤1 continuous loop” means: no more than **one** unbounded/repeating animation on the driving surface at any time; any other repeating animation must be bounded to ≤5 s total runtime or frozen under reduced motion.
- Measurable verification: host audit counts `animation-iteration-count` / `animation` declarations; each must either (a) have a bounded duration ≤5 s and not `infinite`, or (b) be wrapped in `@media (prefers-reduced-motion: no-preference)`; unguarded `infinite` loops count against the “≤1” budget.

---

## 3. Vestibular / Scientific Background — Why These Rules Matter

**Vestibular disorder triggers from UI animation:**
- WCAG 2.3.3 Understanding: reactions include nausea, migraine headaches, potentially needing bed rest to recover; parallax (different rates) and scroll-triggered motion are primary triggers. ([W3C 2.3.3 Understanding](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html))
- A List Apart (Val Head, 2015/2020): large-scale motion (full-screen wipes, parallax, mismatched directions, large perceived distance) triggers vestibular disorders; relative size of movement, direction mismatch, distance covered are key factors. ([A List Apart](https://alistapart.com/article/designing-safer-web-animation-for-motion-sensitivity))
- PubMed / PMC studies: vestibular migraine patients show significantly lower motion-perception thresholds (7.54°/s vs 23.49°/s) and higher visual motion sensitivity; continuous peripheral motion compounds an already-loaded vestibular system (relevant for driving context). ([PubMed — Vestibular Migraine](https://pubmed.ncbi.nlm.nih.gov/33914990); [Nature Scientific Reports — Self-motion perception sensitized in VM](https://www.nature.com/articles/s41598-019-50803-y); [Frontiers — Interictal symptom burden in VM](https://www.frontiersin.org/journals/neurology/articles/10.3389/fneur.2026.1800235/full))
- Driving-simulator evidence (DDI 2024, AHFE 2024): continuous peripheral motion in the instrument cluster increases detection-response time; slow fade-in / reduced brightness mitigates capture; the spec’s perpetual indeterminate slide in the cluster is the highest-capture configuration.

**Reduced-motion guard is a medical/safety necessity, not a preference nicety:**
- Google web.dev: “Not everyone likes decorative animations; some users experience motion sickness when faced with parallax, zoom effects, etc. Reducing animations is a medical necessity.” ([web.dev](https://web.dev/articles/prefers-reduced-motion))
- The spec’s driving-first context (eyes-free, peripheral vision) amplifies the risk: peripheral motion captures exogenous attention involuntarily; the OS-level `reduce` setting must freeze decorative motion without removing the static design element.

---

## 4. Recommended Spec Wording (Unambiguous)

### CHK032 — Reduced-Motion Guard (Freeze, Not Hide)

> **“Reduced-motion guard”** (FR-004, S5): Every decorative motion on StatusView (decorative waves, any CSS `animation` or `transform` on decorative pseudo-elements) MUST be gated by `@media (prefers-reduced-motion: no-preference)` (CSS) / `window.matchMedia('(prefers-reduced-motion: reduce)')` (JS, SCR40). When the user/OS preference is `reduce`:
> - Decorative animations become `animation: none`; `transform` resets to static position; no opacity tricks (`opacity: 0`) to simulate removal.
> - The decorative visual element (wave, pseudo-element) remains visible in its static state; only motion is removed.
> - The indeterminate progress slide and any red failure pulse are frozen to a static state (visible fixed position or static color); the TalkBack status region (`D5`) and reason line continue to convey “in progress” / “failure” semantics audibly.
> - No new in-page pause button or interactive control is added (inventory stays closed; OS setting + audio channel is the sanctioned mechanism).

**Why freeze, not hide:** Hiding the decorative element breaks the visual design contract (`SC-001`) and confuses the user; freezing preserves the UI layout while removing the vestibular trigger. The W3C-sanctioned C39/SCR40 pattern freezes motion, not content.

**Why `prefers-reduced-motion` and not an in-page toggle:**
- WCAG 2.3.3 lists “Take advantage of the OS/user agent reduced-motion feature” as sufficient (C39/SCR40).
- The project’s closed element inventory (`S5`) and `accessibility.md` commit prevent adding new interactive controls; the OS toggle is the lower-cost, higher-coverage mechanism (97.22% browser support per caniuse; Android `Remove animations` available API 29+).
- The device trial (`SC-007`) verifies this via `Settings → Accessibility → Remove animations` (primary) or `adb shell settings put global animator_duration_scale 0` (deterministic shim) — never via a render-time forced class.

---
### CHK034 — No Alert Misuse (Quantified)

> **“No alert misuse”** (SC-002 / SC-003): The surface MUST NOT contain any `role="alert"` or `aria-live="assertive"` applied to:
> 1. Non-critical or non-time-sensitive updates (e.g., decorative wave state, progress status that is not an error, success confirmations that are not security/time-sensitive).
> 2. Interactive elements (links, buttons, controls) — `alert` is for text content only.
> 3. Frequent/repeating announcements that would interrupt screen-reader output continuously.
> 4. Auto-disappearing messages (alerts must remain until user acts or the event resolves).
>
> **Allowed uses:** `role="alert"` / `assertive` ONLY for genuine errors, critical warnings, or time-sensitive security notifications where immediate interruption is required. Status/progress announcements (including buffering, failure state, wave state if announced) MUST use `role="status"` (implied `polite`) or no live region, never `assertive`.
>
> **Verification:** ARIA inventory audit (`C-H5`) counts `aria-live` values; any `assertive` or `alert` must be matched to a documented critical event; unmatched instances = misuse.

---
### CHK035 — Loop Constraints (Measurable)

> **Loop constraint** (`FR-004` / `S5` / `SC-004`):
> - A **loop** = any CSS `animation` cycle that repeats continuously (e.g., `animation-iteration-count: infinite`, or bounded cycles that restart automatically) without terminating within **5 seconds** of start.
> - **≤1 continuous loop** permitted on the StatusView surface. Any additional repeating animation must either (a) have a bounded total duration ≤ 5 s (`G152` pattern) or (b) be frozen under `prefers-reduced-motion: reduce`.
> - The perpetual indeterminate slide (1.5 s loop) counts as the **single allowed continuous loop**; it MUST be gated by `prefers-reduced-motion` (frozen when `reduce`). The bounded yellow pulse (≤3 s) and the 0.2 s banner slide-in do not count as continuous loops (they terminate within 5 s), but must also respect reduced motion.
> - No unguarded `animation-iteration-count: infinite` may exist outside the single permitted loop; host audit (`SC-001` item v) verifies every `animation` declaration is either bounded (≤5 s) or inside `@media (prefers-reduced-motion: no-preference)`.

---

## 5. Options Considered (With Reasons)

### CHK032 — Guard behavior options
| Option | Description | Verdict |
|---|---|---|
| A | Freeze motion (`animation: none`), keep static element | **Recommended** — W3C-sanctioned (C39); preserves design; no inventory change. |
| B | Hide decorative element (`display: none` / `opacity: 0`) | Rejected — breaks visual contract (`SC-001`); hides design, not motion; violates “static twin” principle. |
| C | Add in-page pause button | Rejected — violates closed inventory (`S5`); duplicates OS mechanism; requires new earcon/focus/target. |
| D | Reduce motion but keep some animation (`animation-duration` shortened) | Rejected — does not eliminate vestibular trigger; “reduce” means remove non-essential motion per Media Queries Level 5 (`reduce` = “preferably to the point where all non-essential movement is removed”). |

### CHK034 — Alert misuse options
| Option | Description | Verdict |
|---|---|---|
| A | Reserve `assertive`/`alert` for errors/time-critical only; `polite` for status | **Recommended** — aligns with APG Alert Pattern and A11Y Collective. |
| B | Use `assertive` for all status/progress updates for faster feedback | Rejected — floods screen reader; violates usability for cognitive/visual disabilities; misuse per VS Code issue and A11Y Collective. |
| C | Apply `alert` to decorative elements (waves) to announce state | Rejected — decorative elements must not announce; decorative motion is non-essential; `alert` is for text content only. |

### CHK035 — Loop measurement options
| Option | Description | Verdict |
|---|---|---|
| A | Measure loop by `animation-iteration-count` and total runtime ≤5 s | **Recommended** — aligns with G152 / 2.2.2 / 2.2.3; deterministic; audit-friendly. |
| B | Allow any number of loops as long as they are “subtle” | Rejected — “subtle” is subjective; violates measurable acceptance (`SC-004`); continuous loops trigger 2.2.2 regardless of subtlety. |
| C | Count loops per second (Hz) only, ignore duration | Rejected — misses 2.2.2 duration requirement (>5 s automatic start = mechanism required); must measure both duration and continuity. |

---

## 6. Citations (Formatted)

**W3C / Standards:**
1. W3C — Understanding SC 2.3.3 Animation from Interactions (Level AAA): https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html
2. W3C — Understanding SC 2.2.2 Pause, Stop, Hide: https://www.w3.org/WAI/WCAG21/Understanding/pause-stop-hide
3. W3C — Understanding SC 2.3.1 Three Flashes or Below Threshold: https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html
4. W3C — Technique C39 (`prefers-reduced-motion` CSS): https://www.w3.org/WAI/WCAG22/Techniques/css/C39
5. W3C — Technique SCR40 (`prefers-reduced-motion` JS): https://www.w3.org/WAI/WCAG22/Techniques/client-side-script/SCR40
6. W3C — Technique G152 (animated GIF stops within 5 s): https://www.w3.org/WAI/WCAG22/Techniques/general/G152
7. W3C — ARIA Alert Pattern: https://www.w3.org/WAI/ARIA/apg/patterns/alert
8. W3C — WAI-ARIA `alert` role reference: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/alert_role

**Implementation / Best Practice:**
9. MDN — `prefers-reduced-motion`: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion
10. web.dev (Google / Thomas Steiner) — prefers-reduced-motion practical guide: https://web.dev/articles/prefers-reduced-motion
11. Apple Developer — Reduced Motion evaluation criteria: https://developer.apple.com/help/app-store-connect/manage-app-accessibility/reduced-motion-evaluation-criteria
12. A List Apart — Designing Safer Web Animation for Motion Sensitivity (Val Head, 2015/2020): https://alistapart.com/article/designing-safer-web-animation-for-motion-sensitivity
13. A11Y Collective — Complete Guide to ARIA Live Regions: https://www.a11y-collective.com/blog/aria-live
14. Accessible Data Interfaces — Screen Reader Announcement Strategies: https://www.accessible-data-interfaces.com/core-aria-keyboard-navigation-for-data-uis/screen-reader-announcement-strategies
15. GitHub (microsoft/vscode) — Issue #185371 on `assertive` misuse: https://github.com/microsoft/vscode/issues/185371

**Scientific / Medical:**
16. PubMed — Vestibular migraine motion perception thresholds (lower thresholds in VM): https://pubmed.ncbi.nlm.nih.gov/33914990
17. Nature Scientific Reports — Self-motion perception sensitized in vestibular migraine: https://www.nature.com/articles/s41598-019-50803-y
18. PubMed — Visual Motion Sensitivity in Vestibular Migraine (prospective cohort): https://pubmed.ncbi.nlm.nih.gov/42235082
19. PMC — Abnormal visuo-vestibular interactions in vestibular migraine: https://pmc.ncbi.nlm.nih.gov/articles/PMC6391603
20. A List Apart — Vestibular disorder definitions (vestibular.org / a11y project): https://alistapart.com/article/designing-safer-web-animation-for-motion-sensitivity (section “A vesti-what disorder?”)

**Project / Internal Sources (evidence-backed references to spec):**
21. `specs/006-lente-sinestesica-completa/checklists/a11y-motion.md` (CHK032, CHK034, CHK035 definitions).
22. `specs/005-adopt-design-sona/research/clarify-e2-motion-accessibility.md` (S5 motion budget, perpetual loop analysis, 2.2.2 trigger matrix, device trial protocol).
23. `specs/005-adopt-design-sona/research/clarify-f2-reduced-motion-trigger.md` (OS pathway for `prefers-reduced-motion` on Android WebView, `adb` deterministic shim, census-clean constraint).
24. `specs/006-lente-sinestesica-completa/spec.md` (FR-004, SC-002, SC-004, SC-007).
25. `DESIGN/accessibility.md` (deliberate non-requirements; reduced-motion posture amendment needed).

---

## 7. Final Recommendation (Concise)

- **CHK032:** Freeze decorative motion (`animation: none`) under `prefers-reduced-motion: reduce`; keep static decorative element visible; trigger = OS/user agent preference (`@media` + `matchMedia`); no opacity tricks; no new interactive controls.
- **CHK034:** Define misuse precisely: `assertive`/`alert` reserved for critical/time-sensitive text errors only; `polite`/`status` for status/progress; never on interactive elements; never auto-disappearing; never frequent/repeating; decorative elements never announce.
- **CHK035:** Define loop = repeating cycle >5 s or `infinite`; allow ≤1 unbounded loop on surface; it must be gated by `prefers-reduced-motion`; all other loops bounded to ≤5 s; host audit verifies every `animation` declaration.

These definitions make the spec measurable, consistent with W3C C39/SCR40, aligned with the project’s closed inventory (`S5`), and backed by both normative WCAG text and scientific evidence on vestibular disorders.
