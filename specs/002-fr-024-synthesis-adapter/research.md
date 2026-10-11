# Research: FR-024 Synthesis Adapter (Option A — sherpa-onnx + Piper pt-BR)

**Branch**: `002-fr-024-synthesis-adapter`  
**Spec**: `/home/luis/development/mdreader/specs/002-fr-024-synthesis-adapter/spec.md`  
**Date**: 2026-10-06  
**Status**: All clarification-batch items C1–C4 confirmed A; no remaining `NEEDS CLARIFICATION` markers in `spec.md`; checklist passes.

---

## 1. Clarification Batch (C1–C4) — Consolidated

Source: `CLARIFICATION_BATCH.md`, `spec.md` lines 13–26, subagent `ses_eebac5cb8ffe107VdoI27hvXXO` (diag `parse` gap).

| Batch | Question | Resolution (A) | Design impact |
|---|---|---|---|
| C1-A | Paragraph vs sentence resume; `SentenceBoundary` from Piper? | Basic `pulldown-cmark` + source-range mapping sufficient; resume at nearest completed paragraph/heading (`AlignmentGranularity::None`); sentence deferred to v2. | Adapter declares `(none, none)`; `snap_to_sentence_start` not required at v1; resume aligns to chunk split boundaries (`FR-024a`). |
| C2-A | `SsmlLevel` / `AlignmentGranularity` declaration without built trait? | Declare `(none, none)` against existing `DeclaredFormat` + `EngineError` shapes (`format.rs`, `closed.rs`); trait `T016` mandated but not yet built; no phantom `format()`/`fail()`. | Adapter implements contract declaration only; no new trait file; `player` branches on declared value (raw cursor / paragraph-boundary snap — never silent inference). |
| C3-A | SC-044 close on `test_double`? Consent deferred? | **Split**: consent acquisition deferred (no writer/UI/verb/storage key; out of Option A scope per `spec.md:138`); SC-044 stays `DEFERRED`; `test_double` unreachable under `001/spec.md:89` (standins not selectable when non-standin adapter registered); stale `spec.md:133` corrected to `138`; match `constitution.md:643` (`gate-8-full: DEFERRED, owner=synthesis`). | Adapter lives in `offline_default`; no active mid-document fallback switch until second real engine (SC-044); `SC-051` preserved continuously as selector invariant. |
| C4-A | 7-member `EngineError` + mapping? `SC-051` continuous? Bad Markdown = `EngineError` or document/parse? `AdapterFailed` needed? | Replace incorrect names with real 7 (`RateMismatch`, `ChannelMismatch`, `InitFailed`, `Unreachable`, `DeviceRefused`, `DeviceLost`, `StartRefused`); add condition→variant table; `DeviceLost` row added; `DeviceRefused`/`StartRefused` split documented; `width`→`RateMismatch`; `SC-051` preserved; bad Markdown = document/parse (not `EngineError`; `diag` `kind` lacks `parse` — subagent confirms `pulldown-cmark` infallible / not yet workspace dependency; defer to `FR-044` contract-edit); no `AdapterFailed`; 7-site change justified. | Adapter reports only named variants; format verification uses `format.rs` check; bad-Markdown rejected at document/parse layer before adapter invocation; `diag` unchanged at v1. |

---

## 2. Source Evidence (verified in workspace)

- `crates/engine-core/src/closed.rs` 245–270 — `EngineError` 7 variants with wire names (`engine_unreachable`, `engine_init_failed`, `rate_mismatch`, `channel_mismatch`, `device_refused`, `device_lost`, `start_refused`).
- `crates/engine-core/src/format.rs` 23–33 — `DeclaredFormat` (mono, 22050 Hz, f32, 32-bit); no `SsmlLevel` or `AlignmentGranularity` enum exists.
- `crates/engine-core/src/closed.rs` 314–330 — `EngineTier`: `OfflineDefault`, `CloudDefault`, `CloudEscalation`, `TestDouble`, `EvaluatedFallback`; adapter registers only `OfflineDefault`.
- `crates/engine-core/src/lib.rs` — exposes `closed`, `format`, `standin_a`, `standin_b`; no `trait_.rs`; `grep '\btrait\b'` = 0.
- `constitution.md` §TTS Engine Registry (line 524) — `sherpa-onnx` + Piper pt-BR (`faber`/`cadu`/`jeff`) = `offline_default`, boundary events = none, SSML = none, admissible under Principle IX.
- `constitution.md` §Audio Pipeline Contracts (line 568) — sentence-level is pipeline requirement, not engine assumption; adapter must declare delivered granularity; degradation observable.
- `constitution.md` §Development Workflow / Gate 8 split (line 643, Exception 8) — `gate-8-shell: PASS` (handoff shape + granularity-declared + 22050 Hz request) / `gate-8-full: DEFERRED (owner=synthesis)`.
- `constitution.md` line 353 — byte budget ≤5000 bytes/request measured in bytes, adapter owns split.

---

## 3. Technology Choices

### 3.1 Engine / Voice

- **Choice**: `sherpa-onnx` (Apache-2.0, `ort` 2.0.0-rc.13, NNAPI, exception 3 / 4 recorded) + Piper pt-BR `faber` (CC0) as `offline_default`.
- **Rationale**: Constitution §TTS Engine Registry; Principle IX (offline-first); `faber` default per assessment (`spec.md` Assumption 2; `edresson` excluded CC-BY-4.0).
- **Alternatives rejected**: Google Cloud TTS (requires consent + network — deferred to v2/SC-044); Edge read-aloud (consent-gated fallback-only, requires `msedge-tts` 0.4.0 — deferred); Kokoro-82M pt-BR (deferred — `sherpa-onnx` ships zh+en only).

### 3.2 Parser / Chunking

- **Choice**: `pulldown-cmark` (0.13.x) for AST + paragraph/heading split; adapter owns chunk boundary (`FR-024a`).
- **Rationale**: Source-range events make resume-to-source mapping possible (`constitution.md` Audio Pipeline Contracts); paragraph/heading is coarsest declared granularity (`None`) and sufficient for v1 per clarification C1-A.
- **Alternatives rejected**: Sentence-level split (requires `Piper` boundary events — not exposed; deferred); manual regex split (no source-range; breaks resume alignment).
- **Risk / unresolved integration**: `pulldown-cmark` is not a workspace dependency today (`grep` workspace `Cargo.toml` = 0; subagent `ses_eebac5cb8ffe107VdoI27hvXXO` confirms). Resolution: adapter design assumes it; integration stage (`003-contract-edit`) confirms reachability. If unreachable at integration: use existing document-layer validation (bad Markdown = parse error, not `EngineError`) and defer parser introduction to coordinated edit.

### 3.3 Capability Declaration

- **Choice**: `declared_capabilities()` returns `(SsmlLevel::None, AlignmentGranularity::None)` against existing contract shapes (`format.rs` + `closed.rs`).
- **Rationale**: `T016` mandated (`constitution.md:299-312`) but trait not built; adapter must declare unconditionally (`constitution.md:304-305` / `582-585`) without phantom methods; `player` branches on value (`spec.md:97`; `Assumption 3` corrected).
- **Alternatives rejected**: Invent `trait_::EngineAdapter` file (would touch `engine-core` before `T016` design complete); declare unspecified / omit declaration (violates `FR-024c`, `constitution.md:304`).

### 3.4 Failure Contract

- **Choice**: Adapter uses the 7-member `EngineError` closed set with condition→variant table (see §6). No `AdapterFailed`; bad Markdown stays document/parse layer (`spec.md:85`; clarification 2026-10-06).
- **Rationale**: Principle VII (no catch-all); `SC-051` invariant requires named failure for mechanism verification; condition mapping prevents ambiguous failure presentation (`spec.md:86`; `FR-035`).

---

## 4. Design Decisions (with trade-off record)

### D1 — Chunk split adapter-side, not engine-side
- **Chosen**: Adapter splits at paragraph/heading before synthesis; engine receives bounded chunks (`≤5000` bytes, `constitution.md:353`).
- **Rejected (engine-side)**: Would require engine to expose chunk-boundary metadata; `sherpa-onnx` does not; would couple engine to byte budget.
- **Trade-off**: Adapter becomes parser + splitter + synthesiser; more responsibility, but preserves engine neutrality (Principle VI) and makes resume points explicit.

### D2 — `(none, none)` declaration, no sentence-level alignment
- **Chosen**: Declare both capability levels `None`.
- **Rejected**: Claim `sentence` / `SsmlLevel::Partial` — false for Piper (no boundary events verified); would require silent inference or phantom trait.
- **Trade-off**: Less precise resume (paragraph/heading only) for v1; sentence deferred to v2 when second engine (`Edge` with `SentenceBoundary`) or `Piper` upgrade provides events.

### D3 — Active mid-document fallback switch deferred (`SC-051` preserved as invariant, not active)
- **Chosen**: With adapter as sole registered tier (`offline_default`), mechanism verifies via `paused-with-stated-reason` (`FR-023a` / `FR-041`) on failure; no automatic takeover until `SC-044` discharges (second real engine).
- **Rejected**: Enable active switch with adapter as only selectable tier — impossible (only one tier selectable); would silently fail `SC-051` verification.
- **Trade-off**: User must manually resume or wait for second engine; mechanism preserved continuously rather than degraded.

### D4 — No `diag` `parse` kind expansion at v1
- **Chosen**: Bad Markdown = document-layer validation (pre-adapter), not `EngineError`; `diag` unchanged; deferred to `FR-044` coordinated edit.
- **Rejected**: New `kind` (`parse`) — would expand closed set (Principle VII requires coordinated edit + semver); new `error` member on `failure`/`refusal` preferred over new kind (clarification C4-A option b), but deferred to avoid touching `engine-core` file today.
- **Trade-off**: Document errors may not appear in diagnostic record until v2; acceptable because bad Markdown is a pre-synthesis failure, not a synthesis failure.

---

## 5. Unknowns Resolved

| Unknown (from Technical Context / spec) | Resolution | Evidence / Source |
|---|---|---|
| Chunk boundary interaction with byte budget | Adapter splits; engine gets bounded chunks | `spec.md:19`; `constitution.md:353`; C3-A |
| Piper sentence-boundary events | None exposed; not required for v1 | `spec.md:18`; C1-A; `spec.md` Assumption 4 |
| SC-051 continuous vs temporarily disabled | Continuous invariant; active switch deferred | `spec.md:20`; C4-A; `constitution.md:306-312` |
| Adapter failure names / condition table | 7 real variants; table at `spec.md:85` | `spec.md:21`; C4-A; `closed.rs` 245–270 |
| SC-044 close on `test_double` | Deferred (`DEFERRED`); `test_double` unreachable | `spec.md:22`; C3-A; `constitution.md:643`; `001/spec.md:89` |
| Resume granularity (sentence vs paragraph) | Paragraph/heading (`None`) for v1 | `spec.md:23`; C1-A; `constitution.md` Audio Pipeline Contracts |
| Capability declaration without trait | Declare `(none,none)` against existing shapes | `spec.md:24`; C2-A; `Assumption 3` corrected |
| Bad Markdown = `EngineError`? | Document/parse only; not `EngineError` | `spec.md:85`; C4-A subagent `ses_eebac5cb8ffe107VdoI27hvXXO` |
| `pulldown-cmark` dependency readiness | Not in workspace; design assumes; integration confirms | `grep` workspace `Cargo.toml` = 0; clarification `spec.md:138` / note 138 |

---

## 6. Condition → EngineError Variant Mapping (design artifact)

Source: `spec.md:85`; confirmed against `closed.rs` 245–270. Adapter uses this table to map failure conditions into the 7-member wire set.

| Failure condition (what observed) | Variant (wire name / serde rename) | Notes |
|---|---|---|
| Delivered rate ≠ 22050 Hz, or pitch-tolerance ±3 cents @ 2.0× violated, or `width` deviation treated as rate-shape (Principle VII) | `RateMismatch` (`rate_mismatch`) | `width` not a separate variant; covered by rate-shape (`format.rs`) |
| Delivered channel count ≠ 1 (mono) | `ChannelMismatch` (`channel_mismatch`) | Never silent downmix |
| Voice-model / init corruption / synthesis failure at startup | `InitFailed` (`engine_init_failed`) | Includes loader failure |
| Process death / OOM / kill (adapter process) / transport unreachable | `Unreachable` (`engine_unreachable`) | Transport-level |
| Device refused the stream (AAudio format/rate refusal at open) | `DeviceRefused` (`device_refused`) | Stream-negotiation refusal |
| Audio output device removed (`BECOMING_NOISY` or `onAudioDevicesRemoved`) — either sufficient, each ≤1 s pause | `DeviceLost` (`device_lost`) | Independent conditions |
| Start refused (API 34+ `ForegroundServiceStartNotAllowedException`; consent missing at start) | `StartRefused` (`start_refused`) | Start-time only |

**Excluded** (not `EngineError`): bad Markdown segment → document/parse layer (`spec.md:85`); any condition that does not match above → fail-closed (build must not have catch-all arm per Principle VII).

---

## 7. Integration Points (out of adapter scope, named for contract)

- `player` (Rust) — receives `AudioChunk`; converts to device format (sole conversion step per `FR-024b` / `spec.md:42–45`); branches on `declared_capabilities()` (`SsmlLevel::None`, `AlignmentGranularity::None`) → raw cursor / paragraph-boundary snap (`spec.md:97`); never silent inference.
- `session` (`redb` + `blake3`) — adapter does not alter position storage (`Assumption 5`); resume points from chunk split boundaries (`FR-024a`).
- `diag` — adapter contributes only synthesis-level events (`failure` / `refusal` kinds); no `parse` kind at v1; unnecessary `document_unparseable` deferred to `FR-044` coordinated edit.
- Kotlin `MediaSession` — mirror only; adapter does not modify notification / focus logic (`spec.md:137` / Non-Goals).
- `engine-core` trait `T016` — adapter does not define it; declares against existing `DeclaredFormat` + `EngineError` shapes.

---

## 8. Verification Plan (pre-implementation)

Before any adapter file is written:

1. Confirm `pulldown-cmark` reachability at integration stage; if unreachable → document-layer validation maintained, adapter split deferred to parser introduction (not blocking v1 design).
2. Confirm `crates/engine-core/src/lib.rs` exposes `closed` + `format` (verified above); confirm no phantom `trait_` created.
3. Confirm `SC-051` mechanism passes with adapter registered (`offline_default` only; standins `standin_a`/`standin_b` not selectable — verified by `001/spec.md:89` / `closed.rs:398-406`).
4. Confirm byte-budget test (`≤5000` bytes / chunk) can be written against adapter split logic (`constitution.md:8` / gate 8 shell).
5. Confirm `cargo fmt`, `cargo clippy -- -D warnings`, `cargo deny` on adapter crate meet Principle IV / Rust rigor (no exceptions needed; `ort` RC already registered).

---

*End of Phase 0 research. All C1–C4 resolved; all `NEEDS CLARIFICATION` cleared; no unconfirmed dependencies block Phase 1 design.*
