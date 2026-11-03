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
- **Position advances only on audible progress:** `PositionClock` (`crates/player/src/position.rs`) — never during pre-roll, never on timer (`rewind_for_silence` clamps to `min(REWIND_FRACTION × paused, REWIND_CAP_SECS)`; `snap_to_boundary` caps at `stored_basis_secs - REWIND_CAP_SECS`).

Governance: normative policy in `.specify/memory/constitution.md`; wire formats/interfaces in `contracts/*.md`/`*.json`; rationale in `docs/`; current state in `ROADMAP.md`; feature specs in `specs/` (001–009; FR/SC tokens visible in `005-adopt-design-sona`, `007-sona-complete-design`).

---

## Architecture & Data Flow

### Authority Split
- **Rust = sole playback authority** (`crates/*`). Position/transport/rate/focus/interruption/recovery live here.
- **Kotlin/Android shell (`android-shell/`) = mirror only.** Renders published JSON verbatim; no position/progress/haptics computation. Components: `PlaybackService`, `MediaMirror`, `FocusMirror`, `RoutingMirror`, `Residency`, `MediaSession`, notification.
- **Tauri bridge (`src-tauri/src/lib.rs`)** connects frontend/Kotlin/JNI → Rust; publishes mirror state via channel; builds Kotlin mirror JSON; holds `AppState` (`Mutex<_>` fields; take/release alone, never nested/combined; `session` → `bookmark_store` last; poisoned locks skipped with `tracing::warn`).

### Pipeline (one Rust code path)
```
Kotlin surface (Service / MediaSession / notification / focus) — mirror only
        ↓ intents (wire strings)
Tauri bridge: src-tauri/src/lib.rs (AppState, commands, publish, heartbeat/playback watch)
        ↓ same Rust code path
crates/engine-core   — Engine trait (trait_.rs), closed vocabulary (closed.rs), handoff format (mono f32 @ 22050 Hz, format.rs), stand-ins (standin_a / standin_b)
        ↓ (declared capabilities / caps)
crates/adapter-synthesis — offline_default (sherpa-onnx / Piper pt-BR), chunking/synth, capabilities, format_check, failure taxonomy, registry (reg.rs), voices in assets/
        ↓
crates/player        — transport, PositionClock (audible-progress only), buffering/pre-roll (≤60 s), rate ladder (pitch-preserving in engine), skip/rewind, focus/routing/interruption, backend (cpal 0.18.2, AAudio Android), surface/mirror, earcons
        ↓
crates/session       — redb bookmarks + settings (blake3-256 fingerprint, fingerprint.rs), offer/resume state machine (BOOKMARKS / StoredPosition)
crates/diag          — append-only JSONL diagnostics (30-day / 10 MiB, 11 kinds, 7 violation types; writer.rs / scan.rs / retention)
crates/doc-structure — ordered headings → timeline / FR-038 re-anchor (reanchor)
```

### State, Locks & DI
- **DI:** explicit path deps in `Cargo.toml`; constructor injection; no DI container.
- **AppState** (`src-tauri/src/lib.rs`): `Mutex` fields — `session`, `backend`, `permissions`, `focus_tracker`, `offer`, `pending_open`, `published_route`, `rms_channel`/`last_rms`/`view_hidden`/`last_rms_publish_ms`, `publish_channel`, `surface_resolver`, `headings`, `bookmark_store`, `fingerprint`, `first_yield_ms`, `diag`, `diag_measures`, `document`, `production`, `stream_anchor_secs`, `playback_watch`. Lock order mirrors IPC path (`session` first; `bookmark_store` last); heartbeat/read order mirrors to avoid cycles.
- **Position clock:** `crates/player/src/position.rs` — advances **only** via `on_audible_progress(elapsed_secs)`.
- **Bookmarking:** content-addressed via blake3-256 (`crates/session/src/fingerprint.rs`); `redb` store (`store.rs`).

### Audio & Handoff
- **Format:** Mono `f32` @ `22050` Hz (`crates/engine-core/src/format.rs`, `DeclaredFormat::SAMPLE_RATE = 22050`). Mismatches → named `EngineError` (never silent resample/downmix).
- **Engine contract:** `Engine` (`trait_.rs`) requires `declared_capabilities()`, `set_rate()` (pitch-preserving stretch inside engine; rung selection only — delivered rate must stay declared; deviation beyond `PITCH_TOLERANCE_CENTS` → `EngineError::RateMismatch`), `stream(position_secs, max_secs)` (returns buffered-only chunks; reading never advances position).
- **Backend:** `cpal` 0.18.2 (AAudio on Android). Host CI devices (44.1–48 kHz stereo) return named refusal in `CpalBackend`. Pre-roll ≤ 60 s; resident queue bounded 60 s / 1,323,000 samples.
- **Wire format:** Rust `PascalCase` → JSON `snake_case` (e.g., `TransportState::Playing` → `playing`; `PlaybackRate` → bare number `1.5`; `EngineTier::OfflineDefault` → `offline_default`). `Display` returns wire strings.

### RMS Instrumentation & Waveform (007 / converged)
- **RMS source (Rust):** `crates/player/src/backend.rs` computes RMS of recently consumed speech blocks (lock-free) in `SampleQueue::consume_into()`, stored as `AtomicU32` via `rms: AtomicU32` (`Ordering::Relaxed`). Exposed via `CpalBackend::rms()` and `queue.lock().map_or(0.0, |q| q.rms())`.
- **RMS IPC (Tauri):** `src-tauri/src/lib.rs` maintains `rms_channel: Mutex<Option<tauri::ipc::Channel<f32>>>`, `last_rms`, `last_rms_publish_ms`, `view_hidden`. Commands: `register_rms_listener`; `set_rms_visibility`. `publish_rms` = last-wins coalescing with ~20 Hz throttle (50 ms interval); publishes over `tauri://channel/rms`; polled in playback watch loop.
- **Waveform visualizer (frontend, `status_view.html`):** `#waveform` canvas (aria-hidden="true") — 5-layer smooth ribbons (teal→jade→violet/celeste), cubic-bezier at 1/3 and 2/3 width, per-layer opacity 0.15/0.20/0.25/0.30/0.35, mirrored around center axis; white 1 px center line (0.90); 6 px circular playhead when playing. DPR cap 2×; 30 fps gate (33.33 ms); IPC-event-driven redraw; pause on `visibilitychange`; freeze under `prefers-reduced-motion: reduce`; canvas fallback. Progress via parallel hidden `#progress` (`role="progressbar"`) updated by `renderProgress()` with `aria-valuenow`/`aria-valuetext`; live region `#liveRegion` announces meaningful changes.

### Repeat Button (007 / converged)
- **UI:** `status_view.html` `#btn-repeat`; click → `sendAction('repeat_section')`; SONA tokens; native button (keyboard accessible, focus ring).
- **Bridge:** `src-tauri/src/lib.rs` maps `"skip_back5s" | "repeat_section"` → `BridgeAction::Verb(ShellVerb::SkipBackward)`; reuses existing skip/section + "skip back" earcon (Allegro-trás); restarts current section from beginning.

---

## Key Directories

| Directory | Purpose |
|---|---|
| `crates/engine-core/` | Closed vocabulary (`closed.rs`), `Engine` trait (`trait_.rs`), handoff format (`format.rs` — 22050 Hz mono f32), stand-ins (`standin_a`/`standin_b`), fallback/order |
| `crates/player/` | Playback authority (transport, `PositionClock` at `position.rs`, rate/skip/rewind, focus/interruption, backend/cpal, surface, earcons); CLI at `cli.rs`; lib at `lib.rs` |
| `crates/session/` | `redb` bookmarks/settings (`store.rs`), blake3-256 fingerprint (`fingerprint.rs`), offer/resume state machine |
| `crates/adapter-synthesis/` | Offline adapter `offline_default` (`sherpa-onnx`/Piper pt-BR), chunk/synth, capabilities, `format_check`, failure taxonomy, registry (`reg.rs` — only `offline_default` at v1); voice assets in `assets/` |
| `crates/diag/` | Append-only JSONL diagnostics (`writer.rs`, `scan.rs`); retention; 11 kinds / 7 violation types |
| `crates/doc-structure/` | Ordered headings → timeline / FR-038 `reanchor`; normalization |
| `crates/cleanroom-core/` | Standalone `proptest` helpers (not workspace member — run separately) |
| `src-tauri/` | Tauri bridge (`AppState`, commands, publish, heartbeat/playback watch, `shell_jni.rs`, RMS/visibility, waveform HTML, permissions/surface/failure/diag wiring); `tauri.conf.json` (product `Ressoa`, id `app.mdreader`, v0.1.0, `minSdkVersion 29`) |
| `android-shell/` | Kotlin mirror only (`ShellHost.kt`, `MediaMirror.kt`, `FocusMirror.kt`, `RoutingMirror.kt`, etc.); `PlaybackService`, `MediaSession`, notification, `AudioFocusRequest` — no playback logic |
| `vendor/sherpa-onnx-sys/` | Vendored `zip` patch (`RUSTSEC-2026-0367/0368/0369`); remove when upstream fixes |
| `scripts/` | Gates (`verify.sh` 1–8; sub-gates `check-manifest.sh`, `check-shell-purity.sh`, `check-apk-classes.sh`), `expected-tests.txt` (pinned ~130 targets), commit hook |
| `contracts/` | Normative contracts (`shell-ipc.md`, `surface.md`, `earcon.md`, `ingest.md`, `resume-algorithm.md`, `storage-schema.md`, `diagnostic-trace.md` + JSON schemas) |
| `docs/` | Architecture (`architecture.md`), IPC (`ipc.md`), fallback (`fallback-order.md`), diagnostics (`diagnostic-record.md`, `byte-budget.md`), ADR (`ADR/`), contracts rationale |
| `specs/` | Feature specs / tasks / contracts (`001-android-media-shell` → `009-open-file-from-title`; `005-adopt-design-sona`, `006-lente-sinestesica-completa`, `007-sona-complete-design`, `008-ref-device-emulator`) |
| `tests/` (root) | Integration / contract / UI / audit (`005-red/`, `005-adopt-design-sona-red/`, `contract/`, `chroma_scope_property.rs`, `contract_census.rs`, `surface_inventory.rs`, `us1_*`–`us3_*`) |

---

## Development Commands

```sh
# Format (gate 1) — enforced; never skip
cargo fmt --all -- --check
cargo fmt --all

# Tests (gate 2) — cargo-nextest (NOT plain cargo test); must 2-way diff vs pinned list
cargo nextest run --workspace
cargo nextest run --workspace --profile ci
# Check gate 2 diff locally
cargo nextest list --workspace | sort > /tmp/nextest-live.txt
sort scripts/expected-tests.txt > /tmp/expected.txt
diff -u /tmp/expected.txt /tmp/nextest-live.txt
# Filtered examples (namespaced by crate/feature/task)
cargo nextest run -p player --test prop_transport_math
cargo nextest run -p engine-core --test test_engine_handoff_us1 --test test_standin_conformance_us5 --test prop_pitch --test standin_shape
cargo nextest run -p adapter-synthesis --test prop_chunk

# Docs (gate 3)
cargo doc --workspace --no-deps

# Lint (gate 4) — must be clean (includes clippy -D warnings; workspace lints deny unwrap/expect/panic/todo/dbg/println/print_stdout/print_stderr/allow_attributes/allow_attributes_without_reason/wildcard_dependencies/multiple_crate_versions)
cargo clippy --workspace --all-targets -- -D warnings

# Supply chain (gate 5)
cargo deny check advisories licenses bans sources
cargo audit

# All gates 1–8 (fail-closed, host-checkable)
./scripts/verify.sh

# Sub-gates (invoked by verify.sh)
./scripts/check-manifest.sh      # minSdk 29, mediaPlayback, FGS permissions, no INTERNET
./scripts/check-shell-purity.sh  # mirror-only, vocabulary allowlists, traceability (SC-035 / FR-042)
./scripts/check-apk-classes.sh   # APK class presence (after build)

# Android artifact (gate 9 — authoritative, outside verify.sh; device-checkable only)
cargo tauri android build --target aarch64        # release
cargo tauri android build --target aarch64 --debug

# Build / check
cargo build --workspace
cargo build --release --workspace --locked
cargo check -p player
cargo check --target x86_64-unknown-linux-gnu  # clang linker per .cargo/config.toml

# CLI examples (same code path as IPC — no separate RFC)
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

Pin notes (from `src-tauri/Cargo.toml`, verified by scout): `tauri = "=2.12.1"`, `tauri-build = "=2.7.2"`, edition `2024`; `rust-toolchain.toml`: `channel = "stable"`, components `rustfmt` + `clippy`; MSRV policy recorded (≥ 1.85); `Cargo.lock` committed (use `--locked`); `.cargo/config.toml`: clang linker on `x86_64-unknown-linux-gnu`.

---

## Code Conventions & Common Patterns

### Formatting & Imports
- **Indent:** 4 spaces, no tabs. Line length: 100 soft / 120 hard.
- **Always format:** `cargo fmt` (gate 1 enforced — never skip).
- **Imports:** Group std / external / local, alphabetical within groups; prefer `use crate::` for local.
- **Derives:** `Debug, Clone, Serialize, Deserialize` where appropriate.

### Naming
- Structs/enums/traits: `PascalCase` (`TransportState`, `EngineTier`, `DeclaredFormat`).
- Functions/variables/modules: `snake_case` (`on_audible_progress`, `publish_rms`, `stream_anchor_secs`).
- Constants: `UPPER_SNAKE_CASE` (`SAMPLE_RATE`, `REWIND_FRACTION`, `SKIP_STEP_SECS`, `PITCH_TOLERANCE_CENTS`).
- Test functions: descriptive with underscores (`given_when_then` style; reference FR/SC/task anchors — `005`, `007`, `us1`–`us3`).
- Private helpers: prefix `_` if unused.
- Generics: single uppercase (`T`, `U`, `E`, `I`).

### Types & Error Handling
- Prefer `f64` for positions; `String` owned / `&str` borrowed.
- `Option<T>` / `Result<T, E>` everywhere.
- Custom errors with `thiserror` (`#[derive(Error, Debug)]`); propagate with `?`; `anyhow::Context` where helpful.
- `Eq` precluded by `f64`; use `PartialEq` carefully.
- **Forbidden** (workspace-denied, gate 4 enforced): `unwrap` / `expect` / `panic` / `todo` / `dbg` / `println` / `print_stdout` / `print_stderr`. Suppressions only via `#[expect(lint, reason = "...")]`.

### Async, Runtime & Backend
- `tokio` only in `src-tauri` (`rt`, `time`). Library crates (`crates/*`) are synchronous — no `async`/`await`.
- `cpal` uses blocking output-stream callbacks; position advances only on audible progress (`PositionClock`).
- JNI 0.21 (compat with Tauri 2.12.1); Android `minSdk 29`; `mediaPlayback` + `FOREGROUND_SERVICE*` permissions in manifest (edited directly, not via `gradle` — no `android-shell/build.gradle*` at repo root).

### CLI Protocol (Principle II — same code path as IPC)
- `clap` derive; stdin/args → stdout (JSON default, `--format human` optional); errors → stderr; exit `0` on `Ok`, `1` on `Err`.
- Each crate exposes `pub fn run(args, streams) -> Result<(), CliError>`; `main.rs` forwards locked stdio.
- Thin `main.rs` wrappers delegate to crate `cli::run`; identical behavior for CLI and IPC (`tauri::ipc::Channel`, no `InvokeBody::Streaming`).

### Closed Sets (Principle VII — gate 7 enforced; no catch-all arms in `crates/*/src/*.rs`)
- One declaration site: `crates/engine-core/src/closed.rs` (e.g., `TransportState`, `SessionEnd`, `EngineError`).
- Every consumer matches exhaustively — **no `_ =>` arms** (comments stripped for check). No `Other`/`Unknown` catch-alls.
- Adding variants requires coordinated edits + docs + semver bump + gate 7 passing + `expected-tests.txt` update if new targets.
- Shell purity enforced: no transport/position/rate/gain/duck/skip vocabulary in mutable/arithmetic contexts outside allowlists (`scripts/check-shell-purity.sh`).

### State Management & Patterns
- **Mutex fields in `AppState`;** locks taken alone, released before next (never nested/combined); `BookmarkStore` locked last.
- **Rust-only logic, Kotlin/JNI as transport/mirror.** Zero Android API in Rust crates.
- **Content-addressed bookmarks:** blake3-256 (`crates/session/src/fingerprint.rs`).
- **Diagnostics:** append-only JSONL (`diag/src/writer.rs`); 4 required fields (`v`, `ts_ms`, `kind`, `position_secs`); `additionalProperties: false`; `detail` bound to non-prose `key=value` (≤ 200 chars); retention (30-day / 10 MiB).
- **Coordinator pattern:** Tauri bridge coordinates via `AppState`; crate modules expose pure functions where possible.
- **Position:** advances only on audible progress (`PositionClock`); pre-roll never counts.

---

## Important Files

| File | Role |
|---|---|
| `Cargo.toml` | Workspace members/deps; `[workspace.lints]` (strict deny/warn); `[patch.crates-io]` for vendored `sherpa-onnx-sys`; `[workspace.dependencies]` |
| `Cargo.lock` | Committed; build with `--locked` |
| `rust-toolchain.toml` | Stable + `rustfmt` + `clippy`; MSRV policy |
| `.cargo/config.toml` | Clang linker (`x86_64-unknown-linux-gnu`) |
| `clippy.toml` | Lint baseline marker |
| `deny.toml` | Supply-chain policy (advisories / bans / licenses / sources); deny-list allows `sherpa-onnx`, `ort`, `espeak-rs`, `msedge-tts`; ignores `RUSTSEC-2024-0370` with justification |
| `.config/nextest.toml` | `cargo-nextest` CI profile (`threads-required=1`, `failure-output=final`, `leaks-request=forbid`) |
| `src-tauri/tauri.conf.json` | Tauri config (`minSdkVersion 29`, `distDir ../dist`, `devUrl http://localhost:1420`, voice resources, product `Ressoa`, id `app.mdreader`) |
| `src-tauri/Cargo.toml` | `tauri = "=2.12.1"`, `tauri-build = "=2.7.2"`, edition `2024`; JNI 0.21 |
| `src-tauri/src/lib.rs` | Bridge entry, `AppState` (Mutex fields + RMS channel + heartbeat + playback watch + commands), `shell_jni.rs` |
| `crates/engine-core/src/closed.rs` | Canonical closed vocabulary (one declaration site) |
| `crates/engine-core/src/format.rs` | Handoff shape (`SAMPLE_RATE = 22050`; mono f32) |
| `crates/engine-core/src/trait_.rs` | `Engine` trait + error / capability contracts |
| `crates/player/src/lib.rs` / `cli.rs` | Playback authority + `ShellVerb` / transition logic |
| `crates/player/src/position.rs` | `PositionClock` — audible-progress only; rewind / snap rules |
| `crates/player/src/backend.rs` | `cpal` 0.18.2 backend; `SampleQueue` / `AtomicU32` RMS; `CpalBackend::rms()`; bounded 60 s / 1,323,000 samples |
| `crates/session/src/store.rs` / `fingerprint.rs` | `redb` store + blake3-256 fingerprinting |
| `crates/diag/src/writer.rs` / `scan.rs` / `retention.rs` | Diagnostic writer / scanner / retention |
| `crates/adapter-synthesis/src/reg.rs` | Tier registry (`offline_default` only at v1) |
| `scripts/verify.sh` | Gates 1–8 orchestrator (fail-closed) |
| `scripts/expected-tests.txt` | Pinned nextest target list (two-way diff for gate 2; ~130 targets including `adapter-synthesis::audio_chunk`, `::cli_protocol`, `::prop_chunk`, and `src-tauri` surface/RMS/waveform/repeat targets) |
| `scripts/check-manifest.sh` / `check-shell-purity.sh` / `check-apk-classes.sh` | Manifest (minSdk 29, `mediaPlayback`, FGS, no INTERNET); mirror-only + vocabulary allowlists; APK class presence |
| `docs/architecture.md` | Authority split, pipeline, constitution v2.0.0 anchor, FR/SC tokens |
| `docs/ipc.md` / `fallback-order.md` / `diagnostic-record.md` | IPC contracts, fallback order, diagnostic norms |
| `contracts/*.md` / `*.json` | Normative contracts (shell IPC, surface, earcon, ingest, storage, resume, diagnostics, schemas) |
| `.specify/memory/constitution.md` | Normative governance (v2.0.0, amended 2026-10-28; §VII / §VIII amendments; Exceptions Log; amendment process requiring SYNC IMPACT REPORT) |
| `ROADMAP.md` | Current state (`dev` `0dcd6d5c`; 2026-11-02; 005/006/007/008 converged; 007 28/29; T029 remaining) |
| `specs/007-sona-complete-design/tasks.md` | 007 convergence (28/29); RMS / waveform / repeat complete; T029 gate |

---

## Runtime/Tooling Preferences

- **Language / toolchain:** Rust 2024 edition (workspace resolver v3); stable channel (`rust-toolchain.toml`) + `rustfmt` + `clippy`; MSRV ≥ 1.85 (recorded, not hard-pinned); `cargo` with `--locked` (committed `Cargo.lock`).
- **Build / orchestration:** `cargo build --workspace`; `cargo build --release --workspace --locked`; `cargo tauri android build --target aarch64` (gate 9, outside `verify.sh`). No `npm` / Bun / Node for core; Tauri 2.12.1 bridge is one consumer.
- **Android:** `minSdk 29`; manifest edited directly (`mediaPlayback`, `FOREGROUND_SERVICE*`); JNI 0.21; voice assets in `crates/adapter-synthesis/assets/`; `android-shell/` Kotlin mirror only (no `build.gradle*` at repo root — mirror rendered from published JSON, not compiled independently).
- **Testing framework:** `cargo-nextest` (NOT `cargo test`). `proptest` 1.12.0, `tempfile` 3.14.0, `jsonschema` 0.29.0. Tests split across `crates/*/tests/` (integration/contract/property), `crates/*/src/**` (`#[cfg(test)]` inline), `src-tauri/tests/` (bridge/integration/diagnostic/surface/RMS/waveform/repeat), `tests/` root (spec/contract/UI/audit — `005-red/`, `005-adopt-design-sona-red/`, `contract/`). Many use real `redb` files / temp dirs (no mocks). `crates/cleanroom-core/` is **not** a workspace member (run separately for blocking invariants like `slice_readability_proptest.rs`).
- **Property tests:** persisted failing cases in `*.proptest-regressions` (checked in). Explicit case counts / grid strategies.
- **Lint / format / supply chain:** `rustfmt` + `clippy` strict deny (gate 4 fails closed); `cargo-deny` + `cargo audit` (gate 5); deny-list enforced (gate 6; name-boundary match allows `sherpa-onnx` / `ort` / `espeak-rs` / `msedge-tts`); vendored `sherpa-onnx-sys` via `[patch.crates-io]` with removal condition when upstream fixes `RUSTSEC-2026-0367/0368/0369`. Workspace lints: `unsafe_code = deny`, `missing_docs = deny`, `missing_debug_implementations = deny`, `unreachable_pub = deny`, `unused_results = deny`; clippy denies `unwrap_used`, `expect_used`, `panic`, `unimplemented`, `unreachable`, `todo`, `dbg_macro`, `print_stdout`, `print_stderr`, `allow_attributes`, `allow_attributes_without_reason`, `wildcard_dependencies`, `multiple_crate_versions`.
- **Audio / backend:** `cpal` 0.18.2; `AAudio` on Android; mono f32 @ 22050 Hz; host CI stereo devices (44.1–48 kHz) return named refusal (`CpalBackend` asserted in contracts). `AtomicU32` RMS; `SampleQueue` bounded; `publish_rms` ~20 Hz throttle; waveform canvas (DPR cap 2×; 30 fps gate; `visibilitychange` pause; `prefers-reduced-motion` freeze; aria-hidden canvas + hidden progressbar + live region).
- **CI profile (`.config/nextest.toml`):** `threads-required=1`, `failure-output=final`, `leaks-request=forbid` (prevents race-sensitive transport/math tests from flaking). Target naming: namespaced by crate/module/feature/task (e.g., `player::*`, `engine-core::*`, `src-tauri::*`, `*::prop_*`, `*::us1`).

---

## Testing & QA

### Frameworks & Locations

| Area | What lives there | Notes |
|---|---|---|
| `crates/*/src/**` (`#[cfg(test)]`) | Inline unit tests | Co-located; shape / CLI / protocol checks |
| `crates/*/tests/` | Integration / contract / property | E.g., `session/tests/` (`test_ingest_cli.rs`, `prop_*`), `player/tests/` (integration/contract/feature), `engine-core/tests/contract/`, `adapter-synthesis/tests/` |
| `crates/cleanroom-core/tests/` | Blocking invariants (`slice_readability_proptest.rs`) | **Not workspace member** — run separately |
| `src-tauri/tests/` | Bridge / integration / diagnostic / surface / RMS / waveform / repeat / progress / notification / offer / lockscreen / buffer / play-to-audio | Surface inventory / count; waveform canvas / performance; RMS / visualization; repeat control; progress labels; noscope / purity |
| `tests/` (root) | Spec / contract / UI / audit | `005-red/` (FR-006/007/SC-001, latency oracle / reduced-motion / freeze); `005-adopt-design-sona-red/` (contract drift, extensions empty, FR-006/FR-007, property census closed, SC-001 split/duplication); `contract/`; `chroma_scope_property.rs` / `contract_census.rs` / `surface_inventory.rs`; `us1_*`–`us3_*` (banner/states/rate/blind/waveform / contract audit / notification purity); `005-adopt-design-sona-sc001*`/`sc002`/`sc007`; earcon / lente / reduced-motion invariants. Converged with **007-sona-complete-design** (RMS / waveform / repeat). |
| `specs/**/tests/` | Spec-driven fixtures / evidence | Not always `#[test]` harnesses; fixtures under `tests/device/fixtures/` and `specs/.../tests/device/fixtures/` |

### Patterns
- **Unit / Integration / Contract / Property:** Co-located units, cross-component via authoritative types (`TransportState`, `EngineTier`, `DeclaredFormat`), contract tests between layers, `proptest!` with explicit case counts / grid strategies.
- **Fixtures / replay:** `include_str!` + fixture files (`tests/device/fixtures/`, `specs/.../tests/device/fixtures/`); deterministic JSONL replay through Rust authority.
- **Temp / redb:** Real `redb` files / temp dirs (no mocks); session/recovery via `session::BOOKMARKS` / `store`.
- **Guard / absence:** Fail-closed scans (`test_noscope.rs` enforces no network crates / call sites app-wide; shell purity; manifest; APK classes).
- **Traceability:** Tests reference FR/SC/task/spec anchors (`005`, `007`, `us1`–`us3`); BLOCKING invariants explicitly marked.
- **Regression files:** `*.proptest-regressions` checked in.
- **Exactness:** Contract-critical values assert equality (`rewind` formula; kill/recovery bounds ≤ 30 s).
- **Converged coverage (005 ↔ 007):** Surface / ARIA / contrast / reduced-motion / latency oracle / freeze; RMS / waveform / canvas / performance / DPR cap / 30 fps gate / `visibilitychange`; repeat control; numeric / rate; overflow / empty-title; glassmorphism / chroma scope; decorative elements; earcon / lente invariants. Red suites (`tests/005-red/`, `tests/005-adopt-design-sona-red/`) validate adoption / contract compliance.

### Running & Gates (fail-closed)
- **Runner:** `cargo nextest` (CI profile `.config/nextest.toml`); never `cargo test` for gate 2.
- **Gate 2 (tests):** Two-way diff of `cargo nextest list --workspace | sort` vs `scripts/expected-tests.txt`. Current pinned inventory: ~130 targets (includes `adapter-synthesis::*`, `player::*`, `engine-core::*`, `src-tauri::*` surface/RMS/waveform/repeat, property / integration / contract / schema targets).
- **Full gate sequence (`scripts/verify.sh`):**
  1. format — `cargo fmt --all -- --check`
  2. tests — `cargo nextest run --workspace` + two-way diff vs `expected-tests.txt`
  3. surface-audit / earcon — via nextest filters (`C-H1` … `C-H9`)
  4. documentation — `cargo doc --workspace --no-deps`
  5. lint-drift — manifest `[lints] workspace = true`; root `[workspace.lints]` matches pinned rigour; `cargo clippy --workspace --all-targets -- -D warnings`
  6. supply-chain — `cargo deny check advisories licenses bans sources` + `cargo audit`
  7. deny-list — scan `Cargo.toml`/`Cargo.lock` (name-boundary); allows `sherpa-onnx`/`ort`/`espeak-rs`/`msedge-tts`
  8. closed-set — no `_ =>` match arm in `crates/*/src/*.rs` (comments stripped); no `Other`/`Unknown`
  9. pipeline — shell subset; prints `gate-8-shell: PASS`; `gate-8-full: DEFERRED (owner=synthesis)`
  10. `scripts/check-manifest.sh` (`minSdk 29`, `mediaPlayback`, `FOREGROUND_SERVICE*`, no `INTERNET`)
  11. `scripts/check-shell-purity.sh` (mirror-only, vocabulary allowlists, traceability `FR/SC`)
  12. `C-H1` … `C-H9` scaffold echo
- **Gate 9 (authoritative device artifact):** `cargo tauri android build --target aarch64`. Then `check-manifest.sh` + `check-apk-classes.sh` validate built artifact / classes.
- **Coverage expectations:** No coverage tooling configured. Strict linting, closed-set discipline, shell purity, fail-closed gates (`verify.sh`) are primary QA mechanisms. 005/007 convergence validated via contract/property tests and surface audits.

---

*Document title: Repository Guidelines. Source verification: 4 parallel scouts (core-src / tests-configs / scripts-docs / build-state) completed 2026-11-03. Key updated findings incorporated: `tauri`/`tauri-build` pins (`=2.12.1` / `=2.7.2`), `cargo-nextest` ~130 targets / `expected-tests.txt`, `.specify/memory/constitution.md` v2.0.0 (1468 lines, amended 2026-10-28), `docs/architecture.md` / `ROADMAP.md` (dev `0dcd6d5c`; 007 28/29; T029 remaining), `android-shell/` Kotlin mirror-only, `cpal` 0.18.2 / AAudio / `AtomicU32` RMS / ~20 Hz throttle (`publish_rms`), `AppState` mutex fields + lock order (`session` → `bookmark_store` last), `crates/engine-core/src/closed.rs` single declaration site, 5-layer waveform / DPR cap / 30 fps / `visibilitychange` / `prefers-reduced-motion`, `#btn-repeat` / `BridgeAction::Verb(ShellVerb::SkipBackward)`.*
