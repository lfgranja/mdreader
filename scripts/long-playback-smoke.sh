#!/usr/bin/env bash
# long-playback-smoke.sh — 5+ min playback smoke test (no harness timeout)
# Usage: ./scripts/long-playback-smoke.sh --serial <serial> --duration 360
set -euo pipefail

SERIAL=""
DURATION=360
PKG="app.mdreader"
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"
OUTPUT_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --serial) SERIAL="$2"; shift 2 ;;
    --duration) DURATION="$2"; shift 2 ;;
    --output) OUTPUT_DIR="$2"; shift 2 ;;
    --adb) ADB="$2"; shift 2 ;;
    -h|--help) echo "Usage: $0 --serial <serial> [--duration 360] [--output <dir>]"; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
done

TS=$(date -u +%Y%m%dT%H%M%SZ)
OUTPUT_DIR="${OUTPUT_DIR:-./long-playback-$TS}"
mkdir -p "$OUTPUT_DIR"
adb_cmd() { "$ADB" -s "$SERIAL" "$@"; }

echo "=== Long Playback Smoke (${DURATION}s) ===" | tee "$OUTPUT_DIR/summary.txt"
adb_cmd shell am force-stop "$PKG" >/dev/null 2>&1 || true
sleep 2
adb_cmd shell am start -n "$PKG/.MainActivity" >/dev/null 2>&1 || true
sleep 6

# Ingest a document
adb_cmd shell "run-as $PKG rm -f cache/debug_ingest_result.txt" 2>/dev/null || true
adb_cmd shell am start -n "$PKG/.DebugIngestActivity" --es mode document --es path /data/local/tmp/medium.md >/dev/null 2>&1 || true
sleep 10

BEFORE=$(adb_cmd shell "run-as $PKG wc -c /data/data/$PKG/diagnostic-record.jsonl" | tr -dc '0-9')
echo "Baseline diag size: $BEFORE" | tee -a "$OUTPUT_DIR/summary.txt"

echo "Playing for ${DURATION}s..." | tee -a "$OUTPUT_DIR/summary.txt"
sleep "$DURATION"

# Capture
adb_cmd shell "run-as $PKG cat /data/data/$PKG/diagnostic-record.jsonl" > "$OUTPUT_DIR/diagnostic-record.jsonl" 2>/dev/null || true
adb_cmd logcat -d -t 200 > "$OUTPUT_DIR/logcat.txt" 2>&1 || true
adb_cmd exec-out screencap -p > "$OUTPUT_DIR/screenshot.png" 2>/dev/null || true

# Check for crash/ANR
if grep -E "FATAL EXCEPTION|ANR in $PKG" "$OUTPUT_DIR/logcat.txt" >/dev/null 2>&1; then
  echo "FAIL: Crash/ANR detected" | tee -a "$OUTPUT_DIR/summary.txt"
else
  echo "PASS: No crash/ANR" | tee -a "$OUTPUT_DIR/summary.txt"
fi

# Check underrun
if grep -q "dropped_samples=[1-9]" "$OUTPUT_DIR/diagnostic-record.jsonl" 2>/dev/null; then
  echo "WARN: Dropped samples detected" | tee -a "$OUTPUT_DIR/summary.txt"
else
  echo "PASS: No dropped samples" | tee -a "$OUTPUT_DIR/summary.txt"
fi

# Check preroll cap
MAX_PREROLL=$(grep -o 'preroll_cap_secs=[0-9.]*' "$OUTPUT_DIR/diagnostic-record.jsonl" 2>/dev/null | sort -u | tail -1)
echo "Max preroll_cap: $MAX_PREROLL" | tee -a "$OUTPUT_DIR/summary.txt"

echo "Report: $OUTPUT_DIR" | tee -a "$OUTPUT_DIR/summary.txt"