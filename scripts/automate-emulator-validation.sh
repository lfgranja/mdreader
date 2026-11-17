#!/usr/bin/env bash
# automate-emulator-validation.sh — Validação automatizada no emulador AVD
# Uso: ./scripts/automate-emulator-validation.sh [--serial <serial>] [--package <package>] [--output <dir>]
#
# Automatiza checks estruturais executáveis via ADB no emulador/dispositivo Android.
# Não substitui gates físicos (SC-009 latência de áudio, 10/10 trials manuais).
# Produz relatório em Markdown e logs brutos.

set -euo pipefail

SERIAL="${SERIAL:-}"
PACKAGE="${PACKAGE:-app.mdreader}"
OUTPUT_DIR="${OUTPUT_DIR:-}"
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"

usage() {
  cat <<'EOF'
Usage: automate-emulator-validation.sh [options]
  --serial <serial>   ADB serial (default: first device)
  --package <package> App package (default: app.mdreader)
  --output <dir>      Output directory (default: ./emulator-validation-<timestamp>)
  --adb <path>        Path to adb (default: ~/Android/Sdk/platform-tools/adb)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --serial) SERIAL="$2"; shift 2 ;;
    --package) PACKAGE="$2"; shift 2 ;;
    --output) OUTPUT_DIR="$2"; shift 2 ;;
    --adb) ADB="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

TS=$(date -u +%Y%m%dT%H%M%SZ)
OUTPUT_DIR="${OUTPUT_DIR:-./emulator-validation-$TS}"
mkdir -p "$OUTPUT_DIR"

adb_cmd() {
  if [[ -n "$SERIAL" ]]; then
    "$ADB" -s "$SERIAL" "$@"
  else
    "$ADB" "$@"
  fi
}

pass() { echo "PASS: $1" | tee -a "$OUTPUT_DIR/summary.txt"; }
fail() { echo "FAIL: $1" | tee -a "$OUTPUT_DIR/summary.txt"; }
warn() { echo "WARN: $1" | tee -a "$OUTPUT_DIR/summary.txt"; }
info() { echo "INFO: $1" | tee -a "$OUTPUT_DIR/summary.txt"; }

echo "=== AVD Emulator Validation ===" | tee "$OUTPUT_DIR/summary.txt"
echo "Timestamp: $TS" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Package: $PACKAGE" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Output: $OUTPUT_DIR" | tee -a "$OUTPUT_DIR/summary.txt"
echo "" | tee -a "$OUTPUT_DIR/summary.txt"

# 1. Device connectivity
echo "--- 1. Device connectivity ---" | tee -a "$OUTPUT_DIR/summary.txt"
if adb_cmd version >/dev/null 2>&1; then
  pass "adb available"
else
  fail "adb not available at $ADB"
  exit 1
fi

adb_cmd devices > "$OUTPUT_DIR/devices_raw.txt" 2>&1 || true
DEVICES=$(awk 'NR>1 && $2=="device" {print $1}' "$OUTPUT_DIR/devices_raw.txt" | tr -d '\r')
echo "$DEVICES" > "$OUTPUT_DIR/devices.txt"
if [[ -z "$DEVICES" ]]; then
  fail "No connected device (see $OUTPUT_DIR/devices_raw.txt)"
  cat "$OUTPUT_DIR/devices_raw.txt" >> "$OUTPUT_DIR/summary.txt" || true
  exit 1
fi
if [[ -z "$SERIAL" ]]; then
  SERIAL=$(echo "$DEVICES" | head -1)
fi
pass "Device connected: $SERIAL"

# 2. Device info
echo "" | tee -a "$OUTPUT_DIR/summary.txt"
echo "--- 2. Device info ---" | tee -a "$OUTPUT_DIR/summary.txt"
adb_cmd shell getprop ro.product.model > "$OUTPUT_DIR/device_model.txt" 2>&1 || true
adb_cmd shell getprop ro.build.version.sdk > "$OUTPUT_DIR/device_sdk.txt" 2>&1 || true
adb_cmd shell getprop ro.build.version.release > "$OUTPUT_DIR/device_release.txt" 2>&1 || true
adb_cmd shell getprop ro.product.cpu.abi > "$OUTPUT_DIR/device_abi.txt" 2>&1 || true
adb_cmd shell getprop sys.boot_completed > "$OUTPUT_DIR/boot_completed.txt" 2>&1 || true
MODEL=$(tr -d '\r\n' < "$OUTPUT_DIR/device_model.txt" 2>/dev/null || echo "?")
SDK=$(tr -d '\r\n' < "$OUTPUT_DIR/device_sdk.txt" 2>/dev/null || echo "?")
REL=$(tr -d '\r\n' < "$OUTPUT_DIR/device_release.txt" 2>/dev/null || echo "?")
ABI=$(tr -d '\r\n' < "$OUTPUT_DIR/device_abi.txt" 2>/dev/null || echo "?")
BOOT=$(tr -d '\r\n' < "$OUTPUT_DIR/boot_completed.txt" 2>/dev/null || echo "?")
info "Model: $MODEL, Android: $REL (SDK $SDK), ABI: $ABI, boot_completed: $BOOT"
if [[ "$BOOT" == "1" ]]; then pass "Device boot completed"; else fail "Device not fully booted (boot_completed=$BOOT)"; fi
if [[ "${SDK:-0}" -ge 29 ]] 2>/dev/null; then pass "SDK >= 29 (minSdk satisfied)"; else warn "SDK check inconclusive"; fi

# 3. App installed
echo "" | tee -a "$OUTPUT_DIR/summary.txt"
echo "--- 3. App installation ---" | tee -a "$OUTPUT_DIR/summary.txt"
adb_cmd shell pm list packages | grep -F "$PACKAGE" > "$OUTPUT_DIR/packages.txt" 2>&1 || true
if grep -q "$PACKAGE" "$OUTPUT_DIR/packages.txt" 2>/dev/null; then
  pass "Package installed: $PACKAGE"
  adb_cmd shell dumpsys package "$PACKAGE" > "$OUTPUT_DIR/package_dump.txt" 2>&1 || true
else
  fail "Package not installed: $PACKAGE"
fi

# 4. Manifest/service checks (from dumpsys)
echo "" | tee -a "$OUTPUT_DIR/summary.txt"
echo "--- 4. Service & permissions (dumpsys) ---" | tee -a "$OUTPUT_DIR/summary.txt"
if grep -q "foregroundServiceType" "$OUTPUT_DIR/package_dump.txt" 2>/dev/null; then
  pass "foregroundServiceType found in package dump"
else
  warn "foregroundServiceType not found in dumpsys (may need manual check)"
fi
if grep -qi "mediaPlayback\|PlaybackService" "$OUTPUT_DIR/package_dump.txt" 2>/dev/null; then
  pass "PlaybackService/mediaPlayback reference found"
else
  warn "PlaybackService/mediaPlayback not clearly found in dumpsys"
fi
if grep -q "android.permission.INTERNET" "$OUTPUT_DIR/package_dump.txt" 2>/dev/null; then
  fail "INTERNET permission present (violates pure-offline constraint)"
else
  pass "INTERNET permission not found in dumpsys"
fi
for perm in FOREGROUND_SERVICE MEDIA_PLAYBACK POST_NOTIFICATIONS WAKE_LOCK; do
  if grep -q "$perm" "$OUTPUT_DIR/package_dump.txt" 2>/dev/null; then
    pass "Permission/service type reference: $perm"
  else
    warn "Reference not found in dumpsys: $perm (may be declared differently)"
  fi
done

# 5. App launch test
echo "" | tee -a "$OUTPUT_DIR/summary.txt"
echo "--- 5. App launch test ---" | tee -a "$OUTPUT_DIR/summary.txt"
adb_cmd logcat -c 2>/dev/null || true
ACTIVITY=$(adb_cmd shell cmd package resolve-activity --brief "$PACKAGE" 2>/dev/null | tail -1 | tr -d '\r\n' || true)
echo "$ACTIVITY" > "$OUTPUT_DIR/resolved_activity.txt"
if [[ -n "$ACTIVITY" && "$ACTIVITY" != *"No activity"* ]]; then
  info "Resolved activity: $ACTIVITY"
  adb_cmd shell am start -W -n "$ACTIVITY" > "$OUTPUT_DIR/am_start.txt" 2>&1 || true
  sleep 3
  PID=$(adb_cmd shell pidof "$PACKAGE" 2>/dev/null | tr -d '\r\n' || true)
  echo "$PID" > "$OUTPUT_DIR/pid.txt"
  if [[ -n "$PID" ]]; then
    pass "App launched (pid $PID)"
  else
    warn "App launch command issued but pid not found (may have crashed or different behavior)"
  fi
else
  warn "Could not resolve main activity for $PACKAGE"
fi
adb_cmd logcat -d -t 200 > "$OUTPUT_DIR/logcat_after_launch.txt" 2>&1 || true

# 6. Check for crashes
echo "" | tee -a "$OUTPUT_DIR/summary.txt"
echo "--- 6. Crash check ---" | tee -a "$OUTPUT_DIR/summary.txt"
if grep -E "FATAL EXCEPTION|AndroidRuntime.*$PACKAGE|ANR in $PACKAGE" "$OUTPUT_DIR/logcat_after_launch.txt" >/dev/null 2>&1; then
  fail "Crash/ANR indicators found in logcat"
  grep -nE "FATAL EXCEPTION|AndroidRuntime|ANR" "$OUTPUT_DIR/logcat_after_launch.txt" | head -20 > "$OUTPUT_DIR/crash_indicators.txt" || true
else
  pass "No FATAL EXCEPTION/ANR indicators for $PACKAGE in recent logcat"
fi

# 7. Media/foreground service checks (best-effort)
echo "" | tee -a "$OUTPUT_DIR/summary.txt"
echo "--- 7. Service status (best-effort) ---" | tee -a "$OUTPUT_DIR/summary.txt"
adb_cmd shell dumpsys activity services "$PACKAGE" > "$OUTPUT_DIR/services_dump.txt" 2>&1 || true
if grep -qi "Playback\|media" "$OUTPUT_DIR/services_dump.txt" 2>/dev/null; then
  info "Playback/media service references found in activity services dump"
else
  info "No active playback/media service found (expected if not playing)"
fi

# 8. Generate report
REPORT="$OUTPUT_DIR/VERIF-EMULATOR-AUTO.md"
cat > "$REPORT" <<EOF
# Automated Emulator Validation Report

- Timestamp (UTC): $TS
- Device serial: $SERIAL
- Model: $MODEL
- Android: $REL (SDK $SDK)
- ABI: $ABI
- Package: $PACKAGE
- Boot completed: $BOOT

## Summary

\`\`\`
$(cat "$OUTPUT_DIR/summary.txt")
\`\`\`

## Notes

- This is an automated structural/sanity check, not a substitute for full gate-8-full physical validation.
- SC-009 latency (≤3s play-to-audio), audio output characteristics, Bluetooth/car, telephony/alarm require manual/physical testing.
- Structural contracts (notification content, focus ducking behavior, resume fingerprint semantics, diag JSONL content) need interactive validation per specs/008-ref-device-emulator/quickstart.md.

## Artifacts

All raw outputs are in: \`$OUTPUT_DIR/\`
EOF

echo "" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Report written to: $REPORT" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Done."
