# Research: Debug-only/Test Hooks for SAF Automation (T184/T226)

## Objective
Enable 100% ADB-automatable verification of document/directory ingest paths on physical device/emulator without human SAF picker interaction, while preserving production security.

## Sources (reliability: official/high)
- Android Developers — *Test and debug features* (security risk): test/debug features in production builds are a security risk; debug-only behavior must not ship in release. https://developer.android.com/privacy-and-security/risks/test-debug
- Android Developers — instrumented testing / Espresso setup guidance for `androidTest` and debug variants. https://developer.android.com/training/testing/instrumented-tests / https://developer.android.com/training/testing/espresso/setup
- Tauri v2 Android/mobile development docs (project structure, Kotlin integration, debug workflows). https://v2.tauri.app/develop/ and https://v2.tauri.app/develop/debug/
- Android manifest merge: `src/debug/AndroidManifest.xml` merged only into debug builds — appropriate for exported test-only activities.

## Design (implemented)
- **`DebugIngestHook` / `DebugIngestActivity`** (in `android-shell/src/main/kotlin/app/mdreader/`, compiled into app): debug-only automation entrypoint.
  - Guard: `BuildConfig.DEBUG` checked at runtime; refuses in release (`refused:not-debug`).
  - Exposure: exported activity declared only in `src-tauri/gen/android/app/src/debug/AndroidManifest.xml` (debug source set) with `android:exported="true"`, `excludeFromRecents`, `noHistory`, translucent theme, action `app.mdreader.action.DEBUG_INGEST`.
  - Routing: reads file/directory from `--es path` (or URI extras where applicable), snapshots bytes, computes fingerprint via existing `DocIngest` path, and invokes the same JNI callbacks as production flows (`onDocumentIngested` / `onDirectorySelected` via `DocIngest` debug helpers) so Rust ingest/open behavior is exercised.
  - Directory hook mirrors scan semantics (root files + 1st-level subfolders, natural order, byte cap/extension filtering) and dispatches manifest incrementally as production scan does.
- **Security**: no release manifest export; runtime debug guard; no document content logged beyond paths/result codes; no INTERNET permission.
- **Limitations**: bypasses *native SAF picker UI* (intentional). T184 Scenario 3 (SAF cancel) and Scenario 4 (TalkBack/keyboard traversal of picker/title UI) remain UI/a11y-specific; the programmatic ingest/open path is automated. Full SAF UI E2E would require instrumented tests with UI Automator/Espresso against system picker (OEM/locale fragile) or dedicated test hooks in UI.

## Instrumented test (scaffold)
- `src-tauri/gen/android/app/src/androidTest/.../DebugIngestActivityTest.kt`: smoke test launching `DebugIngestActivity` with missing path; validates debug activity launch/finish path.

## Automation script
- `scripts/automate-t184-t226-hook.sh`: pushes fixtures, launches app, invokes debug hook for document/directory, captures logcat/media/service/screenshot/UI, checks for crashes/ANRs, emits report.
