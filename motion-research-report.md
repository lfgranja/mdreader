# Comprehensive Research Report — CSS Animation Accessibility & "Subtle" Quantification

**Prepared for:** Spec clarification (FR-004 / FR-005, CHK009) — "subtle CSS animations"
**Sources:** W3C WCAG 2.2/2.3, MDN, web.dev (Google), CSS-Tricks, Smashing Magazine, peer-reviewed vestibular literature, Portuguese/Spanish accessibility docs.
**Languages covered:** English, Portuguese (PT), Spanish (ES), scientific (English/abstracts).
**Date:** 2026-10-27

---

## 1. Key Findings (Evidence-Backed)

### 1.1 WCAG Normative & Informative Definitions

| Source | URL | Finding |
|---|---|---|
| W3C — Understanding SC 2.3.3 (AAA) | https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions | **Non-essential motion triggered by interaction must be disable-able.** Essential = required for functionality/information; decorative/parallax/spin are non-essential. Vestibular reactions include dizziness, nausea, migraine, bed rest. |
| W3C — Technique C39 (CSS) | https://www.w3.org/WAI/WCAG22/Techniques/css/C39 | `prefers-reduced-motion` is a **sufficient technique** for SC 2.3.3. Must suppress motion, not just slow it. |
| W3C — Technique SCR40 (JS) | https://www.w3.org/WAI/WCAG22/Techniques/client-side-script/SCR40 | JS detection of `prefers-reduced-motion` must manually stop animations; CSS media query auto-triggers only for CSS-based animations. |
| W3C — SC 2.2.2 (AA) | https://www.w3.org/TR/WCAG22/#pause-stop-hide | Auto-starting motion must be pausable/stoppable/hidable if >5 seconds or parallel to other content. |
| W3C — SC 2.3.1 (A) | https://www.w3.org/TR/WCAG22/#three-flashes-or-below-threshold | Flashing >3 times/second is a seizure risk; decorative animation must stay below this rate. |
| W3C — Animation from Interactions (WG wiki) | https://www.w3.org/WAI/GL/wiki/Animation_caused_by_user_interaction | **"Significant animation" definition:** continues >3 seconds, synchronized with user action (e.g., parallax), affects **>1/3 of viewport**. |

### 1.2 Quantified Thresholds — What the Literature Says

| Dimension | Quantified "Safe / Subtle" Range | Quantified "Significant / Risk" Threshold | Source / Note |
|---|---|---|---|
| **Duration** | 100–400 ms standard; 250 ms common for micro-feedback; 50–200 ms for small elements. | >500 ms becomes cumbersome; >3 s = "significant" per WCAG WG; >5 s requires pause/stop/hide (WCAG 2.2.2 AA). | NN/G (https://www.nngroup.com/articles/animation-duration/), Material Design (https://m1.material.io/motion/duration-easing.html), Smashing Magazine (https://www.smashingmagazine.com/2023/11/creating-accessible-ui-animations/) |
| **Motion distance / area** | <10 px translate = low risk; <~10% viewport = generally safe; small scale delta (1.02) safe. | >1/3 viewport = significant / vestibular trigger; large fly-ins (60vw translate) = high risk; scale(1)→scale(2) = high risk (zoom/vection). | css-animation.com (https://www.css-animation.com/accessible-motion-architecture/vestibular-safe-motion-patterns), WCAG WG wiki |
| **Loop / autoplay** | User-triggered only; non-looping preferred; instant or very short loops (<3 s). | Continuous loop without user control = high risk; autoplay = must be disabled under `reduce`; carousels = remove auto-advance. | web.dev (https://web.dev/learn/accessibility/motion), MDN (https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion) |
| **Flash / frequency** | <3 flashes/second; subtle opacity changes only. | ≥3 flashes/second fails WCAG 2.3.1 A. | W3C SC 2.3.1 |
| **Motion type / property** | `opacity` fade; tiny `translate` (<10 px); `color` / `background-color` change. | `rotate` continuous; `scale` large; multi-layer `translate` (parallax); `translateX(60vw)` or full-page sweep; autoplay. | css-animation.com table; web.dev; Smashing Magazine categories |

### 1.3 Scientific / Peer-Reviewed Evidence on Vestibular Impact

| Source / Study | URL / Citation | Finding |
|---|---|---|
| Vestibular dysfunction prevalence | https://vestibular.org/article/what-is-vestibular/about-vestibular-disorders/ (cited by web.dev) | By age 40, >35% of adults experience some vestibular dysfunction; web motion can trigger nausea, dizziness, migraine. |
| Vestibular perceptual thresholds (systematic review) | https://www.sciencedirect.com/science/article/pii/S0966636223014339 | Threshold elevation after visual motion exposure confirms visual-vestibular mismatch is measurable and clinically significant. |
| Persistent postural-perceptual dizziness (PPPD) | https://pmc.ncbi.nlm.nih.gov/articles/PMC11136745 | PPPD patients have significantly lower motion discrimination thresholds (0.28 vs 0.39 in controls), meaning **even subtle motion is more triggering** for this group. |
| Clinical characteristics of visual motion hypersensitivity | https://pmc.ncbi.nlm.nih.gov/articles/PMC10349011 | Elevated motion discrimination thresholds in clinical groups; motion hypersensitivity is a documented clinical condition, not preference only. |
| W3C Understanding 2.3.3 | https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions.html | Vestibular reactions include dizziness, nausea, migraine headaches, and **potentially requiring bed rest** — severity justifies strict guardrails. |

### 1.4 `prefers-reduced-motion` Behavior — What "Reduce" Actually Means

| Source | URL | Recommended Behavior |
|---|---|---|
| MDN — prefers-reduced-motion | https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion | `reduce` = user enabled OS setting. Must minimize motion; **not necessarily zero motion**, but all non-essential motion removed/replaced. `opacity` and `color` transitions are acceptable replacements. |
| web.dev — prefers-reduced-motion | https://web.dev/articles/prefers-reduced-motion | "Preferably to the point where all non-essential movement is removed." For decorative effects: disable or replace with fade. For essential feedback: slow, not remove (e.g., loader: reduce rotation speed, eliminate scaling, cap duration). |
| CSS-Tricks / Val Head | https://css-tricks.com/accessible-web-animation-the-wcag-on-animation-explained | `reduce` does not mean `none`. Essential state-change animations can stay but must be slowed or simplified (e.g., opacity cross-fade instead of slide). |
| Smashing Magazine (Mercado Libre case) | https://www.smashingmagazine.com/2023/11/creating-accessible-ui-animations/ | Classified animations into 3 groups: (1) non-motion (color/opacity) — safe; (2) non-essential motion (parallax, loop, autoplay, scale >1/3 screen) — **remove entirely**; (3) essential motion (loader) — adapt (reduce rotation, eliminate scale, cap 5s). |
| AccessScan / web.dev | https://accessscan.app/guides/reduced-motion-accessibility | "`reduce` does not mean 'no animation ever.' It means avoid large, directional movement." Best practice: freeze/remove decorative; slow or fade essential. |

### 1.5 Portuguese & Spanish Evidence

- **Português:** https://blog.openreplay.com/pt/prefers-reduced-motion-animacao-acessivel — confirma que `reduce` exige substituição de movimentos não essenciais, não apenas redução de velocidade; recomenda `animation: none` ou substituição por opacidade para decorações.
- **Español:** MDN ES (https://developer.mozilla.org/es/docs/Web/CSS/@media/prefers-reduced-motion) — explica que `reduce` indica preferencia del sistema operativo para minimizar movimiento no esencial; animações de escala/panorama são gatilhos vestibulares.
- **Español / W3C:** https://www.w3.org/WAI/WCAG22/Techniques/css/C39.html (versión española del documento técnico) — confirma técnica suficiente para SC 2.3.3.

---

## 2. Options — How to Specify "Subtle" in the Spec

### Option A: Vague / Subjective (Current Risk)
- **Spec language:** "Subtle CSS animations are allowed."
- **Problem:** No quantification. A developer can claim a 60vw parallax slide, 3-second loop, or scale(2) is "subtle." Fails accessibility and is untestable.
- **Evidence:** WCAG WG defines "significant" (not "subtle") as >3 s, >1/3 viewport, user-triggered. Without the inverse definition, ambiguity remains.

### Option B: Duration-Only Cap (Partial)
- **Spec language:** "Subtle animations must be <300 ms and not loop."
- **Problem:** Duration alone does not prevent vestibular triggers. A 250ms `translateX(60vw)` or `rotate(360deg)` is still a high-risk vection trigger per css-animation.com. Does not cover area, property type, or reduced-motion guard.

### Option C: Multi-Axis Quantified Spec + Reduced-Motion Guard (Recommended)
- **Spec language:** See Section 4 (Recommendation). Defines subtle by: (1) property allowed, (2) max distance / viewport %, (3) max duration, (4) loop/autoplay rules, (5) mandatory `prefers-reduced-motion` behavior.
- **Advantage:** Unambiguous, testable, aligns with WCAG 2.3.3 / 2.2.2 / 2.3.1, backed by scientific evidence.

---

## 3. Scientific Backing Summary

- **Vestibular mechanism:** Visual-vestibular mismatch (vection) is triggered by large, coherent directional flow across the visual field — not by frame rate or performance. Smooth 60fps parallax is *more* harmful than janky 12fps parallax (css-animation.com).
- **Clinical evidence:** PPPD patients show lower motion discrimination thresholds (0.28 ± 0.11 vs 0.39 ± 0.12, p=0.001); motion hypersensitivity is a documented clinical condition (PMC11136745, PMC10349011).
- **Prevalence:** >35% of adults by age 40 experience vestibular dysfunction (vestibular.org, cited by web.dev). "Subtle" for a healthy user may be disabling for another.
- **Severity:** Vestibular reactions include nausea, dizziness, migraine, and bed rest (W3C Understanding 2.3.3). Therefore a conservative (strict) quantification for decorative motion is justified.

---

## 4. Recommendation — What the Spec SHOULD Say for "Subtle CSS Animations"

Based on all evidence above, the spec (FR-004 / FR-005 / CHK009) should replace ambiguous "subtle" with an **exact, multi-axis definition** and an **unconditional reduced-motion guard**.

### 4.1 Mandatory Spec Language (Recommended)

```markdown
"Subtle decorative CSS animation" is defined as ALL of the following:

A. ALLOWED PROPERTIES / EFFECTS ONLY:
   - opacity (fade in/out)
   - color / background-color (gradual change)
   - transform: translate (distance ≤ 10 px OR ≤ 5% of viewport, whichever is larger)
   - transform: scale (delta ≤ 1.05, i.e., 5% growth max)
   - No rotate, no continuous spin, no zoom (scale > 1.05), no parallax (multi-layer translate at different rates).

B. DURATION CONSTRAINT:
   - Total animation duration ≤ 250 ms for user-triggered micro-feedback (hover, focus, click).
   - Maximum 400 ms for small element entrance/exit.
   - Any animation > 3 seconds is considered SIGNIFICANT and is NOT "subtle"; it must be disable-able per WCAG 2.3.3 and must not autoplay without user control.

C. LOOP / AUTOPLAY CONSTRAINT:
   - No infinite loops (animation-iteration-count: infinite) for decorative motion.
   - No autoplay motion (animation-play-state: running by default on load/scroll without user trigger) for decorative effects.
   - User-triggered only; must respect prefers-reduced-motion.

D. AREA / VIEWPORT CONSTRAINT:
   - Animated element must not cover > 1/3 of viewport during motion.
   - Translation distance must not exceed ~10% of viewport dimension in any direction.

E. REDUCED-MOTION GUARD (MANDATORY):
   - All "subtle" decorative animations MUST respond to @media (prefers-reduced-motion: reduce).
   - Under reduce: decorative motion must be removed or replaced by an instant/static state or an opacity-only fade (duration ≤ 200 ms, no translate/scale displacement).
   - Essential animations (e.g., loading indicator) MAY stay under reduce but must: (1) eliminate rotation/scale, (2) cap duration ≤ 5 s, (3) replace with opacity pulse or static indicator.

F. FLASH / SEIZURE GUARD:
   - No flashing or rapid luminance changes > 3 times per second. Any decorative pulse must stay below this rate.
```

### 4.2 What "Subtle" Does NOT Mean (Clarify in Spec)

- Subtle ≠ safe: a 250 ms full-screen slide (`translateX(100vw)`) is subtle by duration but high-risk by displacement.
- Subtle ≠ reduced-motion exempt: all decorative motion, however small, must honor `prefers-reduced-motion: reduce`.
- Subtle ≠ essential: decorative animations must be removable; only functionality-critical animations (e.g., state-change feedback, loader progress) may remain under reduced motion, and then only in adapted form.

---

## 5. Implementation Pattern (CSS Example for Spec Reference)

```css
/* Subtle decorative animation — allowed ONLY under no-preference */
@media (prefers-reduced-motion: no-preference) {
  .subtle-fade-in {
    animation: subtle-fade 200ms ease-out both;
  }
  @keyframes subtle-fade {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }
}

/* MANDATORY reduced-motion fallback */
@media (prefers-reduced-motion: reduce) {
  .subtle-fade-in {
    animation: none;
    opacity: 1;
    /* Or, if continuity is needed: */
    /* animation: subtle-opacity 150ms ease-out both; */
  }
  @keyframes subtle-opacity {
    from { opacity: 0; }
    to   { opacity: 1; }
  }
}
```

---

## 6. Testing / Verification Methods for Spec Compliance

| Test Method | Tool / Approach | What to Verify |
|---|---|---|
| **CSS Media Query Emulation** | Chrome DevTools → Rendering → Emulate `prefers-reduced-motion: reduce` | All decorative animation collapses to static/fade; no sweep remains. |
| **Animation Panel** | DevTools → Animations | Capture triggered animations; verify duration ≤ 250 ms, translate distance ≤ 5% viewport, no loop, no rotate. |
| **Viewport Area Check** | Manual / automated (measure element bounding box vs viewport) | Animated region never exceeds 1/3 viewport. |
| **Reduced-Motion User Preference** | OS-level: macOS System Settings → Accessibility → Reduce motion; Windows → Ease of Access → Display; Android → Accessibility → Remove animations. | Site responds without reload; decorative motion removed. |
| **Flash / Frequency Check** | PEAT (Photosensitive Epilepsy Analysis Tool) for extreme cases; manual frame counting for decorative pulses. | <3 flashes/second. |
| **Manual Vestibular Field Check** | Normal viewing distance; trigger each animation. Ask: does a substantial region of screen flow coherently in one direction? | If yes → fails "subtle" definition regardless of duration. |

---

## 7. Final Evidence-Based Conclusion

The term "subtle CSS animations" is **unacceptably ambiguous** without quantification. Scientific evidence (vestibular thresholds, PPPD studies, prevalence >35%) and normative accessibility standards (WCAG 2.2/2.3, MDN, web.dev) all point to the same conclusion:

- **Decorative motion must be quantified** (duration, distance, property, area, loop, autoplay) and **must honor `prefers-reduced-motion: reduce` unconditionally**.
- **"Subtle" does not grant exemption** from accessibility requirements; it only defines a stricter subset of allowed motion.
- **The recommended spec language (Section 4.1)** converts ambiguity into an auditable, testable contract.

**Recommendation to project (FR-004 / FR-005):** Adopt Option C (multi-axis quantified spec + mandatory reduced-motion guard). Remove any spec clause that permits "subtle" decorative animation without the exact constraints listed above.

---

*Report compiled from 8+ separate search rounds covering English, Portuguese, Spanish, scientific literature, W3C specifications, MDN, web.dev, and peer-reviewed clinical studies.*
