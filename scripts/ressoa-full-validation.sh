#!/usr/bin/env bash
# ressoa-full-validation.sh — Master validation: runs ALL Ressoa device/host checks in one go
# Usage: ./scripts/ressoa-full-validation.sh [--serial <serial>] [--duration 30] [--skip-long] [--skip-appium]
set -euo pipefail

SERIAL=""
DURATION=30
SKIP_LONG=false
SKIP_APPIUM=false
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"
OUTPUT_BASE="./ressoa-validation-$(date -u +%Y%m%dT%H%M%SZ)"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --serial) SERIAL="$2"; shift 2 ;;
    --duration) DURATION="$2"; shift 2 ;;
    --skip-long) SKIP_LONG=true; shift ;;
    --skip-appium) SKIP_APPIUM=true; shift ;;
    --adb) ADB="$2"; shift 2 ;;
    -h|--help) echo "Usage: $0 --serial <serial> [--duration 30] [--skip-long] [--skip-appium] [--output <dir>]"; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$SERIAL" ]]; then
  echo "ERROR: --serial required" >&2
  exit 1
fi

mkdir -p "$OUTPUT_BASE"
PASS=0; FAIL=0; SKIP=0
run_step() { # name, cmd...
  local name="$1"; shift
  echo ""
  echo "=== $name ===" | tee -a "$OUTPUT_BASE/master-summary.txt"
  if "$@"; then
    echo "PASS: $name" | tee -a "$OUTPUT_BASE/master-summary.txt"
    PASS=$((PASS+1))
  else
    echo "FAIL: $name" | tee -a "$OUTPUT_BASE/master-summary.txt"
    FAIL=$((FAIL+1))
  fi
}
skip_step() { echo "SKIP: $1" | tee -a "$OUTPUT_BASE/master-summary.txt"; SKIP=$((SKIP+1)); }

echo "=== Ressoa Full Validation ===" | tee "$OUTPUT_BASE/master-summary.txt"
echo "Serial: $SERIAL" | tee -a "$OUTPUT_BASE/master-summary.txt"
echo "Timestamp: $(date -u)" | tee -a "$OUTPUT_BASE/master-summary.txt"

# 1. Host gates
echo ""
echo "=== 1. Host gates ===" | tee -a "$OUTPUT_BASE/master-summary.txt"
cd /home/luis/development/mdreader
cargo fmt --all -- --check >> "$OUTPUT_BASE/fmt.log" 2>&1 && { echo "PASS: fmt" | tee -a "$OUTPUT_BASE/master-summary.txt"; PASS=$((PASS+1)); } || { echo "FAIL: fmt" | tee -a "$OUTPUT_BASE/master-summary.txt"; FAIL=$((FAIL+1)); }
cargo clippy -p adapter-synthesis -p session -p src-tauri -p player -- -D warnings >> "$OUTPUT_BASE/clippy.log" 2>&1 && { echo "PASS: clippy" | tee -a "$OUTPUT_BASE/master-summary.txt"; PASS=$((PASS+1)); } || { echo "FAIL: clippy" | tee -a "$OUTPUT_BASE/master-summary.txt"; FAIL=$((FAIL+1)); }

# 2. Host tests
echo ""
echo "=== 2. Host tests ===" | tee -a "$OUTPUT_BASE/master-summary.txt"
cargo test -p adapter-synthesis --lib >> "$OUTPUT_BASE/tests-adapter.log" 2>&1 && echo "PASS: adapter-synthesis lib tests" | tee -a "$OUTPUT_BASE/master-summary.txt" || { echo "FAIL: adapter-synthesis tests" | tee -a "$OUTPUT_BASE/master-summary.txt"; FAIL=$((FAIL+1)); }

# 3. Device smoke
echo ""
echo "=== 3. Device automation smoke ===" | tee -a "$OUTPUT_BASE/master-summary.txt"
./scripts/device-automation-smoke.sh --serial "$SERIAL" --duration "$DURATION" --output "$OUTPUT_BASE/device-smoke" >> "$OUTPUT_BASE/device-smoke.log" 2>&1 && { echo "PASS: device smoke" | tee -a "$OUTPUT_BASE/master-summary.txt"; PASS=$((PASS+1)); } || { echo "FAIL: device smoke" | tee -a "$OUTPUT_BASE/master-summary.txt"; FAIL=$((FAIL+1)); }

# 4. Long playback (optional)
if [[ "$SKIP_LONG" == "false" ]]; then
  echo ""
  echo "=== 4. Long playback (5+ min) ===" | tee -a "$OUTPUT_BASE/master-summary.txt"
  ./scripts/long-playback-smoke.sh --serial "$SERIAL" --duration 300 --output "$OUTPUT_BASE/long-playback" >> "$OUTPUT_BASE/long-playback.log" 2>&1 && echo "PASS: long playback" | tee -a "$OUTPUT_BASE/master-summary.txt" || { echo "FAIL: long playback" | tee -a "$OUTPUT_BASE/master-summary.txt"; FAIL=$((FAIL+1)); }
else
  skip_step "long playback"
fi

# 5. Appium (optional)
if [[ "$SKIP_APPIUM" == "false" ]]; then
  echo ""
  echo "=== 5. Appium E2E ===" | tee -a "$OUTPUT_BASE/master-summary.txt"
  export PATH="$PATH:/home/luis/.npm-global/bin:$HOME/.local/bin"
  export ANDROID_HOME="$HOME/Android/Sdk"
  export ANDROID_SDK_ROOT="$HOME/Android/Sdk"
  # Start appium if not running
  if ! curl -s http://127.0.0.1:4723/status >/dev/null 2>&1; then
    appium --port 4723 > "$OUTPUT_BASE/appium-server.log" 2>&1 &
    sleep 12
  fi
  python3 scripts/appium-e2e-full.py >> "$OUTPUT_BASE/appium-e2e.log" 2>&1 && echo "PASS: Appium E2E" | tee -a "$OUTPUT_BASE/master-summary.txt" || { echo "FAIL: Appium E2E" | tee -a "$OUTPUT_BASE/master-summary.txt"; FAIL=$((FAIL+1)); }
else
  skip_step "Appium E2E"
fi

# 6. Maestro smoke
echo ""
echo "=== 6. Maestro smoke ===" | tee -a "$OUTPUT_BASE/master-summary.txt"
export PATH="$PATH:$HOME/.maestro/bin"
maestro test .maestro/smoke.yaml >> "$OUTPUT_BASE/maestro.log" 2>&1 && { echo "PASS: Maestro smoke" | tee -a "$OUTPUT_BASE/master-summary.txt"; PASS=$((PASS+1)); } || { echo "FAIL: Maestro smoke" | tee -a "$OUTPUT_BASE/master-summary.txt"; FAIL=$((FAIL+1)); }

# Summary
echo ""
echo "=== SUMMARY ===" | tee -a "$OUTPUT_BASE/master-summary.txt"
echo "PASS: $PASS" | tee -a "$OUTPUT_BASE/master-summary.txt"
echo "FAIL: $FAIL" | tee -a "$OUTPUT_BASE/master-summary.txt"
echo "SKIP: $SKIP" | tee -a "$OUTPUT_BASE/master-summary.txt"
echo "Report: $OUTPUT_BASE" | tee -a "$OUTPUT_BASE/master-summary.txt"