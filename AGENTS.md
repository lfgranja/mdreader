# Repository Guidelines

## Project Overview

`mdreader` (product **Ressoa**, package `app.mdreader`) is an Android media-shell TTS audiobook reader. Rust owns synthesis and PCM playback as the **sole playback authority**; Kotlin (`android-shell/`) owns `Service`, `MediaSession`, notification, and `AudioFocusRequest` as **mirrors only**. The Tauri 2.12.1 bridge (`src-tauri/`) is one consumer among many. Every crate is CLI-drivable (`clap` derive, stdin/args → stdout, errors → stderr, `--format json`). Target: Android `minSdk 29`. Host-checkable gates 1–8 via `scripts/verify.sh`, device gate 9 via `cargo tauri android build --target aarch64`.

Key principles:
- **Library-first:** Every capability is reachable as a Rust API; CLI is thin.
- **One code path:** Identical behavior for CLI and IPC (`tauri::ipc::Channel`, no `InvokeBody::Streaming`).
- **Zero Android API in Rust:** Kotlin pushes answers across the boundary.
- **Pipeline degrades observably:** Never silently infers SSML/alignment; uses declared capabilities.
- **Closed sets everywhere:** Exhaustive `match`, no `_ =>` arms, no `Other`/`Unknown`; adding variants requires coordinated edits + docs + semver bump.
- **Durability:** 20s heartbeat while `Playing` + event-durable writes; hard kill replays ≤ ~30s.
- **Position advances only on audible progress:** Never during pre-roll, never on timer.

## Architecture & Data Flow

### Authority Split
- **Rust = sole playback authority** (`crates/*`). Position, transport, rate selection, focus/routing/interruption logic live in Rust.
- **Kotlin/Android shell (`android-shell/`) = mirror only.** Renders published JSON verbatim (no position/progress/haptics computation). Components: `PlaybackService`, `MediaMirror`, `FocusMirror`, `RoutingMirror`, `Residency`, `MediaSession`, notification.
- **Tauri bridge (`src-tauri/`)** connects frontend/Kotlin/JNI to Rust; publishes mirror state via channel, builds Kotlin mirror JSON, holds `AppState`.

### Pipeline
```
Kotlin surface (Service/MediaSession/notification/focus) — mirror only
        ↓ intents (wire strings)
Tauri bridge: src-tauri/src/lib.rs (AppState, commands, publish, heartbeat)
        ↓ same Rust code path
crates/engine-core   — Engine trait, closed vocabulary (closed.rs), handoff format (mono f32 @ 22050 Hz), stand-ins
        ↓ (declared capabilities/caps)
crates/adapter-synthesis — offline_default (sherpa-onnx/Piper pt-BR), chunking/synth, capabilities, format_check, failure taxonomy, registry
        ↓
crates/player        — transport, PositionClock (audible-progress only), buffering/pre-roll (≤60 s), rate ladder (pitch-preserving in engine), skip/rewind, focus/routing/interruption, backend (cpal), surface/mirror, earcons
        ↓
crates/session       — redb bookmarks + settings (blake3-256 content fingerprint), offer/resume state machine
crates/diag          — append-only JSONL diagnostics (30-day/10 MiB, 11 kinds, 7 violation types)
crates/doc-structure — ordered headings → timeline/FR-038 re-anchor
```

### State, Locks & DI
- **Single `AppState`** behind `tauri::manage` in `src-tauri`; fields are `Mutex<_>`. Take/release locks alone, no nested/combined holds.
- **Lock order:** follows IPC path (`session` → `bookmark_store` last); heartbeat/read order mirrors to avoid cycles.
- **Poisoned locks:** skipped with `tracing::warn` (best-effort), never panic.
- **DI:** explicit path deps in `Cargo.toml`, constructor injection. No DI container.
- **Position clock:** `crates/player/src/position.rs` — `PositionClock` advances **only** via `on_audible_progress(elapsed_secs)`. Basis updated only on audible progress; `rewind_for_silence` moves cursor back by `min(REWIND_FRACTION × paused, REWIND_CAP_SECS)` clamped to 0 without moving basis; `snap_to_boundary` caps at `stored_basis_secs - REWIND_CAP_SECS`. Skips add exact `SKIP_STEP_SECS` and reset basis.

### Audio & Handoff
- **Format:** Mono `f32` @ 22050 Hz (`crates/engine-core/src/format.rs`, `DeclaredFormat::SAMPLE_RATE = 22050`). Mismatches produce named `EngineError`s (never silent resample/downmix).
- **Engine contract:** `Engine` (`crates/engine-core/src/trait_.rs`) requires `declared_capabilities()`, `set_rate()` (pitch-preserving stretch inside engine; rung selection only), `stream(position_secs, max_secs)`. `stream` returns buffered-only chunks (reading never advances position). Delivered sample rate must remain declared on all rungs; pitch deviation beyond `PITCH_TOLERANCE_CENTS` → `EngineError::RateMismatch`.
- **Backend:** `cpal` 0.18.2 (AAudio on Android). Host CI devices (44.1–48 kHz stereo) return named refusal in `CpalBackend` (asserted in contracts). Pre-roll ≤ 60 s.

### Wire Format
- Rust `PascalCase` → JSON `snake_case` (e.g. `TransportState::Playing` → `playing`, `PlaybackRate` → bare number like `1.5`, `EngineTier::OfflineDefault` → `offline_default`). `Display` returns wire strings.

## Key Directories

| Path | Purpose |
|---|---|
| `crates/engine-core/` | Closed sets, `Engine` trait, handoff format (22050 Hz mono f32), fallback/order, stand-ins (`standin_a`/`standin_b`) |
| `crates/player/` | Transport/position/rate/skip/rewind/focus/interruption/routing/recovery/resume/earcon/backend/buffer/surface; `PositionClock` (audible-progress only) |
| `crates/session/` | `redb` bookmarks/settings, blake3-256 fingerprint, offer/resume state machine (`BOOKMARKS`, `StoredPosition`) |
| `crates/diag/` | Append-only JSONL diagnostics (30-day/10 MiB, 11 kinds, 7 violation types), writer/scanner/retention |
| `crates/doc-structure/` | Headings/timeline, normalization, `reanchor` (FR-038) |
| `crates/adapter-synthesis/` | Offline adapter, chunk/synth, capabilities, format check, failure taxonomy, registry (`offline_default` only at v1), voice assets in `assets/` |
| `crates/cleanroom-core/` | Standalone proptest helpers (not a workspace member) |
| `src-tauri/` | Tauri bridge (`AppState`, commands, publish, heartbeat/playback watch), permissions/surface/failure/diag wiring, `shell_jni.rs` |
| `android-shell/` | Kotlin mirror (Service/MediaSession/notification/focus) — no playback logic |
| `vendor/sherpa-onnx-sys/` | Vendored patch for `zip` (RUSTSEC-2026-0367/0368/0369); remove when upstream fixes |
| `scripts/` | Gates, purity checks, manifest/APK checks, commit hook (`verify.sh`, `check-shell-purity.sh`, `check-manifest.sh`, `check-apk-classes.sh`, `expected-tests.txt`) |
| `contracts/` | Normative IPC/surface/earcon/ingest/storage/resume/diagnostic contracts (`*.md`, `*.json`) |
| `docs/` | Architecture, IPC, diagnostics, fallback order, byte budget, ADRs |
| `specs/` | Feature specs/tasks/checklists/contracts (001-android-media-shell onward) |
| `tests/` | Root integration/contract/UI/audit tests (many artifact/filesystem assertions); includes `005-red/`, `005-adopt-design-sona-red/`, `contract/` |

## Development Commands

```sh
# Format (gate 1)
cargo fmt --all -- --check
cargo fmt --all

# Tests (gate 2) — use cargo-nextest
cargo nextest run --workspace
cargo nextest run -p player --test prop_transport_math
cargo nextest run -p engine-core --test prop_closed_sets

# Docs (gate 3)
cargo doc --workspace --no-deps

# Lint (gate 4) — must be clean
cargo clippy --workspace --all-targets -- -D warnings

# Supply chain (gate 5)
cargo deny check advisories licenses bans sources
cargo audit

# All gates 1-8 (fail-closed)
./scripts/verify.sh

# Android artifact (gate 9, outside verify.sh)
cargo tauri android build --target aarch64
# or debug for APK class checks
cargo tauri android build --target aarch64 --debug

# Build/check
cargo build --workspace
cargo build --release --workspace --locked
cargo check -p player
cargo check --target x86_64-unknown-linux-gnu  # clang linker per .cargo/config.toml

# CLI examples (same code path as IPC)
player play
player rate-up
player skip-back
session --db s.db get doc.txt
session --db s.db put doc.txt --offset-secs 12.5 --first-yield-ms 100 --end-kind paused
engine-core capabilities
engine-core stream --at 12.5 --budget-secs 5.0
diag scan record.jsonl
adapter-synthesis chunk --input doc.md
```

## Code Conventions & Common Patterns

### Formatting & Imports
- **Indent:** 4 spaces, no tabs. Line length: 100 soft / 120 hard.
- **Always format:** `cargo fmt` (gate 1 enforced).
- **Imports:** Group std / external / local, alphabetical within groups, prefer `use crate::`.
- **Derives:** `Debug, Clone, Serialize, Deserialize` where appropriate.

### Naming
- Structs/enums/traits: `PascalCase`
- Functions/variables/modules: `snake_case`
- Constants: `UPPER_SNAKE_CASE`
- Test functions: descriptive with underscores (given-when-then style)
- Private helpers: prefix `_` if unused
- Generics: single uppercase letters (`T`, `U`, `E`, `I`)

### Types & Error Handling
- Prefer `f64` for positions; `String` owned / `&str` borrowed; `Option<T>`/`Result<T, E>` everywhere.
- Custom errors with `thiserror` (`#[derive(Error, Debug)]`), propagate with `?`, use `anyhow::Context` where helpful.
- `Eq` precluded by `f64`; use `PartialEq` carefully.
- **Forbidden:** `unwrap`/`expect`/`panic`/`todo`/`dbg`/`println`/`print_stdout`/`print_stderr` (workspace-denied). Suppressions only via `#[expect(lint, reason = "...")]`.

### Async, Runtime & Backend
- `tokio` only in `src-tauri` (`rt`, `time`). Library crates are synchronous (no `async`/`await`).
- `cpal` uses blocking output-stream callbacks; position advances only on audible progress.
- JNI 0.21 (compat with Tauri 2.12.1).

### CLI Protocol (Principle II)
- `clap` derive; stdin/args → stdout (JSON default, `--format human` optional); errors → stderr; exit 0 on `Ok`, 1 on `Err`.
- Each crate exposes `pub fn run(args, streams) -> Result<(), CliError>`; `main.rs` forwards locked stdio.

### Closed Sets (Principle VII)
- One declaration site in `crates/engine-core/src/closed.rs` (e.g. `TransportState`, `SessionEnd`, `EngineError`).
- Every consumer must match exhaustively — **no `_ =>` arms in `crates/*/src/`** (gate 7 enforced).
- No `Other`/`Unknown` catch-alls. Adding variants requires coordinated edits + docs + semver bump + gate 7 passing.
- Shell purity enforced: no transport/position/rate/gain/duck/skip vocabulary in mutable/arithmetic contexts outside allowlists (`scripts/check-shell-purity.sh`).

### State Management & Patterns
- Mutex fields in `AppState`; locks taken alone, released before next; `BookmarkStore` locked last.
- Rust-only logic, Kotlin/JNI as transport/mirror.
- Content-addressed bookmarks via blake3-256 fingerprint (`crates/session/src/fingerprint.rs`).
- Diagnostics are append-only JSONL with 4 required fields (`v`, `ts_ms`, `kind`, `position_secs`), `additionalProperties: false`, `detail` bound to non-prose `key=value` (≤200 chars).

## Important Files

| File | Role |
|---|---|
| `Cargo.toml` | Workspace members/deps, `[workspace.lints]` (strict deny/warn), `[patch.crates-io]` for vendored `sherpa-onnx-sys` |
| `Cargo.lock` | Committed; use `--locked` |
| `rust-toolchain.toml` | Stable channel + `rustfmt`, `clippy` |
| `.cargo/config.toml` | Clang linker on `x86_64-unknown-linux-gnu` |
| `clippy.toml` | Lint baseline marker |
| `deny.toml` | Supply chain policy (advisories/bans/licenses/sources); deny-list includes TTS/LLM crates except `sherpa-onnx`, `ort`, `espeak-rs`, `msedge-tts`; ignores `RUSTSEC-2024-0370` with justification |
| `.config/nextest.toml` | `cargo-nextest` CI profile (`threads-required=1`, `failure-output=final`, `leaks-request=forbid`) |
| `src-tauri/tauri.conf.json` | Tauri config (product `Ressoa`, identifier `app.mdreader`, v0.1.0, Android `minSdkVersion 29`, `distDir ../dist`, `devUrl http://localhost:1420`, voices resources) |
| `src-tauri/Cargo.toml` | Tauri 2.12.1 (exact), tauri-build 2.7.2 (exact), workspace deps, edition 2024 |
| `src-tauri/src/lib.rs` | Bridge entry, `AppState`, commands, publish, heartbeat/playback watch |
| `crates/engine-core/src/closed.rs` | Canonical closed vocabulary |
| `crates/engine-core/src/format.rs` | Handoff shape (22050 Hz mono f32) |
| `crates/engine-core/src/trait_.rs` | `Engine` trait and error/capability contracts |
| `crates/player/src/lib.rs` / `crates/player/src/cli.rs` | Playback authority and `ShellVerb`/transition logic |
| `crates/player/src/position.rs` | Audible-progress-only `PositionClock`, rewind/snap rules |
| `crates/session/src/store.rs` / `crates/session/src/fingerprint.rs` | `redb` store and blake3-256 fingerprinting |
| `crates/diag/src/writer.rs` / `crates/diag/src/scan.rs` | Diagnostic writer/scanner/retention |
| `crates/adapter-synthesis/src/reg.rs` | Tier registry (`offline_default` only at v1) |
| `scripts/verify.sh` | Gates 1-8 orchestrator (fail-closed) |
| `scripts/expected-tests.txt` | Pinned nextest target list for gate 2 two-way diff |
| `scripts/check-shell-purity.sh` | Enforces mirror-only shell + traceability (SC-035/FR-042) |
| `scripts/check-manifest.sh` | Android manifest checks (`mediaPlayback`, FGS permissions, minSdk 29) |
| `scripts/check-apk-classes.sh` | APK class presence verification |
| `docs/architecture.md`, `docs/ipc.md`, `docs/fallback-order.md`, `docs/diagnostic-record.md` | Core architecture/contracts |
| `contracts/*.md`, `contracts/*.json` | Normative contracts (shell IPC, surface, earcon, ingest, storage, resume, diagnostics) |

## Runtime/Tooling Preferences

- **Language/toolchain:** Rust 2024 edition (workspace resolver v3), stable toolchain (`rust-toolchain.toml` with `rustfmt`, `clippy`), MSRV policy as recorded. Clang linker on Linux x86_64 (`.cargo/config.toml`).
- **Build/orchestration:** Cargo with `--locked` (committed `Cargo.lock`). Tauri 2.12.1 pinned exactly; `tauri-build` 2.7.2 pinned exactly.
- **Android:** `minSdk 29`, manifest edited directly for `mediaPlayback` foreground service + `FOREGROUND_SERVICE*` permissions. JNI 0.21. Voice assets in `crates/adapter-synthesis/assets/`. No `android-shell/build.gradle*` present in repo root scan.
- **Testing:** `cargo-nextest` (not plain `cargo test`). `proptest` 1.12.0, `tempfile` 3.14.0, `jsonschema` 0.29.0. Tests split across crate `tests/` and root `tests/`; many use real `redb` files/temp dirs (no mocks). Property tests checked in with `.proptest-regressions`. Root tests often assert on artifact/filesystem state.
- **Lint/format/supply chain:** `rustfmt` + `clippy` with strict deny set (gate 4 fails closed). `cargo-deny` + `cargo audit` (gate 5). Deny-list enforced (gate 6). Vendored `sherpa-onnx-sys` via `[patch.crates-io]` with removal condition when upstream fixes RUSTSEC-2026-0367/0368/0369.
- **Audio:** `cpal` 0.18.2, AAudio on Android. Host CI devices (44.1–48 kHz stereo) return named refusal in `CpalBackend`.
- **Workspace lints (enforced):** `unsafe_code = deny`, `missing_docs = deny`, `missing_debug_implementations = deny`, `unreachable_pub = deny`, `unused_results = deny`. Clippy: `all/pedantic = deny`, `nursery/cargo = warn (-1)`, and denies `unwrap_used`, `expect_used`, `panic`, `unimplemented`, `unreachable`, `todo`, `dbg_macro`, `print_stdout`, `print_stderr`, `allow_attributes`, `allow_attributes_without_reason`, `wildcard_dependencies`, `multiple_crate_versions`.

## Testing & QA

### Frameworks & Locations
| Area | What lives there | Notes |
|---|---|---|
| `crates/*/src/**` `#[cfg(test)]` | Inline unit tests | Co-located, shape/CLI/protocol checks. |
| `crates/session/tests/` | Session integration tests | `test_ingest_cli.rs`, `test_offer3s.rs`, many `prop_*` targets. |
| `crates/player/tests/` | Player integration/contract/feature tests | `tests/integration/` (recovery/failover/focus/routing/lockscreen/external start), `tests/contract/` (focus intents/removal signals/shell IPC), plus latency/progress/earcon/setup tests. |
| `crates/engine-core/tests/contract/` | Engine handoff/stand-in conformance | `test_engine_handoff_us1.rs`, `test_standin_conformance_us5.rs`. |
| `crates/cleanroom-core/tests/` | Property/validation tests | `slice_readability_proptest.rs` (BLOCKING invariants). |
| `src-tauri/tests/` | Bridge/integration/diagnostic tests | Publish/payload, noscope/purity, resume/restarts audio, play-to-audio measured, in-app play functional, offer/lockscreen, buffer state in diag, etc. |
| `tests/005-red/`, `tests/005-adopt-design-sona-red/` | Spec/design red/gate tests | Feature-gated (census/ARIA/logo/motion/latency/property contracts). |
| `tests/contract/` | Contract docs/tests | e.g. `test_focus_intents_us2_contract.md`. |
| `specs/**/verification/` | Attestations/protocols | Supporting evidence (not always `#[test]` harnesses). |

### Patterns
- **Unit/Integration/Contract/Property:** Co-located unit tests, cross-component integration via authoritative types (`PlaybackSession`, `PositionClock`), contract tests between layers, property tests with `proptest!` and explicit case counts/grid strategies.
- **Fixtures/replay:** Embedded via `include_str!` (e.g. `service_kill_power_loss.jsonl`) and `specs/.../tests/device/fixtures/`. Deterministic JSONL replay through Rust authority.
- **Temp/redb:** Tests use real `redb` files/temp dirs (no mocks). Session/recovery tests read/write bookmarks via `session::BOOKMARKS`/`store`.
- **Guard/absence tests:** Fail-closed scans (e.g. `src-tauri/tests/test_noscope.rs` enforces no network crates/call sites app-wide, no external `http://`/`https://` in served HTML surfaces with W3C namespace allowlist, manifest capability checks).
- **Traceability:** Tests reference FR/SC/task/spec anchors; BLOCKING invariants are explicitly marked.
- **Exactness:** Contract-critical values assert equality (e.g. rewind formula `min(REWIND_FRACTION × paused, REWIND_CAP_SECS)`, bounds ≤30s for kill/recovery).

### Running & Gates
- **Runner:** `cargo nextest` with CI profile (`.config/nextest.toml`: `threads-required=1`, `failure-output=final`, `leaks-request=forbid`).
- **Target naming:** Namespaced by crate/module and feature/US/task (e.g. `player::*`, `engine-core::*`, `src-tauri::*`, `*::prop_*`, `*::us1`).
- **Gate 2:** Must match `scripts/expected-tests.txt` exactly (two-way diff of live `cargo nextest list --workspace` vs pinned list). Includes unit/property/contract/integration/schema targets.
- **Full gate sequence (`scripts/verify.sh`, fail-closed):**
  1. **format** — `cargo fmt --all -- --check`
  2. **tests** — `cargo nextest run --workspace` + two-way diff vs `scripts/expected-tests.txt`
  3. **surface-audit** — `cargo nextest run --workspace -E 'test(surface_audit)'` and `-E 'test(earcon)'` (C-H1..C-H9)
  4. **documentation** — `cargo doc --workspace --no-deps`
  5. **lint-drift** — manifest `[lints] workspace = true`, root `[workspace.lints]` matches pinned rigour set exactly, `[workspace.dependencies]` exists, `cargo clippy --workspace --all-targets -- -D warnings`
  6. **supply-chain** — `cargo deny check advisories licenses bans sources` + `cargo audit` (`webpki-roots` waivers enforced)
  7. **deny-list** — scan manifests/`Cargo.lock` for denied crates (name-boundary match); allows `sherpa-onnx`, `ort`, `msedge-tts`
  8. **closed-set** — no catch-all ` _ =>` match arm in `crates/*/src/*.rs` (comments stripped)
  9. **pipeline** — runs shell subset (`engine-core` handoff/standin/prop_pitch + `player` prop_transport_math), prints `gate-8-shell: PASS`, `gate-8-full: DEFERRED (owner=synthesis)` (full gate 8 deferred per FR-024/Exceptions Log)
  10. `scripts/check-manifest.sh` (minSdk 29, `mediaPlayback`, FGS permissions, no INTERNET)
  11. `scripts/check-shell-purity.sh` (mirror-only, vocabulary allowlists, no `Other`/`Unknown`, engine-id allowlist, traceability FR/SC in `specs/001-android-media-shell/spec.md`↔tests)
  12. C-H1..C-H9 scaffold echo
- **Gate 9:** `cargo tauri android build --target aarch64` (authoritative device artifact). Then `scripts/check-manifest.sh` and `scripts/check-apk-classes.sh` validate built artifact/classes (`MainActivity`, `PlaybackService`, `FocusMirror`, `MediaMirror`, `Residency`, `RoutingMirror`, `TauriActivity`).
- **Coverage expectations:** No coverage tooling configured. Strict linting, closed-set discipline, shell purity, and fail-closed guards are the primary QA mechanisms.