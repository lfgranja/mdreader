# Feature Specification: FR-024 Synthesis Adapter — sherpa-onnx + Piper pt-BR

**Feature Branch**: `002-fr-024-synthesis-adapter`

**Created**: 2026-10-06

**Status**: Draft — output of `/speckit.specify` from assessment `fr-024-synthesis-adapter` (decision: `go`, Option A)

**Input**: Assessment artifacts (`intake.md`, `research.md`, `problem.md`, `concept.md`, `decision.md`) defining the minimum viable synthesis adapter for the `001-android-media-shell` playback shell.

---

## Clarifications

### Session 2026-10-06

- Q: For the synthesis adapter to pass success, must the first chunk of real audio reach the speaker within 3 seconds of the Play press (`SC-009`), or can v1 accept a longer initial delay if pre-roll is available? (SC-009 / performance) → A: Confirm ≤3 s achievable with basic adapter (benchmark on reference device before declaring PASS); `SC-009` is a hard gate for v1. If benchmark fails, adapt with pre-roll/watchdog rather than defer to full synthesis feature.
- Q: Does the Piper pt-BR voice provide sentence-boundary events so the adapter can align resume to sentence start, or is basic `pulldown-cmark` segmentation sufficient for v1? (FR-024 / data model / resume accuracy) → A: Basic `pulldown-cmark` + source-range mapping is sufficient for v1; resume aligns to nearest paragraph/heading boundary initially; `SentenceBoundary` from `Piper` is not required for Option A and may be added later.
- Q: The adapter must respect the byte-budget bound (≤5000 bytes per chunk, `constitution.md` line 353). Should the adapter split Markdown segments into chunks before synthesis to enforce this, or rely on the engine to handle budget internally? (FR-024 / integration / data) → A: Adapter splits by paragraph/heading before synthesis (adapter owns chunk boundary); engine receives bounded chunks; resume points align to split boundaries.
- Q: When integrating the real adapter, must `SC-051` (engine-failure mechanism) remain passing continuously, or can it be temporarily disabled during transition? (FR-023a / integration / risk) → A: Preserve `SC-051` continuously as **selector invariant** (tier order `EngineTier::ORDER`, consent gate T075, 3-s offer, named failure presentation); adapter enters `offline_default`; stand-ins not selectable (`001/spec.md:89`); **active mid-document fallback switch deferred until SC-044 lands** (second real engine required); no regression; bad Markdown = document/parse-level (`spec.md:85`).
- Q: Should adapter declare all named failure categories supported by the engine contract using the correct closed-set wire names (`RateMismatch`, `ChannelMismatch`, `InitFailed`, `Unreachable`, `DeviceRefused`, `DeviceLost`, `StartRefused`) and include the condition→variant table? (FR-035 / closed-set) → A: Yes — replace incorrect names (`format_mismatch`, `engine_crash`, `refusal`, `deferred_grant`) with the 7 real `EngineError` variants; add condition→variant mapping (format→Rate/ChannelMismatch; init/voice corruption→InitFailed; process death/OOM/kill→Unreachable; device refusal→DeviceRefused; start refused→StartRefused); `SC-051` runs with adapter registered (adapters in `offline_default`, standins not selectable per `001/spec.md:89`); bad Markdown input is document/parse-level, not `EngineError`; no new `AdapterFailed` variant needed.
- Q: Should v1 close SC-044 on the `test_double` branch, deferring Google Cloud TTS / consent surface? (SC-044 / consent / deferred) → A: **Split** — consent acquisition deferred (no writer/UI/verb/storage key; out of Option A scope per `spec.md:138`); SC-044 stays `DEFERRED` (not close on `test_double` — unreachable under `001/spec.md:89`; `test_double` inadmissible once adapter registered per `001/spec.md:89`); fix stale `spec.md:133` → `138`; match `constitution.md:643` (`gate-8-full: DEFERRED, owner=synthesis`).
- Q: Should adapter resume use sentence-level alignment, given Piper exposes no boundary events? (FR-024 / data / resume) → A: Paragraph/heading resume via `pulldown-cmark` + source-range mapping for v1 (`AlignmentGranularity::None` declared); `SentenceBoundary` from Piper not required; marker at `FR-024` removed; US/SC corrected to nearest completed paragraph/heading boundary; sentence-level deferred to v2.
- Q: Must adapter declare `SsmlLevel` / `AlignmentGranularity` via `declared_capabilities()` given the trait does not yet exist? (FR-024c / integration / contract) → A: Yes — declare `(none, none)` against existing `DeclaredFormat`/`EngineError` shapes (not an unspecified trait); `FR-024c` and `Assumption 3` corrected; `player` branches on the declared value, which is honored without a raw-cursor degradation branch — for `AlignmentGranularity::None` the paragraph/heading snap always applies, and the raw cursor is reserved for the document-layer / unparseable case only (`CLARIFICATION_BATCH_RESOLVED.md:13`); sentence-level resume stays deferred to v2.
- Batch confirmation 2026-10-06 (C1–C4, subagent deep-dive + /research per candidate): C1-A (paragraph resume, marker already removed, US/SC corrected to paragraph/heading, sentence deferred); C2-A (declaration `(none,none)` against existing shapes, `FR-024c` normative, `Assumption 3` corrected — trait mandated by `T016` not yet built, no phantom `format()`/`fail()`); C3-A **split** (consent deferred; SC-044 stays `DEFERRED`, not close on `test_double`; stale `spec.md:133` → `138` fixed); C4-A (7-member `EngineError` + condition→variant table integrated at `spec.md:85`; `DeviceLost` row added; `DeviceRefused`/`StartRefused` split documented; `width`→`RateMismatch` noted; `SC-051` preserved as invariant / active switch deferred; no `AdapterFailed` — 7-site change not justified). All confirmed by internal source (`crates/engine-core/src/closed.rs` 235-270, `format.rs`, `snap_to_boundary`; `spec.md` cross-references); scientific evidence (Zacks 2007 / Monk 2008 / Grosjean 1980; Yuan OSDI'14 / ExoPlayer / gRPC / W3C SSML 1.1 / GDPR Art.7-9 / AES67 PTP). Checklist re-validated.
- Q: Should bad Markdown input be recorded in `diag` with a new `parse` kind once the contract-edit cycle completes, or stay document-layer only? (FR-024 / diag / failure taxonomy / `spec.md:85`) → A: Clarified 2026-10-06 (subagent `ses_eebac5cb8ffe107VdoI27hvXXO` completed): (i) **Reachability first** — `pulldown-cmark` accepts any byte string (no error type); it is not yet a workspace dependency (`grep` workspace `Cargo.toml` → 0). Under the chosen parser, a parse failure may be unreachable; confirm At `003-contract-edit` stage before any schema change. (ii) **If reachable**, prefer **option (b)** — new `error` member (`document_unparseable`) on the existing `failure`/`refusal` kinds (gated by `FR-044` coordinated edit, no `engine-core` file touched), not a new `kind`, preserving "not an `EngineError`" boundary (`spec.md:85`). (iii) **If unreachable**, declare so explicitly and map bad-Markdown to an existing named member (document-layer validation rejects before adapter invocation; `diag` records synthesis-level events only). For v1: keep document-layer only, no `diag` expansion; deferred to contract-edit cycle; subagent confirms low risk of leaving deferred today but window closes when T073 lands with 9 `kind` variants. `CLARIFICATION_BATCH.md` C4-A mapping preserved (`bad Markdown → document/parse`, not `EngineError`).

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — First audible read aloud from a Markdown document (Priority: P1)

A driver opens a Markdown document in the `mdreader` app, presses Play, and hears the first spoken sentence within 3 seconds — without unlocking the phone, and with the screen turning off shortly after.

**Why this priority**: This is the core promise of the driving-first design (`Principle VIII` / `spec.md` US1). Without real synthesis, the entire shell (`player`, `focus`, `notification`) remains a mechanism proof (`SC-051`) rather than a user-facing product.

**Independent Test**: With the adapter integrated and `standin_a`/`standin_b` retired for this session, start playback on a parsed Markdown file, lock the device, and confirm audio continues uninterrupted for the document duration. The test requires only one real engine (`sherpa-onnx` + `Piper` `faber`) to pass.

**Acceptance Scenarios**:

1. **Given** a document is parsed by `pulldown-cmark` and a session starts, **When** the user presses Play, **Then** audio emerges within ≤3 s (`SC-009`) as mono f32 at 22050 Hz, never as a silent resample or substituted format (`FR-035`).
2. **Given** the device screen turns off or is manually locked, **When** playback is active, **Then** it continues uninterrupted (`spec.md` US1 / `FR-013`).
3. **Given** the adapter delivers its first chunk, **When** the format is exactly mono f32 @ 22050 Hz, **Then** the `player` converts to device format; if the adapter delivers anything else (stereo, different rate, wrong bit-depth), **Then** a named failure (`FR-035`) is surfaced and playback does not start silently.

---

### User Story 2 — Navigation prompt yields, then reading resumes (Priority: P1)

A navigation app requests transient audio focus. The document ducks to ~15–25% (`FR-013`; target ~20%, range 15–25%, app-applied gain) and keeps reading; when focus returns, playback resumes at the nearest completed paragraph/heading boundary (`FR-015` rewind formula; sentence-level deferred to v2).

**Why this priority**: Routing interruptions are where driving gets dangerous; a document that talks over a turn-by-turn prompt teaches the user to turn the app off (`spec.md` US2 / clarification 2026-10-01).

**Independent Test**: Trigger a transient focus event (`AUDIOFOCUS_LOSS_TRANSIENT`) during playback with the adapter active; confirm volume drops to ~15–25% (FR-013 band; app-applied gain, not acoustic output; device-minimum-volume best-effort per FR-013); confirm regain resumes from paragraph/heading boundary with ≤30 s cumulative rewind (`FR-015` / `spec.md` line 28).

**Acceptance Scenarios**:

1. **Given** audio is playing via the adapter, **When** a transient focus request arrives, **Then** volume ducks to ~15–25% and reading continues (`FR-013`).
2. **Given** focus is regained after transient loss, **When** playback resumes, **Then** the cursor moves back min(5% × paused duration, 30 s) from the yield, then snaps to the nearest completed paragraph/heading boundary containing that 5%-back point (cap overrides snap when boundary >30 s; cumulative rewind across supersede per `FR-015`); zero mid-word landing (`Grosjean 1980` / `Bashford&Warren 1987`).
3. **Given** a permanent focus loss (call, alarm), **When** it occurs, **Then** playback pauses immediately (`FR-013a`), a reason expires after 3 s (`FR-041`), and no auto-resume occurs (`FR-014`).

---

### User Story 3 — Engine failure mid-document, fallback continues (Priority: P1)

The adapter fails partway through a spoken segment; a second real engine takes over from the last spoken position (nearest paragraph/heading boundary for the new engine; sentence-level deferred to v2) without requiring the driver to unlock the device.

**Why this priority**: Real-world engines fail (network drop for cloud, memory pressure, voice data corruption). The `engine-core` contract (`FR-023a`) requires a named failure, a bounded 3-second offer, and automatic fallback to the next tier — with the position re-anchored to the nearest paragraph/heading boundary for the new engine (sentence-level deferred to v2; SC-051 preserved as selector invariant, active switch deferred until a second real engine is available).

**Independent Test**: Force the adapter to fail mid-document (simulate by stopping the `sherpa-onnx` process — bad Markdown input is document/parse-level, not `EngineError`) and confirm `SC-051` mechanism works: `FR-023a` named failure presented within 3 s (detection immediate, offer 3 s), fallback continues from nearest paragraph/heading boundary once a second real engine is registered (v1: mechanism preserved; active switch deferred per `SC-051` clarification), notification carries the failure reason (`FR-037`).

**Acceptance Scenarios**:

1. **Given** the adapter is playing, **When** it fails, **Then** a named `EngineError` is surfaced (`FR-023a`) immediately (offer expires at ≤3 s); the notification states the failure (`FR-037`); the fallback tier (next `EngineTier` per `FR-023c`) takes over when available; with adapter as sole registered engine, mechanism is preserved (`SC-051` invariant) and active switch is deferred until SC-044 lands.
2. **Given** the user makes no choice within 3 s, **When** the offer expires, **Then** automatic fallback continues from the nearest paragraph/heading boundary if a selectable tier exists (`FR-023a`); if only `offline_default` is registered, mechanism preserved (`SC-051`) with reason expiring after 3 s (`FR-041`) — position not replayed against failed timeline (`spec.md` line 391).
3. **Given** the user selects an alternative engine, **When** playback resumes, **Then** it resumes from the last spoken position re-anchored to the nearest paragraph/heading boundary for the new engine using the adapter's existing `DocumentSegment` split points (adapter never re-splits; `FR-023a` / `spec.md` line 37 / `adapter-handoff.md` §4); sentence-level deferred to v2; v1 verifies mechanism preservation only (`SC-051`), active takeover deferred (`SC-044`).

---

### Edge Cases

- What happens when the adapter delivers audio at a rate other than 22050 Hz? → Named `EngineError` (`FR-035`); playback does not start; device-format conversion by Rust is never silent.
- How does the system handle a device that refuses 22050 Hz? → Named failure (`FR-035`); never silent resample or substituted format (`spec.md` line 384–389).
- What happens if the adapter fails during pre-roll (≤60 s buffer while paused)? → Pre-roll stops; playback resumes only after manual Play; no continuous synthesis while idle (`spec.md` line 66 / `FR-015c`).
- How does resume behave when a document is edited on disk? → Fingerprint (`blake3`) changes; stored position unreachable; no resume offered (`spec.md` line 52 / `FR-036`).
- How does the adapter signal failure to `player`, and which `EngineError` variant applies? → Adapter uses the 7-member closed set (`RateMismatch`/`ChannelMismatch`/`InitFailed`/`Unreachable`/`DeviceRefused`/`DeviceLost`/`StartRefused`). Condition→variant: format/width/rate mismatch → `RateMismatch` (no separate width variant per `format.rs`; width treated as rate-shape under Principle VII; pitch-tolerance ±3 cents @ 2.0× also `RateMismatch`); delivered channel count ≠ 1 → `ChannelMismatch`; voice-model/init failure → `InitFailed`; process death/OOM/kill → `Unreachable`; output device removed (`ACTION_AUDIO_BECOMING_NOISY` **or** `onAudioDevicesRemoved`, independent — either one sufficient, each held to ≤1 s pause bound per `AGENTS.md:79` / `001/spec.md:653`) → `DeviceLost`; stream-negotiation device refusal (AAudio rate/format refusal at open) → `DeviceRefused`; stream-start refused (API 34+ `ForegroundServiceStartNotAllowedException` / consent missing at start) → `StartRefused`; bad Markdown segment → document/parse error (not `EngineError`; `diag` schema `kind` enum lacks `parse` — clarification 2026-10-06: subagent `ses_eebac5cb8ffe107VdoI27hvXXO` confirms `pulldown-cmark` is infallible and not yet a workspace dependency; if parse failure unreachable, declare so and use existing named member; if reachable, prefer option (b) new `error` member on `failure`/`refusal` kinds via `FR-044` coordinated edit, not new `kind`; deferred to contract-edit cycle, no v1 schema expansion, `SC-051` preserved) `SC-051` preserved continuously as selector invariant (`offline_default` adapter registered; standins not selectable per `001/spec.md:89` / `constitution.md:306-312`); active mid-document fallback deferred to second real engine (`SC-044`).

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-024**: The synthesis adapter MUST deliver mono float32 audio at the declared playback rate of 22050 Hz from a Markdown document input via `sherpa-onnx` and Piper pt-BR voice data (`faber` as default), honoring the `engine-core` handoff contract (mono f32, declared rate, named failure on format mismatch). Adapter declares `AlignmentGranularity::None` (Piper exposes no sentence-boundary events; `sentence` resume deferred to v2); resume aligns to nearest completed paragraph/heading boundary via parser-side `pulldown-cmark` segmentation (`FR-024a`).
- **FR-024a** (derived): The adapter MUST split Markdown source into bounded chunks (≤5000 bytes, `constitution.md` line 353) at paragraph/heading boundaries before synthesis; chunk boundaries become resume-alignment points; if a sentence boundary falls within a paragraph/heading, the paragraph/heading boundary takes precedence (sentence-level resume deferred to v2); the engine receives bounded chunks and does not independently resample or split.
- **FR-024b** (derived): If the adapter delivers audio in any format other than mono f32 @ 22050 Hz, or delivers stereo, or resamples silently, the system MUST surface a named `EngineError` (`FR-035`) and MUST NOT start playback; the `player` conversion to device format is the sole format conversion step.
- **FR-024c** (derived): The adapter MUST declare its capabilities via `declared_capabilities()` (`SsmlLevel::None`, `AlignmentGranularity::None`) per `engine-handoff.md:18-23` and `constitution.md:304-308`; it implements the `engine-core` contract (not an unspecified trait), does not modify `player` transport/position logic, and introduces no second source of truth over playback state (single authority per `constitution.md:82-86`). `Assumption 3` corrected: trait is mandated (`T016`) but not yet built — adapter declares against the existing `DeclaredFormat` + `EngineError` contract shapes (`format.rs`, `closed.rs`).

### Key Entities *(data involved, no implementation details)*

- **DocumentSegment**: A bounded chunk of spoken content derived from Markdown, identified by source range (from `pulldown-cmark`), carrying a `blake3` content hash of the document and a sentence/index offset for resume alignment.
- **Empty-state / partial-load (edits 6-7)**: No `DocumentSegment`s delivered → `AudioChunk` not delivered; notification indeterminate / no progress. Partial load (first N segments only) → adapter synthesizes available segments; resume on regain uses available split points; document-layer error (not `EngineError`). Adapter never delivers partial-then-completes without user action; `DocumentSegment` split points remain canonical.
- **AudioChunk immutability (edit 8)**: `AudioChunk` immutable after delivery; adapter never modifies after delivery; `player` resume uses `source_range_ref` / `duration_ms` without adapter state access; `redb` bookmark authoritative.

- **AudioChunk**: The output unit delivered to `player`: mono f32 @ 22050 Hz, bounded by the engine contract, with no embedded document text (`FR-045` / `spec.md` line 79); pre-roll ≤60 s (`FR-015a`).
- **EngineFailure (named)**: A typed error variant (`FR-035`) carrying the failure reason (the 7-member `EngineError` closed set, `spec.md:86`) and `session_position` — an **echo of the `source_range_ref` of the chunk the adapter delivered**, never adapter-computed (`constitution.md:82-86`, `:486-490`; `FR-024c`). `player` MUST NOT treat the echo as playback authority: it derives the resume position from its own delivered-chunk log under its own ≤30 s replay bound (`FR-015b`). Used by `player` to present `FR-023a` / `FR-037` to the user.
- **EngineTier**: The closed-set ordering (`offline_default` → `cloud_default` → `cloud_escalation` → `test_double` → `evaluated_fallback`) governing fallback selection (`FR-023c`, `constitution.md` §Engine Registry); the adapter lives only in `offline_default`.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-009**: With the adapter active, a user can press Play on a parsed document and audible audio is produced within ≤3 seconds at the transducer (full pipeline: adapter mono f32 @ 22050 Hz → player device-format conversion → speaker; endpoint is transducer, not adapter output; reference device = `001/plan.md:50-53` mid-range arm64, 4x Cortex-A78-class, 6 GB RAM). Pre-roll (`FR-015a`) begins on the press; ≤3 s applies to first audible after Print (cold start), independently of any prior pre-roll; warm-resume (pre-roll buffered) governed by `SC-020` (≤4 s).
**Gate-check sequence (edit 5)**: Requirements review → `SC-009` benchmark on reference device → mechanism-preservation verification (`SC-051` preserved continuously, `SC-035` named failure, `SC-027` position verified, `FR-028` notification integrity) → discharge-readiness assessment (`SC-044` prerequisites explicit, consent deferred, `test_double` inadmissible with adapter registered). PASS requires all four stages in order; `SC-044` deferred is a valid PASS condition for v1.

- **SC-044 (DEFERRED)**: Two real working engines would be required to discharge this gate (`constitution.md:643`; `001/spec.md:1454`; `001/VERIFICATION.md:56` — `PENDING / DEFERRED — synthesis feature`); under Option A (`002/spec.md:138`) only the `offline_default` (`sherpa-onnx` + `Piper`) engine is registered, so `test_double` (admissible only when *no* non-`test_double` adapter is registered, `001/spec.md:89`; `closed.rs:305-312`) is unreachable (`CLARIFICATION_BATCH.md` C3-A / Finding A). Defer to `003-cloud-fallback` / Edge; no consent acquisition surface in v1 (`FR-023c` read-only, `T075`); mechanism verified continuously as invariant (`SC-051`; `spec.md:85` / C4-A).

**SC-044 discharge prerequisites (explicit, edit 4)**: (1) second selectable real engine registered; (2) consent acquisition surface built (`writer` / `UI` / `verb` / `storage key` / `T075`); (3) `test_double` admissible only when *no* adapter registered (`001/spec.md:89`; `closed.rs:305-312`); (4) `SC-051` mechanism preserved continuously (not degraded); (5) `SC-027` / `FR-036` resume verified; (6) `SC-009` benchmark passes; (7) `SC-035` named-failure verified. All must be stated before PASS; with adapter registered, only preservation (not discharge) is verifiable in v1.
- **SC-051 (preserved as invariant)**: The mechanism of engine failure and automatic fallback continues as a selector invariant (`offline_default` adapter registered; `test_double` and stand-ins not selectable per `001/spec.md:89` / `closed.rs:305-312`; `001/spec.md:89`). Active mid-document takeover is deferred until a second selectable real engine is registered (`SC-044`; `constitution.md:643`; `spec.md:85` / C4-A); with adapter as sole registered tier, mechanism verifies via paused-with-stated-reason (`FR-023a` / `FR-041`) on failure, never silent and never auto-resume (`FR-014`).
- **SC-035**: Any format mismatch (stereo, wrong rate, non-float, wrong bit-depth) is surfaced as a named `EngineError`, never as silent resample or silent failure (`FR-035`; `spec.md` line 384–389).
- **SC-027 (transport)**: Playback position advances correctly through adapter-produced chunks (`DocumentSegment` / `AudioChunk`); resume (`FR-036`) finds nearest completed paragraph/heading at or before yield (new engine reuses adapter `DocumentSegment` split points — never re-splits); replay after unexpected end bounded at ≤30 s (`FR-004` / `SC-007`; `FR-015b` = pre-roll delivery, not replay). `blake3` fingerprint invalidates position on edit (`FR-036`). ID non-colliding (not `001/spec.md:1481` rate-consistency).

### Qualitative / Technology-agnostic

- **Notification integrity (R2/R3/R4)**: The lock-screen notification displays correct title + section title + progress (indeterminate/determinate per `FR-028`) while the adapter plays, with controls derived from `PlaybackState` (`FR-018`); no custom RemoteViews, no Live Update promotion.
- **Driving-safety (Principle VIII)**: All control interactions (play/pause, ±5 s skip, rate ladder 7 steps, stop, resume offer) remain reachable while locked, with ≤3 s bounded offer (`FR-032c`) and never auto-resume after unexpected end (`FR-014`; `spec.md` line 59).

---

## Assumptions

- **Assumption 1**: `sherpa-onnx` (Apache-2.0, `ort` 2.0.0-rc.13 with `NNAPI`) publishes and runs a working `aarch64-linux-android` build that produces mono f32 output compatible with `cpal`; `constitution.md` §Engine Registry records this as the selected `offline_default`.
- **Assumption 2**: `Piper` pt-BR voice models (`faber`, `cadu`, `jeff`, CC0; `edresson` excluded for CC-BY-4.0) can be loaded from embedded assets without requiring network at synthesis time, honoring `Principle IX` (offline-first).
- **Assumption 3**: The `engine-core` trait (`T016`) is mandated by `constitution.md:299-312` but **not yet built** (`crates/engine-core/src/lib.rs` exposes `closed`, `format`, `standin_a`, `standin_b` — no `trait_.rs`; `grep '\btrait\b'` returns 0 (no trait declarations in crates/; no `cpal` imports either); `declared_capabilities()` 0 occurrences). The adapter declares `(SsmlLevel::None, AlignmentGranularity::None)` against the existing `DeclaredFormat` + `EngineError` contract shapes (`format.rs`, `closed.rs`); declaration is unconditional (`constitution.md:304-305` / `582-585`); `player` branches on the declared value (paragraph/heading boundary snap — never silent inference; the raw-cursor degradation branch never fires for `AlignmentGranularity::None`, per `CLARIFICATION_BATCH_RESOLVED.md:13`); no phantom `format()`/`fail()` methods exist in the workspace.
- **Assumption 4**: `pulldown-cmark` produces sufficient structural events (headings, paragraphs) for basic resume anchoring at paragraph/heading boundary for v1; perfect sentence-level resume (`FR-036`) may require enhancement to `doc-structure` or `Piper` boundary events later (deferred, not blocking Option A).
- **Assumption 5**: `redb` bookmark store and `blake3` fingerprint remain the sole resume mechanism; the adapter does not alter position storage — only delivers audio that the `player` tracks.

---

## Notes

- This spec describes the adapter only — the `player`, `session`, `diag`, and Kotlin `MediaSession` are out of scope (per `problem.md` Non-Goals / `spec.md` FR-034 clarificação).
- The 3 remaining open questions from the assessment (`problem.md` / `concept.md`) carried forward to `/speckit.clarify`: latency benchmark (`SC-009`); `Piper` boundary events (resolved: none declared, deferred sentence-level to v2); byte-budget interaction (resolved: adapter splits at `FR-024a`). `CLARIFICATION_BATCH.md` C1–C4 resolved 2026-10-06; `SC-051` preservation and `FR-035` refinement resolved at `spec.md:20` / `:21` / `:85`; `diag` `parse`-kind gap (line 85 / note 138) clarified 2026-10-06 — subagent `ses_eebac5cb8ffe107VdoI27hvXXO` (deep-dive): `pulldown-cmark` infallible, not yet in workspace; preference if reachable = option (b) (`error` member on existing `failure`/`refusal` kinds via `FR-044`, no engine-core touch), else declare unreachable; deferred to contract-edit cycle; window closes when T073 lands with 9 `kind` variants.
- No `SSML`, no cloud fallback, no second voice, no diagnostic viewer, and no document picker are included in Option A per `concept.md`; those remain deferred (`spec.md` clarifications 2026-10-01 / 2026-10-05).
