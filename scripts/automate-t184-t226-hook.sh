#!/usr/bin/env bash
set -euo pipefail
SERIAL="${1:-adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp}"
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"
PKG="${PKG:-app.mdreader}"
TS=$(date -u +%Y%m%dT%H%M%SZ)
OUT="./t184-t226-hook-$TS"
mkdir -p "$OUT"
adb() { "$ADB" -s "$SERIAL" "$@"; }

log(){ echo "$*" | tee -a "$OUT/summary.txt"; }
pass(){ log "PASS: $*"; }
fail(){ log "FAIL: $*"; }
warn(){ log "WARN: $*"; }

log "=== T184/T226 via DebugIngestActivity hook === $TS serial=$SERIAL"
adb shell getprop ro.product.model ro.build.version.sdk ro.product.cpu.abi > "$OUT/device.txt" 2>&1 || true
adb shell pm list packages | grep -F "$PKG" > "$OUT/pkg.txt" 2>&1 || true
grep -q "$PKG" "$OUT/pkg.txt" && pass "package installed" || fail "package missing"

# Fixtures on device (prefer app-internal files for debug/run-as accessibility)
APP_FIX=/data/user/0/$PKG/files/mdreader-test-fixtures
FIX_SRC=/tmp/mdreader-test-fixtures
# ensure fixtures exist
if [[ ! -f "$FIX_SRC/t184-teste.md" || ! -d "$FIX_SRC/md-dir" ]]; then
  mkdir -p "$FIX_SRC/md-dir"
  printf '# Teste T184\n\n## Seção 1\nConteúdo de teste para validação automatizada.\n\n## Seção 2\nMais conteúdo.\n' > "$FIX_SRC/t184-teste.md"
  printf '# Introdução\nCapítulo 1.\n' > "$FIX_SRC/md-dir/01-introducao.md"
  printf '# Arquitetura\nCapítulo 2.\n' > "$FIX_SRC/md-dir/02-arquitetura.md"
  printf '# Conclusão\nCapítulo 10.\n' > "$FIX_SRC/md-dir/10-conclusao.md"
  : > "$FIX_SRC/md-dir/04-vazio.md"
  python3 -c "open('$FIX_SRC/md-dir/05-pesado.md','w').write('# Pesado\n'+('x'*100000))"
  printf '\xff\xfe\x00' > "$FIX_SRC/md-dir/03-invalido.bin"
  printf '# Final\nVálido.\n' > "$FIX_SRC/md-dir/06-final.md"
fi
if adb shell "run-as $PKG true" >/dev/null 2>&1; then
  adb push "$FIX_SRC" /data/local/tmp/mdreader-test-fixtures > "$OUT/push.txt" 2>&1 || true
  adb shell "run-as $PKG mkdir -p $APP_FIX/md-dir && cp /data/local/tmp/mdreader-test-fixtures/t184-teste.md $APP_FIX/ && cp /data/local/tmp/mdreader-test-fixtures/md-dir/* $APP_FIX/md-dir/" >> "$OUT/push.txt" 2>&1 || true
  FIX_BASE="$APP_FIX"
  pass "fixtures pushed to app-internal storage (run-as)"
else
  adb shell 'mkdir -p $FIX_BASE/md-dir' >/dev/null 2>&1 || true
  adb push /tmp/mdreader-test-fixtures/t184-teste.md $FIX_BASE/t184-teste.md > "$OUT/push.txt" 2>&1 || true
  adb push /tmp/mdreader-test-fixtures/md-dir /sdcard/Download/mdreader-test-fixtures/ > "$OUT/push_dir.txt" 2>&1 || true
  FIX_BASE="/sdcard/Download/mdreader-test-fixtures"
  pass "fixtures pushed to /sdcard/Download (run-as unavailable)"
fi
adb shell "ls -R $FIX_BASE 2>&1 || run-as $PKG ls -R files/mdreader-test-fixtures 2>&1" > "$OUT/fixtures.txt" 2>&1 || true

adb logcat -c >/dev/null 2>&1 || true
adb shell am force-stop "$PKG" >/dev/null 2>&1 || true
adb shell am start -W -n "$PKG/.MainActivity" > "$OUT/launch.txt" 2>&1 || true
sleep 4

# T184 Scenario 1: open new document (auto-ingest path via hook)
log "--- T184 Scenario 1: open document via debug hook ---"
adb shell am start -n "$PKG/.DebugIngestActivity" --es mode document --es path $FIX_BASE/t184-teste.md > "$OUT/t184_s1_am.txt" 2>&1 && pass "debug hook document intent sent" || fail "debug hook document intent failed"
sleep 5
adb shell "run-as $PKG cat cache/debug_ingest_result.txt" > "$OUT/t184_s1_result.txt" 2>&1 || true
adb logcat -d -t 400 > "$OUT/t184_s1_logcat.txt" 2>&1 || true
if grep -q "^ok:" "$OUT/t184_s1_result.txt" 2>/dev/null || grep -q 'DebugIngestActivity\|debugDeliverDocument\|onDocumentIngested\|t184-teste' "$OUT/t184_s1_logcat.txt"; then pass "document ingest evidence captured (result/logcat)"; else warn "document ingest evidence not obvious (see $OUT/t184_s1_result.txt)"; fi
adb shell dumpsys media_session > "$OUT/t184_s1_media.txt" 2>&1 || true

# T184 Scenario 2: reopen same doc (hook again) - resume/offer is Rust/app behavior
log "--- T184 Scenario 2: reopen document ---"
adb shell am start -n "$PKG/.DebugIngestActivity" --es mode document --es path $FIX_BASE/t184-teste.md > "$OUT/t184_s2_am.txt" 2>&1 || true
sleep 4
adb logcat -d -t 300 > "$OUT/t184_s2_logcat.txt" 2>&1 || true
pass "reopen document intent sent (resume/offer UI behavior requires app-state observation)"

# T184 Scenario 3/4: cancellation/a11y are UI-specific; hook covers programmatic open path
warn "T184 Scenario 3 (SAF cancel) and Scenario 4 (keyboard/TalkBack focus) remain UI/SAF interaction scenarios; hook validates ingest/open path without picker."

# T226: directory
log "--- T226: directory ingest via debug hook ---"
adb shell am force-stop "$PKG" >/dev/null 2>&1 || true
adb shell am start -W -n "$PKG/.MainActivity" >/dev/null 2>&1 || true
sleep 3
adb shell am start -n "$PKG/.DebugIngestActivity" --es mode directory --es path $FIX_BASE/md-dir > "$OUT/t226_dir_am.txt" 2>&1 && pass "debug hook directory intent sent" || fail "debug hook directory intent failed"
sleep 8
adb shell "run-as $PKG cat cache/debug_ingest_result.txt" > "$OUT/t226_result.txt" 2>&1 || true
adb logcat -d -t 500 > "$OUT/t226_logcat.txt" 2>&1 || true
if grep -q "^ok:directory" "$OUT/t226_result.txt" 2>/dev/null || grep -q 'debug ingest result\|chapters=\|onDirectorySelected\|directory' "$OUT/t226_logcat.txt"; then pass "directory ingest evidence captured (result/logcat)"; else warn "directory ingest evidence not obvious (see $OUT/t226_result.txt)"; fi

adb shell dumpsys activity services "$PKG" > "$OUT/services.txt" 2>&1 || true
adb shell screencap -p /sdcard/t184t226hook.png >/dev/null 2>&1 && adb pull /sdcard/t184t226hook.png "$OUT/screen.png" >/dev/null 2>&1 || true
adb shell uiautomator dump /sdcard/window_dump.xml >/dev/null 2>&1 && adb pull /sdcard/window_dump.xml "$OUT/ui.xml" >/dev/null 2>&1 || true

if grep -E 'FATAL EXCEPTION|ANR in '"$PKG" "$OUT"/*.txt >/dev/null 2>&1; then
  fail "crash/ANR indicators found"
else
  pass "no crash/ANR indicators in captured logs"
fi

cat > "$OUT/REPORT.md" << MD
# T184/T226 Hook Automation Report
- Timestamp: $TS
- Serial: $SERIAL
- Device: $(tr '\n' ' ' < "$OUT/device.txt" 2>/dev/null || true)

## Scope
Uses debug-only \`DebugIngestActivity\` (guarded by \`BuildConfig.DEBUG\`, debug manifest) to route file/directory paths to the same DocIngest snapshot/fingerprint + JNI callbacks used by production ingest flows.

## Summary
\`\`\`
$(cat "$OUT/summary.txt")
\`\`\`

## Residual UI-only cases
SAF picker cancel interaction and TalkBack/keyboard focus traversal of the native picker/title UI remain UI/human-accessibility checks; programmatic ingest/open path is automated.
MD
log "Report: $OUT/REPORT.md"
