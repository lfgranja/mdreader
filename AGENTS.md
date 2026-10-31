# Repository Guidelines

## Project Overview

`mdreader` (product **Ressoa**, package `app.mdreader`) — Android media-shell TTS audiobook reader. Rust owns synthesis + PCM playback (sole authority for transport/position/buffering); Kotlin (`android-shell/`) owns `Service`, `MediaSession` (mirror only — never computes position, enforced by check SC-035), notification, `AudioFocusRequest`. Tauri 2.12.1 bridge (`src-tauri/`) is one consumer among many; every crate is CLI-drivable (`clap` derive, stdin/args → stdout, errors → stderr, `--format json`). Target: Android minSdk 29, host-checkable gates 1–8 (`scripts/verify.sh`), device gate 9 (`cargo tauri android build --target aarch64`).

Key principles:
- **I (library-first):** every capability reachable as Rust API; CLI is thin `main.rs`.
- **II (one code path):** identical `run()` for CLI and IPC (`tauri::ipc::Channel`, no `InvokeBody::Streaming`).
- **V:** zero Android API calls in Rust; Kotlin pushes answers (`PermissionSnapshot`, intents) across boundary.
- **VI:** pipeline degrades observably to declared capabilities; never silently infers SSML/alignment.
- **VII / FR-044 / gate 7:** closed sets everywhere, exhaustive `match`, no `_ =>` arms, no `Other`/`Unknown`; adding variant = compile error at every consumer until coordinated edit + docs + semver bump.
- **Durability cadence (FR-004 / T011):** 20 s heartbeat while `Playing` + event-durable writes; hard kill replays ≤ ~30 s.

## Architecture & Data Flow

```
Kotlin surface (Service/MediaSession/notification/focus)
        ↓ intents (wire strings)
Tauri bridge: src-tauri/src/lib.rs (AppState, commands, publish)
        ↓ same Rust code path
crates/player      — transport, position, buffering, pre-roll (≤60 s), rate, rewind/snap math
crates/engine-core — Engine trait, closed-set vocabulary, handoff format (mono f32 @ 22050 Hz), stand-in engines
crates/session     — redb bookmarks + settings (blake3-256 fingerprint), resume-offer state machine
crates/diag        — append-only JSONL diagnostic record (app-private, 30-day + 10 MB trim)
crates/doc-structure — ordered headings → timeline positions, FR-038 re-anchor
crates/adapter-synthesis — offline_default tier (sherpa-onnx 1.13.8 + Piper pt-BR), capabilities, chunking
crates/cleanroom-core — proptest invariant generators (standalone, not workspace member)
```

Audio contract: engine delivers mono `f32` PCM at declared 22050 Hz; engine owns pitch-preserving stretch (`Engine::set_rate`); player selects rung from closed ladder. Rate/channel/width mismatch = named `EngineError` (`rate_mismatch` / `channel_mismatch` / `device_refused` / `device_lost` / `start_refused` / `engine_unreachable` / `engine_init_failed`) — never silent resample/downmix.

State/concurrency: single `AppState` behind `tauri::manage`, fields `Mutex<_>`; each lock taken alone and released before next — no two state locks held together. Position authority: `PositionClock` (`f64` ≥ 0 + `stored_basis_secs`) advances only on audible progress from callback (FR-005), never during pre-roll, never on its own timer. Lock order matches IPC path: `session` → `bookmark_store` (store last), mirroring heartbeat read order — prevents cycles.

Authority split (FR-002 / FR-003 / SC-035): Rust = sole playback authority; Kotlin surface = mirror only (transport/position/progress from `MirrorState` / `SurfaceSnapshot`). No second clock, no mutable playback logic in Kotlin (`check-shell-purity.sh` enforces). Tauri manifest edited directly (`mediaPlayback` foreground service + FGS permissions) — Tauri 2 has no `mobile::update_manifest` / `registerService` plugin API; verify on every Tauri upgrade.

Closed sets (`crates/engine-core/src/closed.rs`): `PlaybackRate` (7 rungs: 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0 — wire JSON bare f64), `TransportState` (playing/paused/stopped/completed/interrupted), `EngineError` (7 values), `SessionEnd` (paused/stopped/completed/interrupted — never playing, FR-044), `EngineTier` (offline_default/cloud_default/cloud_escalation/test_double/evaluated_fallback, ascending order `EngineTier::ORDER`), `FocusOutcome` (duckable_transient / silence_demanding / permanent_loss; regained = separate direction event). Extension = coordinated edit across all consumers + docs + semver bump.

Wire mapping (normative, `contracts/shell-ipc.md`): Rust `PascalCase` → JSON `snake_case` → verb/intent. Example: `TransportState::Playing` → wire `playing`; `PlaybackRate` → bare number (`1.5`); `EngineTier::OfflineDefault` → `offline_default`. All `Display` impls return the wire string; diagnostics rely on it.

Durability (`crates/session/src/store.rs`): `redb` `bookmarks` (key: blake3-256 fingerprint of document bytes — path-independent, edit-sensitive) + `settings` (`last_rate`, `BOOKMARKS` table definition). `DURABLE_CADENCE_MS = 20000`. Atomic crash-safe writes (`BookmarkStore::put`). Best-effort: storage failure = tracing warn, never panic; missing document / unopened store / poisoned lock skips with warn. Only `Stop` and `DocumentEnd` delete rows (FR-032 / FR-032b / FR-032f); focus-loss / `BecomingNoisy` / `swipe_away` / pre-suspend / section-change preserve.

Synthesis (`crates/adapter-synthesis/src/`): `offline_default` tier (`reg.rs`: `REGISTERED_TIERS = [EngineTier::OfflineDefault]`). `chunk::split_document` owns chunk boundaries; `synth::synthesize` produces bounded chunks; `format_check` verifies declared mono f32 @ 22050 Hz; failure taxonomy (`failure.rs`) re-exported. `capabilities` declares `SsmlLevel` / `AlignmentGranularity` vocabulary; `consume` delivers chunks.

## Key Directories

| Path | Purpose |
|---|---|
| `crates/player/` | Playback session: transport (`cli.rs`), position (`position.rs`), rate ladder (`rate.rs`), skip/rewind (`skip.rs`, `rewind.rs`), focus (`focus.rs`), routing (`routing.rs`), interruption (`interruption.rs`), surface (`surface.rs`), backend (`backend.rs`), recovery (`recovery.rs`), resume (`resume.rs`), earcon (`earcon.rs`) |
| `crates/engine-core/` | Closed vocabulary (`closed.rs`), `Engine` trait (`trait_.rs`), fallback selector (`fallback.rs`), handoff format (`format.rs` — `SAMPLE_RATE = 22050`, `CHANNELS = 1`, `SAMPLE_WIDTH_BITS = 32`), stand-in fixtures (`standin_a.rs`, `standin_b.rs`) |
| `crates/session/` | `redb` bookmark/settings store (`store.rs`), blake3 fingerprint (`fingerprint.rs`), offer machine (`offer.rs`), resume algorithm |
| `crates/diag/` | Append-only JSONL writer (`writer.rs`); scanner (`scan.rs`) — 7 closed violation kinds: `prose_in_detail`, `detail_too_long`, `failure_without_error`, `refusal_without_error`, `playing_without_advance`, `session_end_with_playing`, `engine_without_tier` |
| `crates/doc-structure/` | Headings → timeline (`lib.rs`); FR-038 re-anchor |
| `crates/adapter-synthesis/` | Offline adapter (`offline_default`): chunk (`chunk.rs`), synth (`synth.rs`), capabilities (`capabilities.rs`), format check (`format_check.rs`), failure taxonomy (`failure.rs`), registry (`reg.rs`) |
| `src-tauri/src/lib.rs` | `AppState`, commands (`register_android_plugin`, `update_permissions`, `run_mobile_plugin` = `play`/`pause`/`stop`/`diagnose`), publish (`publish`), heartbeat (`HEARTBEAT_PERIOD_SECS = 20`), playback watch (`PLAYBACK_WATCH_PERIOD_MS = 500`), `diag_record`, `surface` (reason getters: `short_reason`, `full_reason`), `failure`, `status_view` |
| `android-shell/app/src/main/` | Kotlin: `Service`, `MediaSession` (mirror), notification, `AudioFocusRequest`. Zero playback logic enforced by `check-shell-purity.sh` |
| `contracts/` | `shell-ipc.md` (verbs/intents/state mirror/case mapping), `diagnostic-trace.md`, `resume-algorithm.md`, `surface.md`, `earcon.md`, `ingest.md`, `storage-schema.md` |
| `specs/001-android-media-shell/` | Feature spec (`spec.md`), tasks (`tasks.md`), contracts, checklists |
| `docs/` | `architecture.md`, `ipc.md`, `diagnostic-record.md`, `fallback-order.md`, `byte-budget.md`, `adr/` (e.g. `adr-002-diag-§VII.md`, `adr-003-earcon-at-focus-loss.md`) |
| `.specify/memory/constitution.md` | Governance (principles, engine registry, verification gates, exceptions log) |
| `vendor/sherpa-onnx-sys/` | Vendored build-fix (`zip 2 → 9`, clears RUSTSEC-2026-0367/0368/0369). Remove when upstream ships (see `PROVENANCE.md`) |

## Development Commands

```sh
# Format (gate 1) — MUST exit 0; never weaken to silence lint
cargo fmt --all -- --check
cargo fmt --all

# Tests (gate 2) — cargo-nextest, NOT plain cargo test
cargo nextest run --workspace
cargo nextest run -p player --test prop_transport_math
cargo nextest run -p engine-core --test prop_closed_sets

# Documentation (gate 3)
cargo doc --workspace --no-deps

# Lint (gate 4 — fail-closed: non-zero = FAIL, never skip)
cargo clippy --workspace --all-targets -- -D warnings

# Supply chain (gate 5)
cargo deny check advisories licenses bans sources
cargo audit

# Gate runner (1–8, fail-closed)
./scripts/verify.sh

# Gate 9 — Android artifact (runs OUTSIDE verify.sh)
cargo tauri android build --target aarch64

# Full workspace build
cargo build --workspace
cargo build --release --workspace --locked

# Single crate / check
cargo check -p player
cargo check --target x86_64-unknown-linux-gnu   # .cargo/config.toml: clang linker

# CLI invocations (Principle II — same code path)
# Workspace members: player, engine-core, session, diag, doc-structure, adapter-synthesis
player play                 # → {"transport":"Playing","position_secs":0,"rate":"1.0","engine_id":"rust-player"}
player rate-up              # moves one rung on closed ladder
player skip-back            # clamped at 0.0
session --db s.db get doc.txt
session --db s.db put doc.txt --offset-secs 12.5 --first-yield-ms 100 --end-kind paused
session --db s.db rate get  # stored rate rung
session --db s.db offer show doc.txt
session --db s.db foreground on|off
engine-core capabilities    # declared vocabulary
engine-core stream --at 12.5 --budget-secs 5.0
diag scan record.jsonl     # 7 violation kinds; exits 0 even with violations; non-zero = read/corrupt error
doc-structure headings headings.json
doc-structure reanchor headings.json --at 75.0
adapter-synthesis capabilities
adapter-synthesis chunk --input doc.md   # segment metadata only (FR-045)
```

## Code Conventions & Common Patterns

**Formatting / imports:** 4-space indent, line length 100 soft / 120 hard; `cargo fmt` always; imports grouped with blank lines between std / external / local crates; alphabetical within groups; prefer `use crate::`; `derive(Debug, Clone, Serialize, Deserialize)` for structs.

**Naming:** Structs/Enums/Traits `PascalCase`; functions/variables `snake_case`; constants `UPPER_SNAKE_CASE`; modules `snake_case`; test names descriptive with underscores (`test_calculates_sum_correctly`); private helpers prefix `_` if unused; generics single uppercase (`T`, `U`, `E`, `I`).

**Types:** `f64` for floating-point (positions); `i32`/`i64` whole numbers; `String` owned, `&str` refs; `Vec<T>` collections; `HashMap<K, V>` mappings; `Option<T>` / `Result<T, E>` everywhere. Custom errors with `thiserror` (`#[derive(Error, Debug)]`); propagate with `?`; `anyhow::Context` where useful. `Eq` precluded by `f64` fields; `PartialEq` exact (quarter-second grid = exact binary fractions). No `unwrap`/`expect`/`panic`/`todo`/`dbg`/`println`/`print_stdout`/`print_stderr` — denied (`Cargo.toml` / `clippy.toml`); only suppression: `#[expect(lint, reason = "...")]` with written reason. Never weaken `Cargo.toml`/`clippy.toml` to silence lint; no blanket `#[allow]`.

**Async / runtime:** `tokio` (`rt`, `time`) ONLY in `src-tauri`. Library crates (`player`, `session`, `engine-core`, etc.) are synchronous; no `async`/`await` in crates. `cpal` blocking output-stream callback; position advances on audible progress from callback.

**Error handling:** Fail-closed. Named `Error` enums (`thiserror` + `tracing`). Rejected inputs = named errors with non-zero exit (`player rate set 3.0` → `"rate must be one of 0.75 | 1.0 | ... | 4.0, got 3"`). Storage failure = best-effort (`tracing` warn, never panic; gap marker `kind: "gap"`). Record append failure = drop event (`kind: "gap"`), playback never interrupted.

**CLI protocol (Principle II):** `clap` derive; stdin/args → stdout (JSON default via `--format json`, `--format human` selectable); errors → stderr; exit 0 on `Ok`, 1 on `Err`. Each crate exposes `pub fn run(args, streams) -> Result<(), CliError>`; `main.rs` forwards locked stdio.

**Closed-set discipline (Principle VII / FR-044):** One declaration site (`engine-core/src/closed.rs`). Every consumer matches exhaustively — no `_ =>` arm in `crates/*/src/` (gate 7 checks via `find` + `awk`, stripping line comments). No `Other`/`Unknown` catch-all; no second list of ladder rungs/engine IDs/tier values. Adding variant = coordinated edit + semver bump + docs update + gate 7 pass.

**Dependency injection / state:** Explicit `Cargo.toml` path dependencies; constructor parameters. `AppState` holds: `PlaybackSession` (`Mutex`), `CpalBackend` (`Option`), `BookmarkStore` (`Mutex`), `FocusTracker` (`Mutex`), `OfferMachine` (`Mutex`), `tauri::ipc::Channel`, permission snapshot (`Mutex<PermissionSnapshot>`), `AppHandle`. No DI container. `BookmarkStore` locked last (after fingerprint/clock/session locks released) — prevents lock cycles; heartbeat order and IPC order (`session` then `store`) mirror this.

**f64 positions:** Live on quarter-second grid (exact binary fractions: 0.0, 0.25, 0.5, 0.75, ...). `Eq` precluded; `PartialEq` exact. Wire form = bare JSON number (`1.5`). Rate ladder rungs serialize as numbers (`1.0` = `R1_0`). Rate change never alters delivered sample rate (`22050 Hz` fixed by engine); player selects rung only.

**Audio / handoff:** Engine delivers mono `f32` @ 22050 Hz (`engine_core/src/format.rs`). Player converts device format ONLY. Rate = pitch-preserving time-stretch inside engine; rate ladder selects only. Mismatch (`rate_mismatch` / `channel_mismatch`) = named `EngineError`, never silent resample/downmix (`FR-033` / `FR-035` / `SC-034`). Pre-roll bounded at 60 s (`MAX_PREROLL_SECS`); position never advances on buffered-only audio (`FR-005`).

**Wire / case mapping (normative, `shell-ipc.md`):** Rust `PascalCase` → JSON `snake_case`. `TransportState` values: `playing` | `paused` | `stopped` | `completed` | `interrupted`. `SessionEnd` (never `playing`): `paused` | `stopped` | `completed` | `interrupted`. `FocusOutcome`: `duckable_transient` | `silence_demanding` | `permanent_loss`; `regained` = direction event. `EngineError`: 7 wire strings (see above). `EngineTier`: `offline_default` / ... / `evaluated_fallback`. `PlaybackRate`: bare number. `ShellVerb`: 15 members (13 transport/session verbs + 2 voice-cycle: `CycleVoiceNext` / `CycleVoicePrev`). `ChooseEngine(id)` = selection (legal from every state, changes no transport); rapid presses honored or explicitly rejected (`PlayerError::IllegalTransition`) — never phantom playing state (`FR-005`).

**Diagnostic record (`diag`):** Append-only JSONL (`writer.rs`); 4 required fields per event (`v`, `ts_ms`, `kind`, `position_secs`); `additionalProperties: false`. Optional `detail` = bound, non-prose `key=value` string (≤ 200 chars, regex `^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+(...)$`). 11 event kinds (`session_start`, `session_end`, `transport_change`, `focus_transition`, `output_device`, `failure`, `refusal`, `trim`, `gap`, `earcon_truncation`, `earcon_emission_failure`). Dual bounds: retention 30 days (`RETENTION_DAYS = 30`), size cap 10 MiB (`SIZE_CAP_BYTES = 10 * 1024 * 1024`). Trim markers (`kind: "trim"`, `trim.dropped_events`, `trim.cutoff_ts_ms`) flushed before retained entries. Gap marker (`kind: "gap"`) on storage-full append — best-effort (`FR-039b`). Read-only; never transmitted (`FR-040`); no in-app surface (`FR-039a`). Position keyed by blake3-256 fingerprint (`FR-036` / `FR-036a`).

**State management / lock discipline:** Every `AppState` field `Mutex<_>`. Every path takes locks alone, releases before next; no nested/combined holds. `BookmarkStore` locked LAST (`session` → fingerprint/clock/session → `store`). Heartbeat: read transport → release → read fingerprint/clock → release → lock store → write → release. Playback watch: same pattern (`feed_audible_progress`, `maybe_republish_progress`, `signal_document_end`). IPC paths (`commands::run_mobile_plugin`, `open_document_bytes`) follow `session`-then-`bookmark_store` order. Poisoned locks = skip + tracing warn, never panic (`FR-039b` / best-effort durability).

**Synthesis / chunking:** `adapter_synthesis` owns chunk boundaries (`chunk::split_document`); engine receives bounded chunks (`FR-024` / `FR-045`). `consume` produces chunks; `format_check` verifies declared mono f32 @ 22050 Hz (`SAMPLE_RATE = 22050`). Capability vocabulary (`SsmlLevel`, `AlignmentGranularity`) declared via `capabilities`; stable across calls. `offline_default` = only selectable tier at v1 (`REGISTERED_TIERS = [OfflineDefault]`); `test_double` admissible only while no non-`test_double` adapter registered (`SC-057` / `SC-051`). `test_double` restricted to `standin_a` / `standin_b` fixtures; never enters production playback; never rendered in diagnostic `engine` field (`Option`, no production code sets it — `rust-player` = backend identifier, not synthesis engine).

**Surface / mirror (`player/src/surface.rs`, `src-tauri/src/surface.rs`):** `MirrorState` = exactly `transport`, `position_secs`, `progress` (determinate: `{position_secs, duration_secs}`; indeterminate: string `"indeterminate"`). `SurfaceSnapshot` = `title` + `section_title` (absent when no heading yet, not `null`, `FR-028c`) + `progress`. `SessionSnapshot` = 4 keys (`transport`, `position_secs`, `rate`, `engine_id`). `ShortReason` / `FullReason` = parallel enums (10 variants, same `snake_case` wire forms): `focus_silence_demanding` (expires `focus_3s`), `focus_permanent_loss` (expires `focus_3s` — supersedes clear resume/duck flags, `FR-014`), `engine_unreachable` / `engine_init_failed` / `engine_rate_mismatch` / `engine_channel_mismatch` / `engine_device_refused` / `engine_device_lost` / `engine_start_refused` (persistent `failure_persistent`), `notification_denied` (persists while denied). Statement language: pt-BR. Focus-loss statement names document title only (`FR-037` / `FR-041`); never quotes body. Transient reasons auto-dismiss at 3 s (`TRANSIENT_DISMISS_MS`); resident pause stays dismissable until user swipe/stop/end; playing = non-dismissable (`FR-028e`).

**Focus / routing (`player/src/focus.rs`, `interruption.rs`, `routing.rs`):** 9 explicit `FocusIntent` arms (`DuckableTransient` → `duckable_transient` + `playing` + duck gain ~20% `DUCK_GAIN` bounds 15–25%, `FR-013` / `ADR-001`; `SilenceDemanding` → `silence_demanding` + `paused`, `FR-013a`; `PermanentLoss` → `permanent_loss` + `paused`, `FR-014`; `FocusRegained` → `regained` direction + resume via rewind; `BecomingNoisy` → `silence_demanding` + `paused` ≤1 s; `DeviceAppeared(id)` → `playing` + same position, never private→speaker without pause-first; `DeviceDisappeared(id)` → `permanent_loss` + `paused`, `FR-016` / `FR-017a`; `AppForegrounded` / `AppBackgrounded` → latch only, no transport change, `FR-032c` / `FR-023a`). Outcome escalation: latest-event-wins supersede (`focus::supersede_flags` — 3 × 3 product, no catch-all). Residency (`shell-ipc.md`): `ResidentStop` (preserves bookmark, `FR-009a`), `SwipeAway` (pauses first under `FR-028e` fallback, preserves bookmark), `DocumentEnd` (tears down residency, deletes bookmark — no resumable position, `FR-032f` / `SC-045` / `SC-049`). Unrecognized lifecycle = `lifecycle_unmapped` (diagnostic), never absorbed.

**Permission gates (`permissions.rs`):** `PermissionSnapshot` (3-member closed set: `granted` / `denied` / `not_requested`, wire `snake_case`). `check_startup` (once at boot): fails closed if `foreground_service_media_playback` is `denied` or `not_requested` → `PermissionFailure::ForegroundServiceMissing` (`foreground_service_missing`); no playing state presented; playback begins only after user opens app (`FR-008` / `FR-010`). `check_pre_play` (before every `Play` / `ExternalStart` / `ResumeAtStored` / `StartOver`): same fail-closed; refusal records named refusal (`refusal` event with `error` field, never phantom playing). Denied notification = NOT a playback blocker; playback starts/stops/pauses fine; surface shows `lock_screen_controls_unavailable` (`FR-021`).

**Engine registry (constitution, `.specify/`):** Admissible engine set is closed. `offline_default`: `sherpa-onnx` + Piper pt-BR (`faber`, `cadu`, `jeff`) — yes (`Principle IX`, offline-first). `cloud_default`: Google Cloud TTS WaveNet — yes (`Principle X`, consent-gated, revocable, never silent default). `cloud_escalation`: Google Neural2 / Chirp 3 HD / Studio; Amazon Polly Neural; Azure Speech Neural — yes, ranked by free quota. `evaluated_fallback`: Microsoft Edge read-aloud (`msedge-tts` 0.4.0) — yes, last, never default, consent-gated, speaks endpoint directly (never via OpenAI-compatible Python wrapper, which discards boundary events). `test_double`: `standin_a` / `standin_b` fixtures ONLY; admissible only while no non-`test_double` adapter registered (`SC-057`); never selectable in production (`EngineTier::is_selectable` filters `TestDouble` when non-`TestDouble` exists). `rust-player` = backend playback identifier (not synthesis engine); never enters fallback order; no `engine_tier`; diagnostic `engine` field `Option` set by no production code.

**Tests / patterns:** `nextest` (not plain `cargo test`); `proptest` for property tests; real `redb` files in temp dirs (never mocks) — rows read back through store's own getters; given-when-then names; `assert_eq!` / `assert_ne!` preferred; deterministic, isolated, full-suite-safe; `tests/contract/` and `tests/integration/` wired via `[[test]]` in owning crate `Cargo.toml` (missing path = broken workspace build). Property tests for closed tables (`prop_transport_math`, `prop_closed_sets`, `prop_rate_ladder`, `prop_resume_boundary`, `prop_bookmark_durable_20s`, etc.). Integration tests (`tests/us1_*`, `us2_*`, `us3_*`). No coverage tooling (`tarpaulin`/`grcov`/`codecov` absent everywhere); no `.github/workflows` — CI = `scripts/verify.sh`.

**Testing & QA**

Test framework: `cargo-nextest` (`.config/nextest.toml`) + `proptest`. Unit tests in `#[cfg(test)]` modules; integration tests in `tests/` (wired via `[[test]]`); property tests (`prop_*.rs`) for closed-set invariants.

Key suites by crate:
- `player`: `prop_transport_math` (rung-shape invariant: rate change never alters delivered handoff at 22050 Hz), `prop_ladder` (ladder order), `prop_skip_clamp` / `prop_skip_wire` / `prop_skip_resident` (skip math), `prop_rewind`, `prop_routing` / `prop_routing_bound`, `prop_focus` / `prop_focus8`, `prop_resume_boundary` / `prop_resume_snap`, `prop_earcon`, `prop_first_audio_3s`, `prop_progress`, `prop_persist`, `prop_preroll_bound`, `prop_idle_drain`, `prop_lifecycle`, `prop_skip`, `prop_surface`, `prop_canonical_guard`, `prop_duck_gain`
- `engine-core`: `prop_closed_sets` (exhaustive match, no catch-all), `prop_fallback` (tier order), `prop_pitch` (pitch preservation within ±50 cents across all 7 rungs), `standin_shape`, `pitch_tolerance`, `test_engine_handoff_us1` (contract: declared mono f32 @ 22050 Hz across transport lifecycle + SC-051 mid-document failover + failure taxonomy)
- `session`: `prop_bookmark_durable_20s` / `prop_bookmark_lifecycle` / `prop_resume` / `prop_offer` / `prop_explicit_resume_survives_expiry` / `prop_fingerprint` / `prop_second_document_preserves_first_bookmark`
- `diag`: `prop_trace` / `prop_record` / `contract_schema` / `contract_trace` / `prop_scanner_canary` / `prop_no_prose` / `contract_schema_negative`
- `adapter-synthesis`: `prop_segment` / `prop_chunk` / `prop_declare` / `prop_format` / `unit_consume` / `audio_chunk`
- `doc-structure`: `prop_headings`

Contract/integration: `tests/contract/` (engine handoff `us1_`, stand-in conformance `us5_`); `tests/integration/` (`us1_*`, `us2_*`, `us3_*` — session recovery, focus duck/pause, routing, lock-screen, failover). `tests/us4_*` = lock-screen / focus-intent contract.

Gate entry (`scripts/verify.sh` — fail-closed, never skips):
1. `format` — `cargo fmt --all -- --check`
2. `tests` — `cargo nextest run --workspace` + two-way target diff (`scripts/expected-tests.txt` vs live list; missing OR unexpected = FAIL)
3. `documentation` — `cargo doc --workspace --no-deps`
4. `lint-drift` — (a) every member manifest has `[lints] workspace = true`; (b) root `[workspace.lints]` equals pinned T001 set (no missing/extra/weakened); (c) `[workspace.dependencies]` intact; (d) `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (fail-closed: moved lint → resolve in code, never weaken level / blanket allow / unreasoned `#[expect]`)
5. `supply-chain` — `cargo deny check advisories licenses bans sources` + `cargo audit`; deny-list scan (`DENY_LIST_PATTERN` — `tts-rs`, `onnx`, `sherpa` (boundary-safe: does NOT match `sherpa-onnx`), `candle`, `burn`, `tch`, `llm`, `llama-cpp-2`, `llama-rs`, `rust-bert`, `openai`, `async-openai`, `openai-api`, `langchain`, `langchain-rust`, `edge-tts-rs`); `sherpa-onnx` / `ort` / `espeak-rs` / `msedge-tts` explicitly NOT banned; `Cargo.lock` must exist (missing = FAIL); `webpki-roots` versions exactly `0.26.11` + `1.0.10` (mechanical bound on RUSTSEC-2026-0381 waiver).
6. `deny-list` — no denied crate in manifest or lockfile (belt-and-braces; `cargo deny check bans` covers lockfile too)
7. `closed-set` — strip line comments (`//`, `///`, `//!`) then check no `_ =>` match arm in `crates/*/src/*.rs`
8. `pipeline` (feature-scoped, split): `gate-8-shell` (PASS: `engine-core` handoff suites + `player::prop_transport_math` — declared mono f32 @ 22050 Hz across rate rungs + transport lifecycle + failure taxonomy); `gate-8-full` (DEFERRED: byte budget < 5000 bytes/request + SSML capability + sentence-level alignment — owner = synthesis / `SC-044`)
9. `android-build` — `cargo tauri android build --target aarch64` (device artifact; runs outside `verify.sh`)

Sub-gates invoked by `verify.sh`: `surface-audit` (`C-H1..C-H9`: hex census → role → accent-only → inventory → aria → movement → contrast → served-identity → short-mix/oracle ≤100ms — deferred per `7909ec0` / T027); `shell-purity` (`check-shell-purity.sh`: 5 sections — forbidden shapes, allowlist enforcement, arithmetic/mutable-state rejection, `SC-057` registry/reference + `test_double` selectability, spec→test traceability (`tasks.md` T-lines)). `check-manifest.sh`: manifest permission declarations (`FOREGROUND_SERVICE*` + `mediaPlayback` service type). `check-apk-classes.sh`: APK class verification. `commit-hook.sh`.

No mock/store mocks: all durable/bookmark/integration tests use real `redb` files (temp dir), read back through store's own `get`. No mock playback/backends: device-independent half (synthesis, fingerprint, totals, offer, rate seed) asserted unconditionally; device-gated half (audio device @ 22050 Hz) responds with named refusal (`Backend::new` refuses on host devices reporting 44.1–48 kHz stereo — same refusal asserted unconditionally, `tests/us*` contract). No tautology tests; delete existing wording/implementation tests on conflict (never re-pin).

No coverage tooling configured; CI = `verify.sh`. No `.github/workflows`. Workspace members: `crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`, `crates/doc-structure`, `crates/adapter-synthesis`, `src-tauri`; `crates/cleanroom-core` standalone (not member).

## Important Files

| File | Role / Note |
|---|---|
| `Cargo.toml` | Workspace manifest; members; `[workspace.lints]` (deny: `unsafe_code`, `missing_docs`, `missing_debug_implementations`, `unreachable_pub`, `unused_results`; `clippy::all`=`deny`, `pedantic`=`deny`, `nursery`/`cargo`=`warn`; `unwrap`/`expect`/`panic`/`todo`/`unimplemented`/`unreachable`/`dbg`/`print_stdout`/`print_stderr`=`deny`); `[patch.crates-io]` (`sherpa-onnx-sys` vendored) |
| `Cargo.lock` | Committed; `--locked` required; never auto-substitute failing crate |
| `rust-toolchain.toml` | Stable channel (`channel = "stable"`, floats, no version pin); components `rustfmt`, `clippy` |
| `clippy.toml` | Pedantic/nursery baseline; no weakenings |
| `deny.toml` | Supply-chain policy (advisories allow RUSTSEC-2024-0370; licenses allow MIT/Apache-2.0/ISC/etc.; deny-list as above; sources crates.io only) |
| `.cargo/config.toml` | `clang` linker on Linux x86_64 |
| `.config/nextest.toml` | `cargo-nextest` CI profile |
| `.specify/memory/constitution.md` | Governance document (principles, engine registry, verification gates, exceptions log); read before design changes |
| `.specify/feature.json` / assessments / templates | Feature tracking / design review records |
| `scripts/verify.sh` | Gate runner 1–8; read to understand fail-closed behavior (missing input = FAIL, never PASS/skip) |
| `scripts/expected-tests.txt` | Pinned nextest target inventory; two-way diff in gate 2 |
| `scripts/check-shell-purity.sh` | Shell-purity gate: verifies Kotlin/Tauri surface has zero playback logic (`SC-035`) |
| `scripts/check-manifest.sh` | Android manifest permission verification (`FOREGROUND_SERVICE*` + `mediaPlayback`) |
| `crates/player/src/lib.rs` | Player crate entry; module list (`surface`, `transport`, `position`, `rate`, `skip`, `rewind`, `focus`, `interruption`, `routing`, `recovery`, `resume`, `earcon`, `backend`, `buffer`) |
| `crates/player/src/cli.rs` | `ShellVerb` (15 members), transition table (5 states × 15 verbs, exhaustive, no catch-all), rate ladder (`LADDER`) |
| `crates/player/src/rate.rs` | Ladder order (`LADDER`), `step_up`/`step_down`, `RateStepper`, `RATE_EFFECTIVE_BOUND_SECS = 2.0` |
| `crates/player/src/transport.rs` | `TransportState`, `Position` (`f64`), `TransportError`, `SkipDirection` (`SkipForward`/`SkipBackward` — ±5.0 s, `FR-018`) |
| `crates/player/src/position.rs` | `PositionClock` (`f64` ≥ 0, quarter-second grid) |
| `crates/player/src/surface.rs` | `MirrorState`, `SurfaceSnapshot`, `SessionSnapshot`, `ShortReason`, `FullReason` (10 variants, wire `snake_case`) |
| `crates/engine-core/src/closed.rs` | Closed-set registry (`PlaybackRate`, `TransportState`, `EngineError`, `SessionEnd`, `EngineTier`, `FocusOutcome`) |
| `crates/engine-core/src/trait_.rs` | `Engine` trait (`set_rate`, `declared_capabilities`, `stream`, etc.) |
| `crates/engine-core/src/format.rs` | Handoff shape: `SAMPLE_RATE = 22050`, `CHANNELS = 1`, `SAMPLE_WIDTH_BITS = 32` (`f32`) |
| `crates/session/src/store.rs` | `redb` `BookmarkStore`; `BOOKMARKS` (key `[u8; 32]`, value `(f64, u64, u8)`); `SETTINGS` (`last_rate`); `DURABLE_CADENCE_MS = 20000`; `StoredPosition`; `OpenLookup`; `clear_bookmark` |
| `crates/session/src/fingerprint.rs` | `blake3`-256 (`BLAKE3_HASH_LEN = 32`); `Fingerprint` |
| `crates/session/src/offer.rs` | `OfferMachine` (resume-offer state machine: open → resolve → expire); `FR-032c` / `FR-032d` forms |
| `crates/diag/src/writer.rs` | `DiagnosticWriter` (append-only, retention 30 d, cap 10 MiB, best-effort); `RecordEvent`; `EventKind` (11 kinds) |
| `crates/diag/src/scan.rs` | `scan_record` (7 violation kinds, oldest-first, exits 0 with violations, non-zero on corrupt/read error) |
| `src-tauri/src/lib.rs` | `AppState` (shared state, `Mutex` fields), `run()` (entry), `commands` (`register_android_plugin`, `update_permissions`, `run_mobile_plugin`), `publish` (`MirrorState` over `tauri::ipc::Channel`), heartbeat (`HEARTBEAT_PERIOD_SECS = 20`), playback watch (`PLAYBACK_WATCH_PERIOD_MS = 500`), `diag_record`, `surface`, `failure`, `status_view`, `permissions` |
| `src-tauri/src/lib.rs:commands` | Command dispatch table (`BridgeAction` classification); `open_document_bytes` (cold-start open: fingerprint → lookup → synthesis → playback); `parse_action` (wire string → `ShellVerb` / `FocusIntent`); `signal_document_end`; `start_parked_reading`; `persist_bookmark`; `delete_bookmark`; `switch_to_document`; `persist_last_rate` |
| `src-tauri/src/surface.rs` | `short_reason` / `full_reason` (10 variants, `snake_case` wire forms, same enum members); `ReasonExpiry` (`focus_3s` vs `failure_persistent`); `SurfaceStatement` |
| `src-tauri/src/permissions.rs` | `PermissionState` (`granted`/`denied`/`not_requested`); `PermissionSnapshot`; `check_startup` / `check_pre_play` gates |
| `src-tauri/src/diag_record.rs` | Diagnostic record wiring (event emission at session lifecycle; `position_secs` keyed by fingerprint) |
| `docs/architecture.md` | High-level shape, crate table, authority split, principle descriptions, engine registry, verification gates |
| `docs/ipc.md` | IPC contract: closed verbs (`ShellVerb` 15 members), state mirror (3 payloads), focus/routing intents (9 arms), residency (3 intents), case mapping table, CLI invocations |
| `docs/diagnostic-record.md` | Record shape (`RecordEvent` 4 required fields, `additionalProperties: false`), 11 event kinds, no-prose invariant (`detail` ≤ 200 chars, regex-bound), writer dual bounds, scanner 7 violations, CLI (`diag scan`) |
| `contracts/shell-ipc.md` | Normative wire strings (verb set, state mirror fields, focus/routing/residency intents, case mapping table, `PermissionSnapshot` structure, reason surfaces) |
| `contracts/diagnostic-trace.md` | Diagnostic record contract (event kinds, pairing invariants, `detail` format, `trim`/`gap` markers, `position_secs` keying) |
| `contracts/resume-algorithm.md` | Resume algorithm (FR-007): sentence snap-back, 5 s cue, ≤30 s / ≤60 s caps, never mid-word, atomic write |

## Runtime / Tooling Preferences

- **Runtime / language:** Rust 2024 edition, stable toolchain (`rust-toolchain.toml`: `channel = "stable"`, no version pin — floats with toolchain bumps; MSRV 1.85, rustc ≥ 1.88 for full build). `clang` linker (`.cargo/config.toml` for `x86_64-unknown-linux-gnu`). No `unsafe` code allowed (`deny`).
- **Android / Tauri:** Tauri 2.12.1 pinned exactly (`Cargo.toml` / `tauri.conf.json`); `jni` 0.21 (0.22 = breaking rewrite); `ndk` 0.9; `ndk-sys` 0.6. Manifest edited directly for media-playback service (`<service android:foregroundServiceType="mediaPlayback">` + `FOREGROUND_SERVICE*` permissions) — Tauri 2 has no `mobile::update_manifest` / `registerService` plugin API (`docs/architecture.md`: verify on every upgrade). Bridge: `register_android_plugin` + `run_mobile_plugin`; constructor `(Activity;)V`; Kotlin `PlaybackService` self-bootstraps from `Application`/`Activity` (`API 34+` FGS-start restrictions apply). Voice asset: `pt_BR-faber-medium` (`assets/voices/pt_BR-faber-medium/` + `espeak-ng-data/`).
- **Package manager:** `cargo`; `--locked` with committed `Cargo.lock`; `cargo deny` + `cargo audit` for supply chain; `vendor/sherpa-onnx-sys` for vendored fix (`PROVENANCE.md`: sha256, diff, exit condition — remove when upstream ships). No secrets in repo/binary (`.env`, `*.pem`, `*.key` gitignored); per-provider revocable consent; Edge endpoint (`msedge-tts`) consent-gated fallback-last (`Principle XI` exception — never default, never via OpenAI-compatible Python wrapper).
- **Test runner:** `cargo-nextest` (not `cargo test`); `proptest` (`1.12.0` pinned in workspace); `tempfile` (`3.14.0`) for temp `redb` files; `jsonschema` (`0.29.0`) for schema validation.
- **Formatter / linter:** `rustfmt` + `clippy` (`all`=`deny`, `pedantic`=`deny`, `nursery`=`warn`, `cargo`=`warn`; `unwrap`=`deny`, `expect`=`deny`, `panic`=`deny`, `unimplemented`=`deny`, `unreachable`=`deny`, `todo`=`deny`, `dbg_macro`=`deny`, `print_stdout`=`deny`, `print_stderr`=`deny`, `allow_attributes`=`deny`, `wildcard_dependencies`=`deny`, `multiple_crate_versions`=`deny`). Suppression: `#[expect(lint, reason = "...")]` only; never weaken `Cargo.toml`/`clippy.toml`; never blanket `#[allow]`. Gate 4 runs `cargo clippy --workspace --all-targets -- -D warnings`; non-zero exit = FAIL (never skip/ignore).
- **Audio / playback:** `cpal` 0.18.2; AAudio. Declared handoff: mono `f32` @ 22050 Hz (`crates/engine-core/src/format.rs`). Backend (`CpalBackend`) opens with device check; host CI devices (44.1–48 kHz stereo) = named refusal (`Backend::new` returns `Err`); same refusal asserted in contract tests (`tests/us*`). Rate stretch = pitch-preserving (`Engine::set_rate`); ladder selects rung. Rate persistence (`FR-019b` / `FR-012`): read-at-start (`BookmarkStore::get_last_rate`) seeds session; write-on-change (`persist_last_rate` called by every rate dispatch that moved rung) stores `last_rate` (`settings` table, key `"last_rate"`); value-blind (no rung value in any shell payload, `FR-012`). `RATE_EFFECTIVE_BOUND_SECS = 2.0` (`player/src/rate.rs`).
- **Synthesis / adapter:** `adapter-synthesis`: `offline_default` (`reg.rs`: `REGISTERED_TIERS = [EngineTier::OfflineDefault]`). `chunk` owns split; `synth` produces bounded chunks; `consume` delivers; `capabilities` declares vocabulary; `format_check` verifies mono f32 @ 22050 Hz; `failure` taxonomy (re-export). `audio_chunk.rs`: chunk metadata (segment boundary, duration, rate, channel, width). `reg.rs`: engine registry mapping (`EngineTier` → adapter ID). `build.rs`: build-time voice asset setup (`espeak-ng-data`, `pt_BR-faber-medium` ONNX + tokens).
- **Storage:** `redb` (embedded key-value); `bookmarks` (`TableDefinition<&[u8; 32], (f64, u64, u8)>`) + `settings` (`TableDefinition<&str, &[u8]>`). Atomic transactions; crash-safe (`FR-004`). `BookmarkStore` (`store.rs`) provides `put`, `get`, `clear_bookmark`, `get_last_rate`, `put_last_rate`, `open` / `close`. Best-effort durability (`FR-039b`): storage failure = `tracing::warn`, never panic; missing input / unopened store / poisoned lock skips with warning; `gap` marker records storage-full append.
- **Supply chain governance (`deny.toml`, `Cargo.toml`):** `crates.io` only; deny-list as above; `deny` + `audit` pass required (gate 5); `--locked`; `patch.crates-io` for vendored `sherpa-onnx-sys`. `Cargo.lock` version of `sherpa-onnx-sys` = patched (path to `vendor/`). `webpki-roots` versions pinned exactly (`0.26.11`, `1.0.10`) for RUSTSEC-2026-0381 waiver. `ort` = 2.0.0-rc.13 (sole pre-recorded exception, `Exceptions Log` entry 4); `sherpa-onnx` = `"=1.13.8"`.
- **Configuration / manifest:** `tauri.conf.json` (product "Ressoa", bundle android `minSdk 29`, `voice` asset path); `AndroidManifest.xml` edited directly (`mediaPlayback` service + `FOREGROUND_SERVICE*` / `FOREGROUND_SERVICE_MEDIA_PLAYBACK` permissions — `check-manifest.sh` verifies). `src-tauri/tauri.android.conf.json` / `tauri.windows.conf.json` for platform config. `Cargo.toml`: workspace `resolver = "3"`; edition implied by `rust-toolchain.toml` (`stable` = 2024 edition when MSRV supports it; workspace uses 2024 features, `clippy::pedantic` denies `unused_results`, etc.).

## Testing Patterns (Quick Reference)

- **Property / invariant:** `proptest!` blocks (`prop_*.rs`); strategies (`rate_strategy`, `transport_state_strategy`) generate from closed sets; assertions on wire-form equality / ladder order / transport lifecycle / closed-set exhaustiveness / fingerprint stability / resume-boundary snap / bookmark durability (`20 s` heartbeat + event-durable writes) / pre-roll bounds (`≤60 s`).
- **Contract / integration:** `tests/contract/` (`test_engine_handoff_us1.rs`, `test_standin_conformance_us5.rs`); `tests/integration/` (`test_session_recovery_us1.rs`, `test_focus_duck_pause_us2.rs`, `test_lockscreen_us4.rs`, `test_routing_us3.rs`, `test_failover_us5.rs`, `test_external_start_us1.rs`). Real `redb` files (`tempfile`); never mocks; read-back through store getters.
- **Unit:** `#[cfg(test)]` modules in each crate (`tests/` root files auto-discover); given-when-then (`fn test_...`); `assert_eq!` / `assert_ne!`; happy + error paths (`Error` variants named, no catch-all).
- **Smoke / CLI:** Every crate binary (`main.rs`) executable with `--format json`; verify by running actual binary (`player play`, `session --db`, `diag scan`, `engine-core capabilities`, `adapter-synthesis chunk`) — not by reading source.
- **Verification command:** `./scripts/verify.sh` (gates 1–8); `cargo tauri android build --target aarch64` (gate 9, outside script). Gate 8 split: `gate-8-shell: PASS` (handoff + rate rungs + failure taxonomy) / `gate-8-full: DEFERRED (owner=synthesis)` (byte budget + SSML + sentence alignment, `SC-044`).
- **No mock playback:** `CpalBackend` requires device @ 22050 Hz mono; host CI devices (44.1–48 kHz stereo) return named refusal (`Err`); same refusal pinned in contract/integration tests. Never mock the refusal — assert the real refusal response.
- **No mock store:** `BookmarkStore` uses real `redb` file; `tests/` open with `tempfile` temp dir; every row read back via `store.get(...)`. `tests/open_document_tests` asserts `open_document_bytes` over real store; `tests/heartbeat_bookmark_tests` asserts durable heartbeat (`20 s`) over real `AppState` with real store; `tests/event_durable_tests` asserts event-durable writes (pause, skip, background, delete) read back through same store.
