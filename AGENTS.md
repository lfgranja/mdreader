# Repository guidance

`mdreader` packages Ressoa, an Android TTS reader. Rust crates own playback; `src-tauri/` bridges Tauri/JNI to those crates, and `android-shell/` contains Android platform code.

## Runtime boundaries

- Keep synthesis, PCM output, transport/rate, position, buffering, and focus policy in Rust. Kotlin handles Android service, `MediaSession`, notification, and audio-focus APIs, forwards platform inputs, and renders Rust-published state. Do not calculate playback state or progress in Kotlin or the frontend.
- The engine handoff is exactly mono `f32` PCM at 22,050 Hz (`crates/engine-core/src/format.rs`). Report rate, channel, or sample-width mismatches as a named `EngineError`; never silently resample or downmix.
- `PositionClock` (`crates/player/src/position.rs`) advances only on audible progress. Timers and pre-roll do not advance it.
- Closed vocabularies live in `crates/engine-core/src/closed.rs`. Keep matches exhaustive; do not add `_ =>`, `Other`, or `Unknown`. Adding a variant requires updating consumers, wire contracts, tests, and docs, and following the versioning procedure.

## Workspace and linting

- The root `Cargo.toml` lists workspace members explicitly. `crates/cleanroom-core` is not a member, so workspace checks omit it; its manifest currently also fails direct Cargo invocation because it inherits root workspace dependencies. Do not assume it is standalone or change workspace membership without confirming intent.
- The Rust toolchain follows moving `stable` and requires `rustfmt` and `clippy` (`rust-toolchain.toml`). Linux x86_64 host linking uses `clang` (`.cargo/config.toml`).
- Root lint tables deny unsafe code, missing public docs/debug implementations, `.unwrap()`/`.expect()`, `panic!`/`todo!`/`unimplemented!`/`unreachable!`/`dbg!`, and stdout/stderr printing. Fix code instead of weakening lint levels or adding blanket `#[allow]`; use narrow `#[expect(..., reason = "...")]` only for justified exceptions.

## Verification

Use `cargo-nextest`; CI and the gate script use it. `.config/nextest.toml` serializes `player`, `session`, and property-test groups.

```sh
cargo fmt --all -- --check
cargo nextest run --workspace --profile ci
cargo nextest run -p player
cargo nextest run -p player -E 'test(on_audible_progress_advances_by_exactly_elapsed)'
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/verify.sh
```

`bash scripts/verify.sh` is the host preflight; it requires `cargo-nextest`, `cargo-deny`, and `cargo-audit`, and defaults `PROPTEST_CASES` to 32. Gate 2 compares live nextest targets against `scripts/expected-tests.txt` in both directions, so update that file when adding or removing test targets. Gate 8 runs the engine-handoff subset; the full synthesis contract is explicitly deferred.

## Android

- Android minSdk is 29 (`src-tauri/tauri.conf.json`). Gate 9 is `cargo tauri android build --target aarch64 --debug`; CI uses Java 17, Android NDK 27.0.12077973, and SDK platform 35.
- Tauri serves `dist/index.html`; the surface-audit test requires it to byte-match `src-tauri/src/status_view.html`. Android's `beforeBuildCommand` copies it into generated app assets.
- Kotlin sources live in `android-shell/src/main/kotlin/`; CI copies them into `src-tauri/gen/android/app/src/main/kotlin/` before building. The tracked manifest at `src-tauri/gen/android/app/src/main/AndroidManifest.xml` is hand-edited and checked by `scripts/check-manifest.sh`.
- Tauri is pinned to 2.12.1 in `src-tauri/Cargo.toml`; re-run the Android checks after changing the Tauri version or manifest.

For normative behavior and wire formats, check the owning spec under `specs/`, `contracts/`, and `.specify/memory/constitution.md`. PR and review requirements are in `.github/CONTRIBUTING.md`; CI setup is in `.github/workflows/ci.yml`.
