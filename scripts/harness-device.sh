#!/usr/bin/env bash
# harness-device.sh — harness automatizado para docs/roteiro-testes.md §§4–6.
#
# Executa os cenários 5.1–5.11 no dispositivo e registra evidências em
# <out>/harness-device-<ts>/. Classificação explícita:
#   AUTO        — pode virar PASS/FAIL somente com sinal determinístico no
#                 diagnostic-record.jsonl (ou sinal contrário: FATAL/ANR).
#   PARCIAL     — observação automatizada incompleta; NUNCA PASS.
#   HUMANO      — requer escuta/TalkBack/picker/inspeção visual; NUNCA PASS.
#   INCONCLUSIVE— sem evento terminal dentro do cap, ou diag indisponível.
#
# Regra de ouro: gates determinísticos e logs são a verdade; afirmações não
# observadas ao vivo são marcadas [NV]. Síntese é mais lenta que tempo real —
# observar até session_end, nunca janela fixa curta.
#
# Uso: ./scripts/harness-device.sh [--serial endpoint] [--out dir]
#                                  [--cap-secs 60] [--long] [--offline] [--help]
# Ambiente: SERIAL, ADB, CAP_SECS.  PKG é fixo (app.mdreader).
# NUNCA usar `set -e`: cada item precisa capturar seu próprio $?.

set -uo pipefail

SERIAL="${SERIAL:-192.168.100.102:5555}"
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"
PKG="app.mdreader"
CAP_SECS="${CAP_SECS:-60}"
OUT_BASE=""
VIEW_URI="content://media/external/file/1000848919"
LONG_RUN=0
OFFLINE_OPT=0
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
  cat <<'EOF'
harness-device.sh — roteiro-testes.md §§4–6 (device ativo + visual/escuta)

Opções:
  --serial ENDPOINT   endpoint adb (default $SERIAL ou 192.168.100.102:5555)
  --out DIR           diretório-base de saída (default .)
  --cap-secs N        cap de observação por cenário, em segundos (default 60)
  --long              habilita 5.9 com janela >= 330 s (5+ min)
  --offline           habilita 5.11 com svc wifi/data disable + trap de restauração
  --help, -h          esta ajuda

Notas:
  - PKG é fixo: app.mdreader.
  - PARCIAL/HUMANO/INCONCLUSIVE nunca são reportados como PASS.
  - exit 0 somente se nenhum cenário AUTO tiver sinal contraditório.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --serial)
      [[ $# -ge 2 ]] || { printf 'missing value for %s\n' "$1" >&2; exit 2; }
      SERIAL="$2"; shift 2 ;;
    --serial=*) SERIAL="${1#*=}"; shift ;;
    --out)
      [[ $# -ge 2 ]] || { printf 'missing value for %s\n' "$1" >&2; exit 2; }
      OUT_BASE="$2"; shift 2 ;;
    --out=*) OUT_BASE="${1#*=}"; shift ;;
    --cap-secs)
      [[ $# -ge 2 ]] || { printf 'missing value for %s\n' "$1" >&2; exit 2; }
      CAP_SECS="$2"; shift 2 ;;
    --cap-secs=*) CAP_SECS="${1#*=}"; shift ;;
    --long) LONG_RUN=1; shift ;;
    --offline) OFFLINE_OPT=1; shift ;;
    --help|-h) usage; exit 0 ;;
    *) printf 'unknown option: %s\n' "$1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ ! "$CAP_SECS" =~ ^[0-9]+$ ]] || (( CAP_SECS < 1 )); then
  printf 'invalid --cap-secs: %s (use a positive integer)\n' "$CAP_SECS" >&2
  exit 2
fi

TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="${OUT_BASE:-.}/harness-device-$TS"
SUMMARY_TXT="$OUT/summary.txt"
SUMMARY_CSV="$OUT/summary.csv"
COMMON="$OUT/common"
CAPTURES="$OUT/captures"

mkdir -p "$COMMON" "$CAPTURES"
for s in 5.1 5.2 5.3 5.4 5.5 5.6 5.7 5.8 5.9 5.10 5.11 6.1; do
  mkdir -p "$OUT/$s"
done

declare -a CSV_ROWS=()
AUTO_FAILS=0
DIAG_MODE="unavailable"
DIAG_ABS="/data/data/$PKG/diagnostic-record.jsonl"
INTERNET_STATUS="unknown"

adb_cmd() { "$ADB" -s "$SERIAL" "$@"; }

log() { printf '%s\n' "$*" | tee -a "$SUMMARY_TXT"; }

csv_field() {
  local s="$1"
  s="${s//$'\n'/ }"
  s="${s//$'\r'/ }"
  s="${s//;/,}"
  printf '%s' "$s"
}

# cenario;classe/status;sinais;evidencia
record() {
  local cenario="$1" classe="$2" status="$3" sinais="$4" evidencia="$5"
  if [[ "$classe" != "AUTO" && ( "$status" == "PASS" || "$status" == "FAIL" ) ]]; then
    status="INCONCLUSIVE"
  fi
  if [[ "$classe" == "AUTO" && "$status" != "PASS" && "$status" != "FAIL" ]]; then
    status="INCONCLUSIVE"
  fi
  CSV_ROWS+=("$(csv_field "$cenario");$(csv_field "$classe:$status");$(csv_field "$sinais");$(csv_field "$evidencia")")
  if [[ "$classe" == "AUTO" && "$status" == "FAIL" ]]; then
    AUTO_FAILS=$((AUTO_FAILS + 1))
  fi
  log "[$cenario] $classe/$status :: $sinais :: $evidencia"
}

# Se o ADB não responder, nenhum cenário pode ser alegado — registrar BLOCKED.
abort_blocked() {
  local reason="$1"
  log "BLOCKED: $reason"
  for s in 5.1 5.2 5.3 5.4 5.5 5.6 5.7 5.8 5.9 5.10 5.11 6.1 6.2 6.3 6.4 4.1-SAF; do
    case "$s" in
      6.2|6.3|6.4|4.1-SAF) record "$s" "HUMANO" "NV" "bloqueado: $reason" "nenhuma" ;;
      *) record "$s" "AUTO" "INCONCLUSIVE" "bloqueado: $reason" "nenhuma" ;;
    esac
  done
  write_summaries
  exit 1
}

write_summaries() {
  {
    printf 'cenario;status;sinais;evidencia\n'
    if (( ${#CSV_ROWS[@]} > 0 )); then
      printf '%s\n' "${CSV_ROWS[@]}"
    fi
  } > "$SUMMARY_CSV"
  {
    printf '\n=== summary.csv ===\n'
    cat "$SUMMARY_CSV"
    printf '\nAUTO contraditorios: %d\n' "$AUTO_FAILS"
    printf 'exit: %d\n' "$(( AUTO_FAILS > 0 ? 1 : 0 ))"
  } >> "$SUMMARY_TXT"
}

# --- Preflight ---------------------------------------------------------------
log "=== Harness device — roteiro §§4–6 ==="
log "Timestamp: $TS"
log "Serial: $SERIAL"
log "Package: $PKG (fixo)"
log "Cap: ${CAP_SECS}s"
log "Out: $OUT"

if [[ ! -x "$ADB" ]]; then
  abort_blocked "adb nao executavel: $ADB (use ADB=...)"
fi
if ! adb_cmd get-state 2>/dev/null | grep -q device; then
  abort_blocked "adb get-state nao reportou 'device' em $SERIAL"
fi
log "PASS preflight: dispositivo alcancavel"

adb_cmd shell getprop ro.product.model > "$COMMON/device.txt" 2>&1 || true
adb_cmd shell getprop ro.build.version.sdk >> "$COMMON/device.txt" 2>&1 || true
adb_cmd shell getprop ro.product.cpu.abi >> "$COMMON/device.txt" 2>&1 || true

adb_cmd shell "dumpsys package $PKG" > "$COMMON/dumpsys-package.txt" 2>&1 || true
adb_cmd shell "pm list packages | grep -F $PKG" > "$COMMON/pm-list.txt" 2>&1 || true
if ! grep -qF "$PKG" "$COMMON/pm-list.txt" 2>/dev/null; then
  abort_blocked "pacote $PKG nao instalado em $SERIAL"
fi
grep -E 'versionName|DEBUGGABLE|targetSdk|minSdk' "$COMMON/dumpsys-package.txt" \
  > "$COMMON/package-info.txt" 2>/dev/null || true

adb_cmd shell "cmd package resolve-activity --brief $PKG" \
  > "$COMMON/resolve-activity.txt" 2>&1 || true
if grep -q 'MainActivity' "$COMMON/resolve-activity.txt" 2>/dev/null; then
  log "PASS preflight: resolve-activity -> MainActivity"
else
  log "WARN preflight: resolve-activity nao confirmou MainActivity (ver common/resolve-activity.txt)"
fi

# Ausência de INTERNET: verdade do dispositivo (dumpsys) + checagem de fonte.
if grep -q 'android.permission.INTERNET' "$COMMON/dumpsys-package.txt" 2>/dev/null; then
  INTERNET_STATUS="present"
else
  INTERNET_STATUS="absent"
fi
if [[ -f "$ROOT/scripts/check-manifest.sh" ]]; then
  bash "$ROOT/scripts/check-manifest.sh" > "$COMMON/check-manifest.txt" 2>&1
  CM_RC=$?
  log "INFO preflight: check-manifest.sh exit=$CM_RC (INTERNET no dispositivo: $INTERNET_STATUS)"
else
  log "WARN preflight: scripts/check-manifest.sh ausente; usando apenas dumpsys"
fi
if [[ "$INTERNET_STATUS" != "absent" ]]; then
  log "WARN preflight: INTERNET aparenta estar declarado — ver common/dumpsys-package.txt"
fi

# --- Helpers de cenário ------------------------------------------------------

# Texto curto (~300 bytes) para ACTION_SEND — sem URLs/nrede.
SEND_TEXT='# Teste Ressoa — ingestão curta para validar sessao e sintese offline. Paragrafo um com texto simples e simbolos normalizados: --- *** ___ e [rotulo](uri:interno). Paragrafo dois com quebras controladas e pontuacao para garantir cerca de trezentos bytes no EXTRA_TEXT do ACTION_SEND. Fim de linha.'

trigger_send() {
  local dir="$1"
  adb_cmd shell "am start -n $PKG/.MainActivity -a android.intent.action.SEND -t text/plain --es android.intent.extra.TEXT '$SEND_TEXT'" \
    > "$dir/am-start.txt" 2>&1
  return $?
}

trigger_view() {
  local dir="$1"
  adb_cmd shell am start -a android.intent.action.VIEW -d "$VIEW_URI" \
    -t text/markdown -f 0x1 "$PKG/.MainActivity" \
    > "$dir/am-start.txt" 2>&1
  return $?
}

# force-stop -> poll pidof (<=5s) -> logcat -c -> run-as id -> truncar diag ->
# confirmar wc -c = 0. Sem run-as, DIAG_MODE=unavailable e seguimos com
# logcat/screenshots (nunca alegando PASS de diag).
reset_scenario() {
  local dir="$1" i=0 pid="" n=""
  DIAG_MODE="unavailable"

  adb_cmd shell am force-stop "$PKG" >/dev/null 2>&1 || true
  while (( i < 5 )); do
    pid="$(adb_cmd shell "pidof $PKG" 2>/dev/null | tr -d '\r\n')"
    [[ -z "$pid" ]] && break
    sleep 1
    i=$((i + 1))
  done
  printf 'force-stop poll=%ds pid=%s\n' "$i" "${pid:-<empty>}" > "$dir/reset.txt"

  adb_cmd logcat -c >/dev/null 2>&1 || true

  if adb_cmd shell "run-as $PKG id" > "$dir/run-as-id.txt" 2>&1; then
    DIAG_MODE="ok"
    if ! adb_cmd shell "run-as $PKG sh -c 'rm -f $DIAG_ABS; : > $DIAG_ABS'" \
        > "$dir/diag-truncate.txt" 2>&1; then
      DIAG_MODE="unavailable"
    fi
  else
    DIAG_MODE="unavailable"
  fi

  if [[ "$DIAG_MODE" == "ok" ]]; then
    n="$(adb_cmd shell "run-as $PKG sh -c 'wc -c < $DIAG_ABS'" 2>/dev/null | tr -d '\r\n[:space:]')"
    if [[ ! "$n" =~ ^0+$ ]]; then
      DIAG_MODE="unavailable"
    fi
  fi
  printf 'diag_mode=%s diag_bytes=%s\n' "$DIAG_MODE" "$n" >> "$dir/reset.txt"
  printf '%s\n' "$DIAG_MODE" > "$dir/diag-mode.txt"
}

pull_diag() {
  local dir="$1"
  if [[ "$DIAG_MODE" == "ok" ]]; then
    adb_cmd shell "run-as $PKG cat $DIAG_ABS" \
      > "$dir/diag.jsonl" 2> "$dir/diag.err" || true
  else
    printf '# diagnostic-record.jsonl indisponivel (run-as falhou)\n' > "$dir/diag.jsonl"
  fi
}

diag_has() {
  local dir="$1" re="$2"
  grep -qE "$re" "$dir/diag.jsonl" 2>/dev/null
}

diag_count() {
  local dir="$1" re="$2" n=""
  n="$(grep -cE "$re" "$dir/diag.jsonl" 2>/dev/null)" || n=0
  printf '%s' "${n:-0}"
}
diag_elapsed_ms() {
  local dir="$1" first last
  first="$(grep -o '"ts_ms"[[:space:]]*:[[:space:]]*[0-9]*' "$dir/diag.jsonl" 2>/dev/null | head -1 | grep -o '[0-9]*$')"
  last="$(grep -o '"ts_ms"[[:space:]]*:[[:space:]]*[0-9]*' "$dir/diag.jsonl" 2>/dev/null | tail -1 | grep -o '[0-9]*$')"
  if [[ "$first" =~ ^[0-9]+$ && "$last" =~ ^[0-9]+$ ]]; then
    printf '%s' "$(( last - first ))"
  else
    printf '0'
  fi
}

crash_signal() {
  grep -qE 'FATAL EXCEPTION|ANR in|ANR in '"$PKG" "$1/logcat.txt" 2>/dev/null
}

collect_evidence() {
  local dir="$1" extra="$2"
  adb_cmd shell "dumpsys media_session" > "$dir/media_session.txt" 2>&1 || true
  adb_cmd shell "dumpsys notification --noredact" > "$dir/notification.txt" 2>&1 || true
  adb_cmd logcat -d -t 500 > "$dir/logcat.txt" 2>&1 || true
  if [[ "$extra" == "audio" || "$extra" == "both" ]]; then
    adb_cmd shell "dumpsys audio" > "$dir/audio.txt" 2>&1 || true
  fi
  if [[ "$extra" == "meminfo" || "$extra" == "both" ]]; then
    adb_cmd shell "dumpsys meminfo $PKG" > "$dir/meminfo.txt" 2>&1 || true
  fi
  adb_cmd shell "uiautomator dump /sdcard/mdreader-ui.xml" > "$dir/uiautomator.txt" 2>&1 || true
  adb_cmd pull /sdcard/mdreader-ui.xml "$dir/ui.xml" > "$dir/uipull.txt" 2>&1 || true
  pull_diag "$dir"
}

# Observa até "kind":"session_end" (cap configurável), nunca janela fixa.
# retorno 0 = terminal observado; 2 = timeout (INCONCLUSIVE, não FAIL).
wait_session_end() {
  local dir="$1" cap="$2" diag="$1/diag.jsonl"
  local deadline=$((SECONDS + cap)) rc=2
  : > "$diag"
  while (( SECONDS < deadline )); do
    if [[ "$DIAG_MODE" == "ok" ]]; then
      adb_cmd shell "run-as $PKG cat $DIAG_ABS" > "$diag" 2>/dev/null || true
    fi
    if grep -qE '"kind"[[:space:]]*:[[:space:]]*"session_end"' "$diag" 2>/dev/null; then
      printf 'session_end\n' > "$dir/terminal.txt"
      rc=0
      break
    fi
    sleep 1
  done
  if (( rc != 0 )); then
    printf 'timeout after %ss\n' "$cap" > "$dir/terminal.txt"
  fi
  pull_diag "$dir"
  return "$rc"
}

wait_for_signal() {
  local dir="$1" re="$2" cap="$3" diag="$1/diag.jsonl"
  local deadline=$((SECONDS + cap))
  while (( SECONDS < deadline )); do
    if [[ "$DIAG_MODE" == "ok" ]]; then
      adb_cmd shell "run-as $PKG cat $DIAG_ABS" > "$diag" 2>/dev/null || true
    fi
    grep -qE "$re" "$diag" 2>/dev/null && return 0
    sleep 1
  done
  return 2
}

# Toque determinístico em resource-id lido do uiautomator XML (PARCIAL: toque
# pode falhar por sobreposição; usado apenas para gerar evidência, nunca PASS).
tap_resource() {
  local xml="$1" rid="$2" node="" bounds=""
  node="$(grep -oE '<node[^>]*resource-id="[^"]*'"${rid}"'"[^>]*>' "$xml" 2>/dev/null | grep -oE 'bounds="[^"]+"' | sed -n '1p')" || true
  [[ -n "$node" ]] || return 2
  bounds="${node#bounds=\"}"; bounds="${bounds%\"}"
  if [[ "$bounds" =~ ^\[([0-9]+),([0-9]+)\]\[([0-9]+),([0-9]+)\]$ ]]; then
    local x=$(( (${BASH_REMATCH[1]} + ${BASH_REMATCH[3]}) / 2 ))
    local y=$(( (${BASH_REMATCH[2]} + ${BASH_REMATCH[4]}) / 2 ))
    adb_cmd shell "input tap $x $y" >/dev/null 2>&1 || return 3
    return 0
  fi
  return 2
}

# Par de screenshots separados por 6 s + sha256 (animação do waveform).
capture_pair() {
  local name="$1" dir="$2"
  local t0="$CAPTURES/${name}.t0.png" t6="$CAPTURES/${name}.t6.png"
  adb_cmd exec-out screencap -p > "$t0" 2> "$dir/screencap-t0.err" || true
  sleep 6
  adb_cmd exec-out screencap -p > "$t6" 2> "$dir/screencap-t6.err" || true
  if [[ -s "$t0" && -s "$t6" ]]; then
    sha256sum "$t0" "$t6" > "$dir/hashes.txt" 2>/dev/null || true
    {
      printf 'file t0: '; file -b "$t0" 2>/dev/null || true
      printf 'file t6: '; file -b "$t6" 2>/dev/null || true
    } > "$dir/dimensions.txt"
    local h0 h6
    h0="$(awk 'NR==1{print $1}' "$dir/hashes.txt" 2>/dev/null)"
    h6="$(awk 'NR==2{print $1}' "$dir/hashes.txt" 2>/dev/null)"
    if [[ -n "$h0" && -n "$h6" && "$h0" != "$h6" ]]; then
      printf 'animation_evidence=yes\n' >> "$dir/dimensions.txt"
      return 0
    fi
    printf 'animation_evidence=no\n' >> "$dir/dimensions.txt"
  fi
  return 2
}

# Expressões do diagnostic-record (toleram espaços no JSON).
RE_START='"kind"[[:space:]]*:[[:space:]]*"session_start"'
RE_END='"kind"[[:space:]]*:[[:space:]]*"session_end"'
RE_TTFA='play_to_audio_ms='
RE_END_PAUSED='\"session_end_reason\"[[:space:]]*:[[:space:]]*\"paused\"'
RE_END_COMPLETED='\"session_end_reason\"[[:space:]]*:[[:space:]]*\"completed\"'
RE_END_STOPPED='\"session_end_reason\"[[:space:]]*:[[:space:]]*\"stopped\"'
RE_MOVE='\"kind\"[[:space:]]*:[[:space:]]*\"transport_change\"'
RE_PAUSE='action=pause'
RE_PLAY='action=play'
RE_SECTION='action=section'
RE_RATE='action=rate'
RE_VOICE='"voice"'
RE_FOCUS='"kind"[[:space:]]*:[[:space:]]*"focus_transition"'
RE_COMPLETED='"transport"[[:space:]]*:[[:space:]]*"completed"'
RE_DROPPED='dropped_samples=[1-9]'
RE_PREROLL='preroll_cap_secs=60'

# --- 5.1 Cold start, documento novo ------------------------------------------
log "--- 5.1 cold start / ACTION_SEND ---"
D="$OUT/5.1"
reset_scenario "$D"
trigger_send "$D"
TRIGGER_RC=$?
wait_session_end "$D" "$CAP_SECS"
TERM_RC=$?
collect_evidence "$D" "none"
printf 'am_start_rc=%s terminal_rc=%s diag=%s\n' "$TRIGGER_RC" "$TERM_RC" "$DIAG_MODE" > "$D/result.txt"
S1="am_start_rc=$TRIGGER_RC;terminal=$(cat "$D/terminal.txt")"
diag_has "$D" "$RE_START" && S1="$S1;session_start=yes"
diag_has "$D" "$RE_TTFA" && S1="$S1;play_to_audio_ms=presente"
diag_has "$D" "$RE_MOVE" && S1="$S1;progressao=transport_change"
diag_has "$D" "$RE_END" && S1="$S1;session_end=yes"
if crash_signal "$D"; then
  S1="$S1;FATAL/ANR=yes"
  record "5.1" "AUTO" "FAIL" "$S1" "$D/{diag.jsonl,logcat.txt,am-start.txt}"
elif [[ "$DIAG_MODE" != "ok" ]]; then
  record "5.1" "AUTO" "INCONCLUSIVE" "$S1;diag=unavailable" "$D/{reset.txt,logcat.txt,am-start.txt}"
elif (( TERM_RC != 0 )); then
  record "5.1" "AUTO" "INCONCLUSIVE" "$S1;sem session_end no cap" "$D/{diag.jsonl,logcat.txt,terminal.txt}"
elif diag_has "$D" "$RE_START" && diag_has "$D" "$RE_TTFA" && diag_has "$D" "$RE_END"; then
  record "5.1" "AUTO" "PASS" "$S1" "$D/{diag.jsonl,logcat.txt,media_session.txt}"
else
  record "5.1" "AUTO" "INCONCLUSIVE" "$S1;sinais incompletos" "$D/diag.jsonl"
fi

# --- 5.2 Primeiro áudio / TTFA (ACTION_VIEW, -f 0x1) -------------------------
log "--- 5.2 ACTION_VIEW / TTFA ---"
D="$OUT/5.2"
reset_scenario "$D"
trigger_view "$D"
TRIGGER_RC=$?
wait_session_end "$D" "$CAP_SECS"
TERM_RC=$?
collect_evidence "$D" "none"
printf 'am_start_rc=%s terminal_rc=%s uri=%s\n' "$TRIGGER_RC" "$TERM_RC" "$VIEW_URI" > "$D/result.txt"
S2="ACTION_VIEW -f 0x1 uri=$VIEW_URI;am_start_rc=$TRIGGER_RC;terminal=$(cat "$D/terminal.txt")"
diag_has "$D" "$RE_TTFA" && S2="$S2;play_to_audio_ms=presente"
diag_has "$D" '"fingerprint"' && S2="$S2;fingerprint=presente"
log "INFO 5.2: baseline historico pos-ciclo 026 ~7506 ms vs 183089 ms antigo [NV] — nao reivindicado"
record "5.2" "PARCIAL" "INCONCLUSIVE" "$S2;escuta [NV]" "$D/{am-start.txt,diag.jsonl,media_session.txt}"

# --- 5.3 Pause/resume (media keyevents) --------------------------------------
log "--- 5.3 pause/resume ---"
D="$OUT/5.3"
reset_scenario "$D"
trigger_send "$D"
wait_for_signal "$D" "$RE_START" 20 || true
adb_cmd shell input keyevent 127 > "$D/keyevent-pause.txt" 2>&1 || true   # KEYCODE_MEDIA_PAUSE
wait_for_signal "$D" "$RE_END_PAUSED|$RE_PAUSE" 25 || true
START_BEFORE="$(diag_count "$D" "$RE_START")"
adb_cmd shell input keyevent 126 > "$D/keyevent-play.txt" 2>&1 || true    # KEYCODE_MEDIA_PLAY
wait_for_signal "$D" "$RE_PLAY" 25 || true
collect_evidence "$D" "none"
START_AFTER="$(diag_count "$D" "$RE_START")"
S3="pause=127;play=126;session_start_before=$START_BEFORE;after=$START_AFTER"
diag_has "$D" "$RE_PAUSE" && S3="$S3;transport_change=pause"
diag_has "$D" "$RE_END_PAUSED" && S3="$S3;session_end reason=paused"
if diag_has "$D" "$RE_PLAY"; then S3="$S3;transport_change=play"; fi
if (( START_AFTER > START_BEFORE )); then S3="$S3;novo session_start=yes"; fi
if crash_signal "$D"; then
  record "5.3" "AUTO" "FAIL" "$S3;FATAL/ANR=yes" "$D/logcat.txt"
elif [[ "$DIAG_MODE" != "ok" ]]; then
  record "5.3" "AUTO" "INCONCLUSIVE" "$S3;diag=unavailable" "$D/{reset.txt,logcat.txt}"
elif diag_has "$D" "$RE_END_PAUSED" && { diag_has "$D" "$RE_PLAY" || (( START_AFTER > START_BEFORE )); }; then
  record "5.3" "AUTO" "PASS" "$S3" "$D/{diag.jsonl,media_session.txt}"
else
  record "5.3" "AUTO" "INCONCLUSIVE" "$S3;sem par pause+retomada" "$D/diag.jsonl"
fi

# --- 5.4 Skip ±5 s -----------------------------------------------------------
log "--- 5.4 skip +/-5s ---"
D="$OUT/5.4"
reset_scenario "$D"
trigger_send "$D"
wait_for_signal "$D" "$RE_START" 20 || true
adb_cmd shell input keyevent 90 > "$D/keyevent-next.txt" 2>&1 || true   # MEDIA_NEXT
adb_cmd shell input keyevent 89 > "$D/keyevent-prev.txt" 2>&1 || true   # MEDIA_PREVIOUS
wait_for_signal "$D" "$RE_SECTION" 15 || true
collect_evidence "$D" "none"
S4="media next(90)/previous(89)"
diag_has "$D" "$RE_SECTION" && S4="$S4;action=section=presente"
diag_has "$D" '"position"[[:space:]]*:[[:space:]]*0' && S4="$S4;clamp0=observado"
if crash_signal "$D"; then S4="$S4;FATAL/ANR=yes"; fi
record "5.4" "PARCIAL" "INCONCLUSIVE" "$S4;toques #btnSkip* exigem UI" "$D/{diag.jsonl,ui.xml}"

# --- 5.5 Rate ----------------------------------------------------------------
log "--- 5.5 rate ---"
D="$OUT/5.5"
reset_scenario "$D"
trigger_send "$D"
wait_for_signal "$D" "$RE_START" 20 || true
collect_evidence "$D" "none"
if [[ -s "$D/ui.xml" ]]; then
  tap_resource "$D/ui.xml" "btnRateUp" || true
  sleep 2
  tap_resource "$D/ui.xml" "btnRateDown" || true
  collect_evidence "$D" "none"
fi
S5="uiautomator + tap btnRateUp/Down quando presentes"
diag_has "$D" "$RE_RATE" && S5="$S5;rate_from/rate_to=presentes"
grep -q 'btnRateUp\|speedPill\|liveRegion' "$D/ui.xml" 2>/dev/null && S5="$S5;controles-no-XML=yes"
if crash_signal "$D"; then S5="$S5;FATAL/ANR=yes"; fi
record "5.5" "PARCIAL" "INCONCLUSIVE" "$S5;audicao [NV]" "$D/{diag.jsonl,ui.xml}"

# --- 5.6 Ciclo de voz --------------------------------------------------------
log "--- 5.6 ciclo de voz ---"
D="$OUT/5.6"
reset_scenario "$D"
trigger_send "$D"
wait_for_signal "$D" "$RE_START" 20 || true
collect_evidence "$D" "none"
if [[ -s "$D/ui.xml" ]]; then
  tap_resource "$D/ui.xml" "btnVoice" || true
  sleep 2
  collect_evidence "$D" "none"
fi
S6="ciclo voice (faber->cadu->jeff->faber)"
diag_has "$D" "$RE_VOICE" && S6="$S6;voice_no_diag=presente"
diag_has "$D" "$RE_RATE" && S6="$S6;transport_change=presente"
if crash_signal "$D"; then S6="$S6;FATAL/ANR=yes"; fi
record "5.6" "PARCIAL" "INCONCLUSIVE" "$S6;timbre audivel [NV]" "$D/diag.jsonl"

# --- 5.7 Retomada 3 s --------------------------------------------------------
log "--- 5.7 retomada 3s / offer dialog ---"
D="$OUT/5.7"
reset_scenario "$D"
trigger_send "$D"
wait_for_signal "$D" "$RE_START" 20 || true
adb_cmd shell input keyevent 127 > "$D/keyevent-pause.txt" 2>&1 || true
wait_for_signal "$D" "$RE_END_PAUSED" 20 || true
adb_cmd shell am force-stop "$PKG" >/dev/null 2>&1 || true
sleep 1
trigger_send "$D"
adb_cmd shell "uiautomator dump /sdcard/mdreader-ui-offer.xml" > "$D/uiautomator-offer.txt" 2>&1 || true
adb_cmd pull /sdcard/mdreader-ui-offer.xml "$D/ui-offer.xml" > "$D/uipull-offer.txt" 2>&1 || true
collect_evidence "$D" "none"
S7="pause -> force-stop -> reabertura;dialogo expira em 3s"
grep -q 'offerDialog\|resume_at_stored\|start_over' "$D/ui-offer.xml" 2>/dev/null && S7="$S7;offerDialog-no-XML=yes"
diag_has "$D" "$RE_END_PAUSED" && S7="$S7;session_end reason=paused"
if crash_signal "$D"; then S7="$S7;FATAL/ANR=yes"; fi
record "5.7" "PARCIAL" "INCONCLUSIVE" "$S7;auto-start no expiry com foreground real [NV]" "$D/{ui-offer.xml,diag.jsonl}"

# --- 5.8 Fim do documento (fixture curta) ------------------------------------
log "--- 5.8 document end / fixture curta ---"
D="$OUT/5.8"
reset_scenario "$D"
# 5.8 exige BLAKE3 novo: reaproveitar o fingerprint do 5.7 faria o app estacionar
# na oferta de retomada (pending_open), sem session_start — ver advisory 2026-11-21.
SEND_TEXT="Cenario 5.8 isolado com fingerprint unico para validar conclusao de documento sem offer pendente. Texto curto com varias frases para garantir dois segmentos e exercitar a exaustao da producao no fim."
trigger_send "$D"
wait_session_end "$D" "$CAP_SECS"
TERM_RC=$?
collect_evidence "$D" "none"
S8="fixture curta;terminal=$(cat "$D/terminal.txt")"
diag_has "$D" "$RE_COMPLETED" && S8="$S8;transport=completed"
diag_has "$D" "$RE_END_COMPLETED" && S8="$S8;session_end_reason=completed"
diag_has "$D" "$RE_END_STOPPED" && S8="$S8;session_end_reason=stopped"
if crash_signal "$D"; then
  record "5.8" "AUTO" "FAIL" "$S8;FATAL/ANR=yes" "$D/logcat.txt"
elif [[ "$DIAG_MODE" != "ok" ]]; then
  record "5.8" "AUTO" "INCONCLUSIVE" "$S8;diag=unavailable" "$D/{reset.txt,logcat.txt}"
elif (( TERM_RC != 0 )); then
  record "5.8" "AUTO" "INCONCLUSIVE" "$S8;sem session_end no cap" "$D/{diag.jsonl,terminal.txt}"
elif diag_has "$D" "$RE_COMPLETED" && diag_has "$D" "$RE_END_COMPLETED"; then
  record "5.8" "AUTO" "PASS" "$S8" "$D/diag.jsonl"
elif diag_has "$D" "$RE_END_STOPPED"; then
  record "5.8" "AUTO" "FAIL" "$S8;terminal=stopped contradiz completed" "$D/diag.jsonl"
else
  record "5.8" "AUTO" "INCONCLUSIVE" "$S8;completed nao confirmado" "$D/diag.jsonl"
fi

# --- 5.9 Longa duração -------------------------------------------------------
log "--- 5.9 longa duracao ---"
D="$OUT/5.9"
reset_scenario "$D"
if (( LONG_RUN == 1 )); then
  LONG_CAP=$CAP_SECS
  (( LONG_CAP < 330 )) && LONG_CAP=330
else
  LONG_CAP=$CAP_SECS
fi
SEND_TEXT="Cenario 5.9 longa duracao com fingerprint unico para medir estabilidade sem underrun em reproducao estendida."
trigger_send "$D"
wait_session_end "$D" "$LONG_CAP"
TERM_RC=$?
collect_evidence "$D" "both"
S9="cap=${LONG_CAP}s;terminal=$(cat "$D/terminal.txt");--long=$LONG_RUN"
diag_has "$D" "$RE_PREROLL" && S9="$S9;preroll_cap_secs=60"
if diag_has "$D" "$RE_DROPPED"; then S9="$S9;dropped_samples>0"; else S9="$S9;dropped_samples=0/none"; fi
diag_has "$D" "$RE_END_STOPPED" && S9="$S9;session_end_reason=stopped"
if crash_signal "$D"; then S9="$S9;FATAL/ANR=yes"; fi
ELAPSED_MS="$(diag_elapsed_ms "$D")"
S9="$S9;elapsed_ms=$ELAPSED_MS"
if (( LONG_RUN == 1 && LONG_CAP >= 330 )); then
  if crash_signal "$D" || diag_has "$D" "$RE_DROPPED" || diag_has "$D" "$RE_END_STOPPED"; then
    record "5.9" "AUTO" "FAIL" "$S9" "$D/{diag.jsonl,logcat.txt,meminfo.txt}"
  elif [[ "$DIAG_MODE" != "ok" ]]; then
    record "5.9" "AUTO" "INCONCLUSIVE" "$S9;diag=unavailable" "$D/{reset.txt,meminfo.txt}"
  elif (( TERM_RC != 0 )); then
    record "5.9" "AUTO" "INCONCLUSIVE" "$S9;sem session_end no cap" "$D/{diag.jsonl,terminal.txt}"
  elif (( ELAPSED_MS >= 300000 )) && diag_has "$D" "$RE_PREROLL" && ! diag_has "$D" "$RE_DROPPED" && [[ -s "$D/meminfo.txt" ]]; then
    record "5.9" "AUTO" "PASS" "$S9;>=5min" "$D/{diag.jsonl,meminfo.txt}"
  else
    record "5.9" "AUTO" "INCONCLUSIVE" "$S9;elapsed<5min-ou-sinais-incompletos" "$D/diag.jsonl"
  fi
else
  record "5.9" "PARCIAL" "INCONCLUSIVE" "$S9;5+ min requer --long;underrun audivel [NV]" "$D/meminfo.txt"
fi

# --- 5.10 Foco de áudio ------------------------------------------------------
log "--- 5.10 foco de audio ---"
D="$OUT/5.10"
reset_scenario "$D"
SEND_TEXT="Cenario 5.10 foco com fingerprint unico para observar transicoes de foco sem offer pendente."
trigger_send "$D"
wait_for_signal "$D" "$RE_START" 20 || true
collect_evidence "$D" "audio"
S10="dumpsys audio capturado"
diag_has "$D" "$RE_FOCUS" && S10="$S10;focus_transition=presente"
if grep -qi 'duck\|focus' "$D/audio.txt" 2>/dev/null; then S10="$S10;audio-focus-lines=yes"; fi
if crash_signal "$D"; then S10="$S10;FATAL/ANR=yes"; fi
record "5.10" "PARCIAL" "INCONCLUSIVE" "$S10;intents duckable_transient/permanent_loss/regained dependem de app externo" "$D/{audio.txt,diag.jsonl}"

# --- 5.11 Offline ------------------------------------------------------------
log "--- 5.11 offline ---"
D="$OUT/5.11"
WIFI_BEFORE="" DATA_BEFORE=""
reset_scenario "$D"
if (( OFFLINE_OPT == 1 )); then
  # ADB e TCP/Wi-Fi: desabilitar rede aqui cortaria a conexao.
  # Agendar desativacao/reativacao LOCALMENTE (nohup) no dispositivo.
  OFF_SECS=$(( CAP_SECS + 30 ))
  adb_cmd shell "nohup sh -c 'svc wifi disable; svc data disable; sleep $OFF_SECS; svc wifi enable; svc data enable' >/dev/null 2>&1 &" > "$D/network.txt" 2>&1 || true
  printf 'offline device-side nohup secs=%s (wifi/data restore local)\n' "$OFF_SECS" >> "$D/network.txt"
  for i in $(seq 1 10); do
    adb -s "$SERIAL" connect "$SERIAL" >/dev/null 2>&1 || true
    adb -s "$SERIAL" get-state >/dev/null 2>&1 && break
    sleep 2
  done
else
  printf 'offline opt-in nao solicitado (--offline): rede nao alterada\n' > "$D/network.txt"
fi
SEND_TEXT="Cenario 5.11 offline com fingerprint unico para validar leitura e sintese sem rede apos desativacao local de wifi e dados."
trigger_send "$D"
wait_session_end "$D" "$CAP_SECS"
TERM_RC=$?
collect_evidence "$D" "none"
# Reativacao de rede e device-side (nohup agendado acima); aguardar Wi-Fi voltar
for i in $(seq 1 15); do
  adb -s "$SERIAL" shell 'settings get global wifi_on' 2>/dev/null | grep -q '^1' && break
  sleep 2
done
S11="INTERNET no dispositivo=$INTERNET_STATUS;offline_opt=$OFFLINE_OPT;terminal=$(cat "$D/terminal.txt")"
if grep -qiE 'UnknownHost|SocketException|failed to connect|NetworkOnMainThread' "$D/logcat.txt" 2>/dev/null; then
  S11="$S11;erro-de-rede-no-logcat=yes"
else
  S11="$S11;erro-de-rede-no-logcat=nao"
fi
if crash_signal "$D"; then S11="$S11;FATAL/ANR=yes"; fi
record "5.11" "PARCIAL" "INCONCLUSIVE" "$S11;continuidade de leitura/sintese [NV];MIUI [NV]" "$D/{network.txt,logcat.txt,dumpsys-package.txt}"

# --- 4.1 SAF (picker nativo) --------------------------------------------------
log "--- 4.1 SAF / picker ---"
D="$OUT/common"
collect_evidence "$D" "none" || true
record "4.1-SAF" "HUMANO" "NV" "ACTION_OPEN_DOCUMENT / ACTION_OPEN_DOCUMENT_TREE via #titleCard e #btnPickDirectory" "common/ui.xml [NV]"
record "4.3-C-H1..C-H9" "HUMANO" "NV" "surface audit / byte-identity: DEFERRED no verify.sh [NV]; gate-8-full tambem DEFERRED" "nenhuma"

# --- 6. Visual / escuta ------------------------------------------------------
log "--- 6. capturas e hash ---"
D="$OUT/6.1"
reset_scenario "$D"
trigger_send "$D"
sleep 3
capture_pair "6.1-waveform" "$D"
CAP_RC=$?
S61="screencap t0/t6 com 6s;sha256 em captures/6.1-waveform.*"
if (( CAP_RC == 0 )); then S61="$S61;hashes divergentes=animacao visivel"; else S61="$S61;animacao nao confirmada"; fi
record "6.1" "PARCIAL" "INCONCLUSIVE" "$S61;congelamento reduced-motion [NV]" "captures/6.1-waveform.t0.png;captures/6.1-waveform.t6.png;$D/hashes.txt"

record "6.2" "HUMANO" "NV" "escuta faber/cadu/jeff; prosodia headings/codigo; asteriscos e simbolos normalizados" "nenhuma [NV]"
record "6.3" "HUMANO" "NV" "TalkBack: anuncios e ordem de foco; scripts/device-a11y-check.py e somente leitura" "nenhuma [NV]"
record "6.4" "HUMANO" "NV" "overflow do diretorio com muitos capitulos (directory-nav-buttons-overflow)" "nenhuma [NV]"

write_summaries
log "=== Done: AUTO contraditorios=$AUTO_FAILS; relatorio=$OUT ==="
if (( AUTO_FAILS > 0 )); then exit 1; fi
exit 0
