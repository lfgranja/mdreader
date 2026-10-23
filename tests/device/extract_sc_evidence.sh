#!/bin/bash
# extract_sc_evidence.sh — Extrai evidência específica por SC a partir dos logs capturados
# Uso: ./extract_sc_evidence.sh <logcat_file> <diag_file> <SC_ID>

set -euo pipefail

LOGCAT="${1:-}"
DIAG="${2:-}"
SC_ID="${3:-}"

if [[ -z "$LOGCAT" || -z "$DIAG" || -z "$SC_ID" ]]; then
    echo "Uso: $0 <logcat_file> <diag_file> <SC_ID>"
    echo "Exemplo: $0 ~/mdreader_evidence/T-02_SC-002_trial1_20261012_143000/logcat.log ~/mdreader_evidence/T-02_SC-002_trial1_20261012_143000/diag_after.jsonl SC-002"
    exit 1
fi

if [[ ! -f "$LOGCAT" ]]; then
    echo "ERRO: logcat file não encontrado: $LOGCAT"
    exit 1
fi

if [[ ! -f "$DIAG" ]]; then
    echo "ERRO: diag file não encontrado: $DIAG"
    exit 1
fi

echo "=== Extração de evidência para $SC_ID ==="
echo "Logcat: $LOGCAT"
echo "Diag:   $DIAG"
echo ""

case "$SC_ID" in
  "SC-001")
    echo "--- SC-001: Playback survives screen-off ---"
    echo "Transport playing entries:"
    grep 'transport.*playing' "$LOGCAT" | head -10
    echo ""
    echo "Position advances (last 5):"
    grep 'position_secs' "$LOGCAT" | tail -5
    ;;

  "SC-002")
    echo "--- SC-002: Headphone/A2DP pause ≤1s ---"
    echo "Routing signals (BecomingNoisy/DeviceDisappeared):"
    grep -E '(BecomingNoisy|DeviceDisappeared|DeviceAppeared)' "$LOGCAT"
    echo ""
    echo "Transport pauses:"
    grep 'transport.*paused' "$LOGCAT"
    echo ""
    echo "Latency analysis (routing signal → transport pause):"
    # Extrai timestamps aproximados
    awk '
    /BecomingNoisy|DeviceDisappeared/ { signal_time=$2; signal_line=$0 }
    /transport.*paused/ && signal_time {
      split(signal_time, st, ":");
      split($2, pt, ":");
      signal_ms = st[1]*3600000 + st[2]*60000 + st[3]*1000 + substr(st[4],1,3);
      pause_ms = pt[1]*3600000 + pt[2]*60000 + pt[3]*1000 + substr(pt[4],1,3);
      latency = pause_ms - signal_ms;
      if (latency >= 0 && latency <= 5000) {
        printf "  Signal: %s\n  Pause:  %s\n  Latency: %d ms %s\n\n", signal_line, $0, latency, (latency <= 1000 ? "✅ PASS" : "❌ FAIL")
      }
      signal_time=""
    }' "$LOGCAT"
    ;;

  "SC-003")
    echo "--- SC-003: Duckable transient 15-25% gain ---"
    echo "Duck gain entries:"
    grep 'focus.duck gain=' "$LOGCAT"
    echo ""
    echo "Gain restore (1.00):"
    grep 'gain=1.00' "$LOGCAT"
    echo ""
    echo "Transport during duck (should stay playing):"
    grep -B2 -A2 'focus.duck gain=' "$LOGCAT" | grep 'transport'
    ;;

  "SC-041")
    echo "--- SC-041: Silence-demanding pause + rewind math ---"
    echo "SilenceDemanding focus loss:"
    grep 'SilenceDemanding' "$LOGCAT"
    echo ""
    echo "Focus regained:"
    grep 'FocusRegained' "$LOGCAT"
    echo ""
    echo "Rewind math:"
    grep -i 'rewind' "$LOGCAT"
    echo ""
    echo "Resume position (sentence snap):"
    grep -E '(resume|cursor|snap)' "$LOGCAT" | head -10
    ;;

  "SC-005"|"SC-012"|"SC-019")
    echo "--- $SC_ID: Diagnostic record scan ---"
    python3 << 'PYEOF'
import json
import sys

logcat_file = sys.argv[1]
diag_file = sys.argv[2]
sc_id = sys.argv[3]

last_playing_pos = None
last_playing_ts = None
violations = []

with open(diag_file) as f:
    for line_num, line in enumerate(f, 1):
        line = line.strip()
        if not line:
            continue
        try:
            rec = json.loads(line)
        except json.JSONDecodeError:
            continue

        kind = rec.get('kind')
        pos = rec.get('position_secs')
        transport = rec.get('transport')

        if kind == 'transport_change' and transport == 'playing':
            if last_playing_pos is not None and pos <= last_playing_pos:
                violations.append({
                    'line': line_num,
                    'prev_pos': last_playing_pos,
                    'curr_pos': pos,
                    'ts': rec.get('ts_ms')
                })
            last_playing_pos = pos
            last_playing_ts = rec.get('ts_ms')

        # SC-012: no auto-audio from unexpected end
        if kind == 'session_end' and transport == 'playing':
            violations.append({
                'line': line_num,
                'type': 'session_end_with_playing',
                'rec': rec
            })

        # SC-019: no session_end with transport:playing
        if kind == 'session_end' and transport == 'playing':
            violations.append({
                'line': line_num,
                'type': 'session_end_playing_violation',
                'rec': rec
            })

if violations:
    print(f"❌ VIOLAÇÕES {sc_id} ({len(violations)}):")
    for v in violations:
        print(f"  Line {v['line']}: {json.dumps(v)}")
else:
    print(f"✅ {sc_id}: Nenhuma violação encontrada no diag.jsonl")

print(f"\nTotal entries scanned: {line_num}")
print(f"Last playing position: {last_playing_pos}")
PYEOF
    ;;

  *)
    echo "SC_ID não reconhecido: $SC_ID"
    echo "SCs suportados: SC-001, SC-002, SC-003, SC-041, SC-005, SC-012, SC-019"
    exit 1
    ;;
esac