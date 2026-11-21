# Bug Fix: Playback does not start after file selection

- **Slug**: playback-not-starting-after-file-selection
- **Fixed**: 2026-11-07
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Playback did not start after file selection because `ShellHost.onActivityResult` returned early — `currentActivity` was `null` when `onActivityResult` fired. The root cause was in `ShellApplication.kt`: `onActivityPaused` nulled `currentActivity` when the MainActivity paused (which happens when the SAF picker opens). Since Android calls `onActivityResult` BEFORE `onActivityResumed`, `currentActivity` was still null when the picker result arrived, so `handlePickResult` was never called, `onDocumentIngested` was never invoked, and the document was never loaded into the session.

The fix keeps `currentActivity` alive across the picker round-trip by only clearing it in `onActivityDestroyed` instead of `onActivityPaused`.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `android-shell/src/main/kotlin/app/mdreader/ShellApplication.kt` | modified | Removed `currentActivity = null` from `onActivityPaused`; moved cleanup to `onActivityDestroyed` |

## Diff Highlights

```kotlin
// Before (broke ingest on picker return):
override fun onActivityPaused(activity: Activity) {
    if (ShellHost.currentActivity === activity) {
        ShellHost.currentActivity = null  // <-- cleared before onActivityResult
    }
}

// After (keeps currentActivity alive across picker round-trip):
override fun onActivityPaused(activity: Activity) {
    // Keep currentActivity alive across the SAF picker round-trip:
    // onActivityResult fires before onActivityResumed, so clearing
    // here would make the result unreachable (the ingest never runs).
    // Clear only on destroy.
}

override fun onActivityDestroyed(activity: Activity) {
    if (ShellHost.currentActivity === activity) {
        ShellHost.currentActivity = null
    }
}
```

## Tests Added or Updated

- No new automated tests added. The fix is in the Android shell lifecycle, which is validated on device.
- Existing 122 Rust unit tests remain passing (no Rust changes).

## Local Verification

- Commands run: `cargo tauri android build --target aarch64 --debug` → APK built successfully
- Manual checks: Device test — tapped title card → selected t184-teste.md → audio played for ~10-14 seconds (AudioTrack frames sent), AAudio stream opened. "Persistable URI permission granted" and "Snapshot written" confirmed in logcat. Playback started automatically after file selection.
- Before fix: "Invoke get_status_view failed: no document loaded" repeated; no audio. After fix: AAudio stream opened, audio frames sent.

## Deviations from Assessment

The assessment listed multiple possible root causes. The actual root cause was `currentActivity` being nulled in `onActivityPaused`, which prevented `ShellHost.onActivityResult` from processing the picker result. This was not explicitly listed in the assessment's root cause hypotheses but is consistent with the "ingest never runs" symptom.

## Follow-ups

- Consider using the Activity Result API (registerForActivityResult) instead of the deprecated onActivityResult pattern
- Add a null-safety check in `ShellHost.onActivityResult` that uses the activity parameter instead of `currentActivity`
- Monitor for cases where the activity is actually destroyed (not just paused) during picker flow
- Keep `src/status_view.html` and `src-tauri/src/assets/index.html` in sync


## 2026-11-21 — REGRESSÃO REABERTA no build `bf7971b6` (device 2201117TY)

Usuário abriram "Hamlet trecho.md" pelo app de arquivos (ACTION_VIEW, URI `content://media/external/file/1000848919`). Evidência ADB:
- UI mostra `document:1000848919.md` em vez de "Hamlet trecho.md" — `queryName` (DocIngest.kt:638-643) não resolve `OpenableColumns.DISPLAY_NAME` para URI MediaStore; `effectiveUri` cai em `content://media/external/file/1000848919` (linha 748).
- `diagnostic-record.jsonl`: `session_start doc=c6c1f42ffcfaa751a49f5d19b5673081ad2d56fe614bba7827185580759b074b`, `action=play`, depois 8+ eventos `action=progress` com **`position_secs: 0.0`** e nenhum `play_to_audio_ms` — PositionClock corretamente não avança sem áudio audível, mas nenhum áudio é produzido.
- MediaSession `active=true`; sem FATAL/ANR no logcat.
- Contraste: sessão via ACTION_SEND no mesmo APK avançou de 0.16→10.4s com `play_to_audio_ms=8827`. Logo o problema é específico do caminho ACTION_VIEW/MediaStore, não do motor de síntese.

Hipótese: URI `content://media/external/file/<id>` sem FLAG_GRANT_READ_URI_PERMISSION e sem display name → DocIngest ingere bytes fallback mas o fingerprint bate com bookmark restaurado (c6c1f42f…) → cold-start `AwaitOffer` estaciona o playback (T233, `offer_route=in_app_dialog`), e a UI não exibe o diálogo porque o payload não foi publicado corretamente para ACTION_VIEW. [NV] confirmar oferta pendente.
