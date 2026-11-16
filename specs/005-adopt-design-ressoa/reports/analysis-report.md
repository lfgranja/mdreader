Analysis of specs/008-ref-device-emulator/ files: duplication, ambiguity, underspecification, inconsistency, missing line refs. No edits made.

=== 1. NEAR-DUPLICATE DESCRIPTIONS ===

- quickstart.md:214 exclusions list == contracts/VERIF-EMULATOR-structural.md:185 exclusions == VERIF-EMULATOR.md:267-274 exclusions (audio-output/SC-009/Bluetooth/car/telephony/10/10) — copied verbatim with no value-add.
- quickstart.md:213 verified-contracts list (manifest/service/notification/focus/diag/resume/rate) == VERIF-EMULATOR.md:255-262 verified list == contracts:184 verification-rules == spec.md:105 FR-001 — near-copies across 4 files.
- plan.md:13 exclusions (audio-output, SC-009, Bluetooth/car, 10/10) is a shorter duplicate of contracts exclusions; missing telephony/alarm and MAY_DUCK audio-volume items present in spec.md/contracts.

=== 2. AMBIGUITY ===

- "structural framework" / "structural verification path" — plan.md:9, contracts:9, data-model.md:5. No exact mapping to which framework contracts (only 3 named contracts referenced; 7 framework contracts implied). Not tied to FR-001 line numbers.
- "verified contracts" — contracts:11, VERIF-EMULATOR.md:255. No contract-file mapping given for manifest/service/focus/diag/resume/rate (only notification/status-surface covered by named file).
- "verified framework contracts" — plan.md:13 references 7 items but contracts file only names 3 files (status-surface, earcon-mapping, surface-audit). Ambiguous whether the other 4 have contract definitions.
- quickstart.md:205 "Open `003-reference-device` parsed Markdown" — ambiguous which file/section (plan.md? data-model.md? spec.md?). No spec.md line reference.

=== 3. UNDERSPECIFICATION ===

a) Contracts missing 4 of 7 framework contracts:
- contracts/VERIF-EMULATOR-structural.md:175-177 references only status-surface.md (notification), earcon-mapping.md, surface-audit.md.
- Verification rules contracts:184 list manifest/service/notification/focus/diag/resume/rate — but manifest/service/focus/diag/resume/rate have NO named contract-file references (only notification via status-surface). Missing 6 contract-file links (only 1 covered).

b) Quickstart step references don't fully match VERIF-EMULATOR.md sections by number:
- quickstart.md:212-217 writes attestation referencing sections 2/3/4/6 but never cites section numbers explicitly.
- quickstart.md:216 references branch `005-adopt-design-eta` (typo; should be `005-adopt-design-sona` per quickstart.md:4 and plan.md:3).
- quickstart.md:203-217 has 7 steps; VERIF-EMULATOR.md has sections 1-7 + attachments. Mapping is implicit, not explicit by section number.

c) Data-model.md missing fields / entities:
- spec.md:115 references `§MoveSpec` (existing `003-reference-device` structural-state entity) but data-model.md:143-150 never includes `§MoveSpec`.
- data-model.md:138 lists attestation fields (feature-id/branch/date/AVD version/verified contracts list/exclusions/reference/sign-off) but does not specify the exact 7 framework contracts by name or reference spec.md lines for each.
- data-model.md:150 says "No new fields added" but doesn't confirm which existing fields are verified by the layer; missing explicit field-level mapping to FR-001.

d) Missing spec.md line-number references:
- contracts/VERIF-EMULATOR-structural.md:175-177 reference contracts but do not cite spec.md FR-001/FR-002/SC-004 line numbers for exclusions/verified items.
- data-model.md:138-141 references contracts but does not cite spec.md line numbers for attestation schema or exclusions.
- quickstart.md:228 references `spec.md` FR-001..FR-006; SC-001..SC-004 but does not cite specific line numbers.
- plan.md:73 mentions `VERIF-EMULATOR.md` should reference contracts but gives no spec.md line refs.

e) quickstart.md step 6 includes telephony/alarm exclusion (line 214) but quickstart.md step 5 (line 212) doesn't reference VERIF-EMULATOR.md section 5 (security/consent boundary) — missing reference to section 5.

=== 4. INCONSISTENCY ===

a) Plan.md exclusions ≠ spec.md FR-002 exclusions:
- plan.md:13 exclusions = audio-output, SC-009, Bluetooth/car routing, 10/10 on-device.
- spec.md:106 FR-002 exclusions = audio-output (`22050 Hz`), `SC-009` ≤3 s, real `MAY_DUCK` audio-volume (~20%), telephony/alarm interruption, Bluetooth/car routing, 10/10 on-device trials.
- Plan.md MISSING: real `MAY_DUCK` audio-volume measurement and telephony/alarm interruption (both present in contracts exclusions line 185 and VERIF-EMULATOR.md section 3 lines 270-271).

b) Data-model.md entities ≠ spec.md entities:
- spec.md:115 references `§MoveSpec` but data-model.md:143-150 does not list it.
- spec.md:115 also references `§MoveSpec` as existing entity; data-model.md mentions `§SurfaceInventory`, `§NotificationSurface`, `§ProgressIndication`, `§RateLadder`, `§DiagnosticEvent`, `§DesignDirState` — missing `§MoveSpec`.

c) Quickstart branch typo vs plan/spec:
- quickstart.md:216 writes `005-adopt-design-eta`; correct branch is `005-adopt-design-sona` (plan.md:3, quickstart.md:4, spec.md:4, data-model.md:4, contracts:4, VERIF-EMULATOR.md:245).

d) Research.md design artifacts claim vs actual file:
- research.md:323 claims `VERIF-EMULATOR.md` has "sections 1-8"; actual VERIF-EMULATOR.md has sections 1-7 (§7 is Attachments/References), so section count is off by 1.

e) VERIF-EMULATOR.md §5 vs agent recommendation (research.md:34 / clarify-sec-chk001):
- §5 (VERIF-EMULATOR.md:285-287) includes explicit secret-prohibition clause (`AGENTS.md` deny-secrets, `security.md` CHK001, `NIST SP 800-218` PW.5, `NIST.IR.8397` §2.4, identifier-pattern `≤200` chars, `error` required on failure/refusal, local-only storage, 30-day/10 MB trim). This FULLY MATCHES `chk001` recommendation.
- However, `chk005` recommendation (research.md:33 / `clarify-sec-chk005`) — explicit clause linking structural failure isolation to `gate-8-full` exclusion and `AGENTS.md` diag constraints — is present in `spec.md` lines 86-97 (`Verification failure isolation and diagnostic boundary`) but is NOT included in VERIF-EMULATOR.md §5. §5 covers security/consent (`chk001`), not failure-isolation (`chk005`). So §5 does not fully cover the combined agent recommendations from the clarification batch; it covers `chk001` exactly but excludes `chk005` linkage.

=== 5. ADDITIONAL OBSERVATIONS ===

- contracts/VERIF-EMULATOR-structural.md:4 references `specs/003-reference-device/contracts/` (status-surface.md, earcon-mapping.md, surface-audit.md) — these are parent-contract paths, correct. But contracts file does not reference any new contracts for manifest/service/focus/diag/resume/rate.
- quickstart.md:228 references `problem.md`: `specs/008-ref-device-emulator/problem.md` — but `problem.md` does not exist in the directory listing; only `plan.md`, `spec.md`, `data-model.md`, `contracts/VERIF-EMULATOR-structural.md`, `quickstart.md`, `VERIF-EMULATOR.md`, `research.md`, `checklists/` exist. Missing file reference.
- plan.md:58 lists `contracts/` as "To be created (Phase 1) — reference contract to 003 contracts / manifest / service / diag" — the contracts file exists (`VERIF-EMULATOR-structural.md`) but does not explicitly reference manifest/service/diag contracts by file name, only by framework behavior in verification rules.
- data-model.md:138-141 mentions relationships reference `status-surface`, `earcon-mapping`, `surface-audit` — consistent with contracts file, but does not reference manifest/service/focus/diag/resume/rate contract relationships.
- No reference in any file to `spec.md` specific line numbers for exclusions (FR-002 line 106, SC-001 line 124, SC-004 line 127) or for framework contracts (FR-001 line 105, FR-006 line 110).

Summary: 4 categories of issues identified with file:line citations; no files edited.
