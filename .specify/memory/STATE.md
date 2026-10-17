# Session state — 004-ressoa-dogfood-build (shell bring-up wave)

Updated: 2026-10-16 (Waves A+B committed; gate 5 waived provisionally, constitution v1.6.1; upstream PR #4059 open; Wave B device verification pending).

## Position
- **Wave A (bootstrap) GREEN ON DEVICE**: `PlaybackService` runs in foreground
  (`isForeground=true foregroundId=1`, channel `mdreader_media`, "Ressoa" holder
  visible in the shade, `POST_NOTIFICATIONS granted=true`). Committed `7ca4c1d`.
- **Wave B (Rust→Kotlin mirror publish) implemented + committed** (`1849761`,
  fix `72f369e`): real JNI bridge (`capture_jvm` at the four JNI entries +
  `publish_mirror_to_kotlin` → `ShellHost.publishMirror`); Kotlin `MediaMirror.kt`
  consumes `consumedUnits/totalUnits` (FR-013 rename, Determinate bar) + dismissal;
  lifecycle relay `AppForegrounded/AppBackgrounded` triggers the JVM capture.
  Host contract 6/6 (`src-tauri/tests/test_kotlin_mirror_payload.rs`, pinned in
  `expected-tests.txt` + purity allowlist). APK (universal-release, ~118 MB)
  installed on the device (`53cf9aa7`), app launched with no JNI errors —
  **device verification pending** (waiting for the phone to be free).
- **Gate 5 RESOLVED PROVISIONALLY**: constitution 1.5.0→1.6.0→1.6.1 (Exceptions
  Log entry 11 + correction PATCH); `deny.toml` ignores RUSTSEC-2026-0367/0368/0369;
  `cargo deny check advisories` → "advisories ok". Upstream track complete:
  issue k2-fsa/sherpa-onnx#4058 (filed per the repo bug template), fork
  `lfgranja/sherpa-onnx` branch `fix/rust-zip9-advisories` (commit `5821f23`),
  PR k2-fsa/sherpa-onnx#4059. Local proof the upgrade is a one-line manifest
  bump: `DOCS_RS=1 cargo check` with `zip = "9"` (resolved 9.0.4) compiles the
  unmodified `build.rs`, exit 0. Undo = delete the 3 ignore blocks + entry 11
  together when a `zip >= 9` sherpa-onnx release lands.
- Gates 1–4 + 8-shell PASS on the Wave B tree; manifest + shell-purity PASS.
- T051 (device trials) + T052 (diag scan) pending; the picker (Wave C) is not
  yet wired, so trials currently need the pre-loaded document route.
- `gate-8-full: DEFERRED (owner=synthesis)` — unchanged (Exceptions Log 8).

## Next
1. **Wave B device verification** (when the phone is free): bring the app to the
   front, press Play, verify logcat clean (`kotlin mirror publish failed` /
   `no entry has captured the JVM handle yet` absent), `dumpsys notification
   --noredact` shows title + section + progress, shade + lock-screen capture.
2. **Wave C — picker trigger**: in-app Open control → Tauri command → Kotlin
   picker (`DocIngest.pickDocument` / `handlePickResult`; gen edits host-local).
3. Full `scripts/verify.sh` re-run (gates 1–7 + 8-shell) to confirm all green.
4. T051 sheets (`tests/device/trials-004.md`) to the user; T052 diag scan after.

## Commits this session (004)
`92129da` (T048–T050 done) · `7ca4c1d` (Wave A bootstrap) · `1849761` (Wave B
JNI publish) · `72f369e` (JNI result fix) · `18dd86a` (entry 11 PROVISIONAL) ·
`7a25821` (v1.6.1 correction + upstream links).

## Verification summary
- Host: gates 1–4 + 8-shell green on the Wave B tree; 6 new tests pinned.
- Device: Wave A verified on-device; Wave B runtime verification pending.
- Upstream: PR #4059 open (no upstream CI on PRs — `test-rust` runs on master);
  `zip 9` source-compatible, so the port needs no `build.rs` change.
