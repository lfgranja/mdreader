# Clarification B3 — Contrast Verification Scope for the Restyled Surface

> **Feature:** `specs/005-adopt-design-sona` (SC-001/SC-006: closed-hex color purity on the notification + in-app StatusView/Lente)
> **Question:** should the spec (a) add its own acceptance criterion that every text/background token pairing on the restyled surface meets WCAG 2.1 AA (4.5:1), or (b) inherit the design system's declared contrast numbers (`DESIGN-SONA/tokens.md` §1) and only verify the shipped values?
> **Scope:** research note; not a spec amendment. Informs a clarification decision only.
> **Method (2026-10-24):** primary-source review (W3C WCAG 2.x Understanding + Techniques; NN/g; ACM CHI 2023; ScienceDirect 2017; Apple HIG; Material Design 3; Deque axe-core; Style Dictionary; Myndex/APCA). Host computation of all 8 closed tokens against `#0B0F1A` (void) and `#141B2D` (stage) with the WCAG 2.1 formula. Note: the task's `#05060A` is not present anywhere in the repo; the declared void is `#0B0F1A`.

## 1. WCAG 2.1 thresholds and what they rest on

- **SC 1.4.3 Contrast (Minimum), Level AA:** normal text ≥ 4.5:1; large text (≥ 18 pt, or ≥ 14 pt bold) ≥ 3:1. AAA (1.4.6): 7:1 / 4.5:1. (W3C, *Understanding SC 1.4.3*, https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html)
- **Formula:** `(L1 + 0.05) / (L2 + 0.05)`, `L = 0.2126R + 0.7152G + 0.0722B`, with per-channel linearization per IEC 61966-2-1 (sRGB). Provenance: ISO 9241-3 / ANSI-HFES-100-1988; the `0.05` term is typical viewing flare from the IEC-4WD reference. (W3C, *G18*, https://www.w3.org/WAI/WCAG21/Techniques/general/G18)
- **Threshold derivation.** 4.5:1 is the contrast a person with 20/40 acuity (the common screening-referral threshold) needs to read text without contrast-enhancing AT; 3:1 is the relaxed figure for large text (thicker strokes survive); 7:1 (AAA) is the 20/80 analogue. (W3C Understanding 1.4.3, citing Arditi-Faye; colorcontrast.org, "WCAG Contrast Requirements," 2026, https://www.colorcontrast.org/blog/wcag-contrast-requirements)
- **Critiques.** Myndex/APCA argues the WCAG 2 formula misjudges real-world contrast — it reports passing ~49% of random color pairs it tested that were actually too low (Myndex, GitHub discussion #131, https://github.com/w3c/low-vision-a11y-tf/discussions/131). WCAG 3 (Silver) proposes an APCA-based visual-contrast-of-text measure keyed to font weight/size; it is a Working Draft, not normative. (W3C, *Visual contrast of text* How-To, https://www.w3.org/WAI/GL/WCAG3/2021/how-tos/visual-contrast-of-text)
- **Named in the task but not verifiable here.** "Alexander & Travis 2021" and "Guardiola et al." were offered as examples of studies questioning WCAG 2.x adequacy; no primary source for either was located in this pass, so they are not cited below. Verifiable critiques: APCA/Myndex above plus the acuity grounding in W3C Understanding 1.4.3.

## 2. Dark-mode-specific evidence

- **Positive-polarity advantage.** Dark text on light backgrounds is read reliably faster and with fewer errors than light text on dark, even at mathematically identical ratios: halation (lateral spread of light in the retina) and astigmatic smearing degrade effective contrast at letter boundaries in dark mode. (NN/g, "Dark Mode," 2020, https://www.nngroup.com/articles/dark-mode/; colorcontrast.org, "Why Dark Mode Text Feels Hard to Read," 2026, https://www.colorcontrast.org/blog/why-dark-mode-text-hard-to-read)
- **Ambient illumination.** Under dark ambient illumination, negative-polarity (white-on-black) thresholds are significantly elevated while positive-polarity thresholds stay low. (ScienceDirect/*Appl. Ergon.* 2017, PubMed 28166901, https://pubmed.ncbi.nlm.nih.gov/28166901)
- **CHI 2023** (ACM, https://dl.acm.org/doi/10.1145/3544548.3581552): across 459 participants, light mode was read reliably faster than dark mode for both glance and paragraph reading; font-grade effects appeared in light mode. Glance-speed findings matter for a driving-first surface.
- **Material Design 3.** Surface-container roles (lowest → highest) are decoupled from elevation; text contrast is evaluated against the surface it paints on, so an elevated/raised card re-checks its foreground against the raised surface luminance. (M3 color roles, https://m3.material.io/styles/color/roles; tone-based surfaces, https://m3.material.io/blog/tone-based-surface-color-m3)
- **Apple HIG.** Minimum 4.5:1, strive for 7:1 for small text; dark mode is not exempt; test content in both appearances. (https://developer.apple.com/design/human-interface-guidelines/dark-mode)
- **Dark mode alone does not satisfy 1.4.3** — WCAG thresholds apply equally to dark themes. (BOIA, 2022, https://www.boia.org/blog/offering-a-dark-mode-doesnt-satisfy-wcag-color-contrast-requirements)

## 3. Empirical basis for 4.5:1 vs 3:1; near-black "degauss"/halation

- 4.5:1 and 3:1 are the AA normal/large-text floors derived above; AAA raises them to 7:1/4.5:1. (W3C Understanding 1.4.3)
- **Upper bound is unspecified in WCAG 2.x.** White-on-near-black (`#FFFFFF` on `#0B0F1A`) computes ≈ 20.6:1 — AAA-compliant yet perceptually harsh for extended reading; Mantlr notes white-on-near-black ≈ 17:1 can be harsh. Halation degrades effective contrast at boundaries; APCA/WCAG 3 is the only work that treats excessive contrast. (colorcontrast.org 2026; mantlr.com, "Dark Mode Design Guide," 2026, https://mantlr.com/blog/dark-mode-design-guide-color-typography-accessibility)
- For this feature (dark-only, glanceable, driving-first) the near-black + ivory pairing at 16.75:1 sits in the high-contrast band that maximizes glance legibility (CHI 2023) while accepting halation for small text — an acceptable trade-off for a non-reading surface, but it must be *measured*, not assumed.

**Host computation of the 8 closed tokens (WCAG 2.1 formula, this pass):**

| Token | Hex | On void `#0B0F1A` | Declared | On stage `#141B2D` |
|---|---|---|---|---|
| `--sona-voice` (text) | `#F2F0E6` | 16.75 | 16.75 | 15.01 |
| `--sona-hush` (text) | `#A9B8CE` | 9.51 | 9.51 | 8.52 |
| `--sona-etwahl` (flow) | `#8FD8E8` | 11.99 | 11.98 | 10.74 |
| `--sona-harp` (rate) | `#E8C46A` | 11.42 | 11.42 | 10.24 |
| `--sona-celeste` (rate) | `#B9A6F5` | 8.96 | 8.96 | 8.03 |
| `--sona-aria` (brand) | `#5AD6A8` | **10.59** | **11.20** | 9.49 |
| `--sona-boundary` (UI) | `#55688F` | 3.43 | 3.43 | **3.07** |
| `--sona-stage` (surface) | `#141B2D` | 1.19 (n/a) | — | — |

All text tokens clear 4.5:1 on both surfaces. `--sona-boundary` clears 3:1 on void (3.43) but is *marginal* on stage (3.07) — if hairlines render on raised cards, re-verify or follow tokens.md's advice and use `--sona-etwahl` for focus rings.

## 4. Verification practice: is token-pair contrast a checkable artifact?

- **axe-core** `color-contrast` rule evaluates *rendered* computed foreground/background colors against WCAG AA thresholds; standard in CI. (Deque, https://dequeuniversity.com/rules/axe/3.5/color-contrast)
- **Token-level guard.** Unit assertions over token pairs (`expect(ratio).toBeGreaterThanOrEqual(4.5)`) so a regression cannot merge; Style Dictionary builds tokens cross-platform; design-system-generator validates the token spec's WCAG AA pairs at generation time. (MFA11y token-guard write-up; benmcosker/design-system-generator, https://github.com/benmcosker/design-system-generator; Style Dictionary, https://github.com/style-dictionary/style-dictionary)
- **Two layers, not one.** axe-core runs on a rendered DOM (runtime colors), not on token JSON; a token-manifest check requires a separate host-side computation. The industry treats *both* as checkable artifacts — design sign-off alone is not the gate.

## Implications for the spec

**Recommendation: option (a), scoped — require a host-verifiable AA check of the shipped pairings; do not inherit the design-declared ratios as spec authority.**

1. **Add an AC** (amend SC-001/SC-006 or add SC-007) that every *text* token paired with its actual rendered background computes ≥ 4.5:1 under the WCAG 2.1 formula, and every *non-text* graphical token (boundary hairline, focus ring, progress track) computes ≥ 3:1 — measured by a deterministic host test over the token manifest and the two real surfaces (void `#0B0F1A`, stage `#141B2D`), not by design sign-off. This mirrors the existing closed-hex host check already mandated by SC-001/SC-006.
2. **Do not inherit declared numbers.** Declared ratios are documentation, not evidence: `--sona-aria` is declared 11.20 but computes 10.59 (-0.61). Inheriting them would bake an arithmetic error into the spec. The design doc keeps its numbers as *intent*; the spec's authority is the host computation.
3. **Flag the marginal case.** `--sona-boundary` at 3.07 on stage is barely above the 3:1 non-text floor. Either (i) re-verify hairlines that render on cards, or (ii) follow tokens.md's existing guidance and use `--sona-etwahl` for focus rings on raised surfaces. Record the decision in the spec.
4. **Keep WCAG 2.1 AA as the gate** (legally referenced, tool-supported, axe-core compatible) for a dark-only, driving-first, non-reading surface, while acknowledging in an Assumption that (i) halation makes near-black + ivory high-contrast glance-optimal but harsh for extended reading, and (ii) the upper bound / excessive-contrast question is out of scope — WCAG 3/APCA is not normative and this feature does not adopt it.
5. **Verification split.** Host deterministic: token-manifest contrast (this computation, automated). Device trial: OEM re-coloring of the notification accent (R2 risk) — already scoped to the single accent `--sona-voice`. In-app rendered surface: axe-core where a DOM exists (the packaged `status_view.html` once wired).
6. **Light mode stays deferred** (per tokens.md §5 and the spec Assumptions): adopting it would require full re-verification of every pairing; the host check written now must be parameterized by surface color so a future light-mode pass re-runs it.