#!/bin/bash
# capture_trial.sh — Captura automatizada de evidência para um trial
# Uso: ./capture_trial.sh "T-01" "SC-001" "1"

set -euo pipefail

TRIAL_ID="${1:-T-XX}"
SC_ID="${2:-SC-XXX}"
TRIAL_NUM="${3:-1}"
TS=$(date -u +%Y%m%d_%H%M%S)
OUT_DIR=~/mdreader_evidence/${TRIAL_ID}_${SC_ID}_trial${TRIAL_NUM}_${TS}

mkdir -p "$OUT_DIR"

echo "=========================================="
echo "  CAPTURA: $TRIAL_ID / $SC_ID trial #$TRIAL_NUM"
echo "  Timestamp: $TS"
echo "  Output: $OUT_DIR"
echo "=========================================="

# Limpar logcat
echo "[1/4] Limpando logcat..."
adb logcat -c

# Capturar logcat em background
echo "[2/4] Iniciando captura de logcat..."
adb logcat -v time \
  -s mdreader:V \
  RustStdoutStderr:V \
  Tauri:V \
  FocusMirror:V \
  RoutingMirror:V \
  MediaMirror:V \
  Residency:V \
  ShellHost:V \
  PlaybackService:V \
  *:S > "$OUT_DIR/logcat.log" &
LOGCAT_PID=$!

# Capturar diag.jsonl inicial
echo "[3/4] Capturando diag.jsonl inicial..."
adb shell run-as app.mdreader cat files/diag.jsonl > "$OUT_DIR/diag_before.jsonl" 2>/dev/null || echo '{}' > "$OUT_DIR/diag_before.jsonl"

echo ""
echo ">>> TRIAL EM ANDAMENTO <<<"
echo "Execute o procedimento do trial agora."
echo "Quando terminar, pressione ENTER para finalizar captura."
echo ""

read -r -p "Pressione ENTER quando o trial acabar..."

# Parar logcat
echo "[4/4] Finalizando captura..."
kill $LOGCAT_PID 2>/dev/null
wait $LOGCAT_PID 2>/dev/null

# Capturar diag.jsonl final
echo "Capturando diag.jsonl final..."
adb shell run-as app.mdreader cat files/diag.jsonl > "$OUT_DIR/diag_after.jsonl" 2>/dev/null || echo '{}' > "$OUT_DIR/diag_after.jsonl"

# Resumo
echo ""
echo "=========================================="
echo "  CAPTURA FINALIZADA"
echo "=========================================="
echo "Arquivos gerados em: $OUT_DIR"
ls -la "$OUT_DIR"

echo ""
echo "Próximos passos:"
echo "  1. Anote em notes.txt: resultado (PASS/FAIL), observações"
echo "  2. Rode: ./extract_sc_evidence.sh $OUT_DIR/logcat.log $OUT_DIR/diag_after.jsonl $SC_ID"
echo "  3. Preencha VERIFICATION.md com a evidência"