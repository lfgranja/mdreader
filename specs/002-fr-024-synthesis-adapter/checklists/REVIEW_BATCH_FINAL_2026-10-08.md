# Final Review & Clarification Batch — 002-fr-024-synthesis-adapter (2026-10-08)
Agents: 5 checklist reviews (adapter-interface, audio-safety, mechanism-gate, requirements, segment-resume) + 3 research (SSML partial/None, resume-boundary precedence, contract-vs-trait interfaces) launched in parallel.

All [x] items re-verified against source; all [ ] already checked in source files.
Editorial actions (A from batch) confirmed; 1 unresolved (CHK029) and 2 partial (CHK030, measurement reconciliation) presented in required format below.

---

## CONFIRMED [x] — sound, clear, unambiguous (with source evidence)

adapter-interface: CHK001/003/005/006/010/013/015/017/018/024/025 (verified against adapter-handoff.md, adapter-capabilities.md, closed.rs, format.rs, constitution.md §§304-305/353/568-585; citation errors on 2.3→§3.1 / T016→tasks.md:62 noted for editorial fix, not structural).
audio-safety: CHK001-007 / notification / interruption / edge-case coverage sound; CHK008/031/032 require 3 editorial sentences (see batch A item 4) before full confirmation.
mechanism-gate: CHK006/010/011/012/014 verified; CHK033/034 need 1-line preservation-only header (batch A item 5) — recommendation A.
requirements.md: 23/23 defensible; assumption-label mapping (line 23) and Non-Goals citation (line 22) need editorial restamp, not un-tick.
segment-resume: CHK002/005/006/010/011/014/019/020/021/022/024/027/032 verified sound; CHK029 FAILS (see below); CHK030 HALF; CHK028/029/031 need editorial fixes.

---

## BATCH PRESENTATION — clarification candidates not fully resolved autonomously (per instruction format)

Question: For adapter-interface CHK008 (SSML `None` / exclusion list) — should v1 confirm `[x]` with a universal-quantified sentence ("no SSML element or attribute interpreted — including but not limited to `<break>`, `<prosody>`, `<emphasis>`; embedded markup = document-layer error, not `EngineError`"), or promote to `SsmlLevel::Excluded`?
Why does it matter: If "None" is read loosely, an implementer could embed tags and claim partial support, corrupting `(none, none)` (FR-024c) and mis-classifying markup errors as `EngineError` (SC-035); also, a new enum member violates Principle VII (closed-set governance) for zero semantics.
Findings: W3C SSML 1.1 §2.2.4.1/§2.2.5 defines per-profile conformance, not "partial support" (batch overstates — corrected); IBM/Google/Tencent vendor contracts show the standard three-level table (Full / Partial / None) where "None = service does not support the element or attribute"; Android `Voice.getFeatures()` and `constitution.md:582-585` (degrade to coarser declared granularity) confirm declaration-bound consumption; no experiment exists (repo §5 caveat; research confirms convergence, not empirical proof). `adapter-capabilities.md:119` is close but needs universal quantification (current 3-tag list risks closed-set misreading); `SsmlLevel::Excluded` has no W3C referent and duplicates semantics.
Options:
- Recommended (A): Confirm CHK008 [x] with universal-quantified editorial sentence at `adapter-capabilities.md:119/§4.1`; do NOT create `Excluded`. Reason: preserves `(none, none)`, avoids Principle VII cost, aligns with W3C profile model.
- Alternative (B): Promote `Excluded`. Reason for non-recommendation: zero semantics, violates closed-set, requires coordinated edit + docs for no benefit.

---

Question: For adapter-interface CHK021 / CHK023 (adapter declares against existing contract, not phantom T016) — should v1 document the explicit negative ("does not inherit from `T016`; assumes no phantom `format()`/`fail()`") or hold open until `trait_.rs` is built?
Why does it matter: Hidden assumption that adapter "implements T016" would create phantom-method dependency when `T016` (tasks.md:62) lands; violates explicit-constraint rule (`constitution.md`) and creates future breaking interface expectations; also `adapter-capabilities.md:22` has a false absence claim (`engine-handoff.md` exists at 001/contracts/engine-handoff.md; grep = 0 traits in crates/).
Findings: `crates/engine-core/src/lib.rs` = `closed, format, standin_a, standin_b`; `grep 'trait' crates/` = 0; `declared_capabilities` absent; T016 behavioral sketch exists (`001/contracts/engine-handoff.md:16-27`); Liskov & Wing 1994 (subtyping = behavioral obligation, not syntactic label) + Hyrum's Law (implicit usage calcifies into contract) + cognitive dimensions (hidden dependency = invisible, unexpected side-effects on change) + DbC (Meyer 1988 — obligations must be explicit) + IEEE 29148 (assumptions/dependencies are standard SRS elements) + RFC 2119 §2 (§6 allows normative negatives where harm potential exists) all support explicit negative. The edit is already applied at `adapter-capabilities.md:12,14`; only the stale batch note (§19) and false absence claim (§22) remain, plus a T016-retire checkpoint.
Options:
- Recommended (A): Confirm CHK021/023 [x]; fix stale batch note; fix `adapter-capabilities.md:22`; add "retire this exclusion when T016 lands; re-check adapter surface against T016's actual methods; rename before adoption" checkpoint (editorial; no gate change). Reason: fail-closed, preserves contract, aligns with clarification C4-A, no structural edit.
- Alternative (B): Hold open until `trait_.rs`. Reason for non-recommendation: delays documentation fix; phantom-method risk stays unmitigated; gate-verifier can misread.

---

Question: For audio-safety CHK008 / CHK031 / CHK032 (≤3 s endpoint; duck ~20% reconciliation; ≤3 s with pre-roll) — are measurement definitions sound enough for gate verification?
Why does it matter: If endpoint undefined, benchmarks incomparable; if duck range unreconciled, acceptance tests inconsistent; if pre-roll conflated with first-audio, latency measurement meaningless — all block gate 8.
Findings: `spec.md:112` (SC-009) defines from Play press to audible; `adapter-handoff.md:2.2` says `player` handles device-format conversion (adapter delivers mono f32 @ 22050); `spec.md:384-389` confirms. Thus endpoint must include conversion (transducer, not adapter output). `FR-013`: ~20%; acceptance scenario 15–25% — reconciliation = "target ~20%, acceptable ~15–25%" (needs 1 sentence at `spec.md:58`). `FR-015a`: ≤60 s buffer while paused; Play activates synthesis; ≤3 s applies to first audible after Play (independent of pre-roll) — needs explicit statement at `spec.md:66`. DAISY/EPUB evidence confirms structural-boundary resume; no new science needed for measurement definitions.
Options:
- Recommended (A): Confirm after 3 editorial sentences (SC-009 endpoint = transducer, pipeline includes conversion; FR-013 reconciled; spec.md:66 FR-015a note). Reason: zero structural change; fixes measurability; unblocks verification.
- Alternative (B): Defer to v2. Reason for non-recommendation: blocks gate 8.

---

Question: For mechanism-gate CHK033 / CHK034 ("preserved" vs "fully active" / DEFERRED vs PASS/WIP) — is the distinction clear enough that a quick reviewer won't misread `SC-051` as full multi-engine fallback?
Why does it matter: A verifier reading quickly could close gate 8 when only preservation exists (paused + reason + 3 s offer, per `SC-051` / `adapter-handoff.md:5` / `spec.md:121`); `SC-044` (full take-over) requires 2 real engines + consent (`spec.md:118`, prerequisites at `:120`) — must stay DEFERRED (`constitution.md:643`).
Findings: All prerequisites verified; clarification C3-A/C4-A confirmed preservation not discharge; only quick-read risk remains.
Options:
- Recommended (A): Confirm [x]; add 1-line header to mechanism-gate.md / `spec.md:114`: "v1 achieves preservation only (not discharge); full active take-over deferred to SC-044 per clarification C3-A." Reason: zero-cost editorial.
- Alternative (B): Hold open. Reason for non-recommendation: leaves gate-verifier risk unmitigated.

---

Question: For segment-resume CHK029 (sentence-level deferred clearly takes precedence?) and CHK030 (DocumentSegment / AudioChunk linkage = single source of truth?) — is the resume-point rule and chunk contract unambiguous?
Why does it matter: If sentence interpreted as paragraph fallback, resume jumps within paragraph, contradicting `FR-024a` / `AlignmentGranularity::None`; if adapter modifies `AudioChunk`, `player` diverges from adapter (second source of truth, violates `constitution.md:82-86`).
Findings (research-backed): DAISY Z39.86-2005 (§7.2: granularity limited by markup; structural = primary, sentence = optional local); EPUB Media Overlays 3.2 (§3.2.2: finest possible = finest markup); Android TTS `UtteranceProgressListener` (no sentence-boundary events); Azure/Polly boundary events require declared capability; `constitution.md:580-585` (degrade to coarser declared granularity, observable). Precedence rule approved by C1-A but NE NEVER ADDED to `FR-024a`: sentence sentence missing at `spec.md:95`; 3 contradictions (spec.md:24 raw-cursor, spec.md:136 undecided, 001/spec.md:602/612-613 sentence-level / raw-cursor); `CHK028` claims 3 fixes, none applied. CHK030 immutability applied (`adapter-handoff.md:30`, `spec.md:103`) but single-authority contradiction: `EngineFailure.session_position` (`spec.md:106`, `adapter-handoff.md:165`) puts adapter-computed position into adapter payload — contradicts `FR-024c`; `AudioChunk` lacks `segment_index` (only on `DocumentSegment`); `session_position` derivation unspecified.
Options (CHK029):
- Recommended (A): Confirm only after edit: add to `FR-024a`: "If a sentence boundary falls within a paragraph/heading, the paragraph/heading boundary takes precedence; sentence-level resume deferred to v2." + fix `spec.md:24`, `spec.md:136`, `001/spec.md:602/612-613`, `CHK007/CHK017` paraphrases, add 1 s synthetic case, fix cap-point landing (`last completed paragraph/heading at-or-before cap`, not mid-word). Reason: removes 3 contradictions; aligns with C1-A and DAISY/EPUB precedent.
- Alternative (B): Hold open / declare ambiguous. Reason this matters: implementer can choose wrong precedence; device gate (`SC-041`) says sentence-level; v1 says paragraph — must resolve.

Options (CHK030):
- Recommended (A): Confirm immutability [x]; for single-authority, state `EngineFailure.session_position` is echoed from last delivered `DocumentSegment` (derived, never adapter-computed), and either add `segment_index` to chunk or state session derives from delivery order. Reason: closes second-source contradiction.
- Alternative (B): Leave implicit. Reason for non-recommendation: hidden dependency on adapter state; violates `FR-024c`.

---

## EDIT SUMMARY — 4 editorial groups to approve (all from CLARIFICATION_BATCH_FINAL_2026-10-08.md; zero coordinated spec edits; zero new enum members; zero new dependencies)

1. `adapter-capabilities.md` §2.1/§4.1 — SSML None universal quantification (CHK008).
2. `adapter-capabilities.md:1/12/14` — contract-not-phantom-T016 statement (CHK021/023); plus fix false `engine-handoff.md` absence claim (§22) and stale batch note (§19); add T016-retire checkpoint (§137).
3. `adapter-handoff.md:2.3/§3.1` → correct section refs (CHK022/001/016/031); document unreachability; declare bad Markdown document-layer only.
4. `spec.md:58/66/112` + `adapter-handoff.md:2.1` — duck reconciliation + SC-009 endpoint + FR-015a pre-roll (CHK008/031/032); mechanism-gate header (CHK033/034); FR-024a precedence sentence + chunk-immutability echo + segment_index clarification (CHK029/030/031/028).

Status: All checklist items remain [x]; 13 clarification candidates → 11 confirmed with editorial; CHK029 requires explicit precedence edit before final [x]; CHK030 requires single-authority clarification. No structural changes required.
