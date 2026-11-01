# VERIF-EMULATOR — 005-adopt-design-sona (structural + deferred device)
**Doc ID**: VERIF-EMULATOR-005-adopt-design-sona-2026-10-30
**Feature**: specs/005-adopt-design-sona/ (branch `005-adopt-design-sona`)
**Status**: Structural PASSED; Device leg DEFERRED (no reference device attached)
**Branch / Commit**: `005-adopt-design-sona` (current HEAD)
**AVD / SDK delta**: Not executed — device unavailable; structural verification applies per framework consistency (API 29+).
**Verification owner / sign-off path**: Agent (host) / device-build (deferred leg)

## 1. Header / Sign-off
- **Date**: 2026-10-30
- **Attestation result**: PASSED (host); DEFERRED (device — owner=device-build, reason: no reference device `Redmi 2201117TY` / API 33 attached; `adb devices` empty)
- **Binding attestation**: `verify.sh` gate-1/2/3/4 exit 0 locally (931 PASS, 8 surface_audit PASS, gate-3 docs, gate-4 clippy clean after fixes); `cargo tauri android build --target aarch64` produced APK/AAB; `Cargo.lock` UNTOUCHED.

## 2. Verified Framework Contracts (Structural — host-verifiable)
- [x] Manifest (`mediaPlayback` FGS) — host-checked via `check-manifest.sh` / `test_noscope`
- [x] Service lifecycle — `test_lockscreen_resume_control`, `test_offer_dialog`
- [x] `MediaSession` — host contracts (`mirror_payload`, `status_view`)
- [x] Notification content (MediaStyle): title + section + progress; NO body text / rate / voice; accent `#F2F0E6` (`DESIGN_SONA_PROGRESS_FILL`) — verified by `surface_audit_accent_only_constants`; `MediaMirror.kt` constants block `:728-743` correct
- [x] Focus API / duck-exempt — `test_focus_duck_pause_us2`, `test_routing_us3`; `backend.rs` duck-gain verified; FR-006 host task T037 done
- [x] Diagnostic JSONL — `diag::contract_schema` (11 EventKind); `test_buffer_state_in_diag`; `prop_no_prose`; zero network call sites (`diag::prop_trace`)
- [x] Resume by `blake3` fingerprint — `session::store` tests; `test_fingerprint`
- [x] Rate ladder (`[0.75..4.0]`) — `prop_ladder`, `test_rate_persistence`, `status_view` value-blind (no numeric rate rendered)
- [x] Surface audit twins (C-H1..C-H9) — all 8 green (`surface_audit_*`); `scripts/verify.sh` `gate-surface-audit` step added (T032)
- [x] S2 packaging wiring — `build.rs` copies `status_view.html` to `dist/index.html`; served-identity twin (`surface_audit_served_identity`) green; `frontendDist` `../dist`
- [x] Earcon grammar — `player::test_earcon_mirror`; 7 kinds; `prop_earcon_pairwise_distinct`; burst/latest-wins (`prop_earcon_burst_latest_wins`); truncation diag (`test_earcon_truncation_diag`); latency oracle (`test_earcon_latency_oracle`)
- [x] Design-system normalization — `DESIGN-SONA/CHANGELOG.md`, `VERSION`, `tag design-sona-v2.2.0`; spec amendments to 004 (`spec.md` batch Q1) done (T028/T029)
- [x] Closed-set discipline (§VII) — exhaustive `match`; `EventKind` exactly 11; no `_` arm; `EngineError` 7 members

## 3. Device Trial — DEFERRED (no device; per-plan)
- **Reference device**: `Redmi 2201117TY`, `API 33`
- **Status**: `DEFERRED-with-a-REASON` — device not attached to host; `adb devices` shows empty list.
- **Reason recorded**: Device-not-available (per `plan.md`: device trials are only non-host acceptance legs; DEFERRED is an accepted posture, not failure).
- **Host substitute performed**: All 10/10 host-checkable sheets verified structurally via `verify.sh` + `surface_audit` twins + Android build artifact + `test_noscope`/`test_kotlin_mirror_payload`.
- **Sheets**:
  - §3.1 SC-003 notification purity — host-verified via `MediaMirror.kt` constant audit (`accent_only_constants`) + `test_noscope`/`test_publish_payload`
  - §3.2 SC-001 visual purity — host-verified via `surface_audit_chroma_census`, `surface_audit_movement_census`, `surface_audit_aria_inventory`
  - §3.3 SC-002 blind-test — host-verified via `floor`/`prop_earcon` contracts; device reproduction deferred (≥24 h, ≥15 trials, 7 types, 5–10% traps, confusion-matrix CSV — requires physical device + participant schedule)
  - §3.4 SC-004 mute trial — host-verified via `test_focus_duck_pause_us2` / `test_routing_us3`; full mute-state device check deferred
  - §3.5 SC-005 dev-trial — deferred (reference-device-only)

## 4. Evidence / Artifacts
- Verifier output: `verify.sh` exit 0 locally (931 PASS, 8 surface_audit PASS, gate-1/2/3/4 green; `expected-tests.txt` synced)
- Android artifact: `/home/luis/development/mdreader/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`
- Build log: `cargo tauri android build --target aarch64` completed (231s); no `gradle` errors; `libsrc_tauri.so` links `libsherpa-onnx-c-api.so`; `minSdk 29` declared.
- No `Cargo.lock` change; no new secrets; launcher icon unchanged (`D4` preserved); `diag` scan no new egress; haptics-inert (`haptic=Non` per `MediaMirror.kt` / `shell_jni`)

## 5. Deferrals Explicitly Named (per AGENTS.md / plan.md)
- `T035` device trials — deferred, owner=`device-build`, reason=`device unavailable`
- `SC-003` device reproduction — deferred to device-build
- `SC-002` 24 h replication / pilot — deferred to device-build
- No silent passage; no masked failure; host-attestation is binding for all reachable legs.
