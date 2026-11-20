# Idea Research: Android 16 Accessibility Compliance (021)

- **Slug**: android-16-accessibility-compliance
- **Created**: 2026-11-20
- **Evidence confidence (overall)**: high (static audit measured directly)

## Users & Demand

- TalkBack users of Ressoa depend on correct live-region/pane announcements; the concern is that Android 16 (API 36) may require `setAccessibilityPaneTitle`/`setAccessibilityLiveRegion` instead of legacy `announceForAccessibility`/`TYPE_ANNOUNCEMENT` — [source: `spec.md` §1] (confidence: high)
- Spec 005 already attests TalkBack 10/10 on the current surface — [source: `spec.md` §2.3] (confidence: high as recorded evidence)

## Prior Art

- 10 Kotlin shell files audited (`ShellHost.kt`, `MediaMirror.kt`, `Residency.kt`, `FocusMirror.kt`, `RoutingMirror.kt`, `DocIngest.kt`, `ShellApplication.kt`, `PlaybackService.kt`, `DebugIngestActivity.kt`, `DebugIngestHook.kt`) — [source: `spec.md` §2.1] (confidence: high)
- Kotlin shell is a strict mirror (SC-035): playback/transport/position/rate logic lives in Rust, Kotlin only renders published JSON — [source: `spec.md` §2.1] (confidence: high)
- Existing assessments/bugs/specs contain no Android 16 accessibility migration item — [source: `spec.md` §2.4] (confidence: high)

## Market & Context

- Cost of doing nothing: if legacy APIs existed, an API-36 target could degrade TalkBack behavior; the audit closes that uncertainty with no code churn.

## Data & Constraints (measured 2026-11-20)

- `grep -rnE 'announceForAccessibility|TYPE_ANNOUNCEMENT|AccessibilityPaneTitle|AccessibilityLiveRegion' android-shell/src/main/kotlin/app/mdreader/` → **no matches** (confidence: high).
- `src-tauri/gen/android/app/build.gradle.kts`: `compileSdk = 36` (line 12), `minSdk = 29` (line 17), `targetSdk = 36` (line 18) (confidence: high).
- `src-tauri/src/status_view.html:504`: `<div class="sr-only" role="status" aria-live="polite" id="liveRegion" aria-atomic="true"></div>`; additional `aria-label`, `aria-labelledby`, `role="button"`, `tabindex="0"`, `role="status"`, `aria-modal` present (confidence: high).
- Manifest declares `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_MEDIA_PLAYBACK`, `POST_NOTIFICATIONS`, `WAKE_LOCK`, `VIBRATE`; **no `INTERNET`** (air-gapped, T106) — [source: `spec.md` §2.2] (confidence: high).
- Tasks in `specs/021-android-16-accessibility-compliance/tasks.md`: 4 concluded verification tasks, 1 pending (gates execution) (confidence: high).

## Evidence Against the Idea

- There is no migration work to perform; creating migration code would be churn against the mirror-only contract. Mitigation: this spec is a verification record with explicit no-code scope. (confidence: high)
- Gate 9 (Android debug build) is environment-dependent; passing API-36 configuration is not proof of a successful build. Mitigation: [NV] marker. (confidence: medium)

## Gaps & Open Questions

- Gate 9 build result [NV] — gate owner to execute.
- Device TalkBack re-check not part of this verification; prior attestation remains the reference.

## Sources

- `specs/021-android-16-accessibility-compliance/spec.md`, `plan.md`, `tasks.md` (host: local repo, policy: allowlisted path)
- `android-shell/src/main/kotlin/app/mdreader/`, `src-tauri/gen/android/app/build.gradle.kts`, `src-tauri/src/status_view.html` (host: local repo)
- `ROADMAP.md` (host: local repo)
