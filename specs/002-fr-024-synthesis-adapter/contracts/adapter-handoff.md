# Contract: Adapter → Player / Engine-Core Handoff

**Branch**: `002-fr-024-synthesis-adapter`  
**Feature**: FR-024 / FR-024a / FR-024b / FR-024c  
**Status**: Design-phase contract (Phase 1); v1: `pulldown-cmark` unreachable in workspace (`grep` → 0) — document-layer only; `T016` not built — adapter declares against existing `DeclaredFormat`/`EngineError` shapes only. Integration deferred to `003-contract-edit` (reachability + trait design finalized).  
**Constitution references**: §Engine-Neutral Core (VI, 299–312); §TTS Engine Registry (524); §Audio Pipeline Contracts (568); §Single Playback Authority (82–86); §Closed Sets (VII); §Development Workflow / Gate 8 (643, Exception 8); §Verification Gates (604–618)

---

## 1. Contract Purpose & Scope

This contract defines the boundary between the new synthesis adapter (`crates/adapter-synthesis/`) and the existing modular-mono workspace (`crates/player`, `crates/engine-core`, `crates/session`). It covers:

1. **Chunk delivery format** — what `player` receives from adapter (`AudioChunk`).
2. **Failure signaling** — how adapter reports failure to `player` (`EngineFailure`, 7-member closed set).
3. **Capability declaration** — how adapter declares what it can deliver (`AdapterCapabilities` / `declared_capabilities()`).
4. **Resume / boundary agreement** — how chunk split points become resume-alignment points (`FR-024a`; `AlignmentGranularity::None`).
5. **Selection / tier rules** — adapter lives only in `OfflineDefault`; `test_double` fixtures not selectable when adapter registered (`SC-051` invariant).

**Out of scope** (per `spec.md` Non-Goals / `spec.md:137`): `player` transport / position logic; Kotlin `MediaSession`; `redb` bookmark format; `diag` schema (`kind` enum); SSML rendering; cloud-tier selection; consent acquisition UI; second-engine active switch (deferred to `SC-044`).

**Format**: Markdown contract with embedded wire-format summaries (suitable for Rust library interface + wire-format documentation). No JSON Schema required — adapter exposes Rust `pub` interface, not HTTP endpoint.

---

## 2. Chunk Delivery Contract (Adapter → Player)

### 2.1 Delivery unit: `AudioChunk`

**Immutable after delivery**: adapter never modifies chunk fields after delivery; `source_range_ref` + `duration_ms` give `player` resume authority without adapter state access (`spec.md:103`; `constitution.md:82-86`); single source of truth enforced.

The adapter delivers audio to `player` as discrete chunks, each linked to a source `DocumentSegment`. `player` converts to device format (sole conversion step — never adapter-side silent resample / downmix).

**Wire / interface shape** (derived from `crates/engine-core/src/format.rs` 23–33 and `spec.md` 99–105):

```rust
// Conceptual interface (not pseudo-code; matches actual format.rs / spec)
pub struct AudioChunk {
    pub pcm_data: Vec<f32>,       // mono; exactly 22050 Hz; exactly 32-bit f32
    pub rate_hz: u32,             // const 22050 (DeclaredFormat::SAMPLE_RATE)
    pub channels: u16,            // const 1 (DeclaredFormat::CHANNELS)
    pub duration_ms: f64,         // derived; used for notification progress
    pub source_range_ref: SourceRange, // links to DocumentSegment for resume
    pub pre_roll_remaining_ms: Option<u32>, // ≤ 60_000 (FR-015a)
}
```

**Constraints (fail-closed)**:

| Constraint | Enforcement point | Failure if violated |
|---|---|---|
| `pcm_data.len() == (duration_ms / 1000.0) * 22050.0` (sample-aligned) | Adapter synthesis + format_check.rs | `InitFailed` if synthesis corrupt; `RateMismatch` if rate wrong |
| Rate exactly 22050 Hz | `format_check.rs` (checks against `DeclaredFormat::SAMPLE_RATE`) | `RateMismatch` (`spec.md:42-45`; `FR-035`; `SC-035`) |
| Channels exactly 1 | `format_check.rs` (checks `CHANNELS`) | `ChannelMismatch` |
| Sample width exactly f32 (32-bit) | Adapter output type / `format_check.rs` | `RateMismatch` (width treated as rate-shape per Principle VII / `spec.md:86`) |
| No embedded text / document content in chunk | Adapter boundary (pre-delivery check against `FR-045`) | Document-layer error; never `EngineError` |
| Chunk derived from `DocumentSegment` with `byte_length ≤ 5000` | Adapter split logic (`chunk.rs`) | Split not performed → chunk exceeds budget; adapter must split before synthesis (`FR-024a`) |

**Player-side contract obligations (existing behavior; adapter must not violate)**:
- Receives `AudioChunk`; converts to device format (only conversion step; `spec.md:384–389`).
- If format mismatch detected: surfaces named `EngineError` (`RateMismatch` / `ChannelMismatch`); does not start silently; does not silent-resample (`FR-024b`; `SC-035`).
- Notification progress (`FR-028`) is determinate iff total duration known at session start; indeterminate if unknown (adapter delivers chunk-by-chunk; total may become known after first chunk — but adapter does not compute this; `player` tracks from session state).
- Pre-roll (`FR-015a`) stops when playback pauses; adapter must not synthesize continuously while idle.

---

## 3. Failure Signaling Contract (Adapter → Player)

### 3.1 Named failure: `EngineFailure`

When adapter cannot deliver a chunk (init failure, format delivery error, process death, device refusal, etc.), it reports via the 7-member `EngineError` closed set. No unreported errors; no silent failure (`FR-023a`; `SC-035`; Principle VII).

**Wire shape** (derived from `crates/engine-core/src/closed.rs` 245–270; wire serde names documented):

```rust
pub enum EngineError {
    Unreachable,     // wire "engine_unreachable"  — process death / OOM / kill / transport unreachable
    InitFailed,      // wire "engine_init_failed"  — voice / loader / init corruption
    RateMismatch,    // wire "rate_mismatch"      — rate / width / pitch-tolerance ±3¢ @ 2.0×
    ChannelMismatch, // wire "channel_mismatch"   — channel count ≠ 1
    DeviceRefused,   // wire "device_refused"     — AAudio refusal at open (stream negotiation)
    DeviceLost,      // wire "device_lost"        — device removed (BECOMING_NOISY OR onAudioDevicesRemoved)
    StartRefused,    // wire "start_refused"      — ForegroundServiceStartNotAllowedException; consent missing
}
```

**Condition → Variant mapping** (adapter must implement; verified at `research.md` §6; `spec.md:85`):

| Condition observed by adapter | Variant to report |
|---|---|
| Format / rate / width / pitch deviation from declared 22050 Hz mono f32 | `RateMismatch` |
| Delivered channel count ≠ 1 | `ChannelMismatch` |
| Voice-model / loader / initialization failure | `InitFailed` |
| Process killed / OOM / adapter unreachable at transport | `Unreachable` |
| Audio output device refused stream negotiation (AAudio rate/format refusal at open) | `DeviceRefused` |
| Output device removed (either `BECOMING_NOISY` OR `onAudioDevicesRemoved`; either sufficient, each held to ≤1 s pause bound) | `DeviceLost` |
| Playback start refused (`ForegroundServiceStartNotAllowedException`; consent missing at start; API 34+ FGS-start restrictions) | `StartRefused` |

**Excluded from `EngineError`:**
- Bad Markdown segment → **document/parse layer only** (`spec.md:85`; `CLARIFICATION_BATCH.md` C4-A; subagent `ses_eebac5cb8ffe107VdoI27hvXXO`). Not `EngineError`; `diag` `kind` enum unchanged; deferred to `FR-044` coordinated edit (`option (b)` from clarification — new `error` member on `failure`/`refusal`, not new `kind`). If `pulldown-cmark` unreachable at integration: document-layer validation holds (rejected before adapter invocation).
- Any condition not matching above → **fail-closed** (build fails; no catch-all arm; Principle VII).

### 3.2 Failure presentation to user (`FR-023a` / `FR-037`)

When failure is reported:
- `player` presents named failure to user immediately (detection immediate; offer expires ≤3 s; `FR-032c`; `FR-041`).
- Notification carries failure reason (`FR-037`).
- If user selects alternative tier (when second engine registered — deferred): resume from nearest paragraph/heading boundary for new engine (`FR-023a`; `FR-015`; `spec.md:59`).
- If no selection / only `OfflineDefault` registered (v1 state): mechanism preserved continuously (`SC-051`; `spec.md:85`); reason expires after 3 s (`FR-041`); no auto-resume (`FR-014`); position replay bounded ≤30 s (`FR-015b`; `spec.md:391`).

---

## 4. Capability Declaration Contract (`FR-024c` / `Assumption 3` corrected)

### 4.1 Declaration interface

Adapter MUST declare capabilities unconditionally (`constitution.md:304-305` / `582-585`). Trait `T016` is mandated but not yet built (`crates/engine-core/src/lib.rs`: no `trait_.rs`; `grep '\btrait\b'` = 0); adapter declares against existing `DeclaredFormat` + `EngineError` contract shapes (`format.rs`, `closed.rs`). No phantom methods.

**Conceptual interface** (based on `engine-handoff.md:18-23` reference in stand-in docs; `spec.md:97`; `Assumption 3`):

```rust
pub struct AdapterCapabilities {
    pub ssml_level: SsmlLevel,           // v1: None
    pub alignment_granularity: AlignmentGranularity, // v1: None
}

pub enum SsmlLevel { None, Partial, Full }
pub enum AlignmentGranularity { None, Word, Sentence }
```

**Note**: `SsmlLevel` and `AlignmentGranularity` enums do **not** currently exist in workspace (`grep` `crates/engine-core/src/` = 0). Adapter design must either:
- Define them in adapter crate (acceptable — adapter owns declaration; `player` branches on value); **OR**
- Use string/constant representation until `T016` trait design completes.

**Recommended design decision (from `research.md` §3.3)**: Define `SsmlLevel` and `AlignmentGranularity` in adapter crate (or `engine-core` ahead of `T016`) with `None` variants, so `declared_capabilities()` has a concrete return type. Do **not** invent a `trait_::EngineAdapter` file prematurely.

### 4.2 Declared values (v1 — Option A)

| Capability | Value | Evidence / Rationale |
|---|---|---|
| `SsmlLevel` | `None` | `sherpa-onnx` + Piper does not honor SSML (`constitution.md` §TTS Engine Registry: SSML = none for `offline_default`); no `<break>` / pitch / prosody adjustments delivered |
| `AlignmentGranularity` | `None` | Piper exposes no sentence-boundary events (`spec.md:18`; `C1-A`); paragraph/heading is coarsest declared precision; sentence-level deferred to v2 (`FR-024`; `constitution.md` Audio Pipeline Contracts) |

**Player branch behavior** (must be implemented in `player`; adapter only provides declaration):
- `SsmlLevel::None` → no SSML rendering; plain-text segment delivery; no `<break>` insertion.
- `AlignmentGranularity::None` → resume snaps to nearest completed `Paragraph` / `Heading` boundary (not sentence); `snap_to_boundary` (boundary-kind-agnostic) is used and sentence-level snapping remains deferred to v2; source-range mapping at node/paragraph level (`constitution.md`: "map at sentence or node granularity — intended precision rather than defect").

**Verification**: `SC-051` mechanism must pass with adapter registered (`offline_default`); `standin_a` / `standin_b` not selectable (`001/spec.md:89`); `test_double` unreachable (`CLARIFICATION_BATCH.md` C3-A); adaptation of `player` to `None` must not break mechanism.

---

## 5. Resume / Boundary Agreement

### 5.1 Chunk split points = resume points

Adapter splits Markdown at `Paragraph` / `Heading` boundaries (`FR-024a`); each split produces a `DocumentSegment`; resume aligns to nearest completed boundary containing yield point (`spec.md:59`; `FR-015`).

**Agreement rules** (adapter ↔ `session` / `player`):

| Agreement | Adapter responsibility | Session / Player responsibility |
|---|---|---|
| Chunk boundary at paragraph/heading | Adapter (`chunk.rs`) splits; produces `DocumentSegment` with `source_range` | `session` stores the bookmark keyed by the `blake3` document fingerprint, with the playback offset as its value (`redb`; `crates/session/src/lib.rs`); `segment_index` is adapter-side numbering — not a bookmark key and not a bookmark field; the adapter never alters position storage (`spec.md` Assumption 5) |
| Resume to nearest completed boundary | Adapter delivers from correct `DocumentSegment`; does not infer sentence start | `player` uses `source_range_ref` + declared `AlignmentGranularity::None` → snap to boundary |
| Document edited (blake3 changes) | Adapter does not alter bookmark logic; delivers same chunks; `session` detects hash change | `session` clears bookmark; no resume offered (`spec.md:52`; `FR-036`) |
| Engine failure mid-document | Adapter reports `EngineFailure` with `session_position` echoing the delivered chunk's `source_range_ref` — never adapter-computed (`constitution.md:82-86`); a failure before the first delivery echoes the `DocumentSegment` in synthesis | `player` validates the echo against its own delivered-chunk log (fail-closed — RFC 7233 §4.2 "MUST NOT attempt to recombine"), pauses + 3-s offer; if no selection, resume derived from that log, ≤30 s replay |

---

## 6. Selection / Tier Rules

### 6.1 Registration (adapter → `engine-core` registry)

- Adapter registers only in `OfflineDefault` (`constitution.md` §TTS Engine Registry).
- No `CloudDefault`, `CloudEscalation`, `EvaluatedFallback` registered at v1 (`SC-044` deferred; `constitution.md:643` Exception 8).
- `test_double` fixtures (`standin_a`, `standin_b`) remain fixtures only; **not selectable** because adapter (non-standin) registered (`001/spec.md:89`; `closed.rs:305-312`).

### 6.2 Selector invariant (`SC-051` / `spec.md:85`)

The `SC-051` mechanism (failure → named failure → 3-s offer → automatic fallback if selectable → resume at boundary) must **continue passing** with adapter registered. Verification method:

1. Register adapter (`OfflineDefault`); disable `test_double` selectability.
2. Force failure (`simulated` via `standin_a` / `standin_b` — but fixtures not selectable; mechanism verified by paused-with-stated-reason on adapter failure; active switch deferred).
3. Confirm: named failure presented within 3 s; notification states reason; if only `offline_default` selectable, mechanism preserved (reason expires after 3 s; no auto-resume; position not replayed against failed timeline — `spec.md:391`).

**No regression permitted**: adapter must not break `SC-051`; active fallback switch deferred until `SC-044` (second real engine) — not a regression; mechanism preserved continuously (`CLARIFICATION_BATCH.md` C4-A; `spec.md:85`).

---

## 7. Wire / Interface Summary (quick reference)

| Direction | Interface / Type | Key constraint / wire name |
|---|---|---|
| Adapter → Player | `AudioChunk` (mono f32 @ 22050 Hz) | `DeclaredFormat` (`format.rs`) |
| Adapter → Player | `EngineFailure` (7-member `EngineError`) | Wire names: `engine_unreachable`, `engine_init_failed`, `rate_mismatch`, `channel_mismatch`, `device_refused`, `device_lost`, `start_refused` (`closed.rs`) |
| Adapter → Core / Player | `AdapterCapabilities` (`SsmlLevel::None`, `AlignmentGranularity::None`) | Unconditional declaration (`constitution.md:304`) |
| Adapter ← Player / Session | `DocumentSegment` boundary agreement (paragraph/heading; ≤5000 bytes) | `FR-024a`; resume at completed boundary |
| Adapter → Registry | `OfflineDefault` only; `TestDouble` not selectable | `SC-051`; `001/spec.md:89`; `constitution.md:643` (Exception 8) |

---

## 8. Verification Checklist (before implementation)

Before any adapter source file is created:

- [ ] Confirm `format.rs` and `closed.rs` shapes unchanged (no new variants added to `EngineError`; no new fields in `DeclaredFormat`).
- [ ] Confirm `SC-051` mechanism passes with adapter registered (fixture not selectable; mechanism preserved; active switch deferred).
- [ ] Confirm `pulldown-cmark` integration stage (`003-contract-edit`) resolves workspace dependency; if unreachable, document-layer validation holds (`R8`).
- [ ] Confirm adapter crate declares `[lints] workspace = true`; `missing_docs` denied (Principle III / Rust rigor).
- [ ] Confirm `cargo fmt` / `cargo clippy -- -D warnings` / `cargo deny` pass with `ort` RC exception (4) preserved; no new exceptions proposed.
- [ ] Confirm `CLARIFICATION_BATCH.md` C1–C4 mapping preserved at `spec.md:85` (7-member table; `DeviceLost` row; `DeviceRefused`/`StartRefused` split; `width`→`RateMismatch`; no `AdapterFailed`).
- [ ] Confirm `SC-044` stays `DEFERRED`; `test_double` unreachable (`001/spec.md:89`); `spec.md:138` corrected from stale `133`; consent acquisition deferred (no writer/UI/verb/storage key).

---

*End of contract design. Implementation deferred to Phase 2 (`tasks.md`) / `003-contract-edit` integration stage. No adapter crate files created at this phase; this contract guides implementation.*
