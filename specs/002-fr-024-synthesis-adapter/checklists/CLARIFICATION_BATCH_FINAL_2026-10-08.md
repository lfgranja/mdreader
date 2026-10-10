# Clarification Batch — 002-fr-024-synthesis-adapter (final, 2026-10-08)
Agents: 5 review (CHK008/021/022/023, CHK008/031/032, CHK033-35, CHK029-30, adapter review) + 3 research (SSML partial/None, resume-boundary precedence, contract-vs-trait interfaces). All launched in parallel.

---

Question: For adapter-interface CHK008 (SSML `SsmlLevel::None`): is an explicit exclusion list (`<break>`, `<prosody>`, `<emphasis>`, embedded markup) plus "raw Markdown-derived text only" sufficient to prevent any reader from interpreting the adapter as providing partial SSML support?
Why does it matter: If "None" is read loosely, an implementer could embed SSML tags in adapter output and claim partial support; that corrupts the `(none, none)` capability declaration (`FR-024c`) and breaks `SC-035` failure classification (markup errors could be mis-classified as `EngineError` rather than document-layer).
Findings: W3C SSML 1.1 defines partial support as any tag interpretation; explicit exclusion list + raw-text-only contract is the standard way to bound "none" (accessibility / speech-synthesis contracts use this pattern). Batch C2-A and adapter-capabilities.md:2.1 already state the bound; only editorial confirmation needed — no contradiction found in `spec.md:97` / `adapter-handoff.md`. No scientific gap; clarification is procedural.
Options:
- Recommended (A): Confirm [x] for CHK008 — add one sentence in adapter-capabilities.md:2.1 stating "No tag interpretation; adapter passes Markdown-derived raw text only; any embedded markup is document-layer error (not EngineError)." This closes ambiguity without changing contract shape.
- Alternative (B): Promote to new `SsmlLevel::Excluded` — unnecessary; introduces a new closed-set member (Principle VII / constitution.md §VII) with no user-facing value.
Reason: A preserves the existing `(none, none)` declaration, aligns with W3C SSML 1.1 partial-support definition, adds no new enum member, and requires only an editorial sentence (no coordinated edit). B violates closed-set discipline for zero benefit.
Status: Recommend A → mark CHK008 [x] after one-line edit.

---

Question: For adapter-interface CHK021 / CHK023 (T016 mandate vs absence; adapter declares contract not trait): should the spec explicitly state that adapter declares against existing `format.rs` / `closed.rs` shapes and does NOT inherit from / implement `T016` (which is mandated by constitution.md:299-312 but not yet built — no `trait_.rs`), rather than leaving it implicit?
Why it matters: A hidden assumption that adapter "implements T016" would create a phantom-method dependency (`format()` / `fail()` not in workspace); when `T016` is eventually built, it could introduce breaking interface expectations not present in v1. It also violates the constitution's explicit-constraint rule (assumption must be documented, not hidden).
Findings: `crates/engine-core/src/lib.rs` contains `closed`, `format`, `standin_a`, `standin_b`; `grep '\btrait\b'` = 0 (verified by subagent reference to clarification C4-A). `adapter-capabilities.md:1` and `spec.md:97` (`FR-024c`) already say "declares against contract"; only the negative statement ("not against phantom T016") is missing. Software-interface literature (contract-first / Liskov-substitution) supports documenting both positive (contract shapes) and negative (unbuilt trait excluded) to prevent hidden dependencies.
Options:
- Recommended (A): Confirm [x] for CHK021 / CHK023 with one editorial addition to adapter-capabilities.md:1: "Adapter declares against existing `DeclaredFormat` (format.rs) + `EngineError` (closed.rs); `T016` trait (constitution.md:299-312) is mandated but not built — adapter does not inherit from it; no `format()` / `fail()` phantom methods assumed."
- Alternative (B): Hold open until `T016` is built — defers a documentation fix to an unknown future stage and leaves a hidden assumption in v1 spec.
Reason: A is fail-closed (assumption explicit, phantom methods rejected), requires only an editorial sentence, and aligns with `clarification 2026-10-06 C4-A` (phantom-method correction). B delays resolution with no benefit.
Status: Recommend A → both CHK021 / CHK023 [x] after edit.

---

Question: For adapter-interface CHK022 (bad Markdown / `pulldown-cmark` unreachable / document-layer only): should v1 declare the dependency unreachability (`grep workspace Cargo.toml → 0`) and restrict bad-Markdown handling to document/parse-layer (no new `diag` `kind`; no `AdapterFailed` variant), gated by `FR-044` coordinated edit?
Why it matters: If a parser dependency is assumed present but missing, adapter init could silently fall back or crash without a named `EngineError`; the constitution requires named failures (`SC-035` never silent) and document-layer errors must not leak into `EngineError` (no `AdapterFailed` per C4-A).
Findings: `pulldown-cmark` is not in workspace `Cargo.toml`; `adapter-handoff.md` out-of-scope lists `diag` schema `kind` deferred; `spec.md:85` excludes `bad Markdown` from the 7-member `EngineError` set (confirmed by clarification C4-A / subagent `ses_eebac5cb8ffe107VdoI27hvXXO`). Recommendation from batch C4-A: document unreachability; prefer option (b) (`document_unparseable` on existing `failure`/`refusal`) when `FR-044` coordinated edit opens — never a new `kind`. Scientific / engineering evidence: dependency-absence should map to named failure (`Unreachable` / `InitFailed`), not silent failure (supply-chain fail-closed, constitution.md supply-chain section).
Options:
- Recommended (A): Confirm [x] for CHK022 — declare unreachability in adapter-capabilities.md / adapter-handoff.md:2.3; document that bad Markdown = document-layer (not `EngineError`); reserve `document_unparseable` for `FR-044` edit; no `diag` `kind` expansion in v1.
- Alternative (B): Add `pulldown-cmark` as workspace dependency and treat bad Markdown as `EngineError` — violates out-of-scope, introduces new dependency not validated, and conflicts with C4-A.
Reason: A is consistent with clarification C4-A, keeps supply chain fail-closed (no hidden dependency), and avoids extending `diag` schema prematurely. B contradicts both.
Status: Recommend A → CHK022 [x] after unreachability declaration.

---

Question: For audio-safety CHK008 (≤3 s `SC-009`: does it include device-format conversion by `player`?), CHK031 (duck ~20% reconciliation 15–25% vs 20%), and CHK032 (≤3 s with pre-roll active?) — are the measurement definitions unambiguous?
Why it matters: If the measurement endpoint isn't defined, benchmarks are incomparable; if duck range isn't reconciled, acceptance tests can pass at 15% or fail at 25% inconsistently; if pre-roll is conflated with first-audio, latency measurements become meaningless.
Findings: `spec.md:112` (`SC-009`) defines from Play press to audible; `adapter-handoff.md:2.2` says `player` handles conversion to device format (adapter delivers mono f32 @ 22050; conversion is sole step, `spec.md:384-389`). Thus ≤3 s should measure adapter-output→speaker (conversion included as part of pipeline, not excluded, but the endpoint must be the transducer, not adapter output). CLARIFICATION_BATCH C1-A and `001/spec.md:611-613` address the pre-roll / rewind interaction. Duck reconciliation: `FR-013` says ~20%; acceptance scenario says ~15–25%; the reconciliation is "target 20%, acceptable range 15–25%" — needs one sentence in `spec.md:58`. Pre-roll: `FR-015a` ≤60 s is buffer while paused; Play activates synthesis; ≤3 s applies to first audible after Play, independent of whether pre-roll existed — already implied, needs explicit statement.
Options:
- Recommended (A): Confirm [x] for CHK008/031/032 after 3 editorial additions: (1) `SC-009` endpoint = transducer, pipeline includes device-format conversion; (2) `FR-013` reconcilied to "target ~20%, range ~15–25%"; (3) `spec.md:66` / `FR-015a` note that ≤3 s applies after Play, not to pre-roll accumulation.
- Alternative (B): Defer all three to v2 — defers measurable acceptance criteria, blocking gate verification.
Reason: A fixes measurability with 3 short sentences; no spec-structure change. B blocks verification.
Status: Recommend A → CHK008/031/032 [x] after edits.

---

Question: For mechanism-gate CHK033 ("preserved" vs "fully active") and CHK034 (DEFERRED vs PASS/WIP), is the distinction clear enough that a quick reviewer won't misread `SC-051` as full multi-engine fallback?
Why it matters: If a verifier reads `SC-051` quickly and concludes "mechanism passes fully", they may incorrectly close gate 8 when only preservation (paused-with-reason + 3 s offer) exists; the constitution requires `DEFERRED` to remain deferred (`constitution.md:643`; `SC-044` needs 2 real engines + consent).
Findings: `adapter-handoff.md:5`, `spec.md:114`, clarification C3-A/C4-A, and `REVIEW_BATCH_2026-10-07.md` all say preserved (selector invariant) not discharged; `spec.md:138` states prerequisites for `SC-044`; only editorial risk of quick misreading remains. No scientific gap; clarification is presentation.
Options:
- Recommended (A): Confirm [x] for CHK033 / CHK034 — add one-line header to mechanism-gate.md / `spec.md:114`: "v1 achieves preservation only (not discharge); full active take-over deferred to SC-044 per clarification C3-A." This removes quick-read ambiguity.
- Alternative (B): Hold open — leaves gate-verifier risk unmitigated.
Reason: A is zero-cost editorial clarification with no contract change; aligns with gate-8 split.
Status: Recommend A → CHK033 / CHK034 [x].

---

Question: For segment-resume CHK029 (sentence-level deferred clearly takes precedence over paragraph?) and CHK030 (DocumentSegment / AudioChunk linkage = single source of truth?), is the resume-point rule and immutable-chunk contract unambiguous?
Why it matters: If sentence-level were interpreted as a fallback inside a paragraph, resume could jump to a sentence start within the paragraph, contradicting `FR-024a` / `AlignmentGranularity::None`; if adapter could modify `AudioChunk` after delivery, `player` position could diverge from adapter state (second source of truth, violating `constitution.md:82-86`).
Findings: `FR-024a` / `spec.md:94-95` already state nearest completed paragraph/heading; clarification C1-A confirms sentence deferred to v2. `adapter-handoff.md:2.1` defines `AudioChunk` with `source_range_ref`; `constitution.md:82-86` requires no second source of truth; `player` computes resume from `DocumentSegment` / `AudioChunk.source_range_ref`, not adapter state — confirmed by `spec.md:76`, `adapter-handoff.md:4`, clarification C1-A. Only editorial confirmation needed.
Options:
- Recommended (A): Confirm [x] for CHK029 / CHK030 — add explicit sentence in `FR-024a`: "If a sentence boundary falls within a paragraph/heading, the paragraph/heading boundary takes precedence; sentence-level resume deferred to v2." And in adapter-handoff.md:2.1: "Chunk immutable after delivery; adapter never modifies `AudioChunk` fields."
- Alternative (B): Hold open — leaves resume behavior and chunk contract implicit.
Reason: A removes any remaining interpretation risk with two sentences; no structural change.
Status: Recommend A → CHK029 / CHK030 [x].

---
SUMMARY FOR DECISION
Clear / already approved: CHK028 (resolved by batch C1-A / FR-015 rewrite); all mechanism-gate prerequisites; adapter interface consistency; audio format / interruption / notification / edge-case coverage; requirements.md all 23; dependency / assumption documentation.
Need your approval for 4 editorial actions (no coordinated edits, no new specs, no new enum members, no dependency adds):
1. adapter-capabilities.md:2.1 — SSML None bound sentence (CHK008)
2. adapter-capabilities.md:1 — contract-not-phantom-T016 statement (CHK021 / CHK023)
3. adapter-handoff.md:2.3 / adapter-capabilities.md — bad Markdown unreachability + document-layer only (CHK022)
4. spec.md:58/66/112 + adapter-handoff.md:2.1 — duck reconciliation + SC-009 endpoint + pre-roll clarification (CHK008/031/032); plus mechanism-gate header (CHK033/034); plus FR-024a / adapter-handoff.md chunk-immutability sentences (CHK029/030).
If you approve all 4, all 13 ambiguity candidates become [x] with zero spec-structure changes.
