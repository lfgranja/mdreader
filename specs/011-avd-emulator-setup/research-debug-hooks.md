# Research: Debug-only/Test Hooks for SAF Automation (T184/T226)

## Objective
Enable fully ADB-automatable verification of document/directory ingest paths on physical device/emulator without human SAF picker interaction, while preserving production security and exercising the same production ingest/open code path.

## Reliable sources (official/high)
- **Android Developers — Test and debug features** (security): test/debug features in production builds are a security risk; keep debug/test behavior out of release. https://developer.android.com/privacy-and-security/risks/test-debug
- **Android Developers — Build instrumented tests / Espresso setup**: instrumented tests run in debug/test contexts; Espresso/AndroidX Test support automation. https://developer.android.com/training/testing/instrumented-tests, https://developer.android.com/training/testing/espresso/setup
- **Android Developers — ADB / activity manager / intents**: `adb shell am start` can launch activities with extras/data for automation; exported components increase attack surface and must be carefully scoped. https://developer.android.com/tools/adb
- **Tauri v2 — Develop/Debug/Mobile**: Tauri Android projects integrate custom Kotlin/Android sources and Gradle; debug workflows are supported. https://v2.tauri.app/develop/, https://v2.tauri.app/develop/debug/
- **Android manifest merge (debug source set)**: resources/manifests in `src/debug` merge only into debug builds — appropriate for debug-only exported test activities.

## Cross-language/security guidance (synthesis)
- Keep automation hooks **debug-only**: runtime guard (`BuildConfig.DEBUG`) + debug-only manifest export (`src/debug/AndroidManifest.xml`) + no release exposure.
- Avoid leaking secrets/document content in logs; return minimal result codes.
- Prefer routing through the **same production pipeline** (snapshot/fingerprint/JNI callbacks/Rust open path) rather than a parallel fake path, so tests validate real behavior.
- SAF native picker UI remains OEM/locale-fragile for full UI automation; a debug hook is a pragmatic, low-risk test seam. UI a11y/picker-cancel can be covered by instrumented UI tests where feasible.

## Implementation (this repo)
- `android-shell/src/main/kotlin/app/mdreader/DebugIngestHook.kt` — debug-only helper; routes file/directory paths to `DocIngest` debug helpers and JNI callbacks (`onDocumentIngested` / `onDirectorySelected`) used by production flows.
- `android-shell/src/main/kotlin/app/mdreader/DebugIngestActivity.kt` — exported **only** in `src-tauri/gen/android/app/src/debug/AndroidManifest.xml` (debug merge); guarded by `BuildConfig.DEBUG`; accepts `--es mode document|directory --es path <path>`; writes result to cache/log for automation.
- `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` — minimal internal debug helpers (reuse snapshot/fingerprint/manifest dispatch).
- `scripts/automate-t184-t226-hook.sh` — ADB automation: pushes fixtures (prefers app-internal storage via `run-as` on debug builds), launches hook, captures `debug_ingest_result.txt`/logcat/media/services/screenshot, checks crashes/ANRs, emits report.
- `src-tauri/gen/android/app/src/androidTest/.../DebugIngestActivityTest.kt` — instrumented smoke scaffold (debug build).

## Verification evidence
- Gate 2 (workspace): `nextest-gate2-retry.log` — **1002 passed (5 slow), 2 skipped**.
- Physical device `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp` (2201117TY, SDK 33, arm64-v8a), debug arm64 APK with hook: `t184-t226-hook-20261105T180552Z/REPORT.md` — document and directory hook intents executed, directory ingest evidence captured, no crash/ANR indicators.
- Residual UI-only: SAF picker cancel and TalkBack/keyboard traversal of native picker/title UI remain manual or require UIAutomator/Espresso against system UI.
