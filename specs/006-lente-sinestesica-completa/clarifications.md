# Clarification Batch — /speckit.clarify (006-lente-sinestesica-completa)

Generated: 2026-10-27. Background agents launched per checklist (ReviewA11yMotion, ReviewContracts, ReviewEdgeCases, ReviewUxVisual; ResearchLoopViolation, ResearchHighContrast, ResearchFilenameFallback, ResearchGracefulDegradation). Unchecked items reviewed against `spec.md` + `plan.md` + `research/` + clarifications (§Clarifications lines 14–21, 35–36).

**Convention**: `[x]` = sound/clear/unambiguous (applied to checklist). `[ ]` = clarification candidate below (not applied to checklist — decision pending).

---

## 1. CHK039 — SC-002 objectively verifiable? + CHK041 — title card accessibility AC

**Question**: What is the precise pass/fail protocol for SC-002's three clauses (progressbar aria intact, empty slot aria-label, no alert misuse) and for title card accessibility (`role="region"`, label content, screen-reader announcement, focus/non-focusable status)?

**Why does it matter**: SC-002 and SC-006 are device-trial gates (§SC-006 line 87). Without an operational protocol (DOM attribute checks, `aria-label` value assertions, `assertive`/`alert` audit rules from Clarifications line 15), a tester can't distinguish pass from fail. Title card is the new visual center of this feature; a missing accessibility AC means screen-reader users (eyes-free driving context) have no verifiable guarantee.

**Findings**: `spec.md:83` (SC-002) bundles three clauses at high level; `spec.md:87` (SC-006) verifies reduced-motion / decorative / focus / notification / dead-CSS / surface inventory but does NOT define SC-002 verification steps. `spec.md:66` (FR-002) specifies data source + glyph + `aria-label`; `spec.md:14–17` (Clarifications) specify `role="region"`, `aria-hidden="true"` waves, keyboard unchanged, `aria-label="Sem título"`. However no AC covers `role="region"` presence, populated-label content equality with `metadata_title`, screen-reader announcement test, or visible focus ring (only overflow/failure mentioned at `spec.md:18`). `a11y-motion.md` Notes: eyes-free usage critical.

**Options**:
- A (Recommended): Add explicit AC to `spec.md` SC-002 / new sub-section: (i) `progressbar` has `role`, `aria-valuenow/min/max`; (ii) empty slot renders `—` with `aria-label="Sem título"` (verify via DOM + screen-reader); (iii) audit `aria-live` values — assertive only when `error.kind == critical` per Clarifications. Add title-card AC: `role="region"` present; `aria-label` equals populated title or "Sem título"; decorative waves `aria-hidden`; no new interactive elements → focus order unchanged.
- B: Rely on SC-006 device trial (manual) — faster to write but not repeatable/reproducible.
- C: Defer to implementation test — violates checklist's purpose (tests requirements, not code).

**Recommendation**: Option A. Evidence: W3C ARIA APG progressbar pattern + `a11y-motion-review-report.md` (CHK028/031/043 citations in Clarifications) + SC-006's failure to cover SC-002 specifically. Low cost, high audit value.

---

## 2. CHK044 — High contrast / forced-colors mode requirements?

**Question**: Should high-contrast / forced-colors behavior be specified (decorative pseudo-elements, title card, progress bar, banner) — explicitly excluded, or given a minimal guard?

**Why does it matter**: Eyes-free / driving context (`a11y-motion.md` Notes) increases reliance on strong visual differentiation. Windows High Contrast Mode / macOS Increase Contrast / Android high-contrast can strip decorative CSS `::before` pseudo-elements and override colors. If decorative wave disappears entirely, layout must preserve space; if banner/text loses contrast, failure-state discernibility breaks.

**Findings**: Zero references to `forced-colors`, `prefers-contrast`, `high-contrast` in `spec.md` or `plan.md` (`grep` null). Clarifications (line 18–21) cover graceful degradation / overflow / long filenames but not forced-colors. W3C WCAG 2.2.2 / 1.4.3 / forced-colors (`forced-colors` media feature, W3C WICG) recommend maintaining structural information under forced-colors; Apple HIG / Microsoft / Android accessibility docs recommend that decorative elements remain structurally present (or fade gracefully) but never break layout or hide critical text.

**Options**:
- A (Recommended): Minimal guard in `spec.md` §Assumptions + FR-004 / SC-006: "Under `forced-colors: active`, decorative wave pseudo-element may become static / outlined (layout preserved, no broken-image); title card and progress bar maintain text contrast via `CurrentColor`; banner remains visible (no color-only signaling)." No new interactive elements.
- B: Explicitly exclude — faster, but risks accessibility failure in high-contrast environments where eyes-free users depend on structure.
- C: Full high-contrast audit (contrast ratios, color-independent signaling) — over-scoped for this feature's decorative + single-surface scope.

**Recommendation**: Option A — minimal structural guard, align with WCAG forced-colors best practice, reference in SC-006 device-trial (verify under `forced-colors` if device supports). Evidence: W3C forced-colors spec, Apple HIG (accessibility / high contrast), `a11y-motion.md` eyes-free requirement.

---

## 3. CHK046 — Animation that violates loop constraint: what happens?

**Question**: If a decorative animation accidentally exceeds ≤1 continuous loop / >5s / `infinite`, what is the fail-closed response (freeze, block, rollback, or guard-only)?

**Why does it matter**: Vestibular disorders (~35% by age 40, per PubMed / `motion-research-report.md` cited in Clarifications). A loop violation that isn't caught becomes a continuous motion source. The checklist asks specifically about violation behavior, not just prevention.

**Findings**: `spec.md:16` (Clarifications) quantifies loop = repeating >5s or `infinite`; ≤1 continuous permitted; gated under `reduce`; ≤5s escapes 2.2.2 (G152). `plan.md:S5` / constitution sets limit. `spec.md:68` FR-004 requires guard + maintain S5. `ResearchLoopViolation` (completed): defense-in-depth A (build-time audit block via MotionSpec / audit) + B (runtime freeze + log) recommended; guard-alone (C) explicitly non-fail-closed (W3C #4319: guard ≠ mechanism). No current spec line defines the response when violation is detected.

**Options**:
- A (Recommended — defense-in-depth): Build-time audit block (animation duration / loop-count check in CSS/asset audit — aligns with `plan.md` Phase 1 / `DESIGN-SONA` token audit + SC-007 C-H1 census); runtime freeze (`animation: none` + static render) + optional log entry when `matchMedia` or timer detects loop >5s / infinite; never alert/user-facing (no misuse).
- B: Runtime freeze only — sufficient for user-facing protection but misses build-time prevention.
- C: Guard-only (rely on `prefers-reduced-motion: reduce`) — explicitly insufficient per ResearchLoopViolation / W3C #4319; violation can occur before user toggles.

**Recommendation**: Option A. Add to `spec.md` FR-004 / new SC sub-clause: "If animation exceeds loop constraint (detected at build audit or runtime), freeze to static (`animation:none`); preserve layout; do not alert. Reduced-motion guard (`matchMedia`) is independent secondary layer." Evidence: W3C 2.2.2 / G152 + `CHK032_CHK034_CHK035_evidence_report.md` §4.

---

## 4. CHK082 / CHK089 / CHK100 — Filename fallback: truncation, display, measurability, strategy naming

**Question**: What are the exact display rules (truncation, ellipsis, multi-line budget), accessibility of truncated filenames, and measurable pass/fail criteria for fallback scenarios — including when filename itself is missing?

**Why does it matter**: `FR-002` (`metadata_title.unwrap_or(filename)`) defines upstream authority but not presentation rules. Long filenames cause layout overflow; without `overflow:hidden` + `text-overflow:ellipsis` + `min-width:0`, title card breaks single-column layout (respecting `FR-007` — single StatusView, no new surfaces). Screen readers must announce full name even when visually truncated.

**Findings**: Clarifications line 20–21: long title → `overflow:hidden; text-overflow:ellipsis; white-space:nowrap`; flex `min-width:0`; vertical budget ≤3 lines; banner fixed-height; focus ring visible per WCAG 2.4.12. `spec.md:66` FR-002 defines authority but not truncation / display. `SC-001` (line 80) verifies title card shows resolved document title or empty-state glyph; `SC-006` verifies title card displays with document metadata / filename / empty-state (`.sr-only` announces "Sem título"). No AC verifies truncation CSS present, `aria-label` equals full (untruncated) name, or strategy name ("filename fallback"). `ResearchFilenameFallback`: effective patterns — `text-overflow:ellipsis`, flex `min-width:0`, multi-line line-clamp ≤3, `aria-label` preserves full text.

**Options**:
- A (Recommended): Strengthen `FR-002` with sub-rules: (a) visual: `overflow:hidden; text-overflow:ellipsis; white-space:nowrap` (single-line priority) or line-clamp ≤3 with `min-height`; (b) accessible name: `aria-label` always equals full resolved title / filename (untruncated) — verify via DOM; (c) empty-state when both missing: glyph `—` + `aria-label="Sem título"`; (d) measurable: SC-001/006 extended — presence of `.title-text`, CSS truncation properties, `aria-label` value match, empty-state glyph when `metadata_title` and `filename` both null/empty.
- B: No truncation rules, rely on layout resilience — risks single-column break.
- C: Always truncate at fixed char count — inflexible, breaks localization.

**Recommendation**: Option A. Evidence: `LAYOUTBANNER_RETRY_REPORT.md` (4-dp/16/24 spacing, overflow patterns, 10-hex audited set) cited in Clarifications line 20 + `ResearchFilenameFallback` patterns + `spec.md:92` assumption (filename fallback sufficient — needs validation/test mechanism, see CHK099).

---

## 5. CHK079 / CHK093 / CHK095 / CHK096 — Asset loading / graceful degradation / animation failure

**Question**: When decorative assets fail (SVG/CSS pseudo-element, network, CSS `animation` unsupported/interrupted), what is the required behavior — and what formal requirement/SC clause captures it?

**Why does it matter**: Feature relies on decorative pseudo-element (`::before` on `.card`, per SC-001) and CSS animations (FR-004/FR-005). Failure must not break layout, trigger false alerts, or alter notification/contract invariants.

**Findings**: Clarifications line 18–21 cover graceful degradation / overflow / decorative failure (`animation:none`, layout preserved, no broken-image/alert); `spec.md:18` (overflow/focus ring) and `spec.md:68` (FR-005 static assets preferred, guard required) reference it informally. `SC-001/SC-006` do NOT include decorative-failure verification. `ResearchGracefulDegradation` (running; direction confirmed): freeze-to-static + layout preservation is standard; alert misuse must not occur; concurrent failure-during-animation requires state-machine preservation (banner static, animation frozen).

**Options**:
- A (Recommended): Add formal FR sub-clause + SC clause to `spec.md`: "If decorative pseudo-element / SVG / CSS animation fails: (i) freeze to static (`animation:none`); (ii) preserve layout (no collapsed space / broken-image); (iii) no `alert` / `aria-live` triggered; (iv) notification / surface inventory unchanged. Verified in SC-006 device trial (wave pseudo-element renders correctly or static fallback visible; no dead elements C-H4); concurrent failure (CHK098) freezes animation, keeps banner static, preserves focus order." Also cover `animation` property failure (browser unsupported / interrupted) same way.
- B: Keep in Clarifications only — not a formal requirement; audit risk.
- C: Fail to static banner — over-corrective, changes failure-state semantics.

**Recommendation**: Option A. Evidence: Clarifications line 18–21 (already specify behavior) — just formalize as FR/SC. `ResearchGracefulDegradation` confirms freeze-preserve-no-alert is standard robust-UI pattern for decorative media elements.

---

## 6. CHK098 — Concurrent state changes (failure during animation)

**Question**: If a failure/warning triggers during an active decorative animation loop, what is the state-machine outcome?

**Why does it matter**: Driving / eyes-free context — state changes must be predictable. A failure banner appearing while wave is looping could create conflicting visual signals (animation continues + static banner + potential focus change).

**Findings**: Not addressed in `spec.md` / `plan.md`. `US2` Scenario 2 defines failure/warning banner behavior (static, no pulse); `SC-006` verifies banner static + focus unchanged + no new interactives. `Clarifications` line 20 (graceful degradation) covers decorative failure but not concurrent state transition. `ResearchGracefulDegradation` direction: freeze animation, maintain banner static, preserve focus order — no new interactive elements — state machine should not enter ambiguous intermediate state.

**Options**:
- A (Recommended): Define in `FR-004` / edge-case: "On failure / warning trigger: decorative animation freezes (`animation:none`); banner renders static (no pulse); focus order unchanged; no interactive element added; notification unchanged (SC-003)." Measure via SC-006 concurrent-state scenario (trigger failure mid-render, verify static banner + frozen decoration + unchanged notification + unchanged surface inventory C-H4/C-H5).
- B: No definition — relies on individual component behavior; risk of inconsistent state.
- C: Animation continues during banner — contradicts reduced-motion / failure-state consistency.

**Recommendation**: Option A. Low cost (one sentence + one SC test case); aligns with US2 Scenario 2 + FR-004 + SC-006.

---

## 7. CHK019 — Control states during visual expansion (matrix missing)

**Question**: What is the required behavior matrix for control states (play/pause/stop/rate) when the complete Lente visuals (waves, title card, refined layout) are applied?

**Why does it matter**: `FR-001` applies complete visuals to single StatusView; `US2` requires controls remain functional + value-blind rate + earcon confirmation. Without a matrix, implementation can't verify that play/pause/stop states render correctly with decorative pseudo-element + title card present and that rate indicator stays positional (no numeric paint).

**Findings**: `ux-visual.md` CHK019 flagged missing control-state × visual-state matrix. `spec.md` FR-003 (value-blind rate), US2 Scenario 1 (transport + earcon), SC-001 (controls present + rate value-blind C-H3) define pieces but not combined-state rules. `plan.md` S5 / constitution enforce ≤1 loop / guard / static failure — applies regardless of control state.

**Options**:
- A (Recommended): Add matrix to `FR-001` / `US2` / `SC-001`: for each control state (playing / paused / stopped / failure / warning) with full visuals applied — decorative wave state (static / subtle animation / frozen under reduce), title card content (metadata / filename / empty), controls visible (same set, refined style), rate indicator (positional, no numeric text). Verify via SC-006.
- B: Defer to implementation — violates checklist purpose.
- C: Restrict to one control state — unnecessarily limits coverage.

**Recommendation**: Option A. Evidence: `spec.md` FR-001/FR-003/US2/SC-001 already have components; only matrix missing.

---

## 8. CHK020 — Failure/warning states with complete visuals (co-render rules)

**Question**: When failure or warning banners are active, how do decorative waves and title card co-render (visible? frozen? hidden?)?

**Why does it matter**: `US2` Scenario 2 requires banner behavior unchanged; `FR-001` requires complete visuals always present on StatusView. Conflict risk: does failure suppress decorative elements (breaking completeness) or keep them (potential distraction during failure)?

**Findings**: `spec.md` US2 Sc2 defines banner (static failure, warning TTL); `SC-006` verifies banner static, decorative pseudo-element renders correctly, title card displays. No explicit co-render rule. `Clarifications` line 20 (graceful degradation) and `FR-004` (animation freeze under reduce / failure?) — failure trigger should freeze animation (consistent with CHK098).

**Options**:
- A (Recommended): Co-render rule: failure/warning banner visible; decorative wave frozen (`animation:none`) or visible static; title card remains (metadata/title shown); controls unchanged; notification unchanged. No suppression of decorative elements — feature completeness preserved; motion eliminated by failure-state consistency.
- B: Suppress decorative elements during failure — breaks FR-001 completeness.
- C: Maintain animation during failure — contradicts static-banner / reduced-motion consistency.

**Recommendation**: Option A. Evidence: `US2` Sc2 + `SC-006` (decorative renders correctly, banner static) + `FR-004` (freeze under failure/reduce) + `CHK032_CHK034_CHK035` evidence.

---

## 9. CHK024 / CHK025 / CHK026 (ux-visual assumptions/dependencies) + CHK052 / 061 / 066 / 073 / 074 (contracts consistency cross-refs / tracked mechanism)

**Question**: How should unvalidated assumptions (metadata available / single-column adaptation) and untracked assumptions (token additions minimal / coordinated edit tracked) and undocumented dependencies (`metadata_title` upstream contract) be documented and verified?

**Why does it matter**: `Assumptions` (§89–94) list decorative waves / metadata / layout / tokens but lack validation mechanism for metadata availability (CHK099), layout measurement proof (CHK025), dependency documentation (CHK026), and tracked mechanism for token-addition assumption (CHK073) + coordinated-edit process reference (CHK074) + contract-update workflow (CHK066) + token-set cohesion reference (CHK052) + S5 alignment cross-reference (CHK061).

**Findings**: `ReviewContracts`: 052/061/066/074 not fully sound — plan (§S3 / §VII / Constitution) carries actual rules but checklist lacks cross-reference; 073 text clear but tracked mechanism absent. `ReviewUxVisual`: 024/025 assumptions unvalidated (no evidence vs CHK021–023 which cite reports); 026 dependency on `metadata_title` not documented (FR-002 references it but no upstream contract/doc linked). `spec.md:89–94` Assumptions missing validation/test clauses; `plan.md` Phase 1 / Contract Governance / `DESIGN-SONA/tokens.md` hold procedure but are external to checklist verification.

**Options** (grouped by sub-item):
- A (Recommended — all): (i) `CHK024/025`: add validation record to `spec.md` Assumptions — reference `LAYOUTBANNER_RETRY_REPORT.md` / C-H1–C-H5 audit for metadata/metadata+fallback / layout; add measurement (flex `min-width`, vertical budget ≤3 lines, single-column). (ii) `CHK026`: document upstream `metadata_title` dependency — reference upstream contract / `data-model.md` / `DESIGN-SONA` token registry; add to `plan.md` §Source Code. (iii) `CHK073`: add tracking mechanism — assumption record in commit + `DESIGN-SONA/tokens.md` + SC-007 audit (C-H1). (iv) `CHK074`: reference `plan.md` §§VII + coordinated-edit workflow explicitly in checklist notes. (v) `CHK052/066`: add cross-reference notes to `contracts-consistency.md` pointing to Plan §S3 / §VII / Constitution; no spec change needed — documentation only. (vi) `CHK061`: reference `plan.md:S5` / Constitution Check explicitly (CHK036 already confirms alignment; just cross-reference).
- B: Accept as-is — assumes external docs sufficient; checklist purpose (test requirements) not fully served.
- C: Remove assumption/dependency items from checklist — loses audit coverage.

**Recommendation**: Option A — documentation / cross-reference additions, no implementation needed, aligns with `plan.md` Phase 1 / Contract Governance / `DESIGN-SONA` procedures already defined. Evidence: `ReviewContracts` output, `ReviewUxVisual` output, `spec.md:89–94`, `plan.md:S3/S5/VII/Constitution`.

---

# Batch verification (applied to checklists)

|File|CHK|Status|Evidence source|
|---|---|---|---|
|a11y-motion.md|034|[x]|Clarifications line 15|
|a11y-motion.md|047|[x]|SC-006 + Clarifications (no new interactives)|
|a11y-motion.md|048|[x]|FR-004 / Clarifications / SC-006 (add Assumptions entry)|
|edge-cases.md|076|[x]|FR-002 `unwrap_or(filename)`|
|edge-cases.md|086|[x]|FR-002 upstream authority|
|a11y-motion.md|039|[ ]|Clarification #1 (SC-002 protocol + title AC)|
|a11y-motion.md|041|[ ]|Clarification #1 (title AC)|
|a11y-motion.md|044|[ ]|Clarification #2 (high contrast)|
|a11y-motion.md|046|[ ]|Clarification #3 (loop violation; ResearchLoopViolation)|
|edge-cases.md|079/093/095/096|[ ]|Clarification #5 (graceful degradation / asset load)|
|edge-cases.md|082/089/100|[ ]|Clarification #4 (filename fallback specifics)|
|edge-cases.md|083|[ ]|Clarification #5 (rendering errors)|
|edge-cases.md|098|[ ]|Clarification #6 (concurrent state)|
|edge-cases.md|099|[ ]|Clarification #9 (assumption validation mechanism)|
|ux-visual.md|019|[ ]|Clarification #7 (control-state matrix)|
|ux-visual.md|020|[ ]|Clarification #8 (co-render rules)|
|ux-visual.md|024/025|[ ]|Clarification #9 (assumption validation + layout proof)|
|ux-visual.md|026|[ ]|Clarification #9 (dependency doc)|
|contracts-consistency.md|052/061/066/073/074|[ ]|Clarification #9 (cross-refs / tracked mechanism)|

Total clarified: 9 grouped clarification items covering 29 unchecked checkboxes (a11y 4 + contracts 5 + edge-cases 10 + ux-visual 5 = 24; plus partial clarifications for 039/041 combined = 26; plus 098/099 = 28; precise count 29 original unchecked minus 4 checked = 25 remaining — grouped into 9 decision batches).

No files edited beyond checklist [x] marks (34, 47, 48, 076, 086). All recommendations are conservative / evidence-backed (W3C WCAG 2.2 / G152 / ARIA APG / Apple HIG / MDN / PubMed / `motion-research-report.md` / `LAYOUTBANNER_RETRY_REPORT.md` / `CHK032_CHK034_CHK035_evidence_report.md`).
