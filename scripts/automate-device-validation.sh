#!/usr/bin/env bash
# automate-device-validation.sh — Validação automatizada em dispositivo Android físico/emulador
# Uso: ./scripts/automate-device-validation.sh --serial <serial> [--package app.mdreader] [--apk <path>] [--fixtures-dir <host-dir>]
#
# Automatiza o máximo possível via ADB: conectividade, instalação, launch, crashes, permissões,
# media/session state, diagnósticos (quando acessíveis), e prepara fixtures.
# SAF picker (T184/T226 cenários com seletor nativo) requer interação UI; este script registra
# evidências automatizáveis e produz relatório.

set -euo pipefail

SERIAL=""
PACKAGE="app.mdreader"
APK=""
FIXTURES_HOST=""
OUTPUT_DIR=""
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --serial) SERIAL="$2"; shift 2 ;;
    --package) PACKAGE="$2"; shift 2 ;;
    --apk) APK="$2"; shift 2 ;;
    --fixtures-dir) FIXTURES_HOST="$2"; shift 2 ;;
    --output) OUTPUT_DIR="$2"; shift 2 ;;
    --adb) ADB="$2"; shift 2 ;;
    -h|--help) echo "Usage: $0 --serial <serial> [--package app.mdreader] [--apk <path>] [--fixtures-dir <dir>] [--output <dir>]"; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
done

TS=$(date -u +%Y%m%dT%H%M%SZ)
OUTPUT_DIR="${OUTPUT_DIR:-./device-validation-$TS}"
mkdir -p "$OUTPUT_DIR"

adb_cmd() { "$ADB" -s "$SERIAL" "$@"; }
pass() { echo "PASS: $1" | tee -a "$OUTPUT_DIR/summary.txt"; }
fail() { echo "FAIL: $1" | tee -a "$OUTPUT_DIR/summary.txt"; }
warn() { echo "WARN: $1" | tee -a "$OUTPUT_DIR/summary.txt"; }
info() { echo "INFO: $1" | tee -a "$OUTPUT_DIR/summary.txt"; }

echo "=== Device Validation (Automated + Evidence) ===" | tee "$OUTPUT_DIR/summary.txt"
echo "Timestamp: $TS" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Serial: $SERIAL" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Package: $PACKAGE" | tee -a "$OUTPUT_DIR/summary.txt"
echo "Output: $OUTPUT_DIR" | tee -a "$OUTPUT_DIR/summary.txt"
echo "" | tee -a "$OUTPUT_DIR/summary.txt"

# 1. Connectivity
adb_cmd version >/dev/null && pass "adb available" || { fail "adb unavailable"; exit 1; }
adb_cmd get-state 2>/dev/null | grep -q device && pass "Device connected: $SERIAL" || { fail "Device not connected"; exit 1; }

# 2. Device info
adb_cmd shell getprop ro.product.model > "$OUTPUT_DIR/model.txt" 2>&1 || true
adb_cmd shell getprop ro.build.version.sdk > "$OUTPUT_DIR/sdk.txt" 2>&1 || true
adb_cmd shell getprop ro.build.version.release > "$OUTPUT_DIR/release.txt" 2>&1 || true
adb_cmd shell getprop ro.product.cpu.abi > "$OUTPUT_DIR/abi.txt" 2>&1 || true
MODEL=$(tr -d '\r\n' < "$OUTPUT_DIR/model.txt"); SDK=$(tr -d '\r\n' < "$OUTPUT_DIR/sdk.txt")
REL=$(tr -d '\r\n' < "$OUTPUT_DIR/release.txt"); ABI=$(tr -d '\r\n' < "$OUTPUT_DIR/abi.txt")
info "Model=$MODEL Android=$REL SDK=$SDK ABI=$ABI"
[[ "${SDK:-0}" -ge 29 ]] 2>/dev/null && pass "SDK >= 29" || warn "SDK check inconclusive"

# 3. Install APK if provided
if [[ -n "$APK" && -f "$APK" ]]; then
  info "Installing APK: $APK"
  adb_cmd install -r "$APK" > "$OUTPUT_DIR/install.txt" 2>&1 && pass "APK installed" || fail "APK install failed (see install.txt)"
fi

# 4. Package and permissions
adb_cmd shell pm list packages | grep -F "$PACKAGE" > "$OUTPUT_DIR/packages.txt" 2>&1 || true
grep -q "$PACKAGE" "$OUTPUT_DIR/packages.txt" && pass "Package installed: $PACKAGE" || fail "Package missing"
adb_cmd shell dumpsys package "$PACKAGE" > "$OUTPUT_DIR/package_dump.txt" 2>&1 || true
if grep -q "android.permission.INTERNET" "$OUTPUT_DIR/package_dump.txt" 2>/dev/null; then
  fail "INTERNET permission present"
else
  pass "No INTERNET permission"
fi
for p in FOREGROUND_SERVICE MEDIA_PLAYBACK POST_NOTIFICATIONS WAKE_LOCK; do
  grep -q "$p" "$OUTPUT_DIR/package_dump.txt" 2>/dev/null && pass "Permission/type ref: $p" || warn "Not found in dumpsys: $p"
done

# 5. Push fixtures
if [[ -n "$FIXTURES_HOST" && -d "$FIXTURES_HOST" ]]; then
  info "Pushing fixtures to /sdcard/Download/mdreader-test-fixtures/"
  adb_cmd shell "mkdir -p /sdcard/Download/mdreader-test-fixtures" >/dev/null 2>&1 || true
  adb_cmd push "$FIXTURES_HOST" /sdcard/Download/mdreader-test-fixtures/ > "$OUTPUT_DIR/push_fixtures.txt" 2>&1 && pass "Fixtures pushed" || warn "Fixture push had issues"
  adb_cmd shell "ls -la /sdcard/Download/mdreader-test-fixtures 2>&1 | head -40" > "$OUTPUT_DIR/fixtures_on_device.txt" 2>&1 || true
fi

# 6. Launch app
adb_cmd logcat -c >/dev/null 2>&1 || true
ACTIVITY=$(adb_cmd shell cmd package resolve-activity --brief "$PACKAGE" 2>/dev/null | tail -1 | tr -d '\r\n' || true)
echo "$ACTIVITY" > "$OUTPUT_DIR/activity.txt"
info "Activity: $ACTIVITY"
if [[ -n "$ACTIVITY" && "$ACTIVITY" != *"No activity"* ]]; then
  adb_cmd shell am start -W -n "$ACTIVITY" > "$OUTPUT_DIR/am_start.txt" 2>&1 || warn "am start issues"
  sleep 4
  PID=$(adb_cmd shell pidof "$PACKAGE" 2>/dev/null | tr -d '\r\n' || true)
  echo "$PID" > "$OUTPUT_DIR/pid.txt"
  [[ -n "$PID" ]] && pass "App launched (pid $PID)" || warn "PID not found"
fi
adb_cmd logcat -d -t 300 > "$OUTPUT_DIR/logcat.txt" 2>&1 || true
if grep -E "FATAL EXCEPTION|ANR in $PACKAGE" "$OUTPUT_DIR/logcat.txt" >/dev/null 2>&1; then
  fail "Crash/ANR indicators found"
  grep -nE "FATAL EXCEPTION|AndroidRuntime|ANR" "$OUTPUT_DIR/logcat.txt" | head -30 > "$OUTPUT_DIR/crash.txt" || true
else
  pass "No crash/ANR indicators in recent logcat"
fi

# 7. Media/session/service state
adb_cmd shell dumpsys activity services "$PACKAGE" > "$OUTPUT_DIR/services.txt" 2>&1 || true
adb_cmd shell dumpsys media_session > "$OUTPUT_DIR/media_session.txt" 2>&1 || true
adb_cmd shell dumpsys notification --noredact > "$OUTPUT_DIR/notifications.txt" 2>&1 || true
grep -qi "$PACKAGE\|mdreader\|Ressoa" "$OUTPUT_DIR/media_session.txt" 2>/dev/null && info "App/media references in media_session dump" || info "No active media session reference found (may be idle)"
grep -qi "$PACKAGE\|mdreader\|Ressoa" "$OUTPUT_DIR/notifications.txt" 2>/dev/null && info "App references in notification dump" || info "No app notification found (may be idle)"

# 8. UI dump
adb_cmd shell uiautomator dump /sdcard/mdreader-ui.xml >/dev/null 2>&1 && adb_cmd pull /sdcard/mdreader-ui.xml "$OUTPUT_DIR/ui.xml" >/dev/null 2>&1 && pass "UI hierarchy dumped" || warn "UI dump unavailable"

# 9. Diagnostics attempt (best-effort)
adb_cmd shell "run-as $PACKAGE ls files 2>&1 | head -20" > "$OUTPUT_DIR/runas_files.txt" 2>&1 || true
grep -qv "run-as: unknown package\|not debuggable\|Permission denied" "$OUTPUT_DIR/runas_files.txt" 2>/dev/null && info "run-as accessible (debuggable build behavior varies)" || info "run-as not accessible (expected for release APK)"

# 10. T184/T226 automation boundary report
cat > "$OUTPUT_DIR/T184-T226-AUTOMATION.md" << 'EOF'
# T184 (Spec 009) / T226 (Spec 010) — Automação ADB e Limites

## Automatizado por este script (sem intervenção humana)
- Conectividade/dispositivo, ABI, SDK
- Instalação do APK correto para o dispositivo
- Permissões e ausência de INTERNET
- Launch da activity principal, pid, crash/ANR check
- Dumps: package, services, media_session, notifications, UI hierarchy
- Push de fixtures de teste para `/sdcard/Download/mdreader-test-fixtures/`

## Limite de automação (SAF picker)
Os cenários completos de T184/T226 exigem interação com o **seletor nativo Android SAF** (`ACTION_OPEN_DOCUMENT` / `ACTION_OPEN_DOCUMENT_TREE`). A automação 100% sem intervenção humana desses fluxos requer:
1. **Hooks de teste/instrumentação** no app (ex.: intent de debug para abrir URI/documento/tree diretamente, ou provider de teste), ou
2. **UIAutomator/Espresso em ambiente instrumentado** com seletores estáveis (variam por fabricante/versão/locale), ou
3. **Validação por contrato/testes de integração** já coberta no workspace (código/contratos implementados; ver tasks/specs).

Os testes de contrato/integração/propriedade das specs 009/010 cobrem a lógica Rust/JNI/contratos; a validação E2E via SAF UI permanece manual ou dependente de hooks de teste.

## Evidência gerada
- `summary.txt`, dumps ADB, `ui.xml`, `logcat.txt`, fixtures no dispositivo.
EOF

# Report
cat > "$OUTPUT_DIR/REPORT.md" << EOF
# Device Validation Report
- Timestamp: $TS
- Serial: $SERIAL
- Model: $MODEL / Android $REL (SDK $SDK) / ABI $ABI
- Package: $PACKAGE

## Summary
\`\`\`
$(cat "$OUTPUT_DIR/summary.txt")
\`\`\`

See \`T184-T226-AUTOMATION.md\` for automation scope/limits.
EOF
echo "Report: $OUTPUT_DIR/REPORT.md" | tee -a "$OUTPUT_DIR/summary.txt"
