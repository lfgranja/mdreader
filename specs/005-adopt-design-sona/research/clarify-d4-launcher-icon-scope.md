# Clarification D4 — Launcher-icon swap: in scope or explicitly out of scope?

> **Feature:** `specs/005-adopt-design-sona` (driving-surface adoption: notification MediaStyle + in-app StatusView)
> **Question:** The DESIGN-SONA dir assigns `--brand-wave-*` to "Marca, ícone do app, landing" (`tokens.md:52–54`) and ships a ready adaptive-icon triple (`assets/logo/ic_launcher_{background,foreground,monochrome}.xml`), but feature 005 claims only two driving surfaces (notification + StatusView), forbids the brand logo on notification/lock-screen (FR-007), and runs a closed driving-surface inventory that excludes un-enumerated elements. The launcher icon is unowned by the spec. Should the feature include an adaptive-icon swap, or record the icon explicitly out of scope (future brand feature)?
> **Scope:** research note; not a spec amendment. Informs a clarification decision only.
> **Method (2026-10-24):** Android adaptive-icon + themed-icon primary docs (developer.android.com); Google Play 512 px listing-icon spec; design-dir asset inspection (three XMLs); host inspection of generated Android resources (`src-tauri/gen/android/...`) and `tauri.conf.json` (no `bundle.icon` array; Tauri 2.12.1 regenerates from `src-tauri/icons/icon.png`). MVP/scope-discipline literature (Wikipedia/Atlassian/Figma MVP definitions; Sparkbox progressive enhancement; Object Edge design-system rollout). No icon-memorability primary study retrieved — flagged as thin in §2.

## 1. What a legal swap entails mechanically (Android primary sources)

**Adaptive icon = two 108×108 dp layers** (foreground + background), vectors preferred; the OEM applies a mask (circle, squircle, rounded-square, teardrop). The inner **66×66 dp is the safe zone** guaranteed never clipped; the outer 18 dp on each side is "extra" content revealed only during motion (parallax/physics). Logo must be ≥48×48 dp and ≤66×66 dp.

Source: Android Developers, *Adaptive icons* (developer.android.com/develop/ui/compose/system/icon_design_adaptive) and the canonical Nick Butcher post (medium.com/google-design/designing-adaptive-icons-515af294c783).

**Themed/monochrome (Android 13+):** a single-layer monochrome drawable whose alpha defines the shape; the system tints it to the user's theme color. The design dir's `ic_launcher_monochrome.xml` paints `#FFFFFF` everywhere with alpha — exactly the required single-color silhouette. The color foreground (`ic_launcher_foreground.xml`) carries the brand-wave strokes (turquesa `#8FD8E8`, jade `#5AD6A8`, celeste `#B9A6F5` + marfim `#F2F0E6`) — permitted **only on a brand surface**, never on a driving surface.

**Wiring:** an `adaptive-icon` XML in `mipmap-anydpi-v26/` referencing `@drawable/ic_launcher_background` and `@drawable/ic_launcher_foreground`, plus an optional `<monochrome android:drawable="@drawable/ic_launcher_monochrome" />`. The repo today has **no** `mipmap-anydpi-v26/` directory and no `adaptive-icon` XML — Tauri 2.12.1 generates a legacy single-layer icon from `src-tauri/icons/icon.png` (the green robot template is still shipped: `src-tauri/gen/android/.../drawable/ic_launcher_background.xml` paints `#3DDC84`).

**Google Play listing icon (separate artifact):** 512×512 px, 32-bit PNG, sRGB, ≤1024 KB, full square (Play applies the 30%-radius rounded mask and drop shadow dynamically); no transparency recommended ("transparent assets will display the background color of Google Play UI"); no drop shadows or corner rounding in the source.

Source: developer.android.com/distribute/google-play/resources/icon-design-specifications.

**Mechanical cost for this repo:**

- (i) replace the generated `drawable/ic_launcher_background.xml` and `drawable-v24/ic_launcher_foreground.xml` with the design-dir versions — or, better, source them from a non-generated path so Tauri's regen cannot clobber them;
- (ii) add `mipmap-anydpi-v26/ic_launcher.xml` (+ round variant);
- (iii) render a 512 px PNG for Play Console;
- (iv) verify the foreground content fits the 66 dp safe zone. The shipped waves run to roughly x=82/y=92 on the 108 viewport, mostly safe, but the jade/celeste strokes near the edge should be checked — clipping would cut the brand marks.

## 2. Evidence on launcher-icon identity effects — honest: thin

Retrieved sources are **specification/guideline documents**, not empirical studies. Android's adaptive-icon rationale is about OEM-shape coherence and motion ("more coherent" icons across launchers), not about recognition or conversion. The Play icon spec argues uniform shapes "help users focus on the artwork" — a UX assertion, not a measured effect. No peer-reviewed icon-memorability or Play-Store-conversion study was retrieved in this pass; treat any such claim as unsupported.

**Themed-icon collapse is real and mechanical, not aesthetic:** Android 13's monochrome layer must be a single color with alpha; the system tints it. A multi-color identity glyph therefore **cannot** survive themed mode as-is — it reduces to a silhouette. The design dir already accepted this split (color foreground vs white monochrome). Consequence: the "brand-wave triad" is invisible to users who enable themed icons; only the silhouette (the Concha Harmônica "R" outline) remains. That weakens the argument that the swap "delivers brand identity" on Android 13+ — it delivers it only to users who keep color icons.

Material product-icon guidance (material.io/design/iconography/product-icons) is the closest practitioner source; it is prescriptive, not evidence-backed.

**Honesty statement:** there is no retrieved evidence that a launcher-icon swap improves outcomes for this app; the maintainer is the single user and the icon is seen only at launcher-open, not during driving. Icon identity is a brand-coherence benefit, not a measured one.

## 3. Scope-management evidence — deferral is supported

**MVP definition (Wikipedia/Atlassian/Figma/codenroll 2026):** "deliberately excludes non-critical features, secondary workflows, aesthetic flourishes, and long-term architectural optimizations" while remaining usable; scope discipline is "ruthless prioritization," not omission of brokenness.

Sources: en.wikipedia.org/wiki/Minimum_viable_product; atlassian.com/agile/product-management/minimum-viable-product; figma.com/resource-library/what-is-a-minimum-viable-product; codenroll.co.il/blog/what-is-an-mvp-a-complete-guide-to-minimum-viable-products-2026.

**Progressive enhancement (Sparkbox; piccalil.li):** build in layers that turn on by capability, so the product is usable at every step rather than only at the end. The current shipped icon (default Tauri/Android robot) is usable; a brand icon is an enhancement layer, not a prerequisite for function.

Sources: sparkbox.com/foundry/MVP_mindset_progressive_enhancement...; piccalil.li/blog/its-about-time-i-tried-to-explain-what-progressive-enhancement-actually-is.

**Design-system rollout (Object Edge, 2022):** audit → assess → build components; component-level updates propagate; the system is a single source of truth. Surface-by-surface adoption is the norm; brand surfaces (icon, landing) are typically last because they depend on the token/component set being frozen.

Source: objectedge.com/blog/strategic-approach-to-rolling-out-a-design-system. (Paywall/JS-heavy pages were not fully retrievable; treated as direction.)

**Applied to this repo:** feature 005 already froze the driving-surface token set at eight members (2026-10-24) and the closed driving-surface inventory. The launcher icon is a **brand surface** ("Marca, ícone do app, landing"), not a driving surface, and its colors (brand-wave triad) are **not** in the closed eight-token set. Adding it now would (i) introduce a third "surface" into a feature that normatively claims two, (ii) require either expanding the closed-hex audit scope to icon files or writing an explicit exemption (monochrome `#FFFFFF` is outside the closed set by Android spec), and (iii) touch Tauri-generated resources that regenerate on build unless relocated. None of those are impossible, but none are required for the feature's acceptance (lock-screen/notification + StatusView).

## 4. Falsifiable considerations — what would change the decision

**Would change to "in scope":**

- (i) maintainer declares the launcher icon part of Option A's "dual mark on existing surfaces" (the Concha Harmônica "R" is already the in-app mark; the icon is the OS-level counterpart), or
- (ii) the default robot icon is judged brand-incoherent enough to block release, or
- (iii) the design-dir asset is verified safe-zone-clean and a non-generated source path is established so Tauri regen cannot clobber it.

**Would keep "out of scope" / defer:**

- (i) the icon stays a brand surface handled by a dedicated future brand feature that also owns the Play Store 512 px listing asset, round-icon variant, and themed-icon QA, or
- (ii) the closed-hex audit exemption for monochrome `#FFFFFF` is not written into this feature's SC-001/SC-006.

**Decisive fact:** the icon is **brand-visible from launcher only** and the driving user interacts **primarily via lock screen, where no logo is allowed (FR-007)**. The swap has zero effect on the driving experience this feature targets; its entire value is brand coherence for a single user opening the launcher.

## 5. Recommendation

**(a) Out of scope now — record the launcher icon as an explicit exclusion, and queue it as the seed task of a future brand feature.** Do not swap in 005. The design-dir adaptive-icon triple is ready and mechanically compliant (safe zone to be verified), so the future feature is a wiring task, not a design task — but it should own the Play Store 512 px asset, the round variant, the themed/monochrome QA, and the safe-zone check together, rather than being bolted onto a driving-surface feature whose audits and closed sets it does not fit.

Option (b) (minimal swap now) is only justified if the maintainer explicitly widens Option A to include brand surfaces — in which case the swap must be accompanied by (i) an explicit SC-001/SC-006 exemption for the icon's brand-wave hexes and monochrome `#FFFFFF`, and (ii) relocation of the icon XMLs out of Tauri's generated tree. Option (c) (constraints only) is subsumed: the mechanical constraints in §1 become the checklist for the future brand feature.

**Evidence strength:** Android mechanics — strong (primary docs). Icon-identity benefit — weak/none retrieved (literature thin; be honest). Scope discipline — moderate (MVP/progressive-enhancement/design-system-rollout sources are practitioner definitions, not experiments, but consistently point the same way).

## Implication for the spec

**Out-of-scope line (recommended):** add to the Assumptions/Exclusions block (alongside the excluded screens Cockpit Eyes-Free, Console Braun & Rams, Sumário Editorial Suíço, Matriz de Telemetria): "Launcher icon / adaptive icon / themed (monochrome) icon and the Google Play 512 px listing icon are out of scope for 005 — brand surfaces, not driving surfaces; handled by a future brand feature. The DESIGN-SONA adaptive-icon triple (`assets/logo/ic_launcher_{background,foreground,monochrome}.xml`) is the ready input; the future feature owns Play Store asset, round variant, themed-icon QA and safe-zone verification. FR-007's no-logo-on-notification/lock-screen rule is unaffected."

**Scoped-task line (only if (b) chosen):** "FR-009 (new): The system MUST ship the DESIGN-SONA adaptive icon (background `#0B0F1A`, foreground waves, monochrome silhouette) as the launcher icon, with a 66 dp safe-zone-compliant foreground, a themed (monochrome) layer, and an explicit exemption of the icon's brand-wave hexes and monochrome `#FFFFFF` from the SC-001/SC-006 closed-hex audit (brand surface, not driving surface). The Play Store 512 px listing icon is a separate artifact owned by the same task."

## Open items for the maintainer

- Whether the future brand feature should also retire the Tauri-generated `src-tauri/icons/icon.png` pipeline or keep it as the source of truth rendered into `mipmap-*` (affects where the XMLs live and whether regen clobbers them).
- Whether the 66 dp safe-zone check on the shipped foreground (waves to ~x=82/y=92) passes as-is, or needs a 2-4 dp inset before the future feature ships.
- Whether themed-icon (monochrome silhouette) QA is part of the future feature's acceptance, or deferred to a separate Android-13-compat pass.