# Repository Guidelines

## Project Overview

`mdreader` (product **Ressoa**, package `app.mdreader`) is an Android media-shell TTS audiobook reader. Rust owns synthesis + PCM playback (`cpal` 0.18.2, AAudio) and is the **sole authority** for position/buffering/transport. Kotlin (`android-shell/`) owns only `Service`, `MediaSession` (mirrors, never computes position), notification, and `AudioFocusRequest` — zero playback logic in Kotlin (enforced by check SC-035). The Tauri 2.12.1 app (`src-tauri/`) is one consumer of the Rust crates, never the only one; every crate is CLI-drivable (`clap` derive, stdin/args → stdout, errors → stderr, `--format json`), and IPC (`tauri::ipc::Channel`, no `InvokeBody::Streaming`) calls the same code.

Target: Android minSdk 29, API-29+ device for on-device trials. Host-checkable gates run without a device.

Key principles:
- **Principle I (library-first):** every capability reachable as a Rust API; CLI is a thin `main.rs`.
- **Principle II (one code path):** identical `run()` for CLI and IPC; no second implementation of any rule.
- **Principle V:** no Android API calls in Rust; Kotlin pushes answers (`PermissionSnapshot`, intents) across the boundary.
- **Principle VI:** pipeline degrades observably to declared capabilities; never silently infers SSML/alignment.
- **Principle VII / FR-044 / gate 7:** closed sets everywhere, exhaustive `match`, no `_ =>` arms, no `Other`/`Unknown`; adding a variant = compile error at every consumer until coordinated.
- **Durability cadence (T011/T035):** durable bookmark row on the 20 s heartbeat while `Playing`, and on every event-durable call site (pause, focus-loss, `BecomingNoisy`/`DeviceDisappeared` ≤1 s pause, section-change skip, pre-suspend background write, `Stop`/`DocumentEnd`). Hard kill replays ≤ ~30 s (FR-004).

## Architecture & Data Flow

```
Kotlin surface (Service/MediaSession/notification/focus)
        ↓ intents (wire strings)
Tauri bridge: src-tauri/src/lib.rs (AppState, commands, publish)
        ↓ same Rust code path
crates/player      — transport, position, buffering, pre-roll (≤60 s), rate, rewind/snap math
crates/engine-core — Engine trait, closed-set vocabulary, handoff format (mono f32 @ 22050 Hz), stand-in engines
crates/session     — redb bookmarks + settings, blake3 content fingerprint, resume-offer state machine
crates/diag        — append-only JSONL diagnostic record (app-private, 30-day + 10 MB trim)
crates/doc-structure — ordered headings → timeline positions, re-anchor
crates/adapter-synthesis — offline_default (sherpa-onnx + Piper pt-BR), capabilities, chunking
crates/cleanroom-core — proptest invariant generators (not a workspace member)
```

Audio contract: engine delivers mono `f32` at declared 22050 Hz; engine resamples, Rust converts device format only. Rate/channel/width mismatch = named `EngineError`, never silent resample/downmix. Rate stretch is pitch-preserving (`Engine::set_rate`); player selects the rung only.

Closed sets (FR-044): rate ladder `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`, transport `Playing|Paused|Stopped|Completed|Interrupted`, 7 `EngineError` values, 4 session-end reasons, 5 engine tiers.

State/concurrency: `AppState` holds one playback session, one lazily-opened audio backend, held `FocusTracker` + `OfferMachine` mutexes, and the publish channel. Every state lock taken alone and released before the next — no two state locks held together (lock-cycle discipline mirrored by IPC paths' `session`-then-`bookmark_store` order).

## Key Directories

|Path|Purpose|
|---|---|
|`crates/engine-core`|Closed-set vocabulary, `Engine` trait, handoff format, stand-in engines (`standin_a`/`standin_b`)|
|`crates/player`|Transport, position, buffering, pre-roll, rate, rewind/snap, focus, interruption, earcon, backend (CPAL)|
|`crates/session`|redb bookmarks + settings store, blake3 fingerprint, resume-offer machine|
|`crates/diag`|JSONL writer, closed-set scanners, scan CLI|
|`crates/doc-structure`|`Heading`, `Document` (headings_before / nearest_boundary_at_or_before / reanchor), CLI|
|`crates/adapter-synthesis`|`offline_default` tier (sherpa-onnx 1.13.8 + Piper pt-BR), capabilities, chunking, failure taxonomy|
|`crates/cleanroom-core`|proptest invariant generators (standalone, not a workspace member)|
|`src-tauri`|Tauri bridge: `AppState`, `commands`, `publish`, `diag_record`, `surface`, `failure`, `status_view`, `shell_jni`, `permissions`|
|`android-shell/app/src/main`|Kotlin: `Service`, `MediaSession`, notification, `AudioFocusRequest`|
|`contracts/`|shell-ipc.md, diagnostic-trace.md, resume-algorithm.md, surface.md, earcon.md, ingest.md, storage-schema.md|
|`specs/001-android-media-shell/`|Current feature spec (spec.md, plan.md, research.md, data-model.md, contracts/, checklists/, quickstart.md)|
|`scripts/`|verify.sh (gates 1–8), check-shell-purity.sh, check-manifest.sh, check-apk-classes.sh, commit-hook.sh, expected-tests.txt|
|`tests/`|Workspace integration tests (`us1_*`, `us2_*`, `us3_*`, contract tests), device trial harness|
|`docs/`|architecture.md, ipc.md, diagnostic-record.md, fallback-order.md, byte-budget.md, adr/|
|`.specify/`|Constitution (`memory/constitution.md`, binding), feature.json, assessments, templates|
|`vendor/sherpa-onnx-sys/`|Vendored crate carrying the build-dependency fix (zip 2→9, RUSTSEC-2026-0367/0368/0369); remove when upstream ships|

## Development Commands

Prereqs: Rust stable (edition 2024, MSRV 1.85; rustc ≥ 1.88 for full build), `cargo-nextest`, Android SDK + API-29+ device for gate 9 / on-device. Linker: `clang` for `x86_64-unknown-linux-gnu` (`.cargo/config.toml`).

```sh
# Formatting (gate 1)
cargo fmt --all -- --check
cargo fmt --all

# Tests (gate 2) — cargo-nextest, not plain cargo test
cargo nextest run --workspace
cargo nextest run -p <crate>
cargo nextest run -p player --test prop_transport_math

# Documentation (gate 3)
cargo doc --workspace --no-deps

# Lint / rigour (gate 4 — must exit 0)
cargo clippy --workspace --all-targets -- -D warnings

# Supply chain (gate 5)
cargo deny check advisories licenses bans sources
cargo audit

# Gate runner (gates 1–8)
./scripts/verify.sh

# Gate 9 — device artifact (runs outside verify.sh)
cargo tauri android build --target aarch64

# Full build
cargo build --workspace
cargo build --release --workspace

# Check without building
cargo check --workspace

# Single crate
cargo build -p player
cargo test -p engine-core
```

Workspace members: `crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`, `crates/doc-structure`, `crates/adapter-synthesis`, `src-tauri`. (`crates/cleanroom-core` is standalone.) `--locked` with committed `Cargo.lock`; never auto-substitute a failing crate.

Test-target wiring: `tests/contract/` and `tests/integration/` targets are **not** auto-discovered — wire them with `[[test]]` in the owning crate's `Cargo.toml` (see `player`, `engine-core`); a missing path breaks every workspace build. `tests/` root files auto-discover in their own crates.

## Code Conventions & Common Patterns

**Rust rigour (constitution "Rust Code Rigor"):**
- `unsafe`, `missing_docs`, `missing_debug_implementations`, `unreachable_pub`, `unused_results` = `deny`.
- `clippy::all`, `clippy::pedantic` = `deny`; `nursery`, `cargo` = `warn`.
- `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg`, `println`/`print_stdout`/`print_stderr` = `deny`.
- Only suppression allowed: `#[expect(lint, reason = "...")]` with a written reason. Never weaken `Cargo.toml`/`clippy.toml` to silence a lint; never blanket `#[allow]`.
- Every workspace member declares `[lints] workspace = true`.

**Errors:** `thiserror` + `tracing`; named `Error` enums (`#[derive(Error, Debug)]`), `Result<T, E>`, `?` propagation, `anyhow::Context` where useful. Failures/refusals record named `error` from the closed seven-value engine taxonomy. Never silence, never substituted format, never a playing state on refused start. Best-effort durability (storage failure = tracing warn, never panic; record append failure = drop event, never interrupt playback).

**CLI (Principle II):** `clap` derive; stdin/args → stdout; errors → stderr; `--format json` default for machine-readable output; error enum variants named, no catch-all. Each crate lib exposes `pub fn run(args, streams) -> Result<(), CliError>`; `main.rs` forwards locked stdio, exits 0 on `Ok`, 1 on `Err`. No `unwrap`/`expect`/panic anywhere in library crates.

**Closed sets:** exhaustive `match` with explicit arms; adding a variant = coordinated edit across all consumers + semver bump + docs. No second list of ladder rungs/engine IDs/tier values (engine IDs live only in the constitution registry).

**Audio / handoff:** mono `f32` @ 22050 Hz; engine owns pitch-preserving stretch; player selects rung; mismatch → named `EngineError`.

**Imports:** group with blank lines between std, external, local crates; explicit imports; sort alphabetically within groups; prefer `use crate::` for local imports.

**Formatting:** 4-space indentation (no tabs); line length 100 soft / 120 hard; always `cargo fmt`; trailing commas in multi-line lists/enums; derive `Debug, Clone, Serialize, Deserialize` for structs when appropriate.

**Naming:** Structs/Enums/Traits `PascalCase`; functions/variables `snake_case`; constants `UPPER_SNAKE_CASE`; modules `snake_case`; test functions descriptive with underscores; private helpers prefix `_` if unused; generic types single uppercase letters (`T`, `U`, `E`, `I`).

**Types:** explicit type annotations for public APIs; `f64` for floating-point; `i32`/`i64` for whole numbers, `u32`/`u64` when non-negative guaranteed; `String` owned, `&str` references; `Option<T>`/`Result<T, E>` appropriately; custom error types with `thiserror`; `Vec<T>` collections, `HashMap<K, V>` key-value; `Box<T>` heap, `Rc<T>`/`Arc<T>` shared ownership.

**Async / runtime:** `tokio` (features `rt`, `time`) only in `src-tauri`. No `async`/`await` in library crates. Player audio uses blocking `cpal` output-stream callback; position clock advances only on audible progress from the callback. The Tauri bridge (`run()`) is the async boundary (`tauri::mobile_entry_point` on Android); spawns `playback_watch_task` and `offer_window_task` holding owned `AppHandle`s, each tick takes every state lock alone — no two locks held together.

**State management / DI:** single shared `AppState` behind `tauri::manage`, every field `Mutex<_>`; each lock taken alone and released before the next; redb `BookmarkStore` locked last (prevents lock cycles). Position authority: `PositionClock` (single `f64` ≥ 0 + `stored_basis_secs`) inside `PlaybackSession` inside `AppState.session`; advances only on audible progress, never on its own, never during pre-roll. No DI container; dependencies are explicit `Cargo.toml` path dependencies and constructor parameters. Injection points: `engine_core::fallback::consent_from(store, registry, engine_id)`, `RateStepper` + `BookmarkStore::get_last_rate()/put_last_rate()` composed by the shell, `session::store::lookup_on_open(store, bytes)`.

**f64 positions:** live on the quarter-second grid (exact binary fractions); `Eq` precluded by `f64` fields, `PartialEq` exact.

**Wire forms:** `snake_case` variant names via `#[serde(rename_all = "snake_case")]` or per-variant `#[serde(rename = "...")]`; `PlaybackRate` serializes as bare JSON f64 number (exact binary fractions). All `Display` impls return the wire string so diagnostics can't drift.

## Important Files

|File|Role|
|---|---|
|`Cargo.toml`|Workspace manifest; members, workspace deps, `[workspace.lints]` rigour set, `patch.crates-io` (vendored `sherpa-onnx-sys`)|
|`Cargo.lock`|Committed lockfile (`--locked` required)|
|`rust-toolchain.toml`|`channel = "stable"` (floats, no version pin); components `rustfmt`, `clippy`|
|`clippy.toml`|Pedantic/nursery baseline; no weakenings|
|`deny.toml`|Supply-chain policy: advisories (one ignore: RUSTSEC-2024-0370), deny-list, license allow-list, sources crates.io only|
|`.cargo/config.toml`|`clang` linker on Linux x86_64|
|`.config/nextest.toml`|`cargo-nextest` CI profile|
|`scripts/verify.sh`|Gates 1–8 runner; fails closed; `gate-8-full` DEFERRED to synthesis feature|
|`scripts/expected-tests.txt`|Pinned nextest target list (two-way diff vs live in gate 2)|
|`crates/engine-core/src/closed.rs`|Closed-set registry (`PlaybackRate`, `TransportState`, `EngineError`, `EngineTier`)|
|`crates/player/src/rate.rs`|Ladder order `LADDER`, `step_up`/`step_down`, `RateStepper`, `RATE_EFFECTIVE_BOUND_SECS = 2.0`|
|`crates/player/src/transport.rs`|`Position`, `TransportError`, `SkipDirection`, `select_rate`|
|`crates/session/src/store.rs`|redb `bookmarks` + `settings` tables; `BOOKMARKS`, `clear_bookmark`|
|`crates/session/src/fingerprint.rs`|blake3-256 content fingerprint + FR-038 re-anchor|
|`crates/session/src/offer.rs`|`OfferMachine` resume-offer state machine|
|`crates/diag/src/writer.rs`|JSONL writer; `ErrorValue` projection of the seven engine wire values|
|`crates/diag/src/scan.rs`|7 closed violations incl. `ProseInDetail`|
|`src-tauri/src/lib.rs`|`AppState`, `HEARTBEAT_PERIOD_SECS = 20`, `PLAYBACK_WATCH_PERIOD_MS = 500`, `commands`, `publish`, heartbeat/event-durable writes, `end_kind_of_pause`/`event_end_kind`/`outgoing_end_kind`|
|`src-tauri/tauri.conf.json`|Product "Ressoa", bundle android minSdk 29, voice asset `pt_BR-faber-medium`|
|`contracts/shell-ipc.md`|Closed verb set, state mirror, focus/routing/residency intents, case mapping|
|`contracts/resume-algorithm.md`|FR-007 resume algorithm (sentence snap-back, 5 s cue, ≤30 s/≤60 s caps, never mid-word, atomic write)|

## Runtime / Tooling Preferences

- **Runtime / language:** Rust 2024 edition, stable toolchain. No Node/Bun tooling is part of the product build (`.opencode/` and lint/config ignores are scaffolding only).
- **Package manager:** `cargo`; `--locked` with committed `Cargo.lock`; `cargo deny` + `cargo audit` for supply chain.
- **Test runner:** `cargo-nextest` (not plain `cargo test` for gates).
- **Formatter / linter:** `rustfmt` + `clippy` (all/pedantic deny, nursery/cargo warn), invoked via gate 4 (`-D warnings`).
- **Android tooling:** Android SDK, API-29+ device, `cargo tauri android build --target aarch64`.
- **Audio:** CPAL 0.18.2, AAudio, declared 22050 Hz mono f32.
- **Tauri:** `2.12.1` pinned exactly (3.0.0-alpha rejected); `jni` 0.21 (0.22 is a breaking rewrite); `ndk` 0.9; `ndk-sys` 0.6. Manifest edited directly (`AndroidManifest.xml`/Gradle) for `<service android:foregroundServiceType="mediaPlayback">` + `FOREGROUND_SERVICE*` permissions — Tauri 2 has no `mobile::update_manifest`/`registerService` plugin API; re-verify on every Tauri upgrade. Bridge: `register_android_plugin` + `run_mobile_plugin`, constructor `(Activity;)V`; Service self-bootstraps from Application/Activity (API 34+ FGS-start restrictions apply).
- **Supply chain:** latest stable only (exceptions need version + reason + risk in commit message); `--locked` + committed `Cargo.lock`; reject advisories, non-crates.io sources, yanked versions; never auto-substitute a failing crate. `deny.toml` deny-list: `tts-rs`, `onnx`, `sherpa`, `candle`, `burn`, `tch`, `llm`, `llama-cpp-2`, `llama-rs`, `rust-bert`, `openai`, `async-openai`, `openai-api`, `langchain`, `langchain-rust`. `sherpa-onnx`, `ort`, `espeak-rs`, `msedge-tts` are explicitly NOT banned. `sherpa-onnx = "=1.13.8"` patched via `vendor/sherpa-onnx-sys`. `ort` 2.0.0-rc.13 is the sole pre-recorded stability exception. No secrets in repo/binary (`.env`, `*.pem`, `*.key` gitignored); per-provider revocable consent; Edge endpoint is consent-gated fallback-last, never default, never via the OpenAI-compatible Python wrapper.

## Testing & QA

Framework: `nextest` + `proptest`; real `redb` files in temp dirs read back through the store's own getters — **never mocks**; `#[cfg(test)]` modules plus wired `[[test]]` targets for `tests/contract/`/`tests/integration/`. Property tests for closed tables (transport transitions, rate ladder, surface inventory). Fail-closed gate inputs: missing `scripts/expected-tests.txt` or missing auditee = FAIL, never skip.

Test conventions: given-when-then names; test both happy path and error cases; prefer `assert_eq!`/`assert_ne!`; deterministic and isolated; full-suite-safe.

Do **not** test wiring/copies/forwarding/mock echoes/source text, tautologies, bare not-throw, non-empty/length-grew, or duplicate same-path rows. Delete existing wording/implementation/incidental-behavior tests when they conflict — never re-pin.

No coverage tooling configured anywhere (`tarpaulin`, `grcov`, `codecov`, `cargo-cov` absent from every `Cargo.toml`; no `coverage` section in `deny.toml`). No CI config files (no `.github`/workflows) — CI is `scripts/verify.sh`.

On-device trials (10/10): lock-screen, focus, routing, ≤3 s play-to-audio, failover.

Surface/audit checks (C-H1..C-H9): hex census, role/accent-only, inventory, aria, movement freeze, contrast, served-identity, short-mix/oracle ≤100 ms.
