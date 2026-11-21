# Repository Guidelines

`mdreader` packages **Ressoa**, an offline-first Android TTS reader for Markdown. Rust crates own all playback authority; Kotlin and the WebView are structural mirrors that render Rust-published state. The Tauri 2.12.1 bridge connects the Android shell and frontend to the Rust core.

## Project Overview

- Product identifier `app.mdreader`; Android `minSdkVersion = 29`; no `INTERNET` permission — documents and audio never leave the device.
- The native TTS path uses vendored `sherpa-onnx-sys` 1.13.8 (zip build-dependency patched to 9 for advisories) with Piper pt-BR voices (`faber`, `cadu`, `jeff`).
- Reference device: `2201117TY` (Redmi Note 11, Android 13 / SDK 33, arm64-v8a).
- Normative behavior lives in `.specify/memory/constitution.md` (v2.0.0) and `specs/*/contracts/`; `docs/architecture.md` summarizes the topology.

## Architecture & Data Flow

```text
Kotlin DocIngest (SAF/ACTION_VIEW/SEND → bytes → BLAKE3 → snapshot)
  → JNI onDocumentIngested → shell_jni::ingest_snapshot
  → open_document_bytes_named (fingerprint, OfferMachine, publish_route)
  → split_document → produce_bounded → adapter_synthesis::synthesize
  → enqueue_chunk_for_generation → CpalBackend::enqueue_pcm
  → audible progress → PositionClock + bookmark (redb, 20 s heartbeat)
  → publish_mirror_to_channel → Kotlin MediaMirror (MediaSession/notification)
                             → WebView events (mirror-publish, chapter-metadata, rms ~20 Hz)
```

- **Rust is the sole authority** for transport, rate, position, buffering, focus and synthesis. Kotlin `ShellHost.postShellVerb(code)` forwards inputs; `MediaMirror` renders payloads verbatim — never computes progress, rate or transport.
- Single synthesis code path shared by CLI and IPC; no `InvokeBody::Streaming`.
- Engine handoff is exactly mono `f32` @ 22,050 Hz (`crates/engine-core/src/format.rs`). Rate/channel/width mismatches are named `EngineError`s — never silent resample or downmix.
- `PositionClock` (`crates/player/src/position.rs`) advances **only** on audible progress; timers and pre-roll do not advance it.
- Closed vocabularies (`crates/engine-core/src/closed.rs`) use exhaustive enums with no `_ =>`, `Other` or `Unknown`. Adding a variant requires coordinated consumers, wire contracts, tests, docs and a version bump.
- `src-tauri/src/surface_audit/` holds structural "twin" tests for the served HTML (aria, contrast, chroma, movement, inventory, served identity).

## Key Directories

| Path | Purpose |
|---|---|
| `crates/engine-core/src/` | Domain types, closed sets, format checks, engine trait, fallback router, stand-ins |
| `crates/adapter-synthesis/src/` | `offline_default` adapter: chunking, synthesis, voices, normalization, prosody, failure taxonomy |
| `crates/player/src/` | Transport state machine, `PositionClock`, CPAL backend, focus, recovery, skip/pre-roll, earcons |
| `crates/session/src/` | `redb` bookmark store (BLAKE3 keys), `OfferMachine`, resume protocol |
| `crates/diag/src/` | Append-only JSONL diagnostic record (11 closed event kinds, 30-day/10 MiB retention) |
| `crates/doc-structure/src/` | Ordered headings, boundary/reanchor helpers |
| `src-tauri/src/` | Tauri bridge (`lib.rs`), JNI shell (`shell_jni.rs`), surface audit |
| `android-shell/src/main/kotlin/app/mdreader/` | Service, `MediaSession`, notification, SAF ingest, mirrors, debug hooks |
| `specs/`, `contracts/` | Normative specs, generated artifacts, wire/format contracts |
| `.specify/scripts/bash/` | SDD prerequisites and scaffolding (`check-prerequisites.sh`, `create-new-feature.sh`) |
| `scripts/ci/` | Deterministic docs-only SDD gates with baselines in `docs/governance/` |

## Development Commands

```sh
# Preflight — always run before suggesting a commit
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
bash scripts/verify.sh                 # gates 1-8 + named checks; PROPTEST_CASES defaults to 32
bash scripts/check-manifest.sh         # generated Android manifest
bash scripts/check-shell-purity.sh     # SC-035 mirror purity

# Targeted tests
cargo test -p src-tauri --test test_directory_resume
cargo nextest run -p engine-core --test test_engine_handoff_us1
cargo nextest run -p player -E 'test(on_audible_progress_advances_by_exactly_elapsed)'

# Android artifacts (CI is preferred — local host may lack JDK 17 / NDK 27 / platform 35)
cargo tauri android build --target aarch64 --debug   # debug
cargo tauri android build --target aarch64           # release

# CI: push to dev → debug build; push to main → debug + release with automatic SemVer;
# workflow_dispatch and v* tags → debug + release.
gh workflow run ci.yml --ref dev
```

## Code Conventions & Common Patterns

- **Style**: 4-space indent; `cargo fmt` is authoritative. Rust naming: PascalCase types, snake_case functions/vars, UPPER_SNAKE_CASE constants. Docs for public items via `///`; modules via `//!`.
- **Lints are deny-level** (`Cargo.toml [workspace.lints]`): no `unsafe`, `unwrap`, `expect`, `panic!`, `todo!`, `dbg!`, stdout/stderr printing, blanket `#[allow]`. Fix code instead; narrow `#[expect(lint, reason = "...")]` only with justification.
- **Errors**: `thiserror` enums with closed variants and meaningful context; `Result<T, E>` and `?`; no catch-all error variants.
- **State**: `AppState` uses `Mutex` for shared data, `AtomicU64 playback_generation` for cancellation, `mpsc` for shell verbs, `OnceLock` for handles. Synthesis runs outside locks and rechecks generation before commit/enqueue.
- **Async/threads**: CPAL callback consumes a PCM queue; backpressure via producer admission; no blocking synthesis in watch ticks on the playback thread.
- **CLI pattern**: every crate is CLI-drivable with `clap`; `--format json|human`; payload on stdout, diagnostics on stderr.
- **Diagnostics**: append via `DiagnosticWriter`; keep `detail` as short `key=value`, never prose. `play_to_audio_ms` is closed at the first consumed sample.
- **Kotlin mirrors**: read Rust payloads verbatim; no local transport/position/rate logic; labels in pt-BR from content, not prose generation.
- **Tests**: `test_*.rs` for contracts/integration, `prop_*.rs` for proptest, `tNNN_*.rs` for task traceability, `contract_*.rs` for schemas.

## Important Files

| Path | Why it matters |
|---|---|
| `Cargo.toml`, `rust-toolchain.toml`, `.cargo/config.toml`, `deny.toml` | Workspace, toolchain, linker (`clang` on host), supply-chain policy |
| `src-tauri/tauri.conf.json` | Frontend dist, `beforeBuildCommand`, Android minSdk, bundled voices |
| `src-tauri/capabilities/default.json` | Tauri ACL — must be versioned; missing capabilities silently break `event.listen` |
| `src-tauri/gen/android/app/src/main/AndroidManifest.xml` | Generated manifest; canonical edits go through `scripts/apply-debug-ingest-overlay.sh`, not direct edits |
| `crates/engine-core/src/format.rs`, `closed.rs` | Handoff format and closed vocabularies — change only with coordinated version bump |
| `crates/player/src/position.rs`, `backend.rs` | Audible-only clock and PCM backend |
| `crates/session/src/store.rs`, `offer.rs` | Bookmarks, resume offers, cold-start routing |
| `src-tauri/src/lib.rs`, `shell_jni.rs` | Bridge entry points, synthesis pipeline, JNI verb channel |
| `scripts/verify.sh`, `scripts/expected-tests.txt` | Authoritative host gates and pinned nextest target inventory |
| `.github/workflows/ci.yml` | Gate 1-8 subset, supply chain, Android debug/release, SemVer |
| `.specify/memory/constitution.md`, `.github/CONTRIBUTING.md` | Governance and review/merge policy |

## Runtime/Tooling Preferences

- **Rust**: moving `stable` channel with `rustfmt` + `clippy`; `clang` linker on Linux x86_64; `cargo-nextest`, `cargo-deny`, `cargo-audit` are required by gates.
- **Android**: Java 17, NDK `27.0.12077973`, platform 35, build-tools 35; Tauri CLI 2.12.x. CI is the preferred build host.
- **Frontend**: `dist/index.html` is prebuilt and served; `beforeDevCommand` is empty. No `package.json` in the repo — do not assume Bun/Node is required for host gates; JS syntax checks use `scripts/check-frontend-module-syntax.sh`.
- **Env vars**: `PROPTEST_CASES=32` (default in `verify.sh`); `SHERPA_ONNX_ARCHIVE_DIR` for host gates; unset for Android builds. The vendored archive makes offline host builds possible.
- **Generated/ignored paths**: never edit `target/`, `src-tauri/gen/**` (except whitelisted tracked files), `dist/`, `.archify/`, keydtores or vendor internals unless changing canonical sources.

## Testing & QA

- `scripts/verify.sh` is the single entrypoint for gates 1-8; every gate is **fail-closed** — missing input is a failure, never a skip. Gate 9 (Android artifact) runs in CI.
- Gate 2 compares the live `cargo nextest list --workspace` against `scripts/expected-tests.txt` in both directions: **any new test target must be added to that file** or the gate fails.
- Property tests use 32 cases locally; deterministic groups (`player`, `session`, `prop_*`) are serialized via `.config/nextest.toml`.
- Surface audit tests (`test(surface_audit)`, `test(earcon)`) enforce C-H1..C-H9 structural invariants; C-H1..C-H9 verbose execution is currently deferred in `verify.sh` — do not claim it ran.
- Gate 8-full (byte budget, SSML, alignment) is explicitly `DEFERRED (owner=synthesis)` — never report it as PASS.
- Device validation scripts (`scripts/device-ttfa-validation.sh`, `automate-*.sh`, `appium-*.py`) are mutating: require explicit authorization before use. Capture `diagnostic-record.jsonl` via `run-as` and treat missing/baseline-mismatched data as `[NV]`.
- SDD docs gates live in `scripts/ci/check_*.py`; baselines in `docs/governance/` are ratchets — update deterministically via `--bootstrap`, never by hand-editing hashes.
- LLM/agent output is informative only; deterministic gates, scripts and CI are the truth. Remote CI/merge/protection claims are `[NV]` until observed via `gh`.
