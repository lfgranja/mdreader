# Data Model: FR-024 Synthesis Adapter (Option A)

**Branch**: `002-fr-024-synthesis-adapter`  
**Spec**: `specs/002-fr-024-synthesis-adapter/spec.md`  
**Design Phase**: Phase 1 (post-`research.md`)  
**Constitution references**: §TTS Engine Registry (`constitution.md` 524), §Audio Pipeline Contracts (568), §Development Workflow / Gate 8 (643, Exception 8), §Tauri Platform Contract / Single playback authority (82–86), §Engine Registry / Tier order (314)  
**Status**: All entities and rules derived from `spec.md` §Key Entities (99–105), clarified via `CLARIFICATION_BATCH.md` C1–C4, confirmed by source (`crates/engine-core/src/closed.rs`, `format.rs`, `standin_a.rs`).

---

## 1. Entities

### E1 — DocumentSegment (chunk-boundary entity)

A bounded chunk of spoken content derived from Markdown, produced adapter-side before synthesis.

| Field | Type / Constraints | Source / Validation |
|---|---|---|
| `content` | `String` (UTF-8) | Raw Markdown text of the segment; no embedded document text in delivered audio (`FR-045` / `spec.md:79`) |
| `source_range` | `(line_start, line_end, col_start, col_end)` or `pulldown-cmark` event range | From parser; enables resume-to-source mapping (`constitution.md` Audio Pipeline Contracts) |
| `boundary_kind` | Enum: `Paragraph` \| `Heading` | Chunk split at paragraph/heading only (`FR-024a`; `AlignmentGranularity::None`) |
| `byte_length` | `usize`; `≤ 5000` | Byte-measured (`constitution.md:353` / `spec.md:19`); adapter splits before synthesis; engine does not resample or split independently |
| `blake3_hash` | `String` (32 hex chars) | Document-level fingerprint (`redb` / session); edit → different hash → no resume (`spec.md:52` / `FR-036`) |
| `segment_index` | `u64` (monotonic within session) | Resume aligns to nearest completed boundary at this index (`FR-015` / `FR-024a`) |
| `resume_point` | `Option<SourceRange>` | Nearest completed paragraph/heading boundary containing the yield point (`spec.md:59` / C1-A); sentence-level deferred to v2 |

**Relationships**:
- `DocumentSegment` → `AudioChunk` (1:N — one segment may yield multiple chunks if split; for v1: 1:1 at paragraph/heading boundary, bounded by 5000 bytes)
- `DocumentSegment` → `Session` (`redb`) via `blake3_hash` (resume reachable only if hash unchanged; document edit = unreachable = no resume offered)
- `DocumentSegment` is independent of `EngineTier`; adapter delivers to `player` regardless of tier selection (`SC-051` mechanism preserved continuously)

**State transitions** (segment lifecycle):

```
Parsed (pulldown-cmark) → Split (adapter) → Bounded (≤5000 bytes) → Synthesised (audio) → Delivered (player) → Completed (resume point recorded)
```

- If `byte_length > 5000`: split at next `Paragraph`/`Heading` boundary (adapter-side; never engine-side)
- If synthesis fails: segment not delivered; failure reported via `EngineFailure` (named, not silent); `player` pauses + offers (`FR-023a` / `FR-041`); resume point stays at last delivered boundary (`spec.md:391`; never replay against failed timeline)

---

### E2 — AudioChunk (delivery unit)

The output unit delivered to `player`; bounded by `engine-core` contract shapes.

| Field | Type / Constraints | Source / Validation |
|---|---|---|
| `pcm_data` | `Vec<f32>` (mono) | Exact format: mono float32 (`FR-024`; `DeclaredFormat` `format.rs`: `SAMPLE_RATE=22050`, `CHANNELS=1`, `SAMPLE_WIDTH_BITS=32`) |
| `rate_hz` | `const 22050` | Deviation → `RateMismatch` (`FR-035`; `closed.rs` 256) |
| `channels` | `const 1` (mono) | Deviation → `ChannelMismatch` (`FR-033`; `closed.rs` 260) |
| `duration_ms` | `f64` | Computed from `pcm_data.len() / rate_hz * 1000`; used for notification progress (`FR-028` / `spec.md`: determinate iff total known at session start) |
| `pre_roll_remaining_ms` | `Option<u32>`; `≤ 60_000` | Pre-roll buffer (`FR-015a`); stops if playback pauses; no continuous synthesis while idle (`spec.md:66`) |
| `embedded_text` | `None` | Explicitly forbidden (`FR-045` / `spec.md:79`); only audio + metadata to `player`; document text never in chunk |
| `source_range_ref` | `SourceRange` | Links back to `DocumentSegment` for resume alignment (`spec.md:101`) |

**Validation rules (fail-closed; no silent resample / downmix)**:

- `pcm_data` must be exactly mono, `f32`, at 22050 Hz — any deviation surfaces `EngineError` (`RateMismatch` or `ChannelMismatch`) via adapter `format_check.rs`; `player` conversion to device format is the sole conversion step (`spec.md:42–45`; `FR-024b`).
- If adapter delivers stereo, wrong rate, wrong bit-depth, or resamples silently: playback does **not** start; named failure presented (`FR-035` / `SC-035`).
- No embedded `String` / document text — checked at adapter boundary (pre-delivery) to honor `FR-045`.

---

### E3 — EngineFailure (named failure entity)

Typed error variant carrying failure reason + current session position, used by `player` to present `FR-023a` / `FR-037`. Not an adapter-internal exception — it crosses the adapter→player boundary.

| Field | Type / Constraints | Source / Validation |
|---|---|---|
| `variant` | `EngineError` (closed 7-member enum) | `RateMismatch` \| `ChannelMismatch` \| `InitFailed` \| `Unreachable` \| `DeviceRefused` \| `DeviceLost` \| `StartRefused` (`closed.rs`: 245–270); wire names serialized (`rate_mismatch`, etc.) |
| `condition` | `String` (≤ 200 chars) | Identifier-pattern only (constitution diagnostic record rule); descriptive of observed condition (e.g., `rate_mismatch: delivered 24000 Hz`) |
| `session_position` | `SourceRange` | Echo of the delivered chunk's `source_range_ref` — the `DocumentSegment.resume_point`, falling back to its `source_range` — never adapter-computed (`constitution.md:82-86`); a failure before the first delivery echoes the segment in synthesis, `player` derives the resume position from its own delivered-chunk log, replay bounded at ≤30 s (`FR-015b`; `spec.md:391`) |
| `offer_expires_at` | `Timestamp` (≤ 3 s from detection) | Offer expires at ≤3 s (`FR-023a`; `FR-032c`; notification carries failure reason `FR-037`) |
| `tier_order_at_failure` | `EngineTier::OfflineDefault` | Adapter is sole registered tier; no selectable fallback until second real engine registered (`SC-044`; `001/spec.md:89`; `constitution.md:643`) |

**Condition → Variant mapping** (design contract from `spec.md:85`; verified at `research.md` §6):

| Observed condition | Variant |
|---|---|
| Format / rate / width mismatch (width treated as rate-shape per Principle VII) | `RateMismatch` |
| Channel count ≠ 1 | `ChannelMismatch` |
| Voice / init / loader corruption | `InitFailed` |
| Process death / OOM / transport unreachable | `Unreachable` |
| Device refused stream (AAudio rate/format refusal at open) | `DeviceRefused` |
| Output device removed (`BECOMING_NOISY` OR `onAudioDevicesRemoved`) — either sufficient, each ≤1 s | `DeviceLost` |
| Start refused (`ForegroundServiceStartNotAllowedException`; consent missing at start) | `StartRefused` |

**Constraints**:
- No `AdapterFailed` — not needed (7-site change not justified; `spec.md:85`; C4-A).
- No new `kind` (`parse`) at `diag` — bad Markdown stays document-layer; deferred to `FR-044` coordinated edit (`spec.md:26`; `CLARIFICATION_BATCH.md` C4-A; subagent `ses_eebac5cb8ffe107VdoI27hvXXO`).
- Failure is named, surfaced, never silent (`FR-023a` / `SC-035`).

---

### E4 — EngineTier (closed-set selection context)

Not a data object stored per chunk; it is the registration context for adapter selection.

| Tier | Adapter status (v1) | Selectable? | Rationale |
|---|---|---|---|
| `OfflineDefault` | **Registered** (`sherpa-onnx` + Piper) | **Yes** | Constitution §TTS Engine Registry; Principle IX; only tier at v1 |
| `CloudDefault` | Not registered | No | Requires consent + network; deferred (`SC-044`) |
| `CloudEscalation` | Not registered | No | Deferred |
| `TestDouble` (`standin_a`, `standin_b`) | Fixtures only | **No** when adapter registered (`001/spec.md:89`; `closed.rs:398-406`) | Only selectable when *no* non-standin adapter registered |
| `EvaluatedFallback` (Edge / `msedge-tts`) | Not registered | No | Consent-gated fallback; deferred (`SC-044`; `constitution.md` §XI exception) |

**Selection invariant** (`SC-051` / `spec.md:85`):
- With adapter registered (`OfflineDefault`), mechanism verified continuously via `paused-with-stated-reason` (`FR-023a` / `FR-041`) on failure; offer ≤3 s; no auto-resume (`FR-014`); position replay bounded ≤30 s (`FR-015b`).
- Active mid-document fallback switch deferred until second selectable real engine (`SC-044`); no regression; `test_double` unreachable (`CLARIFICATION_BATCH.md` C3-A).

---

### E5 — AdapterCapabilities (declaration entity)

Declared by adapter via `declared_capabilities()` against existing contract shapes (`format.rs` + `closed.rs`); trait `T016` mandated but not yet built (`Assumption 3` corrected; `spec.md:97`; `research.md` §3.3).

| Capability | Declared value (v1) | Contract reference | Player branch behavior |
|---|---|---|---|
| `SsmlLevel` | `None` | `engine-handoff.md:18–23` (referenced in `standin_a.rs` / `standin_b.rs`); `format.rs` has no `SsmlLevel` enum — declaration is unconditional (`constitution.md:304-305` / `582-585`) | Raw cursor (no SSML rendering; no `<break>` / pitch adjustment) |
| `AlignmentGranularity` | `None` | `spec.md:94`; `FR-024c`; `constitution.md` Audio Pipeline Contracts (sentence-level is pipeline aspiration, not engine assumption) | Paragraph/heading boundary snap via the boundary-kind-agnostic `snap_to_boundary`; sentence-level snapping deferred to v2; resume aligns to nearest completed `Paragraph` / `Heading` boundary (`spec.md:59`; `FR-015`) |

**Constraints on declaration**:
- Unconditional — must be declared; omission = violation of `FR-024c` / `constitution.md:304`.
- Against existing shapes only — no phantom `format()` / `fail()` methods; adapter does not invent `trait_` file (`research.md` §3.3; `crates/engine-core/src/lib.rs` has no `trait_.rs`; `grep '\btrait\b'` = 0).
- `player` branches on declared value — never silent inference (`spec.md:97`; `Assumption 3` corrected).

---

## 2. Validation Rules (cross-entity)

All rules derived from `spec.md`, `constitution.md`, and source verification (`crates/engine-core/src/`). Applied at adapter boundary (pre-delivery to `player`) unless noted.

| Rule ID | Rule | Applies to | Source |
|---|---|---|---|
| R1 | `AudioChunk.pcm_data` is exactly mono `f32` @ 22050 Hz | AudioChunk | `FR-024`; `format.rs:27-32`; `SC-035` |
| R2 | Any format deviation → named `EngineError` (not silent) | AudioChunk / Adapter output | `FR-024b`; `SC-035`; `spec.md:42-45` |
| R3 | `DocumentSegment.byte_length ≤ 5000` (byte, not char); split at `Paragraph` / `Heading` | DocumentSegment | `FR-024a`; `constitution.md:353`; `spec.md:19` |
| R4 | Chunk split owns boundary; engine does not independently resample / split | Adapter / Engine contract | `FR-024a`; `spec.md` Non-Goals (no engine redesign) |
| R5 | `blake3_hash` unchanged → resume possible; edit → unreachable → no resume | Session / DocumentSegment | `spec.md:52`; `FR-036`; `constitution.md` §Session / bookmark |
| R6 | Resume at nearest completed `Paragraph` / `Heading` boundary (not sentence, v1) | DocumentSegment / Session | `FR-024`; `spec.md:23`; C1-A; `AlignmentGranularity::None` |
| R7 | Failure = named `EngineError` (7 members); condition→variant mapped; no `AdapterFailed` | EngineFailure / Adapter | `FR-035`; `spec.md:85`; C4-A; `closed.rs:245-270` |
| R8 | Bad Markdown = document/parse layer (not `EngineError`; no `diag` `parse` kind at v1); deferred to `FR-044` coordinated edit | Adapter input / Diag | `spec.md:85`; C4-A; `CLARIFICATION_BATCH.md` C4-A; subagent `ses_eebac5cb8ffe107VdoI27hvXXO` |
| R9 | Capability declared `(None, None)` unconditionally; no phantom trait / methods | AdapterCapabilities | `FR-024c`; `spec.md:97`; `Assumption 3` corrected; `constitution.md:304-305` |
| R10 | Pre-roll ≤60 s; stops on pause; no continuous synthesis while idle | AudioChunk / Session | `FR-015a`; `spec.md:66`; `constitution.md` §Audio Pipeline Contracts |
| R11 | `EngineTier` = `OfflineDefault` only at v1; `test_double` not selectable; `SC-051` mechanism preserved continuously | Adapter registration / Selection | `SC-051`; `spec.md:85`; `001/spec.md:89`; `closed.rs:314`; `constitution.md:643` (Exception 8) |

---

## 3. State Transitions (adapter → session → player)

```
Document (Markdown) → [pulldown-cmark parse] → AST + SourceRange
  → [adapter split] → DocumentSegment[] (≤5000 bytes each; Paragraph/Heading boundary)
    → [adapter synth (sherpa-onnx + Piper)] → AudioChunk[] (mono f32 @ 22050 Hz)
      → [format_check (format.rs)] → either Delivered → player
                                → or Named Failure (EngineFailure, 7-member)
                                  → player pauses + 3-s offer (FR-023a)
                                  → if no selection / only OfflineDefault registered
                                    → mechanism preserved (SC-051); reason expires after 3 s (FR-041)
                                    → resume from nearest completed Paragraph/Heading (FR-015; ≤30 s replay bound)
      → [redb bookmark + blake3 fingerprint] → resume reachable if hash unchanged
```

**Critical transition points (no silent inference)**:
- Parse → Split: adapter owns; if parser unreachable at integration, document-layer validation holds (bad Markdown rejected before adapter invocation; `R8`).
- Synth → Deliver: format check mandatory (`R1`/`R2`); never silent resample.
- Deliver → Failure: named variant mandatory (`R7`); never silent failure or unreported error.
- Failure → Resume: position re-anchored to nearest completed boundary (`R6`); replay bounded ≤30 s (`FR-015b`); no replay against failed timeline (`spec.md:391`).

---

## 4. References (absolute paths)

- Spec: `/home/luis/development/mdreader/specs/002-fr-024-synthesis-adapter/spec.md`
- Plan (this design input): `/home/luis/development/mdreader/specs/002-fr-024-synthesis-adapter/plan.md`
- Research (resolves all C1–C4): `/home/luis/development/mdreader/specs/002-fr-024-synthesis-adapter/research.md`
- Constitution: `/home/luis/development/mdreader/.specify/memory/constitution.md` (§TTS Engine Registry 524; §Audio Pipeline Contracts 568; §Development Workflow / Gate 8 643, Exception 8; §Tauri Platform Contract / Single authority 82–86)
- Engine contract source (verified): `/home/luis/development/mdreader/crates/engine-core/src/closed.rs`, `/home/luis/development/mdreader/crates/engine-core/src/format.rs`, `/home/luis/development/mdreader/crates/engine-core/src/lib.rs`
- Stand-in fixtures (test double rules): `/home/luis/development/mdreader/crates/engine-core/src/standin_a.rs`, `standin_b.rs`
- Feature checklist (passes): `/home/luis/development/mdreader/specs/002-fr-024-synthesis-adapter/checklists/requirements.md`
- Clarification batch: `/home/luis/development/mdreader/specs/002-fr-024-synthesis-adapter/CLARIFICATION_BATCH.md`
