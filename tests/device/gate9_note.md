# Gate-9 / T086 — Device-bound verification (DEFERRED — hardware available)

**Status**: Device plugged (adb installed). Server cannot complete gate-9 or T086 because:
- `tauri android init` required (Android project not fully initialized; only stub T028a exists)
- Android SDK / Gradle / NDK not configured in server environment
- Trial requires API-29+ device with audio routing, screen-off, focus, routing

**User action required on development workstation (not server)**:
1. `cd src-tauri && cargo tauri android init` (full Android project)
2. `cargo tauri android build --target aarch64` (gate 9 artifact)
3. Connect plugged Android (API 29+) via USB / ADB
4. `adb install app.apk` + run `tests/device/trials.md` 10/10 per SC

**Evidence in repo (no agent self-attestation)**:
- T024-T031 implemented (subagent-B, different agent/model)
- T019-T022b written RED (subagent-A, different agent/model)
- T087 attestation updated (`VERIFICATION.md` 7a018e1 + device-trial procedure)
- All commits atomic (constraint 6)
