#!/usr/bin/env bash
# automate-t184-t226-adb.sh — Best-effort ADB automation for T184/T226 + evidence.
# SAF native pickers (ACTION_OPEN_DOCUMENT/TREE) cannot be fully automated without
# app test hooks or instrumented UI tests with stable selectors across OEMs/locales.
# This script: prepares fixtures, collects evidence, attempts UI introspection, and
# documents exactly what remains manual or requires a test hook.
set -euo pipefail
SERIAL="${1:-adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp}"
PACKAGE="${2:-app.mdreader}"
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"
TS=$(date -u +%Y%m%dT%H%M%SZ)
OUT="./t184-t226-evidence-$TS"
mkdir -p "$OUT"
adb() { "$ADB" -s "$SERIAL" "$@"; }

echo "T184/T226 ADB evidence run $TS serial=$SERIAL" | tee "$OUT/summary.txt"

# Device/app state
adb shell getprop ro.product.model ro.build.version.sdk ro.product.cpu.abi > "$OUT/device.txt" 2>&1 || true
adb shell pm path "$PACKAGE" > "$OUT/pm_path.txt" 2>&1 || true
adb shell dumpsys package "$PACKAGE" | grep -E "versionName|versionCode|installer" | head -10 > "$OUT/version.txt" 2>&1 || true

# Fixtures (already pushed by device validation; ensure present)
adb shell "ls /sdcard/Download/mdreader-test-fixtures 2>&1; ls /sdcard/Download/mdreader-test-fixtures/md-dir 2>&1" > "$OUT/fixtures.txt" 2>&1 || true

# Launch and capture
adb logcat -c >/dev/null 2>&1 || true
adb shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
adb shell am start -W -n "$PACKAGE/.MainActivity" > "$OUT/am_start.txt" 2>&1 || true
sleep 4
adb shell pidof "$PACKAGE" > "$OUT/pid.txt" 2>&1 || true
adb logcat -d -t 500 > "$OUT/logcat.txt" 2>&1 || true
adb shell dumpsys media_session > "$OUT/media_session.txt" 2>&1 || true
adb shell dumpsys notification --noredact > "$OUT/notification.txt" 2>&1 || true
adb shell dumpsys activity services "$PACKAGE" > "$OUT/services.txt" 2>&1 || true
adb shell screencap -p /sdcard/t184t226.png >/dev/null 2>&1 && adb pull /sdcard/t184t226.png "$OUT/screen.png" >/dev/null 2>&1 || true
adb shell uiautomator dump /sdcard/window_dump.xml >/dev/null 2>&1 && adb pull /sdcard/window_dump.xml "$OUT/ui.xml" >/dev/null 2>&1 || true
adb shell dumpsys window | grep -E 'mCurrentFocus|mDreamingLockscreen|mFocusedApp' > "$OUT/window_focus.txt" 2>&1 || true

# Best-effort: if keyguard not blocking, try to tap likely center areas (non-destructive) and dump
# NOTE: Tauri WebView content is not exposed as standard Android nodes; UIAutomator cannot
# reliably find .meta-box. This is a probe only.
if ! grep -q 'mDreamingLockscreen=true' "$OUT/window_focus.txt" 2>/dev/null; then
  echo "Keyguard not dreaming; attempting safe UI probe taps (best-effort)" >> "$OUT/summary.txt"
  # Probe tap near top-center where title/meta box often sits; record before/after UI dumps.
  adb shell input tap 540 420 >/dev/null 2>&1 || true
  sleep 2
  adb shell uiautomator dump /sdcard/window_dump2.xml >/dev/null 2>&1 && adb pull /sdcard/window_dump2.xml "$OUT/ui_after_tap.xml" >/dev/null 2>&1 || true
  adb shell screencap -p /sdcard/t184t226-2.png >/dev/null 2>&1 && adb pull /sdcard/t184t226-2.png "$OUT/screen_after_tap.png" >/dev/null 2>&1 || true
else
  echo "Device keyguard/lockscreen active; UI automation blocked without unlock credentials (not provided)." >> "$OUT/summary.txt"
fi

# Crash check
if grep -E "FATAL EXCEPTION|ANR in $PACKAGE" "$OUT/logcat.txt" >/dev/null 2>&1; then
  echo "FAIL: crash/ANR indicators found" >> "$OUT/summary.txt"
else
  echo "PASS: no crash/ANR indicators in captured logcat" >> "$OUT/summary.txt"
fi
[[ -s "$OUT/pid.txt" ]] && echo "PASS: app process present" >> "$OUT/summary.txt" || echo "WARN: app pid not found" >> "$OUT/summary.txt"

cat > "$OUT/T184_T226_STATUS.md" << 'MD'
# T184/T226 Automation Status

## Executed automatically (ADB, no human intervention)
- Device/app verification, install confirmation, launch, crash/ANR scan
- media_session/notification/service dumps
- Screenshot + UI hierarchy dumps + window focus/keyguard state
- Test fixtures present on device: `/sdcard/Download/mdreader-test-fixtures/` and `md-dir/`
- Best-effort UI probe tap (only if device unlocked)

## Not fully automatable with current app/build
- **T184 Scenarios 1–4 / T226 Scenarios 1–5** require selecting files/directories via **Android SAF native picker** (`ACTION_OPEN_DOCUMENT` / `ACTION_OPEN_DOCUMENT_TREE`).
- Tauri/WebView UI (`.meta-box`) is not exposed as stable Android UIAutomator nodes; OEM picker UIs vary by device/locale/version.
- Release APK on this device is not `run-as` accessible; no debug test hook found for direct URI/path ingestion from ADB.

## What would make it 100% automatable
- Add a debug-only/test build hook (e.g., exported test activity or intent/command accepting a content URI/file path, guarded by build type/permission) that routes to the same `DocIngest`/open paths.
- Or add instrumented Android tests (Espresso/UIAutomator) with controlled storage/picker test harness.

## Contract/integration coverage
Specs 009/010 implementation and Rust/JNI contract/property/integration tests are tracked in tasks (T158–T183, T185–T225 marked done). This evidence package covers device-side ADB validation; SAF UI E2E remains the residual manual/hook-dependent gap.
MD

echo "Evidence: $OUT" | tee -a "$OUT/summary.txt"
