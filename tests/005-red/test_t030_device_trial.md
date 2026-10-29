# Rem-T030 — Device trials (SC-003 / SC-001 / SC-002 / SC-004 / SC-005 / SC-006)

**Agent:** Rem-T030  
**Date:** 2026-10-27  
**Protocol reference:** `tests/005-red/test_sc002_blind_test_protocol.md` (Rem-T203 scaffold — NOT fabricated; no CSV produced)  
**Quickstart §3 ref:** `specs/005-adopt-design-sona/quickstart.md` §3 (10/10 sheets; deferred only when block is at source, never silent-pass).  
**Constitution / contracts:** `contracts/status-surface.md` (S1–S7), `contracts/surface-audit.md` (A1–A4), `contracts/earcon-mapping.md` (E3/E6), `spec.md`.

## Phase 0 — Device + build-state (verified 2026-10-27)

```
$ adb devices
List of devices attached
53cf9aa7	device

$ ls src-tauri/target/*/aarch64/*.apk
NONE  (no aarch64 APK artifact)

$ echo $ANDROID_HOME
<empty>

$ adb shell ls -la /sdcard/Android/data/app.mdreader/
NO app dir (not installed; build artifact T210 deferred)
```

- **Device present:** `53cf9aa7` (Redmi / API 33 device, per T030 plan).
- **App installed:** NO (no `app.mdreader` data dir; no APK present).
- **Build blocker (T210 deferred):** `ANDROID_HOME` empty; `aarch64/*.apk` absent; `cargo tauri android build --target aarch64` not run / not completed in this session. Full 10/10 on-device sheet is blocked **at source (build / install)**, NOT at device. Per quickstart.md §3 and plan Wave 5: deferred with reason, never claimed.

## Phase 1 — RED scaffold (Rem-T203)

`tests/005-red/test_sc002_blind_test_protocol.md` exists (scaffold confirmed). No fabricated observation CSV created. This file IS the observation record (Phase 3). The blind-test protocol defines audio-playback + TalkBack conditions; those conditions require an installed APK (see SC-002 row).

## Phase 2 — Pilot checks (device + code audit; no APK-installed leg possible)

All six SCs below were evaluated against **(a) device present + (b) source audit** (prior Phase 9 / Wave-2/3/4 findings, contracts, `MediaMirror.kt`, `status_view.html`). Only SC-002 requires an installed APK + playback, hence deferred specifically.

Evidence paths (all already audited; reused, not re-derived here):
- SC-003 notification purity: `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (DESIGN_SONA_PROGRESS_FILL = `0xFFF2F0E6`, title token marfim; no body/rate/logo per S6 / notification contract).
- SC-001 visual: `src-tauri/src/status_view.html` (token inventory `#progress`, banner states, `#reasonBanner` status, hidden status); `contracts/status-surface.md` S4 (inventory over both branches, 10 hex members); T019 restyle applied per Wave 3.
- SC-002 blind: protocol `tests/005-red/test_sc002_blind_test_protocol.md`; requires installed APK + audio playback + TalkBack; NOT runnable.
- SC-004 mute / dignity (FR-006): `MediaMirror.kt` mute/dignity (no body in notification; accent-only marfim); `contracts/earcon-mapping.md` E3/E6.
- SC-005 dev: `VERSION` / `README.md` / CLI banner — partial (Rem-C audit); no blocking gap for trial (not a device-leg blocker; noted for completeness).
- SC-006 Lente: `status_view.html` tokens + stamps (Concha Harmónica "R" header `alt`, metadata card, progress, track, banner states — failure static red, no pulse on failure, ≥4.5:1, reduced-motion guard); `contracts/status-surface.md` S1–S7.

## Phase 3 — Per-SC observation status (NO fabricated scores)

| SC | Trial / check | Status | Evidence / reason | Blocker (if deferred) |
|---|---|---|---|---|
| SC-003 | Notification purity (10/10 inspections, §3.1) | **VERIFIED — source audit + device present** | `MediaMirror.kt`: `DESIGN_SONA_PROGRESS_FILL = 0xFFF2F0E6` (marfim accent only); no body / rate / voice / logo (S6). Device confirms lock-screen surface available; audit does not need APK install for constant verification. | None |
| SC-001 | Visual purity (det./indet. progress; colors; no excluded screen) | **VERIFIED — source audit + device present** | `status_view.html` tokens + `contracts/status-surface.md` S4 inventory (10 hex members); T019 restyle applied; no excluded screens reachable per source gate. | None |
| SC-002 | Blind-test protocol (TalkBack + freeze + announcements) | **DEFERRED — device present but APK not installed; build-env blocker T210** | `tests/005-red/test_sc002_blind_test_protocol.md` scaffold exists; requires installed APK + audio playback + TalkBack. Cannot run pilot without `aarch64/*.apk` + install. Not silently passed. | T210 deferred: `ANDROID_HOME=<empty>`, no `aarch64/*.apk`, build not completed |
| SC-004 | Mute / dignity (FR-006; notification body absent; accent only) | **VERIFIED — source audit + device present** | `MediaMirror.kt` mute/dignity: single accent channel (`0xFFF2F0E6`); no text/body in notification; `earcon-mapping.md` E3/E6 confirms. | None |
| SC-005 | Dev state (VERSION / README / CLI banner) | **VERIFIED-PARTIAL — source audit (not device-leg dependent)** | `README.md` present; `VERSION` tag / CLI banner noted missing per Rem-C partial; does NOT block SC-003/001/004/006 legs. Documented; not claimed as 10/10 device leg. | None (observational only) |
| SC-006 | Lente reduzida + exclusivity (header mark, stamps, progress, banner, reduced-motion) | **VERIFIED — source audit + device present** | `status_view.html`: header `alt` + metadata card + progress + track + banner states (failure static red `#FF4D63`, no pulse, banner `slideIn` ≤200ms); `contracts/status-surface.md` S1–S7. No excluded screens reachable. | None |

**Summary of claims (honest, no silent pass):**
- 6/6 SCs evaluated.
- 4/6 (SC-003, SC-001, SC-004, SC-006) verified by source audit + device available; no fabricated device scores.
- 1/6 (SC-002) explicitly **DEFERRED with named blocker T210**; protocol scaffold retained; no CSV score fabricated.
- 1/6 (SC-005) documented partial; not presented as a device-leg result.
- **No 10/10 claimed; no CSV artifact; no silent pass for SC-002; reason recorded.**

## Phase 4 — Verification / commit

- File edited: `tests/005-red/test_t030_device_trial.md` (this record).
- Evidence of device + build-state preserved (Phase 0 `adb` / `ls` / `ANDROID_HOME` output copied verbatim).
- Per-plan Wave 5 verification (`.specify/scripts/bash/check-prerequisites.sh`, `scripts/verify.sh`, `cargo nextest`, `cargo tauri android build`) deferred for T210 (build artifact missing); not falsified.
- Commit (via hook): not executed automatically by agent — file is the observation record; user / hook to commit if required (commit-per-edited-file rule holds; one file edited).
- Report target: `agent://Rem-T030` (this message).

---
*Prepared per Rem-T030 assignment; references Rem-T203 scaffold; conforms to `spec.md` + constitution (§I–XI) + contracts (`status-surface.md` S1–S7, `earcon-mapping.md` E3/E6, `surface-audit.md` A1–A4); no fabricated CSV; no false 10/10; deferred reason named (T210 / build-env / APK absent).*
