# Clarification — Candidate C1: Celeste/Jade Paint Scope (online research)

**Status:** READ-ONLY research — no edits performed. Proposes findings only.
**Scope:** Whether celeste `#B9A6F5` (and jade `#5AD6A8`) should be promoted from reserved/brand-only to painted UI status in 007, where, and whether that can be resolved autonomously.

---

## (a) Relation to current implementation

- `data-model.md` §SonaToken lists `--sona-celeste #B9A6F5` as **RESERVED, unpainted this feature**; painting it anywhere in audited files is a census violation (C-H1: "`#B9A6F5` paint anywhere (celeste vacancy violation)"). Jade `#5AD6A8` (+ erratum `#5FD0A0`) is **banned paint anywhere** (player-color prohibition).
- `tokens.md` §2 assigns role semantics: jade = brand identity only (Onda 2, "PROIBIDO na interface operacional do player"); celeste = rate/speed (Song of Celerity / Onda 3), "Nunca falha, nunca aviso, nunca fluxo". Brand-wave tokens (`--brand-wave-clarity/healing/cadence`) are scoped to "Marca, ícone do app, landing" — not StatusView.
- `status-surface.md` S3: "`--sona-celeste` (vacancy semantics, batch C2): the token STAYS in the lexicon but paint of it is a census violation — the sanctioned scope is empty, not the token." S8: decorative wave "No new token member (reuse only — S3)." S12: token set unchanged (10 members); additions only if unavoidable per §VII.
- `spec.md` 007 currently assumes the opposite: "SonaToken (expanded): 12+ members (original 10 + jade #5AD6A8 + celeste #B9A6F5 promoted to painted)" and FR-001/SC-001 expect wave ribbons "teal-to-violet" and waveform "teal→jade→violet". This is a direct conflict with C-H1/S3/data-model. Resolving it is the subject of this candidate.

## (b) Importance

- **High / blocking.** 007's visual fidelity (wave ribbons teal→violet, waveform ribbons teal→jade→violet) depends on painting celeste and jade in operational UI. The audit (C-H1) and contract (S3) currently forbid exactly that. Without a decision, either the implementation violates gates or the visuals diverge from the proposal. This is not cosmetic: it touches the banned-paint list, the closed token set, and §VII change discipline.

## (c) Evidence-backed recommendation

### 1. Contrast (WCAG 2.1/2.2 AA; thresholds text ≥4.5:1, non-text ≥3:1)

Computed relative luminance (standard sRGB formula) over the two effective backgrounds:

| Pair | Ratio | Verdict |
|---|---|---|
| `#B9A6F5` over `#0B0F1A` (void) | **8.96:1** | PASS text (AA+AAA) and non-text |
| `#B9A6F5` over `#141B2D` (stage) | **8.03:1** | PASS |
| `#5AD6A8` over `#0B0F1A` | **10.59:1** | PASS |
| `#5AD6A8` over `#141B2D` | **9.49:1** | PASS |
| `#8FD8E8` over void/stage | 11.99 / 10.74 | PASS (reference) |

Translucent overlay check: celeste blended at α 0.10–0.50 over void/stage keeps white primary text contrast ≥5.04:1 at α=0.5 (and higher at lower α); at α≤0.30 all blends keep ≥9:1. So moderate translucency (≤30–40%, per glassmorphism best-practice guidance) does not break text contrast. Source: hues.page contrast data for #b9a6f5 (9.83:1 on black) corroborates; W3C WCAG 2.2 SC 1.4.3/1.4.11.

**Conclusion on contrast:** contrast is NOT the blocker. Celeste and jade both exceed thresholds by wide margins on both backgrounds, including at moderate translucency. The blocker is role semantics + governance, not legibility.

### 2. Role semantics (design-system practice)

- **Material Design 3** color roles: primary/secondary/tertiary are accent roles for emphasize/de-emphasize; tertiary is explicitly for "smaller elements that need special emphasis but don't require immediate attention, such as a badge or notification" (m3.material.io/styles/color/the-color-system; color/roles). M3's guidance: "Use caution when changing color roles for visual effect." Secondary/tertiary are de-emphasis accents, not decorative fills. (sources: m3.material.io color roles; developer.android.com Wear color roles).
- **Apple HIG**: "Apply your app's accent color judiciously. Using your brand color too broadly can overwhelm your interface and dilute its impact. Minimize its use on controls and instead use it intentionally for primary actions or status indicators." Brand should move into the content layer, not controls. (developer.apple.com/design/human-interface-guidelines/branding, updated 2026-09-09).
- **W3C DTCG 2025.10**: tokens are the single source of truth; roles are semantic, and `$`-prefixed reserved keys exist precisely to separate "defined but not applied" decisions from applied ones. Governance = coordinated, versioned edits, not in-place convenience. (w3.org/community/reports/design-tokens/CG-FINAL-format-20251028).
- **Semantic color ownership** (designsystemscollective): roles describe intent; "primary/secondary" are visual emphasis, not semantic status. Promoting a token to painted status should follow a defined semantic role, not aesthetic convenience.

Applied to this project: celeste's defined role is **rate/speed (Song of Celerity)** and jade's is **brand identity only (Onda 2)**. Painting them as decorative ribbon fills would be "changing color roles for visual effect" (M3 caution) and "using brand color too broadly" (Apple HIG) — it dilutes both roles. Jade in operational UI is explicitly prohibited by tokens.md §2. Celeste in decorative ribbons would make it a decoration token, contradicting its rate/speed semantics and the "never fluxo" rule.

### 3. Multi-layer translucent ribbons

- The arxiv study on transparent overlapping charts (arxiv.org/html/2412.16242v1) and glassmorphism guidance (timgraf.com; medium glassmorphism best practices) converge on: keep transparency ≤30–40% for readability; provide an accessibility fallback layer; decorative translucent layers that convey no information are exempt from WCAG 1.4.11 non-text contrast, but text overlaid on them must still pass 1.4.3.
- My blend computation confirms celeste at ≤30% alpha over void/stage keeps all text contrast ≥9:1 (≥4.5). At 40–50% it still passes but narrows margin. Recommendation: cap decorative ribbon alpha ≤0.30 and keep text on solid surfaces (stage/void), not on blended ribbons.

### 4. Should decorative waveform layers reuse brand-wave tokens vs introduce new paints?

- **Reuse is preferred and sufficient.** The brand-wave tokens (clarity=etwahl, healing=aria, cadence=celeste) already encode the teal→jade→violet triad the proposal depicts. Introducing new paint tokens would violate the closed-set discipline (constitution §VII, S3, S12) and add no semantic value.
- **But "reuse" must respect paint scope.** Reusing a brand-wave token as a painted fill on the operational surface (canvas/DOM) is still "painting" it. The audit C-H1 bans jade paint anywhere and celeste paint anywhere. So reuse in a *static SVG asset* (like `waves-top.svg`, which is not in the auditee list A1) is safe and non-violating; reuse as *canvas fill* or *CSS color* in `status_view.html` is a violation unless the contract is amended.
- Therefore: decorative ribbons should be delivered as **static SVG assets** (consistent with FR-005 "Static assets preferred (SVG/CSS)" and S8 "Static SVG preferred") using the brand-wave palette inside the SVG, not as painted hex in the audited HTML/canvas. This keeps C-H1 green without amendment. The reactive canvas waveform (FR-008) should use only already-sanctioned paints (`--sona-etwahl` teal for the active flow, `--sona-voice` white for progress line/playhead, `--sona-boundary` track) and must not paint jade/celeste unless the contract is amended.

## (d) Autonomous resolution vs. user decision

**Not fully autonomous.** Two distinct sub-decisions:

1. **Celeste painted in the speed-control rate area** — already autonomously resolved by prior clarification (numeric rate painted in speed-control area only; S4/S10 amended). Contrast 8.96/8.03 PASS. This is safe and sanctioned. No further action.

2. **Celeste/jade painted in decorative waveform layers (canvas or CSS)** — NOT safe to resolve autonomously because it requires:
   - Amending C-H1 (remove the "`#B9A6F5` paint anywhere" and jade banned-paint clauses) — a contract edit, not a code edit.
   - Amending data-model.md banned-paint list and tokens.md §2 role rules (jade "PROIBIDO na interface operacional"; celeste "nunca fluxo").
   - Resolving the role-dilution concern (M3 caution, Apple HIG judicious use): painting brand colors as decorative fills contradicts their defined roles. This is a product/design judgment, not an engineering fact.
   - §VII coordinated edit (3-file + source + commit + re-run T033–T039).

**Recommended path (autonomous-safe portion):** keep jade banned and celeste reserved in operational UI; deliver decorative ribbons as static SVG assets using the brand-wave palette (no audit violation, no contract amendment, no role dilution); restrict the reactive canvas waveform palette to sanctioned paints (teal/white/boundary). This satisfies the proposal's visual intent (teal→violet ribbons as decorative top layer) while preserving role discipline and audit green.

**Requires user decision:** whether to formally promote celeste (and possibly jade) to painted status in the reactive waveform canvas layer, accepting role dilution and a §VII amendment. If yes, scope must be narrowly defined (e.g., "celeste+jade permitted ONLY as decorative (aria-hidden) canvas ribbon fills at ≤30% alpha; never as text/UI state/indicator") and the banned-paint clauses updated accordingly. Contrast evidence supports paintability; the open question is governance/role intent, which is a human decision.

## Sources

- W3C WCAG 2.2 SC 1.4.3 (Contrast Minimum) and 1.4.11 (Non-text Contrast) — w3.org/WAI/WCAG22/Understanding/contrast-minimum.html, non-text-contrast.html.
- hues.page contrast data for #b9a6f5 (9.83:1 on black, 2.13:1 on white) — corroborates computed 8.96:1 over #0B0F1A.
- Material Design 3 color roles — m3.material.io/styles/color/roles, m3.material.io/styles/color/the-color-system ("Tertiary roles are for smaller elements that need special emphasis... Use caution when changing color roles for visual effect").
- Apple HIG Branding — developer.apple.com/design/human-interface-guidelines/branding ("Apply your app's accent color judiciously... Minimize its use on controls").
- W3C DTCG Format Module 2025.10 — w3.org/community/reports/design-tokens/CG-FINAL-format-20251028 (tokens as single source of truth; reserved keys).
- Semantic color ownership — designsystemscollective.com (primary/secondary are visual emphasis, not semantic status).
- Glassmorphism best practices — timgraf.com (accessibility fallback layer); medium.com glassmorphism guide (transparency ≤30–40%).
- arxiv.org/html/2412.16242v1 — color-name-aware optimization for transparent overlapping charts.
- Project contracts: `specs/005-adopt-design-sona/contracts/status-surface.md` (S3, S8, S12), `specs/005-adopt-design-sona/contracts/surface-audit.md` (C-H1), `specs/005-adopt-design-sona/data-model.md` (§SonaToken banned paints), `DESIGN-SONA/tokens.md` (§2 role rules), `specs/007-sona-complete-design/spec.md`.