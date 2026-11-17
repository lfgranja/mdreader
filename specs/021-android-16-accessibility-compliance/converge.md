# Converge: 021 Android 16 (API 36) Accessibility Compliance

Status: **CONVERGED** (verification-only spec; no code changes required).

## Implementation Summary

Spec 021 is a verification record, not a code change. The ROADMAP NEXT item "Migrar chamadas residuais de `announceForAccessibility` no shell Kotlin para API 36" is already satisfied: the Kotlin shell is a strict mirror (SC-035) with no accessibility-announcement logic, and `targetSdk`/`compileSdk` are already 36.

### Evidence (verified this cycle, read-only)

- `src-tauri/gen/android/app/build.gradle.kts`: `compileSdk = 36` (line 12), `targetSdk = 36` (line 18), `minSdk = 29` (line 23).
- `android-shell/`: zero references to `announceForAccessibility` / `TYPE_ANNOUNCEMENT` / `AccessibilityLiveRegion` / `AccessibilityPaneTitle` / `setAccessibilityPaneTitle` / `setAccessibilityLiveRegion` (grep across all 10 Kotlin files).
- `src-tauri/src/status_view.html`: `aria-live="polite"` present (line 499) with `role="status"`, `aria-atomic="true"`, plus `aria-label`/`aria-labelledby`/`role="button"`/`tabindex="0"`/`aria-modal` surface.
- Manifest permissions: `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_MEDIA_PLAYBACK`, `POST_NOTIFICATIONS`, `WAKE_LOCK`, `VIBRATE`; no `INTERNET` (air-gapped, T106).

### Gate 9 (Android build)

- Command: `cargo tauri android build --target aarch64 --debug` (run during this cycle).
- Outcome: **PASS** — Rust library compiled for aarch64 (minSdk 29) AND Gradle APK packaging completed. Artifacts:
  - APK: `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk` (210 MiB)
  - SHA-256: `75104ca62c08239ab71ca9c770c416d21c0f041e221a0e306e5cf8f2c5fd5836`
  - AAB: `src-tauri/gen/android/app/build/outputs/bundle/universalDebug/app-universal-debug.aab`
- No Kotlin/manifest code changed; build plumbing only. Gate 9 fully green.

### Non-Changes

- No migration code created (nothing to migrate).
- No manifest edit (targetSdk 36 already via build.gradle.kts).
### Unresolved / Deferred
- Gradle APK packaging completed (this cycle, PASS); signed release build still requires authorization (keystore/credentials) — tracked as open blocker in `.specify/validations/v0.1.0-rc2/validate.md`.
- TalkBack full gesture automation remains blocked on secure-settings capability (carried from `device-webview-a11y-gates` validation).

### Acceptance Criteria Status

- [x] `targetSdk = 36` confirmed in `build.gradle.kts`.
- [x] Zero references to `announceForAccessibility`/`TYPE_ANNOUNCEMENT` in shell Kotlin.
- [x] Frontend maintains `aria-live` and ARIA attributes.
- [x] Gate 9 (Android debug build) — Rust compile PASS + Gradle APK packaging PASS (APK sha256 75104ca62c08239ab71ca9c770c416d21c0f041e221a0e306e5cf8f2c5fd5836). Signed build remains authorization-gated.
