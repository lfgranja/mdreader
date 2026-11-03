# Repository Guidelines

> Synthesized from 4 parallel scouts (core-src / tests-configs / scripts-docs / build-state) on 2026-11-03. Source of truth: repo root + `.specify/memory/constitution.md` (v2.0.0, amended 2026-10-28, 1468 lines); contracts/; docs/; scripts/; `ROADMAP.md` (dev `0dcd6d5c`, 2026-11-02; 005/006/007/008 converged; 007 at 28/29; T029 remaining).

---

## Project Overview

`mdreader` (product **Ressoa**, package `app.mdreader`) is an Android media-shell TTS audiobook reader. **Rust owns synthesis and PCM playback as the sole playback authority**; Kotlin (`android-shell/`) owns `Service`, `MediaSession`, notification, and `AudioFocusRequest` as **mirrors only**. The Tauri 2.12.1 bridge (`src-tauri/`) is one consumer among many. Every crate is CLI-drivable (`clap` derive, stdin/args → stdout, errors → stderr, `--format json`). Target: Android `minSdk 29`. Host-checkable gates 1–8 via `scripts/verify.sh`; device gate 9 via `cargo tauri android build --target aarch64`.

Key principles (from `docs/architecture.md` / `.specify/memory/constitution.md` §VII/§VIII):
- **Library-first:** Every capability reachable as a Rust API; CLI is thin (`main.rs` → `cli::run`).
- **One code path:** Identical CLI and IPC (`tauri::ipc::Channel`, no `InvokeBody::Streaming`).
- **Zero Android API in Rust:** Kotlin pushes answers across the boundary.
- **Pipeline degrades observably:** Never silently infers SSML/alignment; uses declared capabilities (`crates/adapter-synthesis/src/reg.rs`: `offline_default` only at v1).
- **Closed sets everywhere:** Exhaustive `match`, no `_ =>` arms, no `Other`/`Unknown`; adding variants requires coordinated edits + docs + semver bump + gate 7 passing (`crates/engine-core/src/closed.rs` is the single declaration site for `TransportState`, `SessionEnd`, `EngineError`).
- **Durability:** 20 s heartbeat while `Playing`; event-durable writes; hard kill replays ≤ ~30 s.
- **Position advances only on audible progress:** `PositionClock` (`crates/player/src/position.rs`) — never during pre-roll, never on timer.

Governance: normative policy in `.specify/memory/constitution.md`; wire formats/interfaces in `contracts/*.md`/`*.json`; rationale in `docs/`; current state in `ROADMAP.md`; feature specs in `specs/` (001–009).

---

## Architecture & Data Flow

### Authority Split
- **Rust = sole playback authority** (`crates/*`). Position/transport/rate/focus/interruption/recovery live here.
- **Kotlin/Android shell (`android-shell/`) = mirror only.** Renders published JSON verbatim; no position/progress/haptics computation.
- **Tauri bridge (`src-tauri/src/lib.rs`)** connects frontend/Kotlin/JNI → Rust; publishes mirror state via channel; builds Kotlin mirror JSON; holds `AppState` with strict lock ordering.

### Pipeline (one Rust code path)
```
Kotlin surface (Service / MediaSession / notification / focus) — mirror only
        ↓ intents
Tauri bridge: src-tauri/src/lib.rs (AppState, commands, publish, heartbeat/playback watch)
        ↓ same Rust code path
crates/engine-core   — Engine trait, closed vocabulary, handoff format (mono f32 @ 22050 Hz), stand-ins
        ↓ (declared capabilities)
crates/adapter-synthesis — offline_default (sherpa-onnx / Piper pt-BR), chunking/synth, registry
        ↓
crates/player        — transport, PositionClock (audible-progress only), buffering/pre-roll (≤60 s), rate/skip/rewind, focus/routing/interruption, backend (cpal 0.18.2, AAudio Android), surface/mirror, earcons
        ↓
crates/session       — redb bookmarks + settings (blake3-256 fingerprint), offer/resume state machine
crates/diag          — append-only JSONL diagnostics (30-day / 10 MiB)
crates/doc-structure — ordered headings → timeline / FR-038 re-anchor
```

### State, Locks & DI
- **DI:** explicit path deps in `Cargo.toml`; constructor injection; no DI container.
- **AppState** (`src-tauri/src/lib.rs`): `Mutex` fields taken alone/released before next; lock order `session` first, `bookmark_store` last; poisoned locks skipped with `tracing::warn`.
- **Position clock:** advances only via `on_audible_progress(elapsed_secs)`.
- **Bookmarking:** content-addressed via blake3-256 (`crates/session/src/fingerprint.rs`).

### Audio & Handoff
- **Format:** Mono `f32` @ `22050` Hz. Mismatches → named `EngineError` (never silent resample/downmix).
- **Engine contract:** requires `declared_capabilities()`, `set_rate()` (pitch-preserving; deviation beyond `PITCH_TOLERANCE_CENTS` → `EngineError::RateMismatch`), `stream(position_secs, max_secs)` returns buffered-only chunks.
- **Backend:** `cpal` 0.18.2 (AAudio on Android). Host CI stereo devices return named refusal. Pre-roll ≤ 60 s; bounded resident queue.
- **Wire format:** Rust `PascalCase` → JSON `snake_case`. `Display` returns wire strings.

---

## Key Directories

| Directory | Purpose |
|---|---|
| `crates/engine-core/` | Closed vocabulary, `Engine` trait, handoff format (22050 Hz mono f32), stand-ins, fallback/order |
| `crates/player/` | Playback authority (transport, `PositionClock`, rate/skip/rewind, focus/routing/interruption, backend/cpal, surface, earcons); CLI |
| `crates/session/` | `redb` bookmarks/settings, blake3-256 fingerprint, offer/resume state machine |
| `crates/adapter-synthesis/` | Offline adapter `offline_default` (sherpa-onnx/Piper pt-BR), chunk/synth, capabilities, registry (only `offline_default` at v1) |
| `crates/diag/` | Append-only JSONL diagnostics, retention (30-day/10 MiB) |
| `crates/doc-structure/` | Ordered headings → timeline/re-anchor |
| `crates/cleanroom-core/` | Standalone `proptest` helpers (not workspace member) |
| `src-tauri/` | Tauri bridge (`AppState`, commands, publish, heartbeat/playback watch), JNI shell bridge |
| `android-shell/` | Kotlin mirror only (Service/MediaSession/notification/focus) — no playback logic |
| `vendor/sherpa-onnx-sys/` | Vendored patch (remove when upstream fixes) |
| `scripts/` | Gates (`verify.sh` 1-8), checks, `expected-tests.txt` |
| `contracts/` | Normative contracts and JSON schemas |
| `docs/` | Architecture, IPC, diagnostics, ADRs |
| `specs/` | Feature specs/tasks/contracts (001-009) |
| `tests/` | Integration/contract/UI/audit tests |

---

## Development Commands

```sh
# Format (gate 1)
cargo fmt --all -- --check
cargo fmt --all

# Tests (gate 2) — cargo-nextest required
cargo nextest run --workspace
cargo nextest run --workspace --profile ci
# Verify test list matches expected
cargo nextest list --workspace | sort > /tmp/nextest-live.txt
sort scripts/expected-tests.txt > /tmp/expected.txt
diff -u /tmp/expected.txt /tmp/nextest-live.txt

# Docs (gate 3)
cargo doc --workspace --no-deps

# Lint (gate 4) — must be clean
cargo clippy --workspace --all-targets -- -D warnings

# Supply chain (gate 5)
cargo deny check advisories licenses bans sources
cargo audit

# All gates 1-8
./scripts/verify.sh

# Android build (gate 9, device-checkable)
cargo tauri android build --target aarch64
cargo tauri android build --target aarch64 --debug

# Build/check
cargo build --workspace
cargo build --release --workspace --locked
cargo check -p player
cargo check --target x86_64-unknown-linux-gnu
```

---

## Code Conventions & Common Patterns

### Formatting & Imports
- Indent 4 spaces, no tabs; line length 100 soft/120 hard
- Always run `cargo fmt` (gate 1 enforced)
- Imports: group std/external/local, alphabetical within groups; prefer `use crate::`
- Derives: `Debug, Clone, Serialize, Deserialize` where appropriate

### Naming
- Structs/enums/traits: `PascalCase`
- Functions/variables/modules: `snake_case`
- Constants: `UPPER_SNAKE_CASE`
- Test functions: descriptive with underscores
- Generics: single uppercase (`T`, `U`, `E`, `I`)

### Types & Error Handling
- Prefer `f64` for positions; `String` owned/`&str` borrowed
- `Option<T>`/`Result<T, E>` everywhere
- Custom errors with `thiserror`; propagate with `?`
- **Forbidden (workspace-denied):** `unwrap`/`expect`/`panic`/`todo`/`dbg`/`println`/`print_stdout`/`print_stderr`. Suppress only via `#[expect(lint, reason = "...")]`

### Async, Runtime & Backend
- `tokio` only in `src-tauri`; library crates are synchronous
- `cpal` blocking callbacks; position advances only on audible progress
- JNI 0.21, Android `minSdk 29`

### CLI Protocol (same code path as IPC)
- `clap` derive; stdin/args → stdout (JSON default), errors → stderr; exit 0 on Ok, 1 on Err
- Each crate exposes `pub fn run(args, streams) -> Result<(), CliError>`; thin `main.rs` forwards stdio

### Closed Sets
- One declaration site: `crates/engine-core/src/closed.rs`
- Exhaustive matches, **no `_ =>` arms** (comments stripped), no `Other`/`Unknown`
- Adding variants requires coordinated edits + docs + semver bump + gate 7 passing

### State Management
- Mutex fields taken alone, released before next (never nested); `BookmarkStore` locked last
- Rust-only logic; Kotlin/JNI transport/mirror only, zero Android API in Rust crates
- Content-addressed bookmarks via blake3-256
- Diagnostics append-only JSONL with strict schema

---

## Important Files

| File | Role |
|---|---|
| `Cargo.toml` | Workspace members/deps, `[workspace.lints]` strict denies |
| `Cargo.lock` | Committed (use `--locked`) |
| `rust-toolchain.toml` | Stable + rustfmt + clippy |
| `.cargo/config.toml` | Clang linker on x86_64-unknown-linux-gnu |
| `deny.toml` | Supply-chain policy |
| `.config/nextest.toml` | cargo-nextest CI profile |
| `src-tauri/tauri.conf.json` | Tauri config (minSdk 29, product Ressoa, id app.mdreader) |
| `src-tauri/Cargo.toml` | Tauri 2.12.1 pins, JNI 0.21 |
| `src-tauri/src/lib.rs` | Bridge entry, AppState, commands, RMS/waveform plumbing |
| `crates/engine-core/src/closed.rs` | Canonical closed vocabulary |
| `crates/engine-core/src/format.rs` | Handoff shape (22050 Hz mono f32) |
| `crates/engine-core/src/trait_.rs` | Engine trait/contracts |
| `crates/player/src/position.rs` | PositionClock (audible-progress only) |
| `crates/player/src/backend.rs` | cpal backend, RMS, bounded queue |
| `crates/session/src/store.rs` / `fingerprint.rs` | redb store + blake3-256 |
| `crates/diag/src/writer.rs` / `scan.rs` | Diagnostics |
| `crates/adapter-synthesis/src/reg.rs` | Tier registry (offline_default only) |
| `scripts/verify.sh` | Gates 1-8 orchestrator |
| `scripts/expected-tests.txt` | Pinned nextest target list (diff for gate 2) |
| `scripts/check-*.sh` | Manifest/shell-purity/apk-classes checks |
| `.specify/memory/constitution.md` | Normative governance (v2.0.0) |
| `ROADMAP.md` | Current state |

---

## Runtime/Tooling Preferences

- **Language/toolchain:** Rust 2024 edition, stable channel, rustfmt+clippy, MSRV ≥ 1.85; use `--locked`
- **Build/orchestration:** cargo workspace, `cargo tauri android build` for Android (gate 9)
- **Android:** minSdk 29, manifest edited directly, JNI 0.21, Kotlin mirror only
- **Testing:** `cargo-nextest` (not plain `cargo test`); proptest, tempfile, jsonschema; real redb/temp dirs; regression files checked in
- **Lint/format/supply chain:** rustfmt+clippy strict denies, cargo-deny+cargo-audit
- **Audio/backend:** cpal 0.18.2, AAudio on Android, mono f32 @ 22050 Hz, AtomicU32 RMS, throttled publishing
- **CI profile:** threads-required=1, failure-output=final, leaks-request=forbid

---

## Testing & QA

### Locations
| Area | Notes |
|---|---|
| `crates/*/src/**` (`#[cfg(test)]`) | Inline unit tests |
| `crates/*/tests/` | Integration/contract/property tests |
| `crates/cleanroom-core/tests/` | Blocking invariants (not workspace member) |
| `src-tauri/tests/` | Bridge/integration/diagnostic/surface/RMS/waveform/repeat tests |
| `tests/` (root) | Spec/contract/UI/audit tests (005-red, 007 convergence) |

### Running & Gates
- **Gate 1:** format check
- **Gate 2:** nextest + two-way diff vs `scripts/expected-tests.txt` (~130 targets)
- **Gate 3:** docs build
- **Gate 4:** clippy -D warnings (strict workspace lints)
- **Gate 5:** cargo-deny + cargo audit
- **Gate 6:** deny-list scan (allows sherpa-onnx/ort/espeak-rs/msedge-tts)
- **Gate 7:** closed-set (no `_ =>` in crates, no Other/Unknown)
- **Gate 8:** pipeline/shell checks
- **Gate 9:** Android build + manifest/apk class checks

**Notes:** No coverage tooling configured. Strict linting, closed sets, shell purity, and fail-closed gates are primary QA mechanisms.
