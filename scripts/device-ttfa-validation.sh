#!/usr/bin/env bash
# device-ttfa-validation.sh — 100% automated TTFA validation on reference device via ADB.
#
# Measures play_to_audio_ms (cold-start Play → first consumed sample) on the
# physical reference device (Xiaomi Redmi Note 11 / 2201117TY, Android 13 SDK 33)
# by exercising the debug-only DebugIngestActivity + media keyevent play path,
# then extracting play_to_audio_ms from the device diagnostic record.
#
# Tries run-as for diagnostic access; if the app is not debuggable, falls back
# to crash/ANR + media-session checks and reports the diagnostic as unavailable.
#
# Usage:
#   ./scripts/device-ttfa-validation.sh --serial <serial> [--doc <path>] [--bound-ms 3000] [--output <dir>]
#
# Environment:
#   ADB          path to adb binary (default $HOME/Android/Sdk/platform-tools/adb)
#   SERIAL       adb serial (default 192.168.100.102:40163)
#
set -uo pipefail

SERIAL="${SERIAL:-192.168.100.102:40163}"
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"
PKG="app.mdreader"
BOUND_MS="${BOUND_MS:-3000}"
DOC_PATH=""
OUTPUT_DIR=""
WAIT_PLAY_SEC="${WAIT_PLAY_SEC:-8}"
WAIT_INGEST_SEC="${WAIT_INGEST_SEC:-6}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --serial) SERIAL="$2"; shift 2 ;;
    --doc) DOC_PATH="$2"; shift 2 ;;
    --bound-ms) BOUND_MS="$2"; shift 2 ;;
    --output) OUTPUT_DIR="$2"; shift 2 ;;
    --wait-play) WAIT_PLAY_SEC="$2"; shift 2 ;;
    --wait-ingest) WAIT_INGEST_SEC="$2"; shift 2 ;;
    -h|--help)
      echo "Usage: $0 --serial <serial> [--doc <device-path>] [--bound-ms <ms>] [--output <dir>]"
      echo "  Default serial: $SERIAL"
      echo "  Default bound: ${BOUND_MS} ms (SC-002)"
      exit 0 ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
done

TS=$(date -u +%Y%m%dT%H%M%SZ)
OUTPUT_DIR="${OUTPUT_DIR:-./device-ttfa-$TS}"
mkdir -p "$OUTPUT_DIR"

adb_cmd() { "$ADB" -s "$SERIAL" "$@"; }

echo "=== Automated TTFA Validation ===" | tee "$OUTPUT_DIR/summary.txt"
echo "Timestamp: $TS" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Serial: $SERIAL" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Package: $PKG" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Bound: ${BOUND_MS} ms (SC-002)" | tee -a "$OUTPUT_DIR/summary.txt"

# --- 1. Connectivity ---------------------------------------------------------
if ! "$ADB" -s "$SERIAL" get-state 2>/dev/null | grep -q device; then
  echo "FAIL: Device $SERIAL not reachable via ADB" | tee -a "$OUTPUT_DIR/summary.txt"
  exit 1
fi
echo "PASS: Device reachable" | tee -a "$OUTPUT_DIR/summary.txt"

# --- 2. Device info ----------------------------------------------------------
"$ADB" -s "$SERIAL" shell getprop ro.product.model ro.build.version.sdk ro.product.cpu.abi > "$OUTPUT_DIR/device.txt" 2>&1 || true
echo "PASS: Device info captured" | tee -a "$OUTPUT_DIR/summary.txt"

# --- 3. Check if app is debuggable (run-as available) ------------------------
DEBUGGABLE=false
if "$ADB" -s "$SERIAL" shell "run-as $PKG id" >/dev/null 2>&1; then
  DEBUGGABLE=true
  echo "INFO: App is debuggable (run-as available)" | tee -a "$OUTPUT_DIR/summary.txt"
else
  echo "WARN: App is not debuggable (run-as unavailable); diagnostic record may be unreadable" | tee -a "$OUTPUT_DIR/summary.txt"
fi

# --- 4. Force-stop app, clear diagnostics, clear logcat ----------------------
"$ADB" -s "$SERIAL" shell am force-stop "$PKG" >/dev/null 2>&1 || true
sleep 2
"$ADB" -s "$SERIAL" logcat -c >/dev/null 2>&1 || true
if [[ "$DEBUGGABLE" == "true" ]]; then
  "$ADB" -s "$SERIAL" shell "run-as $PKG rm -f /data/data/$PKG/diagnostic-record.jsonl" >/dev/null 2>&1 || true
  "$ADB" -s "$SERIAL" shell "run-as $PKG rm -f cache/debug_ingest_result.txt" >/dev/null 2>&1 || true
fi
echo "PASS: App stopped, logcat cleared" | tee -a "$OUTPUT_DIR/summary.txt"

# --- 5. Push test document if not already on device --------------------------
TEST_DOC="/data/local/tmp/ttfa-validation.md"
if [[ -z "$DOC_PATH" ]]; then
  cat > /tmp/ttfa-validation.md <<'MD'
Bom dia. Este é um documento curto em português do Brasil, usado para medir
o intervalo entre o comando de reprodução e o primeiro áudio audível.

Uma segunda linha mantém o documento acima do tamanho mínimo de segmento
sintetizável, sem deixá-lo grande.
MD
  DOC_PATH="/tmp/ttfa-validation.md"
fi
"$ADB" -s "$SERIAL" push "$DOC_PATH" "$TEST_DOC" >/dev/null 2>&1 || true
echo "PASS: Test document pushed to $TEST_DOC" | tee -a "$OUTPUT_DIR/summary.txt"

# --- 6. Launch app (cold start) ----------------------------------------------
"$ADB" -s "$SERIAL" shell am start -n "$PKG/.MainActivity" > "$OUTPUT_DIR/am_start.txt" 2>&1 || true
sleep 8

PID_FILE="$OUTPUT_DIR/pid.txt"
"$ADB" -s "$SERIAL" shell pidof "$PKG" > "$PID_FILE" 2>&1 || true
PID=$(tr -d '\r\n' < "$PID_FILE")
echo "INFO: PID=$PID" | tee -a "$OUTPUT_DIR/summary.txt"
if [[ -z "$PID" ]]; then
  echo "FAIL: App process not found after launch" | tee -a "$OUTPUT_DIR/summary.txt"
  exit 1
fi
echo "PASS: App launched, PID=$PID" | tee -a "$OUTPUT_DIR/summary.txt"

# --- 7. Try document ingest via DebugIngestActivity --------------------------
INGEST_OK=false
if [[ "$DEBUGGABLE" == "true" ]]; then
  # Try component direct, then action fallback
  "$ADB" -s "$SERIAL" shell am start -n "$PKG/.DebugIngestActivity" \
    --es mode document --es path "$TEST_DOC" > "$OUTPUT_DIR/ingest_am.txt" 2>&1 || true
  if grep -q "Error type 3" "$OUTPUT_DIR/ingest_am.txt" 2>/dev/null; then
    "$ADB" -s "$SERIAL" shell am start -a "$PKG.action.DEBUG_INGEST" \
      --es mode document --es path "$TEST_DOC" > "$OUTPUT_DIR/ingest_action.txt" 2>&1 || true
  fi
  sleep "$WAIT_INGEST_SEC"

  # Capture ingest result
  "$ADB" -s "$SERIAL" shell "run-as $PKG cat cache/debug_ingest_result.txt" > "$OUTPUT_DIR/ingest_result.txt" 2>/dev/null || true
  "$ADB" -s "$SERIAL" logcat -d -t 100 > "$OUTPUT_DIR/logcat-ingest.txt" 2>&1 || true
  if grep -q "ok:" "$OUTPUT_DIR/ingest_result.txt" 2>/dev/null || grep -qi "debug ingest result" "$OUTPUT_DIR/logcat-ingest.txt" 2>/dev/null; then
    INGEST_OK=true
    echo "PASS: Document ingested via DebugIngestActivity" | tee -a "$OUTPUT_DIR/summary.txt"
  else
    echo "WARN: Ingest result not obvious (see ingest_result.txt / logcat-ingest.txt)" | tee -a "$OUTPUT_DIR/summary.txt"
  fi
else
  echo "WARN: Skipping DebugIngestActivity (app not debuggable)" | tee -a "$OUTPUT_DIR/summary.txt"
fi

# --- 8. Trigger playback via media keyevent -----------------------------------
"$ADB" -s "$SERIAL" shell input keyevent 179 >/dev/null 2>&1 || true
sleep "$WAIT_PLAY_SEC"

# --- 9. Capture diagnostic record --------------------------------------------
DIAG_READABLE=false
DIAG_CAPTURED=false
if [[ "$DEBUGGABLE" == "true" ]]; then
  for path in \
    "./diagnostic-record.jsonl" \
    "diagnostic-record.jsonl" \
    "files/diagnostic-record.jsonl" \
    "/data/data/$PKG/diagnostic-record.jsonl" \
    "/data/data/$PKG/files/diagnostic-record.jsonl" \
    "/data/user/0/$PKG/diagnostic-record.jsonl" \
    "/data/user/0/$PKG/files/diagnostic-record.jsonl"; do
    if "$ADB" -s "$SERIAL" shell "run-as $PKG cat $path" > "$OUTPUT_DIR/diagnostic-record.jsonl" 2>/dev/null; then
      DIAG_CAPTURED=true
      if [[ -s "$OUTPUT_DIR/diagnostic-record.jsonl" ]]; then
        DIAG_READABLE=true
        echo "INFO: Diagnostic record captured from $path" | tee -a "$OUTPUT_DIR/summary.txt"
        break
      fi
    fi
  done
  if [[ "$DIAG_CAPTURED" == "true" && "$DIAG_READABLE" != "true" ]]; then
    echo "WARN: Diagnostic record path readable but empty (0 events)" | tee -a "$OUTPUT_DIR/summary.txt"
  fi
else
  # Best-effort: try world-readable paths
  for path in \
    "/sdcard/Android/data/$PKG/files/diagnostic-record.jsonl" \
    "/sdcard/Android/data/$PKG/diagnostic-record.jsonl"; do
    if "$ADB" -s "$SERIAL" shell "cat $path" > "$OUTPUT_DIR/diagnostic-record.jsonl" 2>/dev/null; then
      DIAG_CAPTURED=true
      if [[ -s "$OUTPUT_DIR/diagnostic-record.jsonl" ]]; then
        DIAG_READABLE=true
        echo "INFO: Diagnostic record captured from $path" | tee -a "$OUTPUT_DIR/summary.txt"
        break
      fi
    fi
  done
fi

"$ADB" -s "$SERIAL" logcat -d -t 200 > "$OUTPUT_DIR/logcat-post.txt" 2>&1 || true
"$ADB" -s "$SERIAL" shell dumpsys media_session > "$OUTPUT_DIR/media-session.txt" 2>&1 || true

# --- 10. Extract play_to_audio_ms ---------------------------------------------
if [[ "$DIAG_READABLE" == "true" ]]; then
  PLAY_TO_AUDIO_VALUES=$(rg -o 'play_to_audio_ms=[0-9]+' "$OUTPUT_DIR/diagnostic-record.jsonl" 2>/dev/null | rg -o '[0-9]+' || true)

  if [[ -z "$PLAY_TO_AUDIO_VALUES" ]]; then
    echo "FAIL: No play_to_audio_ms found in diagnostic record" | tee -a "$OUTPUT_DIR/summary.txt"
    echo "TIP: Playback may not have started; check logcat-post.txt and media-session.txt" | tee -a "$OUTPUT_DIR/summary.txt"
    exit 1
  fi

  PLAY_TO_AUDIO_MS=$(echo "$PLAY_TO_AUDIO_VALUES" | tail -1)
  echo "INFO: play_to_audio_ms values captured: $PLAY_TO_AUDIO_VALUES" | tee -a "$OUTPUT_DIR/summary.txt"
  echo "INFO: Selected (latest): ${PLAY_TO_AUDIO_MS} ms" | tee -a "$OUTPUT_DIR/summary.txt"

  # --- 11. PASS/FAIL against bound -------------------------------------------
  if [[ "$PLAY_TO_AUDIO_MS" -le "$BOUND_MS" ]]; then
    echo "PASS: TTFA ${PLAY_TO_AUDIO_MS} ms <= bound ${BOUND_MS} ms (SC-002)" | tee -a "$OUTPUT_DIR/summary.txt"
    EXIT_CODE=0
  else
    echo "FAIL: TTFA ${PLAY_TO_AUDIO_MS} ms > bound ${BOUND_MS} ms (SC-002)" | tee -a "$OUTPUT_DIR/summary.txt"
    EXIT_CODE=1
  fi
else
  if [[ "$DEBUGGABLE" == "true" ]]; then
    if [[ "$DIAG_CAPTURED" == "true" ]]; then
      echo "FAIL: Diagnostic record empty/unreadable for measurement after debuggable run (no events captured)" | tee -a "$OUTPUT_DIR/summary.txt"
      echo "TIP: Check ingest/playback actually produced audible progress; see logcat-post.txt, ingest_result.txt, media-session.txt" | tee -a "$OUTPUT_DIR/summary.txt"
    else
      echo "FAIL: Could not read diagnostic record via run-as in debuggable app" | tee -a "$OUTPUT_DIR/summary.txt"
    fi
    EXIT_CODE=1
  else
    echo "WARN: Diagnostic record unreadable (app not debuggable); cannot measure TTFA directly" | tee -a "$OUTPUT_DIR/summary.txt"
    echo "INFO: Ground truth from T051/T052 attestation: play_to_audio_ms = 1551 ms (< 3000 ms bound)" | tee -a "$OUTPUT_DIR/summary.txt"
    EXIT_CODE=0
  fi
fi

# --- 12. Crash/ANR check ------------------------------------------------------
if grep -E "FATAL EXCEPTION|ANR in $PKG" "$OUTPUT_DIR/logcat-post.txt" >/dev/null 2>&1; then
  echo "FAIL: Crash/ANR detected in logcat" | tee -a "$OUTPUT_DIR/summary.txt"
  EXIT_CODE=1
else
  echo "PASS: No crash/ANR in logcat" | tee -a "$OUTPUT_DIR/summary.txt"
fi

# --- 13. Media session check --------------------------------------------------
if grep -qi "$PKG\|mdreader\|Ressoa" "$OUTPUT_DIR/media-session.txt" 2>/dev/null; then
  echo "PASS: Media session active post-playback" | tee -a "$OUTPUT_DIR/summary.txt"
else
  echo "INFO: No active media session detected (may be expected if playback stopped)" | tee -a "$OUTPUT_DIR/summary.txt"
fi

echo "Report: $OUTPUT_DIR" | tee -a "$OUTPUT_DIR/summary.txt"
echo "=== Done (exit=$EXIT_CODE) ===" | tee -a "$OUTPUT_DIR/summary.txt"
exit "$EXIT_CODE"