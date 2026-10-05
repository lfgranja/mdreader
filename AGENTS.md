# AGENTS.md

> Greenfield: no `Cargo.toml`, `crates/`, `src-tauri/`, or `scripts/verify.sh` yet.
> Only `specs/`, `.specify/memory/constitution.md`, and Spec Kit scaffolding exist.
> Do not assume code layout — `plan.md` describes the *intended* layout.

## Read first

- `.specify/memory/constitution.md` (v1.2.0) is binding and overrides convenience.
  Amendments need a Sync Impact Report header + Exceptions Log entry in the same commit.
- Current feature: `specs/001-android-media-shell/` — `spec.md`, `plan.md`,
  `research.md`, `data-model.md`, `contracts/`, `checklists/`, `quickstart.md`.
  Branch `001-android-media-shell` tracks it.
- Workflow commands: `.opencode/commands/speckit.*.md`
  (`specify` → `clarify` → `plan` → `tasks` → `implement`).

## Intended architecture (from `plan.md`, not yet built)

- Cargo-workspace modular monolith: `crates/player` (transport, position, buffering,
  pre-roll ≤60 s, rate, rewind math), `crates/session` (`redb` bookmarks + settings,
  `blake3` fingerprint), `crates/diag` (JSONL record), `crates/engine-core` (trait +
  stand-in engines). Tauri app is one consumer, never the only one; no organiser-only crates.
- **Rust owns synthesis + PCM playback** (`cpal` 0.18.2, AAudio); sole authority for
  position/buffering/transport. Kotlin (`android-shell/`) owns only `Service`,
  `MediaSession` (mirror, never computes position), notification, `AudioFocusRequest`.
  Zero playback logic in Kotlin — enforced by check (SC-035).
- Every crate is CLI-drivable (`clap` derive, stdin/args → stdout, errors → stderr,
  `--format json`); IPC (`tauri::ipc::Channel`, no `InvokeBody::Streaming`) calls the same code.

## Verify (constitution gates; `scripts/verify.sh` is the entrypoint once created)

Prereqs: Rust 1.85+ (Edition 2024), `cargo-nextest`, Android SDK + one API-29+ device.

```sh
cargo fmt --all -- --check
cargo nextest run --workspace
cargo doc --workspace --no-deps
cargo deny check advisories licenses bans sources && cargo audit
cargo tauri android build --target aarch64   # gate 9: must produce device artifact
./scripts/verify.sh                          # gates 1–8 once it exists
```

Host-checkable without a device: rewind/snap math, 7-rung ladder, fingerprint,
trim bounds, closed-set exhaustiveness, manifest check, shell-purity check.
On-device trials are 10/10 (lock-screen, focus, routing, ≤3 s play-to-audio, failover).

## Constraints agents get wrong

- Rust rigor: `unsafe/missing_docs/missing_debug/unreachable_pub/unused_results` deny;
  `clippy::all,pedantic` deny, `nursery,cargo` warn; `unwrap/expect/panic/todo/dbg/println`
  deny — use `thiserror` + `tracing`. Only suppression is
  `#[expect(lint, reason = "...")]`; never weaken `Cargo.toml`/`clippy.toml` to silence a lint.
  Every workspace member declares `[lints] workspace = true`.
- Closed sets everywhere: rate ladder `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`, transport,
  focus outcomes, error kinds, session-end reasons. No `Other`/`Unknown`, no `_ =>` in
  `crates/*/src` (gate 7). Engine IDs live only in the constitution registry — never a second
  list. Adding a variant = coordinated edit + semver bump + docs.
- Supply chain fail-closed: latest stable only (exceptions need version + reason + risk in
  commit msg); `--locked` + committed `Cargo.lock`; reject advisories, non-crates.io sources,
  yanked versions; never auto-substitute a failing crate. Deny-list + copyleft-permitted
  allow-list are in the constitution; `ort` 2.0.0-rc.13 is the sole pre-recorded stability
  exception. No secrets in repo/binary (`.env`, `*.pem`, `*.key` gitignored); per-provider
  revocable consent; Edge endpoint is consent-gated fallback-last, never default, never via
  the OpenAI-compatible Python wrapper.
- Android manifest has no plugin API (`mobile::update_manifest`, `registerService` do not
  exist in Tauri 2): edit `AndroidManifest.xml`/Gradle directly for
  `<service android:foregroundServiceType="mediaPlayback">` + `FOREGROUND_SERVICE*`
  permissions; re-verify on every Tauri upgrade. `Service` is Kotlin for a structural reason
  (ActivityManager reflection); bridge is `register_android_plugin` + `run_mobile_plugin`,
  constructor `(Activity;)V`; Service self-bootstraps from Application/Activity (API 34+
  FGS-start restrictions apply). Pinned: Tauri **2.12.1** (not 3.0.0-alpha).
- Audio handoff: engine delivers mono f32 at declared **22050 Hz**; engine resamples,
  Rust converts device format only. Rate/channel/width mismatch = named `EngineError`,
  never silent resample/downmix. Rate stretch is an engine-contract op (pitch-preserving).
- Notification shows title + current section title + progress only — no body text,
  elapsed/remaining, rate, or voice. Determinate iff total known at session start, else
  indeterminate for the whole session. Non-dismissable while playing; pause is the route
  to dismissal. Focus: `*_MAY_DUCK` → ~20% duck, keep reading; call/alarm/permanent loss →
  pause + reason expiring everywhere after 3 s, never auto-resume. `BECOMING_NOISY` +
  `onAudioDevicesRemoved` both held to the ≤1 s pause bound.
- Diagnostic record: append-only JSONL in app-private storage, 30-day + 10 MB oldest-first
  trim, never transmitted, no in-app surface, no document text in any entry
  (`detail` ≤200 chars identifier-pattern or typed fields only; `error` required on
  `failure` *and* `refusal`). Position keyed by `blake3` content hash (edit = different
  document = no resume); Stop/document-end clears it; kill/power-loss replay ≤30 s.
