# Clarification Batch — 002-fr-024-synthesis-adapter / checklist review 2026-10-07

Status: All checklist [x] items already sound (28 + 39 + 43 + 16 + 35 = 161 checked, 0 unchecked). Batch CLARIFICATION_BATCH_2026-10-07.md already APPROVED 2026-10-07 with 3 subagent reviews (CHK024, CHK014, CHK031) and 3 parallel /research agents (resume boundary / parser fallback / gate-check). Underlying spec edits (FR-015 rewrite + cap-override repair + adapter-handoff citations + positive-assertion criteria) still pending — presented to user below for final confirmation before edit.

---

Question: For very short interruptions (≤3 s paused), should the boundary-level resume (nearest completed paragraph/heading) dominate over the proportional rewind (min(5%×paused, 30 s)), and is a de-minimis floor needed? Should the cap-override (>30 s boundary) land at the last completed boundary ≤ cap or at raw 30 s? And should `AlignmentGranularity::None` with adapter registered use paragraph/heading snap (never raw-cursor degradation)?

Why does it matter: Determines effective replay cost for the most common interruption regime (brief focus loss / notification). If rewind dominates, a 1 s pause replays ~0.05 s; if snap dominates it can replay a full paragraph (≤30 s cap). The spec's two-phase sequence (rewind then snap) implies snap dominates, but precedence is unstated and FR-015's raw-cursor degradation conflicts with 002's paragraph snap when adapter declares `None`. A missing floor/de-minimis risks near-zero rewind with undefined behavior; a wrong cap-override can land mid-word (unrecoverable at `None`).

Findings:
- Event Segmentation Theory (Zacks & Swallow 2007 Psych Bulletin; Zacks et al. 2007; Powers & Scerbo 2023 Human Factors) supports boundary-level resume at prose-event scale, with coarse breakpoints reducing resumption lag vs fine breakpoints.
- Resumption-lag studies (Monk et al. 2008 JEP:Applied; Labonté et al. 2021 Frontiers Psychol 12:659451) find duration effects start at ~5 s; at 2 vs 4 s duration is empirically inert (F<1 for 2/4 s), while alignment effects are significant — confirming snap dominates magnitude at very short pauses.
- Arithmetically 5% of ≤3 s = ≤150 ms, at/below phonemic-restoration quantization (~50–100 ms; Bashford & Warren 1987; Grosjean 1980); rewind contributes nothing measurable, snap carries all supported benefit.
- Spec.md:59 defines ordered composition: back min(5%×paused,30s) → snap to nearest completed paragraph/heading boundary containing that point; cap overrides snap when boundary >30 s — but cap-override landing point is ambiguous (raw 30 s possibly mid-paragraph / mid-word at `None`).
- RESUME_ALIGNMENT_RESEARCH.md:109-118 (repo-verified) confirms `None` → paragraph/heading snap; raw-cursor degradation never fires with adapter registered (chunk-delivery boundaries are real, with audio timing behind them).
- CLARIFICATION_BATCH_RESOLVED.md:13 (APPROVED 2026-10-07) requires FR-015 two-part rewrite (not yet edited); underlying edit still pending.
- No de-minimis floor supported by literature; a phase-skip (paused ≲1 s → skip rewind computation, 5% = 50 ms, below perceptual quantization) is defensible only as documented engineering simplification, not a behavioral threshold.

Options:
- Recommended (matches approved batch): Keep error-close (no de-minimis threshold); apply CLARIFICATION_BATCH_RESOLVED.md:13 plus cap-override repair: land at the last completed paragraph/heading boundary at-or-before the cap point (satisfies ≤30 s ceiling + word-onset floor + boundary alignment). State explicitly that proportional term selects the boundary and snap dominates magnitude (effective rule: start of paragraph/heading containing yield−min(5%×paused,30s)). Document that for very short pauses snap can replay a full paragraph (≤30 s cap). Amend FR-015 degradation so `None` + adapter parser mapping = paragraph/heading snap (never raw cursor); reserve raw cursor only where no parser boundary exists. Fix CHK007 paraphrase. Add synthetic SC-027 case.
- Reason: Matches spec's two-phase sequence; aligns with EST + resumption-lag evidence; respects constitution gate-8 split (deferred discharge, preserved mechanism); fixes already-approved edit; avoids arbitrary constant unsupported by literature.
- Alternative (not recommended): Add fixed-second floor (e.g., rewind ≥1 s) — contradicts boundary-dominance rationale and adds unsupported constant; also risks over-replay on very brief ducks.

---

Question: If `pulldown-cmark` is unreachable as workspace dependency (grep workspace Cargo.toml → 0) and may remain unreachable, should v1 declare bad Markdown at document-layer only with no diag expansion, and — if later reachable — should a new failure be expressed as option (b) a new `error` member on existing `failure`/`refusal` kinds (not a new diag `kind`, not added to 7-member `EngineError`)?

Why does it matter: Closed-set governance (Principle VII; FR-044; constitution.md §Closed Sets) requires every condition surface as a named failure with a recovery action; a catch-all silences unhandled conditions. The adapter declares against `EngineError` shapes and never observes input content; a member with no recovery consumer is drift. Choice between (a) new diag kind, (b) new error member on existing kinds, or (c) nothing determines which enums and contracts must be edited in the coordinated FR-044 edit.

Findings:
- `pulldown-cmark` accepts any byte string (no error type); not in workspace Cargo.toml; condition likely unreachable today.
- EngineError is exactly 7 members (`RateMismatch`, `ChannelMismatch`, `InitFailed`, `Unreachable`, `DeviceRefused`, `DeviceLost`, `StartRefused`), each tied to a recovery action the player takes (failover, refuse-start, ≤1 s pause…). Bad Markdown has no recovery row for the adapter; no `AdapterFailed` justified.
- Classification axis is recovery domain, not cause (repo precedent: `width` folded into `RateMismatch`; split when recovery differs; move domain when recovering actor changes).
- Option (b) preferred: new `error` member (`document_unparseable`) on existing `failure`/`refusal` kinds via FR-044 coordinated edit — never touches `EngineError`; never adds a new diag kind (would invite kind proliferation and touch kind-dispatched consumers). Must also amend one-list invariant (diag `error` = engine members re-exported + document-layer subset) — found conflict: option (b) as recorded says "no engine-core file touched", but `contract_schema.rs:58-63` requires same closed set.
- Fail-closed enforced by exhaustiveness checking (`prop_closed_sets.rs`; `contract_schema_negative.rs` rejects unknown wire strings).
- Literature (Ousterhout §4.2; Saltzer & Schroeder 1975 fail-safe; Laprie 1995 taxonomy; Wadler 1998 Expression Problem; SemVer 2 / no `#[non_exhaustive]` in repo) all support defining error in terms of caller recovery and coordinated growth.
- Minor doc inconsistency: `checklists/mechanism-gate.md:63` (CHK032) conflates 9 `kind` variants with 7 `EngineError` + failure + refusal.

Options:
- Recommended (matches approved batch): v1 = document-layer only, declare unreachability explicitly (`pulldown-cmark` unreachable, infallible if reachable); at `003-contract-edit`, if reachable, execute option (b) full FR-044 edit (new `error` member on `failure`/`refusal`, amend one-list invariant, never `EngineError`, record rule "fold when recovery identical / split when different / move domain when actor changes / no member when unreachable" in `adapter-handoff.md`).
- Reason: Keeps v1 closed; respects adapter's recovery-domain boundary; avoids kind proliferation; makes later edit loud (breaking match) per build-enforced governance.
- Alternative (not recommended): New diag `kind` (`parse`) — wrong axis (session event vs condition) and touches kind-dispatched trim/gap consumers unnecessarily.

---

Question: Should SC-051 be defined as a three-conjunct selector invariant (adapter registered in `offline_default`; mechanism preserved under failure = named `EngineError` within bound → paused-with-stated-reason → 3 s offer expiry → no auto-resume → notification carries reason; continuity across integration), with "preserved" (inductive invariant, verified now via stand-in drill + paused-state drill) explicitly distinct from "discharged" (SC-044: liveness conjunct with second selectable real engine + consent + `test_double` admissibility restored)? And must positive-assertion criteria be added to prevent vacuous pass?

Why does it matter: Safety (invariant at any state) vs liveness (eventually takes over) decomposition (Alpern & Schneider 1985/1987) means preservation is verifiable at N=1 but does not prove failover capability; conflating preserved with discharged overstates reliability. The adapter is sole registered tier (`test_double` inadmissible per `001/spec.md:89`), so the takeover action is not runtime-exercisable — must be stated explicitly rather than implied.

Findings:
- SC-051 definition already matches recommended shape (preserved as invariant; adapter registered; `test_double`/stand-ins closed; mechanism preserved via paused-with-stated-reason; active take-over deferred to SC-044 at `002/spec.md:120`; never silent / never auto-resume per `FR-014`).
- 001 definition stronger: live failover drill between `standin_a`/`standin_b` honouring same contract; "SC-051 verifies recovery mechanism ONLY; SC-044 remains the evidence… NOT discharged by SC-051" (`001/spec.md:1459`, `:1454-1459`).
- Safety/liveness decomposition: preservation = safety conjunct (verifiable at N=1); discharge = safety ∧ liveness (needs full environment). Vacuous-pass pitfall: without positive assertions, "no crash" passes trivially.
- Degraded-mode discipline (IEC 61508): bounded dwell + explicit escalation path matches spec bounds (3 s offer `FR-023a`; reason expires 3 s `FR-041`; no auto-resume; replay ≤30 s).
- Fault-injection drills (`standin_a`/`standin_b` fixtures + `crates/engine-core` contract-level tests) are legitimate instrument; must retain even when adapter is sole tier (loses runtime take-over exercise but keeps mechanism verification).
- Terminology drift: `adapter-handoff.md:177` reworded from "mechanism passes" to "preserved as invariant" (CLARIFICATION_BATCH_RESOLVED item 6); must carry vocabulary consistently.
- Coverage gap by design: with adapter registered, mechanism verification reduces — must be stated explicitly (`chk.md` asserts "preserved continuously" without quantifying loss); amendment: add explicit statement that takeover action is no longer runtime-exercisable while adapter is sole tier, mitigated by retaining stand-in contract-level drill.

Options:
- Recommended: Keep SC-051 as three-conjunct invariant; distinguish preserved vs discharged explicitly; add positive-assertion criteria (named `EngineError` variant within ≤3 s; notification carries stated reason; pause entered; position not replayed against failed timeline); state coverage gap; carry preserved/discharged vocabulary; do NOT close SC-051 on preserved alone = failover capability.
- Reason: Matches Alpern–Schneider decomposition; avoids vacuous pass; respects gate-8 split (constitution.md:643, deferred `SC-044`); keeps stand-in drill; prevents overstatement.
- Alternative (not recommended): Treat preserved as fully passing mechanism — overstates reliability and ignores deferred liveness conjunct.

---

Status of edits required before lock (none block checklist [x] — they are spec-quality refinements aligned to already-approved clarification):
1. FR-015 two-part rewrite (spec.md:59 + 001/spec.md:611-613 reconciliation) — approved, unedited.
2. Cap-override repair (last completed boundary ≤ cap point) — derived from findings, not yet in spec.md.
3. Adapter-handoff.md citation fixes (2.3→5, 98, 205; clause for FR-024a) — minor.
4. CHK007 paraphrase fix ("containing 5%-back point") — minor.
5. Synthetic SC-027 case (1 s pause, known boundaries → current paragraph start, ≤30 s) — test-design.
6. SC-051 positive-assertion criteria + coverage-gap statement — checklist/amendment.
7. `mechanism-gate.md:63` (CHK032) 9-kind / 7-error clarification — doc fix.

Recommendation to user: Confirm approved clarification options above (all three align with CLARIFICATION_BATCH_RESOLVED.md:13) and authorize edits 1–7. No new clarification candidates need autonomous resolution — evidence is sufficient. If any option diverges from recommendation, specify before edits proceed.
