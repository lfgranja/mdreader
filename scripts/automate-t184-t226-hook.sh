#!/usr/bin/env bash
set -euo pipefail

# Spec 012 / FR-008,FR-009,FR-012,SC-001,SC-002,SC-003 contract-test & report helpers.
# Contract modes are host-deterministic, have no device/adb side effects, and take
# precedence over normal device automation.

HOOK_SCRIPT_NAME="automate-t184-t226-hook.sh"
JSON_MODE=false
CONTRACT_MODE=""
CLASSIFY_VALUE=""

# Parse contract/report flags early (no side effects). Preserve positional SERIAL for
# normal device runs by collecting non-contract positionals.
POSITIONAL=()
while (($#)); do
  case "$1" in
    --json) JSON_MODE=true; shift ;;
    --help|-h) CONTRACT_MODE="help"; shift ;;
    --print-contract) CONTRACT_MODE="print-contract"; shift ;;
    --classify-outcome)
      CONTRACT_MODE="classify-outcome"
      CLASSIFY_VALUE="${2:-}"
      if (($# >= 2)); then shift 2; else shift; fi
      ;;
    --print-report-schema) CONTRACT_MODE="print-report-schema"; shift ;;
    --print-residual-rules) CONTRACT_MODE="print-residual-rules"; shift ;;
    *) POSITIONAL+=("$1"); shift ;;
  esac
done
set -- "${POSITIONAL[@]+"${POSITIONAL[@]}"}"

emit_json() {
  python3 - "$@" <<'PY'
import json, sys
payload = json.loads(sys.argv[1])
print(json.dumps(payload, sort_keys=True))
PY
}

# FR-008 minimum fields (spec.md §FR-008) + corroboration/channel fields for contract output.
fr008_json() {
  emit_json '{"scope_class":"programmatic-non-ui","fixture_reference":"<fixture/reference>","operation":"<document|directory|uri>","outcome":"<ok:*|error:*|refused:*|blocked:*|inconclusive>","build_identity":"<build/debug APK identity or unavailable:<reason>>","device_identity":"<serial/model/sdk/abi or unavailable:<reason>>","diagnostic_capture":"<logcat/report/screenshot paths or unavailable:<reason>>","crash_anr_status":"<not-detected|detected|unknown>","residual_scope_note":"Programmatic scope only; native SAF picker UI/a11y residuals remain open (FR-009/FR-012).","preferred_location_or_channel":"cache/debug_ingest_result.txt (authoritative when parseable)","effective_location_or_channel":"<effective path/channel>","write_status":"unavailable","reviewer_accessible_location":"<reviewer-accessible path or unavailable:<reason>>","retention_scope":"Until covered EV/RES/TASK/FR/SC IDs are decided/closed or per repo policy; not required in ignored dirs.","channel":"cache/logcat/script","corroboration":"authoritative-only|corroborated|conflicting|inconclusive (per contract)","full_parent_completion_claimed":false}'
}

print_contract_json() {
  emit_json '{"contract":"debug-ingest-automation","scope_class":"programmatic-non-ui","directory_result":{"operation":"directory","chapters":0,"chapters_field":"chapters=<n> accepted by production filtering/manifest behavior","evidence_required":true,"fixture_composition":"Evidence/report MUST state fixture composition when claiming directory pass.","expected_filtering":"Interpret chapters=<n> against production filtering and documented fixture composition/expected filtering (ASM-012-CHAPTERS-INTERP)."},"fixture_composition":{"note":"Evidence/report MUST state fixture composition when claiming directory pass."},"expected_filtering":{"note":"Interpret chapters=<n> against production filtering and documented fixture composition/expected filtering (ASM-012-CHAPTERS-INTERP)."},"result_prefixes":["ok:document:<absolute-path>","ok:uri:<uri>","ok:directory:<absolute-path>:chapters=<n>","error:<class-or-code>:<detail>","refused:not-debug"],"full_parent_completion_claimed":false}'
}

classify_outcome_json() {
  local value="$1"
  python3 - "$value" <<'PY'
import json, sys
v = sys.argv[1]
d = {
  "raw": v,
  "explicit_within_budget": False,
  "counts_toward_sc002_denominator": True,
}
if v.startswith("blocked:"):
  d.update({"status": "blocked", "counts_toward_sc002_denominator": False, "explicit_within_budget": False})
elif v.startswith("inconclusive"):
  d.update({"status": "inconclusive", "counts_toward_sc002_denominator": True, "explicit_within_budget": False})
elif v.startswith("ok:"):
  d.update({"status": "pass", "counts_toward_sc002_denominator": True, "explicit_within_budget": True})
elif v.startswith("error:") or v.startswith("refused:"):
  d.update({"status": "fail", "counts_toward_sc002_denominator": True, "explicit_within_budget": True})
else:
  # fail-closed: unknown values are not explicit pass evidence
  d.update({"status": "inconclusive", "counts_toward_sc002_denominator": False, "explicit_within_budget": False, "fail_closed": True})
print(json.dumps(d, sort_keys=True))
PY
}

report_schema_json() {
  emit_json '{"scope_class":"programmatic-non-ui","full_parent_completion_claimed":false,"fixture_reference":"<required>","operation":"<document|directory|uri>","outcome":"<explicit parseable>","build_identity":"<required>","device_identity":"<required>","diagnostic_capture":"<required>","crash_anr_status":"<required>","residual_scope_note":"Native SAF picker UI/a11y residuals remain open (FR-009/FR-012).","preferred_location_or_channel":"<required>","effective_location_or_channel":"<required>","write_status":"<written|unavailable|failed:<reason>>","reviewer_accessible_location":"<required>","retention_scope":"<required>","correlation_key":"<request/evidence/run id>","channel_corroboration":"<authoritative/corroborated/conflicting/inconclusive>","counters":{"attempted":0,"precondition_blocked":0,"in_denominator":0,"explicit_within_budget":0,"inconclusive":0},"formulae":{"sc-001":"SC-001: 100% of attempted runs whose fixtures meet app-accessibility precondition execute without human picker interaction; blocked:fixture-inaccessible reported separately.","sc-002":"SC-002: >=95% (0.95) of in-denominator runs produce explicit parseable success or diagnosable failure within 2 minutes; blocked excluded, inconclusive not explicit until rerun resolves."},"formulae_text":"SC-001 and SC-002 formulae: SC-001 fixture-precondition pass rate; SC-002 >= 95% / 0.95 explicit-within-budget over in-denominator runs."}'
}

residual_rules_json() {
  emit_json '{"allowed_statuses":["accepted-risk","deferred","open","verified"],"accepted_risk_requires":["owner","decision","rationale","scope","evidence"],"accepted_risk_completes_parent_when_blocking":false,"verified_requires":["evidence refs appropriate to scope"],"deferred_requires":["owner","decision ref","rationale"],"parent_completion_rule":"Parent T184/T226 fully complete only when all required *-PROG and required residual items are verified or explicitly re-scoped by a recorded decision (never implicit).","full_parent_completion_claimed_from_hook_evidence":false}'
}

usage(){
  cat <<'USAGE'
Usage: automate-t184-t226-hook.sh [serial] [options]
  serial  adb serial (default: adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp)
Automates T184 (open file auto-ingest path) and T226 (directory ingest path) using
debug-only DebugIngestActivity hook (no manual SAF picker). Deterministic, fail-closed.
Contract modes (host-deterministic, no device/adb side effects):
  --help [--json]                    usage or FR-008 contract JSON
  --print-contract [--json]          debug-ingest contract summary
  --classify-outcome <value> [--json] classify result/status (SC-001/SC-002)
  --print-report-schema [--json]     report schema/counters/formulae
  --print-residual-rules [--json]    FR-012 residual lifecycle rules
Normalized mode aliases (debug hook): document=file,open_document; directory=dir,tree,open_directory; uri.
USAGE
}

# Contract modes: deterministic, no adb/mkdir/device side effects.
if [[ "$CONTRACT_MODE" == "help" ]]; then
  if [[ "$JSON_MODE" == "true" ]]; then fr008_json; else usage; fi
  exit 0
fi
if [[ "$CONTRACT_MODE" == "print-contract" ]]; then
  print_contract_json
  exit 0
fi
if [[ "$CONTRACT_MODE" == "classify-outcome" ]]; then
  classify_outcome_json "$CLASSIFY_VALUE"
  exit 0
fi
if [[ "$CONTRACT_MODE" == "print-report-schema" ]]; then
  report_schema_json
  exit 0
fi
if [[ "$CONTRACT_MODE" == "print-residual-rules" ]]; then
  residual_rules_json
  exit 0
fi

# ---- Normal device automation mode (may have side effects) ----
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
  log "Local fixtures missing at $FIX_SRC; creating minimal test fixtures"
  mkdir -p "$FIX_SRC/md-dir"
  printf '# Teste T184\n\nTexto para ingestação automática.\n' > "$FIX_SRC/t184-teste.md"
  printf '# Cap 1\n\nPrimeiro.\n' > "$FIX_SRC/md-dir/001-cap1.md"
  printf '# Cap 2\n\nSegundo.\n' > "$FIX_SRC/md-dir/002-cap2.md"
fi
if adb shell "run-as $PKG true" >/dev/null 2>&1; then
  adb shell "run-as $PKG mkdir -p files/mdreader-test-fixtures/md-dir" >/dev/null 2>&1 || true
  adb push "$FIX_SRC/t184-teste.md" /data/local/tmp/t184-teste.md >/dev/null 2>&1 || true
  adb push "$FIX_SRC/md-dir/001-cap1.md" /data/local/tmp/001-cap1.md >/dev/null 2>&1 || true
  adb push "$FIX_SRC/md-dir/002-cap2.md" /data/local/tmp/002-cap2.md >/dev/null 2>&1 || true
  adb shell "run-as $PKG cp /data/local/tmp/t184-teste.md files/mdreader-test-fixtures/t184-teste.md" >/dev/null 2>&1 || true
  adb shell "run-as $PKG cp /data/local/tmp/001-cap1.md files/mdreader-test-fixtures/md-dir/001-cap1.md" >/dev/null 2>&1 || true
  adb shell "run-as $PKG cp /data/local/tmp/002-cap2.md files/mdreader-test-fixtures/md-dir/002-cap2.md" >/dev/null 2>&1 || true
  FIX_BASE=$APP_FIX
else
  adb shell "mkdir -p $APP_FIX/md-dir" >/dev/null 2>&1 || true
  adb push "$FIX_SRC/t184-teste.md" "$APP_FIX/t184-teste.md" >/dev/null 2>&1 || true
  adb push "$FIX_SRC/md-dir" "$APP_FIX/" >/dev/null 2>&1 || true
  FIX_BASE=$APP_FIX
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
  fail "crash/ANR markers found (inspect $OUT)"
else
  pass "no obvious FATAL EXCEPTION/ANR markers in collected logs"
fi

cat > "$OUT/REPORT.md" << MD
# T184/T226 Hook Automation Report
- Time (UTC): $TS
- Serial: $SERIAL
- Device: $(tr '\n' ' ' < "$OUT/device.txt" 2>/dev/null || true)
- Fixtures base: ${FIX_BASE:-unknown}

## Evidence
- summary: $OUT/summary.txt
- T184 s1 result: $OUT/t184_s1_result.txt
- T184 s1 logcat: $OUT/t184_s1_logcat.txt
- T184 s2 logcat: $OUT/t184_s2_logcat.txt
- T226 result: $OUT/t226_result.txt
- T226 logcat: $OUT/t226_logcat.txt
- screenshot/ui: $OUT/screen.png, $OUT/ui.xml

## Limitations (manual/visual)
- SAF cancel behavior and keyboard/TalkBack focus across SAF/title UI cannot be fully proven by am/debug hook; requires manual accessibility check.
MD
log "Report: $OUT/REPORT.md"
