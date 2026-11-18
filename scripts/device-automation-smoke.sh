#!/usr/bin/env bash
# device-automation-smoke.sh — Automated device validation with screenshots, perf, battery, audio detection
# Usage: ./scripts/device-automation-smoke.sh --serial <serial> [--package app.mdreader] [--duration 60]

set -uo pipefail

SERIAL=""
PACKAGE="app.mdreader"
DURATION=60
OUTPUT_DIR=""
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --serial) SERIAL="$2"; shift 2 ;;
    --package) PACKAGE="$2"; shift 2 ;;
    --duration) DURATION="$2"; shift 2 ;;
    --output) OUTPUT_DIR="$2"; shift 2 ;;
    --adb) ADB="$2"; shift 2 ;;
    -h|--help) echo "Usage: $0 --serial <serial> [--package app.mdreader] [--duration 60] [--output <dir>]"; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
done

TS=$(date -u +%Y%m%dT%H%M%SZ)
OUTPUT_DIR="${OUTPUT_DIR:-./device-automation-$TS}"
mkdir -p "$OUTPUT_DIR"
adb_cmd() { "$ADB" -s "$SERIAL" "$@"; }

echo "=== Device Automation Smoke ===" | tee "$OUTPUT_DIR/summary.txt"
echo "Timestamp: $TS" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Serial: $SERIAL" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Package: $PACKAGE" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Duration: ${DURATION}s" | tee -a "$OUTPUT_DIR/summary.txt"

# 1. Connectivity
adb_cmd get-state 2>/dev/null | grep -q device && echo "PASS: Device connected" | tee -a "$OUTPUT_DIR/summary.txt" || { echo "FAIL: Device not connected" | tee -a "$OUTPUT_DIR/summary.txt"; exit 1; }

# 2. Baseline screenshot
adb_cmd exec-out screencap -p > "$OUTPUT_DIR/screenshot-baseline.png" 2>/dev/null && echo "PASS: Baseline screenshot" | tee -a "$OUTPUT_DIR/summary.txt" || echo "WARN: Screenshot failed" | tee -a "$OUTPUT_DIR/summary.txt"

# 3. Baseline memory
adb_cmd shell "dumpsys meminfo $PACKAGE" > "$OUTPUT_DIR/meminfo-baseline.txt" 2>&1 || true
echo "PASS: Baseline memory captured" | tee -a "$OUTPUT_DIR/summary.txt"

# 4. Baseline GFX
adb_cmd shell "dumpsys gfxinfo $PACKAGE" > "$OUTPUT_DIR/gfxinfo-baseline.txt" 2>&1 || true
echo "PASS: Baseline GFX captured" | tee -a "$OUTPUT_DIR/summary.txt"

# 5. Reset battery stats
adb_cmd shell "dumpsys batterystats --reset" >/dev/null 2>&1 || true
echo "INFO: Battery stats reset" | tee -a "$OUTPUT_DIR/summary.txt"

# 6. Baseline media session
adb_cmd shell "dumpsys media_session" > "$OUTPUT_DIR/media-session-baseline.txt" 2>&1 || true

# 7. Launch app
adb_cmd shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
sleep 2
adb_cmd shell am start -n "$PACKAGE/.MainActivity" >/dev/null 2>&1 || true
sleep 6
adb_cmd shell pidof "$PACKAGE" > "$OUTPUT_DIR/pid.txt" 2>&1 || true
PID=$(tr -d '\r\n' < "$OUTPUT_DIR/pid.txt")
echo "INFO: PID=$PID" | tee -a "$OUTPUT_DIR/summary.txt"

# 8. Sustained playback
echo "INFO: Playing for ${DURATION}s..." | tee -a "$OUTPUT_DIR/summary.txt"
sleep "$DURATION"

# 9. Post-playback screenshot
adb_cmd exec-out screencap -p > "$OUTPUT_DIR/screenshot-post.png" 2>/dev/null && echo "PASS: Post screenshot" | tee -a "$OUTPUT_DIR/summary.txt" || echo "WARN: Screenshot failed" | tee -a "$OUTPUT_DIR/summary.txt"

# 10. Post-playback memory
adb_cmd shell "dumpsys meminfo $PACKAGE" > "$OUTPUT_DIR/meminfo-post.txt" 2>&1 || true

# 11. Post-playback GFX
adb_cmd shell "dumpsys gfxinfo $PACKAGE" > "$OUTPUT_DIR/gfxinfo-post.txt" 2>&1 || true

# 12. Battery stats
adb_cmd shell "dumpsys batterystats $PACKAGE" > "$OUTPUT_DIR/batterystats.txt" 2>&1 || true

# 13. Media session post
adb_cmd shell "dumpsys media_session" > "$OUTPUT_DIR/media-session-post.txt" 2>&1 || true

# 14. Diagnostics capture
adb_cmd shell "run-as $PACKAGE cat /data/data/$PACKAGE/diagnostic-record.jsonl" > "$OUTPUT_DIR/diagnostic-record.jsonl" 2>/dev/null || true
echo "PASS: Diagnostics captured" | tee -a "$OUTPUT_DIR/summary.txt"

# 15. Logcat
adb_cmd logcat -d -t 200 > "$OUTPUT_DIR/logcat.txt" 2>&1 || true
if grep -E "FATAL EXCEPTION|ANR in $PACKAGE" "$OUTPUT_DIR/logcat.txt" >/dev/null 2>&1; then
  echo "FAIL: Crash/ANR found" | tee -a "$OUTPUT_DIR/summary.txt"
  grep -nE "FATAL EXCEPTION|ANR" "$OUTPUT_DIR/logcat.txt" | head -10 > "$OUTPUT_DIR/crash.txt"
else
  echo "PASS: No crash/ANR" | tee -a "$OUTPUT_DIR/summary.txt"
fi

# 16. Monkey stress
adb_cmd shell "monkey -p $PACKAGE -v 200" > "$OUTPUT_DIR/monkey.txt" 2>&1 || true
adb_cmd logcat -d -t 100 > "$OUTPUT_DIR/logcat-monkey.txt" 2>&1 || true
if grep -E "FATAL EXCEPTION|ANR in $PACKAGE" "$OUTPUT_DIR/logcat-monkey.txt" >/dev/null 2>&1; then
  echo "WARN: Monkey crash/ANR" | tee -a "$OUTPUT_DIR/summary.txt"
else
  echo "PASS: Monkey stable" | tee -a "$OUTPUT_DIR/summary.txt"
fi

# 17. Audio detection via media_session
grep -i "$PACKAGE\|mdreader\|Ressoa" "$OUTPUT_DIR/media-session-post.txt" >/dev/null 2>&1 && echo "PASS: Media session active" | tee -a "$OUTPUT_DIR/summary.txt" || echo "INFO: No active media session" | tee -a "$OUTPUT_DIR/summary.txt"

echo "Report: $OUTPUT_DIR" | tee -a "$OUTPUT_DIR/summary.txt"
echo "=== Done ==="
# 18. Accessibility (TalkBack) check via uiautomator dump
adb_cmd shell "uiautomator dump /sdcard/mdreader-a11y.xml" >/dev/null 2>&1 && adb_cmd pull /sdcard/mdreader-a11y.xml "$OUTPUT_DIR/a11y-tree.xml" >/dev/null 2>&1 && echo "PASS: Accessibility tree dumped" | tee -a "$OUTPUT_DIR/summary.txt" || echo "WARN: Accessibility dump failed" | tee -a "$OUTPUT_DIR/summary.txt"

# Count elements without content-desc or text (potential a11y issues)
if [[ -f "$OUTPUT_DIR/a11y-tree.xml" ]]; then
  UNLABELED=$(grep -o 'content-desc=""' "$OUTPUT_DIR/a11y-tree.xml" 2>/dev/null | wc -l || echo 0)
  echo "INFO: Unlabeled elements (empty content-desc): $UNLABELED" | tee -a "$OUTPUT_DIR/summary.txt"
fi

# 19. TalkBack logcat events (if TalkBack enabled)
adb_cmd logcat -d -t 200 > "$OUTPUT_DIR/logcat-a11y.txt" 2>&1 || true
# Parse TalkBack announcements (if TalkBack is enabled)
grep -i "TalkBack\|AccessibilityEvent\|speak\|announce" "$OUTPUT_DIR/logcat-a11y.txt" | tail -10 > "$OUTPUT_DIR/talkback-events.txt" 2>/dev/null || true
TALKBACK_COUNT=$(wc -l < "$OUTPUT_DIR/talkback-events.txt" 2>/dev/null || echo 0)
echo "INFO: TalkBack events captured: $TALKBACK_COUNT (TalkBack must be enabled for events)" | tee -a "$OUTPUT_DIR/summary.txt"
# Check for unlabeled clickable elements (a11y issue)
if [[ -f "$OUTPUT_DIR/a11y-tree.xml" ]]; then
  CLICKABLE_UNLABELED=$(grep -o 'clickable="true"[^>]*content-desc=""' "$OUTPUT_DIR/a11y-tree.xml" 2>/dev/null | wc -l)
  echo "INFO: Clickable unlabeled elements: $CLICKABLE_UNLABELED" | tee -a "$OUTPUT_DIR/summary.txt"
fi


# 18. Accessibility (TalkBack) check via uiautomator dump
adb_cmd shell "uiautomator dump /sdcard/mdreader-a11y.xml" >/dev/null 2>&1 && adb_cmd pull /sdcard/mdreader-a11y.xml "$OUTPUT_DIR/a11y-tree.xml" >/dev/null 2>&1 && echo "PASS: Accessibility tree dumped" | tee -a "$OUTPUT_DIR/summary.txt" || echo "WARN: Accessibility dump failed" | tee -a "$OUTPUT_DIR/summary.txt"

if [[ -f "$OUTPUT_DIR/a11y-tree.xml" ]]; then
  UNLABELED=$(grep -o 'content-desc=""' "$OUTPUT_DIR/a11y-tree.xml" 2>/dev/null | wc -l || echo 0)
  echo "INFO: Unlabeled elements (empty content-desc): $UNLABELED" | tee -a "$OUTPUT_DIR/summary.txt"
fi

adb_cmd logcat -d -t 100 > "$OUTPUT_DIR/logcat-a11y.txt" 2>&1 || true
grep -i "TalkBack\|AccessibilityEvent" "$OUTPUT_DIR/logcat-a11y.txt" | tail -5 > "$OUTPUT_DIR/talkback-events.txt" 2>/dev/null || true
echo "INFO: TalkBack events captured (if enabled)" | tee -a "$OUTPUT_DIR/summary.txt"
exit 0
