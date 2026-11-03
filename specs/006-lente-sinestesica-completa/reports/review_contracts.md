# Contracts Consistency Review — CHK050 to CHK074 (006 Lente Sinestésica Completa)

Evidence base: `specs/006-lente-sinestesica-completa/` (spec.md, plan.md, checklists/*), `specs/005-adopt-design-sona/contracts/` (status-surface.md, surface-audit.md, earcon-mapping.md), `contracts/surface.md`, `.specify/memory/constitution.md` (§VII, Principles VII, S1–S6).

---

## Completeness

### CHK050 — Surface count = exactly 2 (StatusView + MediaStyle notification)
**VERDICT: PASS** (with minor ambiguity)
Evidence: spec.md §FR-007 says "maintain surface count = 2"; plan.md §Constitution Check S1 says "Exactly 2 surfaces"; contracts/status-surface.md S1 defines exactly TWO styled surfaces (notification + StatusView); contracts/surface.md §Resident-paused notification clarifies the resident notification is a DISTINCT surface but the clause governs it separately — the 2-surface count is preserved. Requirement is explicit in spec and plan.
Clarification question: Does "exactly 2 surfaces" include the resident-paused notification surface as the second, or is the resident notification considered part of the media notification? The contract splits them but counts them as the same notification id ("one id is one surface") with mutually exclusive presence.

### CHK051 — Notification purity fully specified (accent-only, no body/rate/voice/mark)
**VERDICT: PASS**
Evidence: spec.md §FR-006: "Notification remains accent-only (no body/rate/voice/mark)"; plan.md §S6 references S6; contracts/status-surface.md S6 (MediaStyle constants block) defines ONLY accent `--sona-voice #F2F0E6` (`:728-743`); contracts/status-surface.md §FR-007 (notification purity) defines the invariant: no brand logo, no sound-wave graphic, accent-only + title + progress. Clear and complete.

### CHK052 — Closed token set with guidance on additions
**VERDICT: UNCLEAR**
Evidence: spec.md §FR-001: "respecting closed token set where feasible"; §Assumptions: "Token set may need minimal coordinated additions only if unavoidable; prefer reuse"; plan.md §S3 references S3 and says "Closed token set (prefer reuse; additions need justification)". There is guidance (prefer reuse; additions only unavoidable) but the exact mechanism (what document needs update) relies on contract discipline (§VII) rather than a specific token registry reference. Not ambiguous, but could be tighter.
Clarification question: Which document is the authoritative token registry — `data-model.md` §SonaToken? Should spec.md explicitly cite it?

### CHK053 — Contract update requirements when tokens/layout change
**VERDICT: PASS**
Evidence: spec.md §FR-007: "Update contracts if tokens/layout change"; contracts/status-surface.md S7 (Change discipline): "Any element addition, token addition, or motion addition = coordinated edit to this contract + `data-model.md` + the spec (§VII discipline, semver treatment at release)"; plan.md §Contracts notes coordinated edit per §VII.

### CHK054 — Surface inventory (A1–A4, no dead `display:none`)
**VERDICT: PASS**
Evidence: spec.md §SC-005 references C-H4/C-H5; contracts/surface-audit.md A2 defines C-H4 (element inventory — anything outside §SurfaceInventory or missing one of 9 entries; structural-dead removals must be removals, no `display:none` corpses); A4 (whitelist discipline) reinforces. The checklist references A1-A4 explicitly. Requirement is clear, though "A1-A4" is shorthand for the audit sections.

---

## Clarity

### CHK055 — "Coordinate changes" unambiguous (what requires coordinated edit)
**VERDICT: UNCLEAR**
Evidence: contracts/status-surface.md S7 states coordinated edit applies to "element addition, token addition, or motion addition"; plan.md references §VII Coordinated edit; constitution.md §VII (Closed Sets, No Catch-Alls) says closed sets must not gain catch-alls. But what exactly requires coordinated edit? The spec mentions token/layout changes (§FR-007) and contracts mention element/token/motion additions. There is no explicit list mapping which spec changes trigger which contract files.
Clarification question: For feature 006, is a decorative wave CSS pseudo-element addition an "element addition" requiring contract update? If title card adds a `.card` container, does that trigger C-H4 inventory update?

### CHK056 — "Prefer reuse" vs "additions need justification" clearly defined
**VERDICT: UNCLEAR**
Evidence: spec.md §Assumptions: "Token set may need minimal coordinated additions only if unavoidable; prefer reuse"; plan.md §S3 says "Closed token set (prefer reuse; additions need justification)". The distinction is present but the bar for "justification" is not defined (what document records it? does it require design-dir alignment? semver bump?).
Clarification question: Where is justification recorded — in commit message, in contract changelog, or in a dedicated justification document? Does addition of one decorative token trigger a MINOR or PATCH version bump per §VII?

### CHK057 — 10 token members / closed set definition referenced
**VERDICT: GAP**
Evidence: contracts/status-surface.md S3 references `data-model.md` §SonaToken and the design-dir ledger (`DESIGN-SONA/tokens.md`). contracts/surface.md refers to DESIGN-SONA palette tokens (title `#F2F0E6`, section `#A9B8CE`, progress `#55688F`, yellow `#FFD400`, red `#FF4D63`, green prohibited). contracts/surface-audit.md A4 references the allowed hex set owned by contract + `data-model.md`. However, the spec.md for 006 does NOT explicitly reference the 10-member set or `data-model.md` §SonaToken. The contracts-consistency checklist notes this as a gap ("Are the 10 token members or closed set definition referenced?").
Clarification question: Should spec.md include a normative reference to `data-model.md` §SonaToken (e.g., "All painted colors MUST be members of the 10-token closed set defined in `data-model.md` §SonaToken")?

### CHK058 — "No new surfaces" unambiguous (StatusView expansion vs new surface)
**VERDICT: PASS**
Evidence: spec.md §Overview: "while keeping the single StatusView surface"; §FR-001: "Apply complete Lente Sinestésica visuals to single StatusView"; §FR-007: "maintain surface count = 2"; contracts/status-surface.md S1: exactly two surfaces; S7: any element addition requires coordinated edit but does not create a new surface. The distinction is clear: expanding the existing StatusView (new decorative pseudo-element, title card) ≠ creating a new screen/surface. Plan §S1 confirms.

---

## Consistency

### CHK059 — Requirements align with feature 005 contracts (status-surface.md, surface-audit.md)
**VERDICT: PASS**
Evidence: spec.md §FR-001, §FR-003, §FR-004, §FR-005, §FR-006, §FR-007 all reference or mirror the 005 contract clauses. Plan §Contracts explicitly references `contracts/status-surface.md`, `contracts/surface-audit.md`, `contracts/earcon.md`. The checklist itself notes alignment. No contradictions detected: S5 movement limits are preserved, S6 notification purity is preserved, S3 token rules are preserved.

### CHK060 — Notification requirements consistent across spec and plan (accent-only invariant)
**VERDICT: PASS**
Evidence: spec.md §FR-006 and §US3 (SC-003) both state accent-only; plan.md §S6 references S6; contracts/status-surface.md S6 defines the constants block; contracts/surface.md §FR-007 defines notification purity. All consistent.

### CHK061 — Movement/animation requirements align with S5 in constitution
**VERDICT: PASS** (with clarification needed on decorative animation count)
Evidence: spec.md §FR-004: reduced-motion guard; §FR-005: static preferred, no new continuous loops beyond S5; plan.md §S5 references S5; contracts/status-surface.md S5 defines continuous motion set = {pill}, reduced-motion freeze, ≤3s TTL. The decorative wave animation (if CSS animation) must comply with S5 limits. Spec says "must not create new continuous loops beyond S5 limits" which aligns. But does the decorative wave animation count as a continuous loop? If it loops continuously, it would violate S5 (only pill allowed). The spec says "≤1 continuous loop" in plan constraints, implying decorative animation should NOT loop continuously.
Clarification question: Is decorative wave animation allowed to loop continuously? If yes, does it violate S5's single-loop limit? Should it be bounded (e.g., one-shot or short TTL)?

### CHK062 — Rate requirements consistent (value-blind, positional only) across all stories
**VERDICT: PASS**
Evidence: spec.md §FR-003: "Rate remains value-blind (positional only), no painted numeric rate"; §US2 Scenario 1: "rate remains value-blind"; plan.md §S3 references value-blindness; contracts/status-surface.md S4 (Value-blindness invariant) defines zero painted numeric rate; contracts/surface.md §Rate defines no value display. Consistent across spec, plan, contracts, user stories.

---

## Acceptance Criteria Quality

### CHK063 — SC-003 (Notification purity unchanged — accent-only) objectively verifiable
**VERDICT: PASS**
Evidence: SC-003 is a binary invariant: only `--sona-voice` accent painted in notification; contracts/status-surface.md S6 defines the constants block; contracts/surface-audit.md C-H3 defines the hex-check for MediaStyle (only `#F2F0E6`). Objective, grep-verifiable, byte-comparable. The checklist asks "Can it be objectively verified?" — yes, via C-H3 audit.

### CHK064 — SC-005 (Surface inventory valid; no dead `display:none`; aria inventory correct) objectively verifiable
**VERDICT: PASS**
Evidence: SC-005 references C-H4 (element inventory — no extra/hidden/dead elements; structural removals) and C-H5 (aria inventory — labels, progressbar roles, live region, no legacy alert+polite pair). Both are defined in contracts/surface-audit.md A2 and are verifiable via host audit scripts (`verify.sh`). Clear, measurable.

### CHK065 — Contract compliance criteria defined for any token additions
**VERDICT: GAP**
Evidence: There is no explicit acceptance criterion defining how a token addition is verified against the contract. The contracts (status-surface.md S7) say coordinated edit is required, and data-model.md is the authority, but the spec does not define a measurable acceptance criterion like "Any new hex value appears in `data-model.md` §SonaToken and passes C-H1 census". The checklist notes this as a gap.
Clarification question: Should SC add an explicit criterion: "Any token addition must be registered in `data-model.md` §SonaToken and pass host hex census (C-H1) before merge"?

---

## Scenario Coverage

### CHK066 — Requirements for contract update workflow if changes needed
**VERDICT: UNCLEAR**
Evidence: spec.md §FR-007 mentions updating contracts; contracts/status-surface.md S7 defines coordinated edit process. But the spec does not describe the workflow steps (e.g., which file is edited first, which files must be synchronized, whether version bump is required). Plan §Contracts references coordinated edit per §VII, but does not detail the sequence.
Clarification question: Should the spec include a step-by-step contract update workflow (e.g., 1. Update contract, 2. Update data-model.md, 3. Update spec, 4. Verify via verify.sh)?

### CHK067 — Requirements defined for notification during all StatusView states with complete UI
**VERDICT: PASS** (with note)
Evidence: spec.md §US3 (notification during complete UI) covers notification purity; contracts/status-surface.md covers notification consistently regardless of StatusView state (S6 is independent of StatusView content). Since the notification surface is separate from StatusView, the notification requirements apply uniformly. The spec does not explicitly state "for all StatusView states" but since notification is a distinct surface, this is implicitly covered.
Clarification question: Should spec.md explicitly state that notification purity holds independently of StatusView state (e.g., whether title card is present or empty, whether decorative waves load)?

### CHK068 — Requirements defined for surface audit after changes
**VERDICT: PASS**
Evidence: spec.md §SC-005 references surface audit; contracts/surface-audit.md A3 describes where checks run (`cargo nextest`, `verify.sh`); plan.md §Implementation Strategy step 7 mentions surface-audit checks. The checklist references Plan §Implementation Strategy. Requirements are present but brief.
Clarification question: Does the spec need an explicit audit trigger condition (e.g., "After any contract edit, run `scripts/verify.sh` gate `surface-audit` before merge")?

---

## Edge Case Coverage

### CHK069 — Behavior defined if token additions violate closed set
**VERDICT: GAP**
Evidence: No explicit behavior is defined in spec.md or contracts for what happens if a token addition violates the closed set (e.g., uses a non-SonaToken hex, or introduces a new color family). The contracts say coordinated edit is required (S7) and audit fails (C-H1), but there is no described remediation behavior (fail-closed? rollback?).
Clarification question: Should spec.md or contracts define the failure mode: "If a token addition violates the closed set, the host audit (C-H1) fails and the change is blocked (fail-closed per Principles III–V)"?

### CHK070 — Behavior defined if layout change affects surface count
**VERDICT: GAP**
Evidence: No explicit behavior is defined. The contract (S1) defines exactly two surfaces; spec (FR-007) says maintain surface count = 2. But if a layout change accidentally introduces a new screen or hidden handler, the audit (C-H4) detects it. There is no explicit spec clause stating the remediation.
Clarification question: Should there be an explicit clause: "Any layout change that introduces a third styled surface or hidden route fails the gate (C-H4 / surface count)"?

### CHK071 — Requirements for removing dead CSS specified
**VERDICT: GAP**
Evidence: contracts/status-surface.md S4 mentions structural-dead removals; contracts/surface-audit.md C-H4 requires removals (not `display:none`). Spec.md SC-005 mentions fixing C-H4. But there is no explicit requirement in spec.md defining when and how dead CSS is removed (e.g., "If an element is removed, its CSS rules must also be removed from the audited file").
Clarification question: Should spec.md include a dead-CSS removal requirement: "When an element is removed from SurfaceInventory, its associated CSS selectors must be removed from `status_view.html` and pass C-H4 audit"?

---

## Dependencies & Assumptions

### CHK072 — Dependency on existing contracts (005-adopt-design-sona/contracts) documented
**VERDICT: PASS**
Evidence: plan.md §Source Code references explicitly lists `specs/005-adopt-design-sona/contracts/status-surface.md`, `surface-audit.md`, `earcon-mapping.md`; contracts/status-surface.md header references feature 005; contracts/README.md establishes naming convention. Dependency is clearly documented.

### CHK073 — Assumption "token set may need minimal coordinated additions only if unavoidable" tracked
**VERDICT: PASS** (with tracking gap)
Evidence: spec.md §Assumptions explicitly states the assumption; plan.md §S3 references it; contracts/status-surface.md S7 references coordinated edit for token additions. The assumption is tracked in the spec document. However, there is no separate tracking mechanism (e.g., a risk register or assumption validation task) beyond the assumption line.
Clarification question: Should there be a validation step (e.g., in verification or plan) confirming whether any token addition was unavoidable? Should it reference a justification entry?

### CHK074 — Coordinated edit process requirement clear (follows §VII)
**VERDICT: UNCLEAR**
Evidence: contracts/status-surface.md S7 references §VII discipline; plan.md §Constitution Check references §VII Coordinated edit; constitution.md §VII (Closed Sets, No Catch-Alls) defines the principle. But the spec.md does not explicitly describe the coordinated edit process (what files, in what order, with what version treatment). It relies on the contract and constitution references.
Clarification question: Should spec.md include an explicit coordinated-edit procedure (e.g., "Any contract/token change requires: (a) update contract file, (b) update `data-model.md`, (c) update spec.md, (d) version bump per §VII, (e) verify via verify.sh")?

---

## Summary Table

| CHK | Verdict | Key Evidence / Gap |
|-----|---------|-------------------|
| 050 | PASS | Spec FR-007, Plan S1, Contract S1 |
| 051 | PASS | Spec FR-006, Contract S6 |
| 052 | UNCLEAR | Guidance present; registry reference implicit |
| 053 | PASS | Spec FR-007, Contract S7 |
| 054 | PASS | Spec SC-005, Contract C-H4/C-H5 |
| 055 | UNCLEAR | §VII referenced; no explicit mapping for 006 elements |
| 056 | UNCLEAR | "Prefer reuse" stated; justification mechanism undefined |
| 057 | GAP | 10-token set / `data-model.md` not referenced in spec |
| 058 | PASS | Spec FR-001/FR-007, Contract S1 |
| 059 | PASS | Plan contracts list aligns with 005 contracts |
| 060 | PASS | Spec FR-006, Plan S6, Contract S6 consistent |
| 061 | PASS | Spec FR-004/FR-005, Plan S5, Constitution S5; decorative loop clarification needed |
| 062 | PASS | Spec FR-003/US2, Contract S4, Plan S3 |
| 063 | PASS | Contract C-H3 + audit gives objective verification |
| 064 | PASS | Contract C-H4/C-H5 + audit gives objective verification |
| 065 | GAP | No measurable token-addition compliance criterion defined |
| 066 | UNCLEAR | Contract update mentioned; workflow steps undefined |
| 067 | PASS (note) | Notification independent of StatusView; clarification possible |
| 068 | PASS | Audit references in SC-005, Plan Implementation, Contract A3 |
| 069 | GAP | No failure-mode behavior specified for closed-set violation |
| 070 | GAP | No failure-mode behavior specified for surface-count change |
| 071 | GAP | Dead CSS removal not explicitly required in spec |
| 072 | PASS | Plan §Source Code, Contract header reference |
| 073 | PASS | Spec §Assumptions tracks assumption; validation mechanism unclear |
| 074 | UNCLEAR | §VII referenced; explicit process steps not in spec |

---

## Proposed Clarifications (Ambiguous / Gap items)

**For UNCLEAR/GAP items (CHK052, 055, 056, 057, 061, 065, 066, 069, 070, 071, 073, 074):**

1. **CHK052 / 057**: Add a normative reference in spec.md: "All painted colors MUST be members of the closed token set defined in `specs/005-adopt-design-sona/data-model.md` §SonaToken (10 members; see contract `status-surface.md` S3 and audit `surface-audit.md` C-H1)."
2. **CHK055**: Clarify in spec.md or contracts what elements in 006 require coordinated edit: decorative pseudo-element (`::before` on `.card`), title card (`.card`), refined control layout. Does a CSS-only layout change trigger coordinated edit?
3. **CHK056**: Define justification mechanism: where is the justification recorded? Suggest: "Any token addition requires a justification entry in the feature's commit message and an update to `DESIGN-SONA/tokens.md` (post-v2.2.0 alignment); no addition without justification passes audit."
4. **CHK057**: Explicitly list or reference the 10 token members in spec.md or add a cross-reference line.
5. **CHK061**: Clarify decorative animation loop constraint: "Decorative wave animations MUST NOT loop continuously (must be either one-shot or bounded by a TTL ≤3 s); only the pill (`.pill`) is permitted as a continuous loop per S5."
6. **CHK065**: Add an SC clause: "SC-007: Any token addition passes C-H1 census and is registered in `data-model.md` §SonaToken; unregistered hex values fail the audit (fail-closed)."
7. **CHK066 / 074**: Define the coordinated edit workflow explicitly: sequence of files, version treatment, audit trigger.
8. **CHK069 / 070 / 071**: Add failure-mode clauses: what happens when the audit fails? Suggest referencing Principles III–V (fail-closed gates) and specifying rollback/block behavior.
9. **CHK073**: Add a validation task in verification/plan: "Confirm whether any token addition was unavoidable; if yes, record justification; if no, revert."

All PASS items have sufficient evidence in the spec, plan, contracts, and constitution. GAP/UNCLEAR items are missing either explicit references, measurable criteria, or failure-mode definitions required by the contract discipline (§VII, Principles III–V).
