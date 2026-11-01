# Repository Guidelines

## Project Overview

`mdreader` (product **Ressoa**, package `app.mdreader`) is an Android media-shell TTS audiobook reader. Rust owns synthesis and PCM playback (sole authority for transport/position/buffering/rate); Kotlin (`android-shell/`) owns `Service`, `MediaSession`, notification, and `AudioFocusRequest` as mirrors only (enforced by purity checks). The Tauri 2.12.1 bridge (`src-tauri/`) is one consumer among many. Every crate is CLI-drivable (`clap` derive, stdin/args → stdout, errors → stderr, `--format json`). Target: Android `minSdk 29`, host-checkable gates 1–8 via `scripts/verify.sh`, device gate 9 via `cargo tauri android build --target aarch64`.

Key principles:
- **Library-first:** every capability reachable as a Rust API; CLI is thin.
- **One code path:** identical behavior for CLI and IPC (`tauri::ipc::Channel`, no `InvokeBody::Streaming`).
- **Zero Android API in Rust:** Kotlin pushes answers across the boundary.
- **Pipeline degrades observably:** never silently infers SSML/alignment; uses declared capabilities.
- **Closed sets everywhere:** exhaustive `match`, no `_ =>` arms, no `Other`/`Unknown`; adding variants requires coordinated edits + docs + semver bump.
- **Durability:** 20s heartbeat while `Playing` + event-durable writes; hard kill replays ≤ ~30s.

## Architecture & Data Flow

```
Kotlin surface (Service/MediaSession/notification/focus) — mirror only
        ↓ intents (wire strings)
Tauri bridge: src-tauri/src/lib.rs (AppState, commands, publish, heartbeat)
        ↓ same Rust code path
crates/player      — transport, position (audible-progress only), buffering/pre-roll (≤60 s), rate ladder, skip/rewind, focus/routing/interruption, surface/mirror, backend
crates/engine-core — Engine trait, closed vocabulary (`engine-core/src/closed.rs`), handoff format (mono f32 @ 22050 Hz), fallback, stand-in fixtures
crates/session     — redb bookmarks + settings (blake3-256 content fingerprint), offer/resume state machine
crates/diag        — append-only JSONL diagnostics (30-day/10 MiB, 11 kinds, 7 violation types)
crates/doc-structure — ordered headings → timeline/FR-038 re-anchor
crates/adapter-synthesis — offline_default (sherpa-onnx/Piper pt-BR), chunking, synth, capabilities, format_check, failure taxonomy, registry
crates/cleanroom-core — proptest utilities (standalone, not a workspace member)
```

**Audio/handoff:** engine delivers mono `f32` PCM @ 22050 Hz; rate is pitch-preserving inside the engine; player selects rung only. Mismatches produce named `EngineError`s (never silent resample/downmix). Position advances only on audible progress from the callback (never during pre-roll, never on timer).

**State/locks:** single `AppState` behind `tauri::manage`, fields `Mutex<_>`. Take/release locks alone, no nested/combined holds. Lock order follows IPC path (`session` → `bookmark_store` last); heartbeat/read order mirrors to avoid cycles. Poisoned locks are skipped with `tracing::warn` (best-effort), never panic.

**Authority split:** Rust = sole playback authority; Kotlin = pure mirror. Shell purity enforced by `scripts/check-shell-purity.sh` (SC-035/FR-042).

## Key Directories

| Path | Purpose |
| --- | --- |
| `crates/engine-core/` | Closed sets, `Engine` trait, handoff format (`SAMPLE_RATE=22050`, `CHANNELS=1`, `f32`), fallback/order, stand-ins |
| `crates/player/` | Transport/position/rate/skip/rewind/focus/interruption/routing/recovery/resume/earcon/backend/buffer/surface |
| `crates/session/` | `redb` bookmarks/settings, blake3 fingerprint, offer machine, resume logic |
| `crates/diag/` | JSONL writer/scanner, retention/caps, violation detection |
| `crates/doc-structure/` | Headings/timeline, normalization, `reanchor` (FR-038) |
| `crates/adapter-synthesis/` | Offline adapter, chunk/synth, capabilities, format check, failure taxonomy, registry |
| `crates/cleanroom-core/` | Standalone proptest helpers (not in workspace) |
| `src-tauri/` | Tauri bridge (`AppState`, commands, publish, heartbeat/playback watch), permissions/surface/failure/diag wiring |
| `android-shell/` | Kotlin mirror (Service/MediaSession/notification/focus) — no playback logic |
| `vendor/sherpa-onnx-sys/` | Vendored patch for `zip` (RUSTSEC-2026-0367/0368/0369); remove when upstream fixes |
| `scripts/` | Gates, purity checks, manifest/APK checks, commit hook |
| `contracts/` | Normative IPC/surface/earcon/ingest/storage/resume/diagnostic contracts |
| `docs/` | Architecture, IPC, diagnostics, fallback order, byte budget, ADRs |
| `specs/` | Feature specs/tasks/checklists/contracts (001-android-media-shell onward) |
| `tests/` | Root integration/contract/UI/audit tests (many artifact/filesystem assertions) |

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

# Lint (gate 4)
cargo clippy --workspace --all-targets -- -D warnings

# Supply chain (gate 5)
cargo deny check advisories licenses bans sources
cargo audit

# All gates 1-8 (fail-closed)
./scripts/verify.sh

# Android artifact (gate 9, outside verify.sh)
cargo tauri android build --target aarch64

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

**Formatting/imports:** 4-space indent, 100 soft / 120 hard line length; always `cargo fmt`. Imports grouped (std / external / local), alphabetical within groups, prefer `use crate::`. Derive `Debug, Clone, Serialize, Deserialize` where appropriate.

**Naming:** `PascalCase` (structs/enums/traits), `snake_case` (functions/variables/modules), `UPPER_SNAKE_CASE` (constants), descriptive test names with underscores. Private helpers may prefix `_` if unused; generics use single uppercase (`T`, `U`, `E`, `I`).

**Types/errors:** Prefer `f64` for positions; `String` owned / `&str` borrowed; `Option<T>`/`Result<T, E>` everywhere. Custom errors with `thiserror` (`#[derive(Error, Debug)]`), propagate with `?`, use `anyhow::Context` where helpful. `Eq` precluded by `f64`; use `PartialEq` carefully. **No** `unwrap`/`expect`/`panic`/`todo`/`dbg`/`println`/`print_stdout`/`print_stderr` (denied). Suppressions only via `#[expect(lint, reason = "...")]`.

**Async/runtime:** `tokio` only in `src-tauri` (`rt`, `time`). Library crates are synchronous (no `async`/`await`). `cpal` uses blocking output-stream callbacks; position advances only on audible progress.

**CLI protocol (Principle II):** `clap` derive; stdin/args → stdout (JSON default, `--format human` optional); errors → stderr; exit 0 on `Ok`, 1 on `Err`. Each crate exposes `pub fn run(args, streams) -> Result<(), CliError>`; `main.rs` forwards locked stdio.

**Closed sets (Principle VII):** One declaration site in `crates/engine-core/src/closed.rs`. Every consumer must match exhaustively — **no `_ =>` arms in `crates/*/src/`** (gate 7 enforced). No `Other`/`Unknown` catch-alls. Adding variants requires coordinated edits + docs + semver bump + gate 7 passing.

**State/DI:** Explicit path deps in `Cargo.toml`, constructor injection. `AppState` holds `Mutex<_>` fields; locks taken alone and released before next; `BookmarkStore` locked last. No DI container.

**Wire/case mapping:** Rust `PascalCase` → JSON `snake_case`. Examples: `TransportState::Playing` → `playing`, `PlaybackRate` → bare number (e.g. `1.5`), `EngineTier::OfflineDefault` → `offline_default`. `Display` returns wire strings.

**Audio specifics:** Mono `f32` @ 22050 Hz (`crates/engine-core/src/format.rs`). Pitch-preserving stretch inside engine; ladder selection only. Pre-roll ≤ 60 s; position never advances on buffered-only audio. Rate persistence via `last_rate` in settings (read at start, written on rung changes).

**Diagnostics:** Append-only JSONL, 4 required fields (`v`, `ts_ms`, `kind`, `position_secs`), `additionalProperties: false`. `detail` bound to non-prose `key=value` (≤200 chars). 11 kinds; dual bounds (30d / 10 MiB) with `trim`/`gap` markers. Scanner reports 7 violation types.

## Important Files

| File | Role |
| --- | --- |
| `Cargo.toml` | Workspace members/deps, `[workspace.lints]` (strict deny/warn set), `[patch.crates-io]` for vendored `sherpa-onnx-sys` |
| `Cargo.lock` | Committed; use `--locked` |
| `rust-toolchain.toml` | Stable channel + `rustfmt`, `clippy` |
| `.cargo/config.toml` | Clang linker on `x86_64-unknown-linux-gnu` |
| `clippy.toml` | Lint baseline marker |
| `deny.toml` | Supply chain policy (advisories/bans/licenses/sources); deny-list includes TTS/LLM crates except explicitly allowed (`sherpa-onnx`, `ort`, `espeak-rs`, `msedge-tts`) |
| `.config/nextest.toml` | `cargo-nextest` config |
| `scripts/verify.sh` | Gates 1-8 orchestrator (fail-closed) |
| `scripts/expected-tests.txt` | Pinned nextest target list for gate 2 two-way diff |
| `scripts/check-shell-purity.sh` | Enforces mirror-only shell + traceability (SC-035/FR-042) |
| `scripts/check-manifest.sh` | Android manifest checks (`mediaPlayback`, FGS permissions) |
| `scripts/check-apk-classes.sh` | APK class presence verification |
| `src-tauri/tauri.conf.json` | Tauri config (product `Ressoa`, Android `minSdk 29`, voice resources) |
| `src-tauri/src/lib.rs` | Bridge entry, `AppState`, commands, publish, heartbeat/playback watch |
| `crates/engine-core/src/closed.rs` | Canonical closed vocabulary |
| `crates/engine-core/src/format.rs` | Handoff shape (22050 Hz mono f32) |
| `crates/player/src/lib.rs` / `crates/player/src/cli.rs` | Playback authority and `ShellVerb`/transition logic |
| `crates/session/src/store.rs` / `crates/session/src/fingerprint.rs` | redb store and blake3 fingerprinting |
| `crates/diag/src/writer.rs` / `crates/diag/src/scan.rs` | Diagnostic writer/scanner |
| `crates/adapter-synthesis/src/reg.rs` | Tier registry (`offline_default` only at v1) |
| `docs/architecture.md`, `docs/ipc.md`, `docs/diagnostic-record.md` | Core architecture/contracts |
| `contracts/*.md`, `contracts/*.json` | Normative contracts (shell IPC, surface, earcon, ingest, storage, resume, diagnostics) |

## Runtime/Tooling Preferences

- **Language/toolchain:** Rust 2024 edition, stable toolchain (`rust-toolchain.toml`, components `rustfmt`, `clippy`); MSRV policy as recorded. Clang linker on Linux x86_64.
- **Build/orchestration:** `cargo` with `--locked` (committed `Cargo.lock`). Workspace resolver v3. Tauri 2.12.1 pinned.
- **Android:** `minSdk 29`, manifest edited directly for `mediaPlayback` foreground service + `FOREGROUND_SERVICE*` permissions. JNI 0.21 (compat with Tauri 2.12.1). Voice assets in `crates/adapter-synthesis/assets/`.
- **Testing:** `cargo-nextest` (not plain `cargo test`). `proptest` 1.12.0, `tempfile` 3.14.0, `jsonschema` 0.29.0. Tests split across crate `tests/` and root `tests/`; many use real `redb` files/temp dirs (no mocks). Property tests checked in with `.proptest-regressions`. Root tests often assert on artifact files.
- **Lint/format/supply chain:** `rustfmt` + `clippy` with strict deny set (gate 4 fails closed). `cargo-deny` + `cargo audit` (gate 5). Deny-list enforced (gate 6). Vendored `sherpa-onnx-sys` via `[patch.crates-io]`.
- **Audio:** `cpal` 0.18.2, AAudio on Android. Host CI devices (44.1–48 kHz stereo) return named refusal in `CpalBackend` (asserted in contract tests).

## Testing & QA

- **Frameworks:** `cargo-nextest`, `proptest`, `jsonschema` for schema validation.
- **Locations:** unit tests in `#[cfg(test)]` modules; integration/contract/property tests in `crates/*/tests/` and root `tests/`. Some tests explicitly declared via `[[test]]` in crate `Cargo.toml`s.
- **Patterns:** given-when-then naming, `assert_eq!`/`assert_ne!`, real I/O (temp `redb`), exhaustive closed-set/property assertions, contract tests validating schemas and wire forms. Many root tests perform filesystem assertions against `src-tauri/src/*` artifacts.
- **Running:** prefer `cargo nextest run --workspace`. Run specific crate/test with `-p` or target names from `scripts/expected-tests.txt`. Tests may read files relative to workspace root.
- **Gates:** `./scripts/verify.sh` runs gates 1-8 (format, tests+two-way diff vs `expected-tests.txt`, docs, lint-drift, supply-chain, deny-list, closed-set `no _ =>`, pipeline). Gate 8 split: `gate-8-shell` PASS, `gate-8-full` DEFERRED pending synthesis (byte budget/SSML/sentence alignment). Gate 9 is Android build outside the script. Purity/manifest/APK checks run as sub-gates.
- **Constraints:** no coverage tooling configured; strict linting; closed-set discipline enforced; shell purity enforced.