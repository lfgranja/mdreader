# Session state — 004-ressoa-dogfood-build (implementation + HOST GREEN)

Updated: 2026-10-16 (T048 host gates, T049 manifest/purity, T050 gate 9 all green; T051/T052 device-blocked).

## Position
- **T048 VERIFICATION GREEN** (`target/verify-004-final.log`, `VERIFY_EXIT=0`):
  gates 1 (format) → 2 (nextest, vs `expected-tests.txt`) → 3 (doc) → 4
  (lint-drift) → 5 (supply-chain: `cargo deny` + `cargo audit`, rustls→0.23.46)
  → 6 (deny-list) → 7 (closed-set, no `_ =>`) → 8-shell (pipeline, shell bounds) all PASS.
  `gate-8-full: DEFERRED (owner=synthesis)` per constitution Exceptions Log 8
  (SC-044 closure not claimed). Full discharge still gated on the 002 synthesis owner.
- **T049 PASS** (scripts report 0 unresolved): `check-manifest.sh` PASS
  (service `mediaPlayback`, `FOREGROUND_SERVICE*`, `VIBRATE`, `POST_NOTIFICATIONS`, minSdk 29);
  `check-shell-purity.sh` PASS (135 tokens resolved, 1 exempted).
  Naming-boundary guard tightened (`src-tauri/tests/test_noscope.rs`); 6/6.
- **T050 GREEN — device artifact**: `cargo tauri android build --target aarch64`
  exit 0 → `app-universal-release.apk` (113 MB) + `.aab`.
  SHA-256 `cc5f08d55ccfb2ba8bfbeb2ddfcd9d08e4126f08af9dd771e1b946aef430f328`.
  `libsrc_tauri.so` exports 4 native entry points and the Kotlin `external fun`s
  resolve (incl. `_00024`-mangled `DocIngest_00024Blake3Hasher_nativeBlake3`).
- **ADB BLOCKED**: the reference phone is not attached (`adb devices` empty).
  Install (`adb install -r`) + hash re-verification on device (T050 tail)
  + trial pulls (T052) are pending re-plugging the phone with USB debugging on.
- Tasks `[X]`: T048, T049, T050. Open: T051 (on-device trials, human-run, 7 × 30 min
  locked commute, 10/10 per SC class), T052 (`diag scan` on the pulled JSONL).

## Next
1. Re-plug reference phone (`2201117TY`, USB debugging ON) → `adb install -r <apk>` → `adb shell run-as app.mdreader cat files/diag.jsonl` only after trials.
2. User runs T051 with `tests/device/trials-004.md` sheet (app label = "Ressoa" on the launcher).
3. T052: `adb pull` + `diag scan` (conforming baseline + negative control planted in a copy).
4. **Scope fact (reported, not fixed)**: the Rust→Kotlin mirror *publish*
   transport that feeds the lock-screen notification (title + section +
   progress) is still stubbed and has no caller — `publish_mirror_to_kotlin`
   is a no-op, `ShellHost.publishMirror` is only declared.
   Now unblocked because the jni 0.22 blocker was just resolved (jni 0.21
   alignment), and `src-tauri/src/lib.rs` records both the deferral reason
   (FR-024/002 synthesis feature) and `startForegroundHolder()` placeholder
   standing in for it. Required inputs (surface resolver + transient
   dismissal + haptic id) would need to be threaded into
   `publish_mirror_to_channel`. Treat as a 002/phase-12 follow-up unless the
   user opts to promote it into 004; `quickstart.md` §3 notification criteria
   cannot pass without it. 001 VERIFICATION.md §11 lists this as an unowned
   follow-up (no task own it).

## Commits this session (004)
`b3f6d48` (T019 JNI bytes-forward + FR-013 units) · `b617df6` (prop warnings) ·
`62ee013` (T040b/T046) · `f6e25ed` (T049 manifest/purity) · `af39a29` (rustls
lock bump) · `06a87ce` (T048 gate 7, 2 catch-alls) · `fec3816` (T050 naming
boundary: Ressoa on painted surfaces) · `43e12fc` (T050 gate 9: jni 0.21
alignment + JNI signatures).

## Verification summary
- cargo nextest `--workspace` = 769 passed (gates 1–2). Carried warnings: none
  new. fmt + clippy repo-wide pre-existing lint issue untouched (verify.sh
  does not run clippy).
- Gate 9 APK installed-not-yet-possible: device detached.
- Constitution gate 8 (full, byte budget) still DEFERRED — owner=synthesis
  (SC-044); host surface stays `gate-8-shell: PASS`.
- Tests as proof of honesty: every RED/GREEN pair in this session
  (prop_ladder streams_input, naming-boundary guard ×2, diag scan ×3,
  gate-7 catch-all removal ×2) ran before the corresponding fix.
- Host clippy repo-wide remains pre-broken (root Cargo.toml priority error +
  engine-core/cli glue); out of scope per AGENTS.md; not a verify.sh gate.