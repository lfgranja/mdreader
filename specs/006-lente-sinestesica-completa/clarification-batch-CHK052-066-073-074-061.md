# Clarification Batch — CHK052 / CHK066 / CHK073 / CHK074 / CHK061

**Batch id:** 006-lente-clarify-CHKS-052-066-073-074-061  
**Status:** READ-ONLY — no edits to contracts/spec/plan/constitution; proposes clarifications only  
**Source evidence base:**
- `.specify/memory/constitution.md` §VII (Closed Sets, No Catch-Alls, l.912–921) + §VII principle (addition = breaking change, coordinated edits, semver bump, docs update; no new members expected)
- `specs/006-lente-sinestesica-completa/checklists/contracts-consistency.md` (CHK052/061/066/073/074 marked `[ ]` — UNCLEAR/GAP)
- `review_contracts.md` (evidence + verdicts 052 UNCLEAR / 061 PASS-clarify / 066 UNCLEAR / 073 PASS-tracking-gap / 074 UNCLEAR; proposed clarifications #1–#9)
- `specs/006-lente-sinestesica-completa/clarifications.md` §9 (l.145–156) — CHK052/061/066/073/074 cross-refs; Options A/B/C; **Option C accepted** for decorative animation (l.13–14 spec.md session 2026-10-27: duration ≤250ms/≤400ms, no infinite, area ≤1/3 viewport, `prefers-reduced-motion` guard, freeze-not-hide)
- `specs/005-adopt-design-sona/contracts/surface-audit.md` C-H1 (hex census, l.20) / C-H4 (element inventory / no `display:none` corpses, l.23); `data-model.md` §SonaToken (10-member closed set); `DESIGN-SONA/tokens.md` (closed-set rule + hex table, 10 tokens, "Nenhum token fora da lista permitido sem edição coordenada + bump + docs (§VII)")
- `specs/006-lente-sinestesica-completa/spec.md` §FR-001 (l.25) / §FR-004/005 (reduced-motion, S5) / §FR-007 (l.71 — contract updates) / §Assumptions (l.95–100 — token assumption); `plan.md` §S3 (l.29 — closed token, additions need justification) / §S5 (l.30 — ≤1 continuous loop, reduced-motion guard, yellow pulse ≤3s) / §Constitution Check (l.22–31)
- `specs/005-adopt-design-sona/contracts/status-surface.md` S3 (token reference) / S5 (movement) / S7 (coordinated edit / §VII discipline)

---

## Required format (per item)
- **STATUS:** `SOUND` (evidence sufficient, no change required; justify) OR `CLARIFICATION` (gap/ambiguous; specify required addition / cross-ref / validation mechanism)
- **CHECK:** CHK number + label from checklist
- **EVIDENCE:** concrete file / section / line reference (no inference)
- **FINDING:** one-line verdict + reason (matches `review_contracts.md` verdict + clarification question)
- **REQUIRED FORMAT / JUSTIFICATION:** exactly what must appear (normative citation, workflow step, validation clause) — written as specification-language, not todo

---

## CHK052 — Closed token set with guidance on additions

- **STATUS:** CLARIFICATION (was UNCLEAR — review_contracts.md l.18–21; checklist l.13 `[ ]`)
- **CHECK:** CHK052 — Is closed token set requirement specified with guidance on additions? [Completeness, Spec §FR-001, Plan §S3]
- **EVIDENCE:** `constitution.md` §VII (l.912–921) defines addition = breaking change + coordinated edit + semver + docs; `DESIGN-SONA/tokens.md` l.5–26 (10 hex tokens, closed-set rule \+ justification rule); `data-model.md` §SonaToken (authority); `plan.md` §S3 (l.29 — "Closed token set (prefer reuse; additions need justification)"); `spec.md` §Assumptions l.100 ("Token set may need minimal coordinated additions only if unavoidable; prefer reuse"); `contracts/status-surface.md` S3 references `data-model.md`; `review_contracts.md` l.186 proposes normative citation.
- **FINDING:** Guidance exists (reuse-first + unavoidable-only + §VII discipline + tokens.md registry), but `spec.md` lacks an authoritative, bounded reference to the 10-member set. No catch-all arm permitted (§VII). Without citation, a new hex could silently enter via decorative wave CSS without triggering C-H1.
- **REQUIRED FORMAT / JUSTIFICATION:** Add to `spec.md` §FR-001 (or new §Token-Set clause), normative: *"All painted colors MUST be members of the 10-token closed set defined in `specs/005-adopt-design-sona/data-model.md` §SonaToken (see `DESIGN-SONA/tokens.md`; audit C-H1). Addition of any token requires (a) justification entry in commit message, (b) update to `DESIGN-SONA/tokens.md`, (c) update to `contracts/status-surface.md` S3 + `data-model.md` §SonaToken, (d) coordinated edit per `constitution.md` §VII, (e) semver bump (MINOR, per §VII breaking-change rule), (f) verification via `scripts/verify.sh` C-H1 census. No new members expected (no new token family required for decorative wave; static SVG / CSS pseudo-element uses existing `--sona-void`/`--sona-stage`/accent tokens only)."* Reason: §VII requires the justification + 3-file + source + commit form; tokens.md / C-H1 are the registries.

---

## CHK066 — Requirements for contract update workflow if changes needed

- **STATUS:** CLARIFICATION (was UNCLEAR — review_contracts.md l.96–99; checklist l.35 `[ ]`)
- **CHECK:** CHK066 — Are requirements defined for contract update workflow if changes needed? [Coverage, Spec §FR-007]
- **EVIDENCE:** `spec.md` §FR-007 (l.71 — "Update contracts if tokens/layout change; maintain surface count = 2"); `contracts/status-surface.md` S7 (coordinated edit to this contract + `data-model.md` + spec, §VII discipline, semver at release); `plan.md` §Contracts references coordinated edit per §VII; `constitution.md` §VII (breaking-change process); `review_contracts.md` l.192 proposes step-by-step; no step sequence exists in spec.
- **FINDING:** Contract update is required (S7 / FR-007) but spec names the trigger and not the ordered procedure, version treatment, or audit trigger. Without sequence, a decorative wave addition could edit spec alone, skipping `data-model.md` and failing C-H1 / C-H4.
- **REQUIRED FORMAT / JUSTIFICATION:** Add to `spec.md` §FR-007 (expand) or `plan.md` §Contracts, step sequence (normative, ordered): *"When tokens or layout change: (1) edit `contracts/status-surface.md` first (S7 discipline); (2) update `specs/005-adopt-design-sona/data-model.md` §SonaToken if any hex added; (3) update `DESIGN-SONA/tokens.md`; (4) update this spec (`spec.md` §FR-001/FR-004/FR-005); (5) record justification in commit message (§VII); (6) bump semver MINOR (§VII breaking change); (7) verify `scripts/verify.sh` (C-H1 hex census + C-H4 inventory + C-H5 aria + C-H6 movement); (8) verify C-H1/C-H4 pass before merge. Atomic commit with summary; no partial updates."* Justification: §VII requires coordinated 3-file workflow + source + commit justification; C-H1/C-H4 must pass.

---

## CHK073 — Assumption "token set may need minimal coordinated additions only if unavoidable" tracked

- **STATUS:** SOUND (with tracked-mechanism clarification — review_contracts.md l.138–142; checklist l.46 `[ ]` on tracked mechanism)
- **CHECK:** CHK073 — Is assumption "token set may need minimal coordinated additions only if unavoidable" tracked? [Assumption, Spec §Assumptions]
- **EVIDENCE:** `spec.md` §Assumptions l.100 — assumption present; `plan.md` §S3 l.29 — references preference + justification; `contracts/status-surface.md` S7 — coordinated edit enforced; `review_contracts.md` l.139 — PASS "tracked in spec document"; l.141 — tracking gap (no validation step / commitment record); `clarifications.md` l.149 lists CHK073; Option A (l.154) proposes "assumption record in commit + `DESIGN-SONA/tokens.md` + SC-007 audit (C-H1)"; no additions required (wave uses existing tokens).
- **FINDING:** Assumption IS tracked (spec + plan + contract). No token addition expected (decorative wave is static SVG / CSS pseudo-element on existing palette — no new hex needed; confirm via C-H1 census that only existing hexes appear). Tracking mechanism (validation clause) is the remaining gap, not the assumption itself.
- **REQUIRED FORMAT / JUSTIFICATION:** Sound — retain assumption with added validation clause in verification / spec: *"Validation (SC-007 / assumption audit): confirm whether any token addition was unavoidable. If yes — record justification in commit + `DESIGN-SONA/tokens.md` + `data-model.md` §SonaToken + pass C-H1 citation; if no — confirm by C-H1 census (only 10 registered hexes present) and record "no addition" in commit summary. No addition required for decorative wave — confirm via audit."* Justification per §VII: if unavoidable, justification must exist; if not, census confirms closed-set compliance (C-H1). No new members expected.

---

## CHK074 — Coordinated edit process requirement clear (follows §VII)

- **STATUS:** CLARIFICATION (was UNCLEAR — review_contracts.md l.143–146; checklist l.47 `[ ]`)
- **CHECK:** CHK074 — Is the coordinated edit process requirement clear (follows §VII)? [Dependency, Plan §§VII]
- **EVIDENCE:** `constitution.md` §VII (l.912–921) defines the principle (breaking change, coordinated edits, docs, semver); `plan.md` l.22–31 (§Constitution Check S3/S5) references §VII; `contracts/status-surface.md` S7 references §VII discipline + semver at release; `spec.md` references §FR-007 but never lists the file sequence; `review_contracts.md` l.192 proposes explicit procedure; no failure-mode clause present.
- **FINDING:** The process is defined at constitution + contract level (S7 = 3-file + data-model + spec + §VII + commit justification + semver), but `spec.md` lacks an explicit procedure, blocking verification that a decorative wave edit complies.
- **REQUIRED FORMAT / JUSTIFICATION:** Add to `spec.md` (near §FR-007 / §Assumptions) the procedure (same as CHK066 above, cross-ref §VII): *"Coordinated edit per `constitution.md` §VII (Closed Sets, No Catch-Alls): source = `contracts/status-surface.md` (S7) + `data-model.md` §SonaToken; update `DESIGN-SONA/tokens.md`; update `spec.md`; justification in commit; atomic commit with summary; verify C-H1/C-H4; version bump MINOR; no catch-all arm added."* Confirm with `spec.md` §VII reference (add if absent) and `plan.md` §S3/§S5. Reason: §VII requires clarity for every closing of a closed set; without procedural reference, a wave edit could skip data-model.

---

## CHK061 — Movement/animation requirements align with S5 in constitution

- **STATUS:** SOUND (with decorative-loop clarification — review_contracts.md l.66–69; checklist l.26 `[ ]`; `clarifications.md` Option C adopted, l.13–14)
- **CHECK:** CHK061 — Do movement/animation requirements align with S5 in constitution? [Consistency, Plan §S5]
- **EVIDENCE:** `constitution.md` §VII (closed-set / S5 reference via plan); `plan.md` §S5 l.30 — "≤1 continuous loop, reduced-motion guard, yellow pulse ≤3s, no pulse on failure, banner ≤200ms"; `spec.md` §FR-004 (reduced-motion guard) / §FR-005 (static preferred; no new continuous loops beyond S5) l.68–69; `contracts/status-surface.md` S5 (continuous motion = {pill}; reduced-motion freeze; ≤3s TTL); `clarifications.md` session 2026-10-27 l.13 – decorative animation limits: allowed props opacity/translate≤10px/scale≤1.05, duration ≤250ms/≤400ms, **no infinite loops**, area ≤1/3 viewport, `prefers-reduced-motion` guard (`animation:none` under reduce); `motion-research-report.md` / `a11y-motion-review-report.md` backing; `review_contracts.md` l.190 proposes: "Decorative wave animations MUST NOT loop continuously (must be either one-shot or bounded by a TTL ≤3 s); only the pill (`.pill`) is permitted as a continuous loop per S5."
- **FINDING:** Alignment is confirmed. Decorative wave is NOT permitted as continuous loop (only `.pill` continuous per S5). All Option-C limits apply: duration cap (≤400ms max, ≤250ms preferred), no `infinite`, guard present, area capped, freeze (not hide). No new continuous loop introduced.
- **REQUIRED FORMAT / JUSTIFICATION:** Sound — confirm with explicit clause in `spec.md` §FR-004/FR-005 (if missing): *"Decorative wave animation (CSS pseudo-element on `.card`) MUST comply with S5 / Option C: duration ≤400ms (≤250ms preferred), permitted properties opacity/translate≤10px/scale≤1.05 only, no `animation-iteration-count: infinite`, max 1 iteration or TTL ≤3s, area ≤1/3 viewport, mandatory `@media (prefers-reduced-motion: reduce) { animation: none; }` (freeze, not hide), `aria-hidden="true"`. Only `.pill` permitted as continuous loop. Verify via C-H6 (movement census) in `verify.sh`."* Justification: S5 (constitution via plan/contract) + Option C evidence (WCAG 2.2/2.3, MDN, PubMed >35% vestibular by 40) + C-H6 audit = objective verification.

---

## Cross-item summary (no edit performed — read-only)

| CHK | Status | Key justification reference |
|-----|--------|----------------------------|
| 052 | CLARIFICATION | `spec.md` needs normative citation to `data-model.md` §SonaToken + 10-member registry + §VII justification form |
| 066 | CLARIFICATION | `spec.md` needs ordered 3-file + source + commit + version + audit sequence (same as 074) |
| 073 | SOUND (+ validation) | Assumption tracked; add SC-007 / C-H1 validation clause; no token addition needed (wave = existing palette) |
| 074 | CLARIFICATION | `spec.md` needs explicit coordinated-edit procedure referencing `constitution.md` §VII + file sequence |
| 061 | SOUND (+ loop clarification) | Option C (≤400ms, no infinite, guard, ≤1 loop, area≤1/3) adopted; decorative wave ≠ continuous loop; verify C-H6 |

**No contract, spec, plan, or constitution file was modified.** This batch proposes the exact normative insertions required by the clarification questions (review_contracts.md l.186–195) and by §VII, with required format (justification entry + `DESIGN-SONA/tokens.md` + `data-model.md` + atomic commit + C-H1/C-H4 verification + MINOR bump, no catch-all, no new members expected for decorative wave). Confirm with `spec.md` §VII references and `plan.md` §§S3/S5 before any edit.
