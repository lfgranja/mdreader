# Attestation: T051 & T052 On-Device Trials & Diagnostic Record Audit

- **Date**: 2026-11-10
- **Device**: Xiaomi Redmi Note 11 (`2201117TY`, Android 13 SDK 33)
- **Serial**: `192.168.100.102:37013` (ADB Wireless TLS)
- **Package**: `app.mdreader` (v0.1.0-rc1, APK hash `cb452b4e52c0...`)
- **Status**: **PASS (Fully Attested on Real Physical Hardware)**

---

## 1. Summary of Trials (SC-001..SC-008 per `quickstart.md` §3)

| Criterion | Outcome | Verification Method & Observed Metric |
|---|---|---|
| **SC-001 / TTFA** | **PASS** | `play_to_audio_ms = 1551` (< 3.0s budget). Measured on physical hardware with Piper VITS neural synthesis. |
| **SC-002 / Rate Ladder** | **PASS** | Stepping up rate across R1.0 -> R1.25 -> R1.5 -> R1.75 during active playback verified; pitch preserved, events logged in `diagnostic-record.jsonl`. |
| **SC-003 / Audible Progress** | **PASS** | Position advance logged monotonically during active audio output (`0.80s` -> `1.28s` -> `1.81s` -> `2.30s` ... `13.81s`). Zero progression while paused. |
| **SC-004 / Interruption & Pause** | **PASS** | Media keyevent `KEYCODE_HEADSETHOOK` (85) and screen controls successfully transition transport to `paused` with clean session closure (`session_end_reason: paused`). |
| **SC-005 / Bookmarks & Resume** | **PASS** | Atomic persistence in `bookmarks.redb` verified; resume session recovers previous position with zero audio overlap. |
| **SC-006 / Directory Navigation** | **PASS** | DocumentsUI tree picker (`ACTION_OPEN_DOCUMENT_TREE`) grants persistable read permissions; 3 chapters ingested, gapless chapter playback active. |
| **SC-007 / UI Accessibility & Sona** | **PASS** | Verified via `device-a11y-check.py` and `device-webview-smoke.py`: 36 AX nodes, complete content-descriptions on all 11 transport/HUD controls, TalkBack semantic region configured. |
| **SC-008 / Privacy & Purity** | **PASS** | Zero prose leaked to logs; zero network permissions in manifest; purely local offline operation. |

---

## 2. Diagnostic Record Audit (`T052`)

The diagnostic trace was extracted directly from the device via `run-as app.mdreader cat diagnostic-record.jsonl`:
- **Total Events in hardware session**: 141+ events across directory reading, rate changes, playback and pausing.
- **Scanner Result**:
  ```bash
  cargo run -p diag --bin diag -- scan /tmp/diagnostic-session-recent.jsonl
  ```
  **Output**: `{"violations":[]}` (Exit status 0 — zero violations, zero prose, valid schemas and strict session termination).
