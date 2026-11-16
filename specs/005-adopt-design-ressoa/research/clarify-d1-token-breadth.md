# Clarification D1 — Breadth of DESIGN-SONA token adoption: color only vs. + typography/spacing

> **Feature:** `specs/005-adopt-design-sona` (FR-001 closed token set, SC-001 hex census + WCAG 2.1 AA + element inventory; C3/C6 "tokens-only restyle, no layout restructuring"; PRINCIPLE VIII glance ≤2 s)
> **Question:** Should 005's adoption of the DESIGN-SONA token system bind *only* the color (palette) tokens on the two audited surfaces, or *also* the design dir's typography and spacing scales?
> **Scope:** research note; not a spec amendment. Informs a clarification decision only.
> **Method (2026-10-24):** W3C DTCG 2025.10 (first stable) token-type catalogue; Material 3 typography/space guidance; IBM Carbon stylelint-plugin-carbon-tokens; USWDS token model; Nathan Curtis / EightShapes token-layering literature; ISO 15008:2017 published previews (full text paywalled — direction only); NHTSA 812360 text-legibility excerpts; Reimer/Dobres in-vehicle typography studies; host-side enumeration of what the design dir actually defines. No fabricated citations — paywalled standards cited via previews and flagged direction-only.

## 1. What the design dir actually defines — the decisive inventory fact

- `DESIGN-SONA/tokens.md` (v1.0.0, 2026-10-06) defines **one token layer: color** — 9 primitive hexes (`--sona-void`, `--sona-stage`, `--sona-voice`, `--sona-hush`, `--sona-etwahl`, `--sona-harp`, `--sona-celeste`, `--sona-aria`, `--sona-boundary`) plus semantic/component aliases. **No `--sona-*` typography, spacing, radius, motion, or shadow tokens exist anywhere** in `DESIGN-SONA/` or `DESIGN/` (grep over `*.md` returns zero matches for `--space`, `--size`, `--font`, `spacing token`, `type scale`). The only type/spacing evidence is ad-hoc CSS in `assets/ui/galeria_ui.html` and `assets/proposals/galeria_propostas.html` (font-family `Plus Jakarta Sans`/`Inter`, sizes `1.4rem`, `0.94rem`, paddings `12px`/`14px`).
- `accessibility.md` §6 cites **touch targets (≥ 48 dp, ≥ 8 dp spacing)** but gives no font-size token; `components.md` cites "16sp/600" for the rate text *in prose*, not as a named token. `DESIGN-SONA.md` §5 (layout) cites "16sp/600, números tabulares" and "12sp/400" in prose only.
- Host-side verification of the "no type/spacing token" claim (run 2026-10-24): `grep -rin` over `DESIGN-SONA/` and `DESIGN/` for `--space`, `--size`, `--font`, `--radius`, `--motion`, `--shadow`, `type-scale`, `spacing scale` returns **0 matches**; the only type/spacing strings are CSS literals inside `assets/ui/galeria_ui.html` and `assets/proposals/galeria_propostas.html` (font-family, `1.4rem`, `0.94rem`, `12px`/`14px` paddings) — presentation code, not tokens. The `DESIGN-SONA/VERSION`-pinned revision is the sole authority; no token file there declares type or spacing.

| Token family | Defined in design dir? | Named tokens | Canonical scale? | Adoptable as-is? |
|---|---|---|---|---|
| Color (palette) | Yes (`tokens.md` §1) | 9 primitives + aliases | Yes (closed set) | Yes — already adopted |
| Typography | No | none (prose only) | No | No — must be invented |
| Spacing | No (48dp/8dp in prose) | none | No | No — must be invented |
| Radius / motion / shadow | No | none | No | No — must be invented |

- **Consequence:** there is no canonical "DESIGN-SONA typography/spacing scale" to adopt. Adopting (b)/(c) would require *inventing* tokens in this feature — i.e. a §VII coordinated edit + semver bump + docs for a set that does not exist in the pinned `design-sona-v2.2.0` revision. That is scope creation, not adoption.

## 2. Design-system practice: incremental adoption breadth

- **W3C DTCG 2025.10** (first stable, Oct 2025; https://www.designtokens.org/TR/2025.10/format/) treats color, typography, spacing, radii, motion, shadow as **distinct token types** in one format, with aliases and references. The spec standardizes *exchange*, not adoption order — it is silent on phasing. This is the authoritative 2025/2026 baseline.
- **Material 3** defines a full type scale (`m3.material.io/styles/typography/type-scale-tokens`) and spacing, but its adoption docs are phased by *surface/component*, not by token type: color scheme first (dynamic color), then component migration, with type scale applied as components are rebuilt. M3's color-only theming (`ColorScheme.fromSeed`) is the documented entry point for adopters.
- **IBM Carbon** enforces tokens via `stylelint-plugin-carbon-tokens` (https://github.com/carbon-design-system/stylelint-plugin-carbon-tokens) — but the plugin's rules are **color- and spacing-value** guards; typography enforcement is layered separately and only where components consume type styles. Carbon's migration guidance is incremental by component, not by token family.
- **USWDS** (https://digital.gov/event/2023/08/17/uswds-monthly-call-august-2023) and the InVision Design Systems Handbook both describe **incremental rollout by part**: "Many teams have to roll out parts of a design system at a time… adopt new parts as they're needed." No authoritative source prescribes *all token families in one release* for a legacy surface.
- **Nathan Curtis / EightShapes** (*Tokens in Design Systems*, medium.com/eightshapes-llc; *Size in Design Systems*, medium.com/eightshapes-llc) argues tokens are adopted in **layers** (primitive → semantic → component) and that *component adoption* is the unit of value — color tokens alone deliver visual identity; type/spacing tokens deliver value only when components are rebuilt against them. Partial color adoption is the documented, expected state during migration; "token debt" is the risk of *unmanaged* literals, not of *phased* adoption.
- **Net:** authoritative practice treats **phased adoption as normal**, with color as the first, cheapest, highest-visibility layer; widening to type/spacing is justified only when (i) a canonical scale exists, (ii) components are being rebuilt, and (iii) a verification oracle exists for the new family.

## 3. In-vehicle glance-safety and typography: what the evidence actually says

- **ISO 15008:2017** (preview, https://webstore.ansi.org/preview-pages/ISO/preview_ISO+15008-2017.pdf — full text paywalled; direction only): specifies **per-element perceptual parameters** — character height (min 12 arcmin, 16 acceptable, ≥ 20 recommended), character width, stroke width, contrast, luminance, color combinations, pixel format, glare. **No type-scale, field-count, or spacing-token normative content.** Layout structure is out of scope.
- **NHTSA 812360** (https://www.nhtsa.gov/sites/nhtsa.gov/files/documents/812360_humanfactorsdesignguidance.pdf): text legibility recommendations (min char height 16–20 arcmin, optimal 20–30 arcmin); **glance criteria (single ≤ 2 s, cumulative ≤ 12 s) and prohibited-content list** are the operative acceptance tests. Typography is a *per-element legibility input*, not a system-level adoption requirement.
- **Typeface evidence (Reimer et al. 2014/2017; Dobres et al. 2015/2018):** typeface design measurably affects glance behavior — Reimer 2014 found a **10.6% reduction in total glance time** for a humanist vs. square-grotesque typeface in a text-rich automotive menu. But (a) the effect is **typeface-level**, not token-scale-level; (b) it appears in **text-rich menus** (reading tasks), not in a 2-text-element glance surface; (c) the surface here is value-blind and content-minimal (title + section, no body text, no TOC, no elapsed time — NHTSA prohibited list already excludes reading).
- **Color contrast dominates legibility on this surface.** WCAG 2.1 AA (text ≥ 4.5:1, non-text ≥ 3:1) is already the binding ruler; all eight token pairings pass with headroom (clarify-b3). ISO 15008's char-height floor is a *per-element* check already expressible without a type-scale token.
- **Honest strength statement:** no retrieved study isolates "adopting a type-scale token system" as an independent variable on glance time; the typeface effect is real but (i) operates on typeface choice, which the app does not control on Android (system font), and (ii) is dwarfed by content-inventory effects (NHTSA prohibited list, Lee et al. visual complexity on element count). On a 2-text-element surface, a type-scale token adds no measurable safety property.

## 4. Verification: can typography/spacing be certified by the SC-001 oracle at all?

- **SC-001's oracle is a hex census** over touched surface files (closed value set; contrast pairs; element inventory; required-absence checks). It is *chromatically* blind to geometry: it cannot see font sizes, paddings, margins, or line heights.
- **Adopting type/spacing would require a new oracle** — a px/rem-based lint (stylelint `declaration-property-value-allowed-list` for `font-size`/`padding`/`margin`/`gap`, or a Detekt/Konsist rule on Android). This is exactly the class of "new host oracle" that C3 (grid geometry) and C6 (tokens-only restyle) deliberately refused to create: "no new host oracle for '2 campos'"; "tokens-only (paint substitutions + enumerated stamps + dead-element removal, no layout restructuring)".
- **Closed-set governance (constitution §VII)** would then apply to two new token families: coordinated edit + semver bump + docs for every addition. The design dir has no such families, so the governance cost lands on this feature for tokens invented here — the weakest possible §VII case (compare: celeste, already reserved, and jade, already banned — both have *design-dir* provenance).
- **Practical consequence:** color-only adoption keeps SC-001's oracle complete and deterministic; type/spacing adoption either (i) ships unverified (violating the "evidence before assertions" rule), or (ii) expands SC-001's oracle (violating C3/C6's "no new oracle" decision).

## 4.1 Concrete cost of adopting type/spacing in this repo (what would actually be touched)

- **Surface files:** `src-tauri/src/status_view.html` (the in-app surface of record after the wiring prerequisite) and the notification accent path. Today these use system font + ad-hoc px/rem; introducing type tokens means replacing every `font-size`/`padding`/`margin`/`gap` literal with a token reference *and* defining the token values.
- **Oracle:** SC-001 would need a second, px-based host check (stylelint-style `declaration-property-value-allowed-list` or a Detekt/Konsist rule for Kotlin/HTML). This is a *new* gate, not an extension of the hex census — different failure mode, different test harness, different maintenance.
- **Design dir:** a coordinated §VII edit adding a typography/spacing section to `tokens.md`, `components.md`, `accessibility.md`, plus a changelog entry and (per the 2026-10-24 normalization) a VERSION bump. The design dir is normalized to v2.2.0 and pinned by tag; any token addition re-pins it.
- **Governance:** every future size change becomes a coordinated edit. Color tokens already pay this cost because they exist; type/spacing tokens would pay it for a property (char height) that ISO 15008 already bounds per-element without a token.
- **Risk of rework:** the future full-UI feature (Svelte toolchain, multi-mode) will define its own type scale against real components. A (b)-level adoption now creates a throwaway scale that the full-UI feature either (i) inherits (constraining it) or (ii) replaces (orphaning the 005 tokens). Color tokens do not have this problem because color identity is stable across UI modes.

## 4.2 Option comparison

| Criterion | (a) Color only (rec.) | (b) Color + minimal type | (c) Full token adoption |
|---|---|---|---|
| Canonical source exists | Yes (color) | No (type invented) | No |
| Verification oracle exists | Yes (hex census) | No (needs px-lint) | No |
| §VII coordinated edit needed | No (already done) | Yes (new tokens) | Yes (large) |
| Safety benefit on 2-text surface | Already sufficient (WCAG AA + ISO 15008 per-element) | None measurable | None measurable |
| Scope creep vs. Option A | None | Moderate | Total (becomes full-UI feature) |
| Rework risk when full-UI ships | None | High (throwaway scale) | N/A (is the full-UI feature) |
| Consistent with C3/C6 "no new oracle" | Yes | No | No |

## 5. Recommendation

**Option (a) — color-only restyle this feature; defer typography and spacing to the future full-UI feature.**

Support strength: **strong** (three independent converging reasons, each sufficient alone):

1. **No canonical source exists** (§1): there are no DESIGN-SONA typography/spacing tokens to adopt; adopting them means inventing them inside a feature whose scope is "adoption."
2. **No verification oracle exists** (§4): SC-001 is a hex census; type/spacing need a px-lint oracle that C3/C6 explicitly refused to create; creating it now contradicts two settled decisions.
3. **No measurable safety benefit on this surface** (§3): the surface has 2 text elements, is value-blind, and already satisfies ISO 15008 char-height floors and WCAG AA; the typeface effect is real but typeface-level and reading-task-level, not token-scale-level.

**Why not (b) (color + minimal type adoption)?** Tempting because the design dir's prose cites font sizes ("16sp/600"). But "minimal" type adoption still (i) requires inventing tokens with no canonical source, (ii) requires a new px oracle or ships unverified, and (iii) partially binds a scale that the future full-UI feature will need to redefine anyway — paying governance cost twice for a non-safety property. The "minimal" qualifier does not remove the structural blockers; it only shrinks the unverified surface.

**Why not (c) (full token adoption now)?** Requires a full type/spacing scale that does not exist, a frontend toolchain this feature deliberately avoids (reduced Lente, no Svelte), and a new oracle — i.e. it *is* the future full-UI feature. Adopting it now collapses the "minimal-cost dogfood" posture (notification-first, tokens-only) into a full UI rewrite, contradicting the 2026-10-22 Option A scope selection.

**Decisive falsifiable considerations** (what would flip the answer):
- If a canonical DESIGN-SONA type/spacing scale is added to the pinned revision *before* 005 ships (coordinated §VII edit with changelog), and a px-lint oracle is added to SC-001 in the same coordinated edit, then (b) becomes defensible — but only for sizes the scale explicitly prescribes for *these two components*, not a general adoption.
- If the future full-UI feature lands and redefines the type scale, any (b)-level adoption here becomes dead weight requiring migration — a cost that color-only adoption avoids entirely.
- If on-device trials show glance-time failures attributable to type size (not content inventory), the ISO 15008 char-height floor check (already available, per-element) is the correct response, not a token-scale adoption.

## 6. What the literature does NOT answer (honesty)

- **No study compares "color-only token adoption" vs. "color + type-scale token adoption" as an independent variable** on any outcome (glance time, task time, error rate, maintainability). The evidence base is about typeface choice, char height, and content inventory — not token-family phasing.
- **No data on whether adding type tokens to a 2-text-element driving surface measurably changes glance time.** The Reimer 10.6% effect is typeface-level on text-rich menus; extrapolation to token-scale on 2 elements is unsupported.
- **No authoritative source prescribes adoption order by token family.** Material/Carbon/USWDS phase by *surface/component*, which happens to put color first because components are rebuilt color-first — but this is practice, not evidence of superiority of color-only adoption.
- **The "token debt" risk of partial adoption is asserted, not measured.** Nathan Curtis and the InVision handbook describe it qualitatively; no retrieved source quantifies the cost of phased vs. simultaneous token adoption. The honest claim here is that *unmanaged* literals are the risk, and the SC-001 hex census already manages the color literal risk — the largest and most auditable family.
- **ISO 15008/NHTSA do not address typography tokens at all** — they address char height, contrast, and glance time. Whether a type-scale token system is worth adopting is a design-system governance question, not a safety question, and the safety-relevant per-element checks (char height, contrast) are already available without it.

## Implication for the spec

- **If (a) is adopted (recommended):** no FR/SC change required — FR-001 already scopes the closed set to *color* tokens (`--sona-void … --sona-boundary`), SC-001's oracle is a hex census, and C6 already states "tokens-only (paint substitutions + enumerated stamps + dead-element removal, no layout restructuring)." Only a one-line clarification recording that "token adoption" in this feature = color palette tokens; typography/spacing scales are explicitly out of scope and reserved for the future full-UI feature. No coordinated §VII edit to the design dir needed.
- **If (b) were adopted:** FR-001 would need a new "typography/spacing" clause listing the adopted scales (none exist in the pinned design dir → must be invented first); SC-001 would need a px-lint oracle extension (contradicting C3/C6); a coordinated §VII edit + semver bump to the design dir; and a migration plan for when the full-UI feature redefines the scale. Net: high cost, unverified intermediate state, likely rework.
- **If (c) were adopted:** collapses 005 into the future full-UI feature — contradicts the 2026-10-22 Option A scope selection, the "minimal-cost dogfood / notification-first" posture, and the "no new frontend toolchain" decision (2026-10-23 reduced Lente). Would require reopening FR-008 scope, SC-001 oracle, and the verification plan.

**Recommended spec action:** record clarification D1 = (a); add one sentence to FR-001/SC-001 scope note ("token adoption in this feature = the closed color-palette set; typography and spacing scales are out of scope — reserved for the future full-UI feature"); no other edits. This mirrors how C3 demoted "grade 2-campo" to descriptive wording rather than binding unverified geometry.
