# Guia de Captura de Logs para Trials On-Device (T086)

Este documento descreve como capturar **logcat** e **diag.jsonl** durante os trials para evidência no `VERIFICATION.md`.

---

## 1. Preparação do Ambiente

### 1.1 Device conectado
```bash
adb devices
# Deve mostrar: 53cf9aa7  device
```

### 1.2 Limpar logs antes de cada trial
```bash
adb logcat -c
```

### 1.3 Iniciar captura contínua (em terminal separado)
```bash
# Opção A: logcat filtrado para mdreader (recomendado para trials)
adb logcat -v time -s mdreader:V RustStdoutStderr:V Tauri:V FocusMirror:V RoutingMirror:V MediaMirror:V Residency:V ShellHost:V PlaybackService:V *:S > ~/mdreader_trial_$(date +%Y%m%d_%H%M%S).log

# Opção B: logcat completo (para debug profundo)
adb logcat -v time > ~/mdreader_full_$(date +%Y%m%d_%H%M%S).log
```

---

## 2. Estrutura dos Logs Esperados

### 2.1 Tags principais do mdreader

| Tag | Origem | Conteúdo |
|-----|--------|----------|
| `mdreader` | Rust `tracing` (player, session, diag, engine-core) | Transport changes, position updates, focus decisions, rewind math, engine handoff |
| `RustStdoutStderr` | Tauri stdout/stderr | Panics, errors, diagnostic output |
| `Tauri` | Tauri IPC | Plugin registration, command dispatch |
| `FocusMirror` | Kotlin `FocusMirror.kt` | AudioFocusRequest callbacks, intent forwarding |
| `RoutingMirror` | Kotlin `RoutingMirror.kt` | `ACTION_AUDIO_BECOMING_NOISY`, `onAudioDevicesRemoved` |
| `MediaMirror` | Kotlin `MediaMirror.kt` | MediaSession state updates |
| `Residency` | Kotlin `Residency.kt` | Resident service lifecycle, notification posts |
| `ShellHost` | Kotlin `ShellHost.kt` | Bootstrap, JNI channel init |
| `PlaybackService` | Kotlin `PlaybackService.kt` | Service commands (PLAY/PAUSE/STOP) |

### 2.2 Padrões de log críticos por trial

#### T-01 (SC-001) — Screen-off playback
```
mdreader: transport: "playing" position_secs=12.3
mdreader: transport: "playing" position_secs=17.8
... (continua avançando durante 5 min tela off)
```

#### T-02 (SC-002) — Headphone pull
```
RoutingMirror: onAudioDevicesRemoved: device=..., privacy=Private
mdreader: RoutingSignal::DeviceDisappeared -> Pause { latency_ms: 250 }
mdreader: transport: "paused" position_secs=45.2
```
**Latência** = timestamp do `RoutingMirror` → timestamp do `transport: "paused"` (deve ser ≤1000ms)

#### T-03 (SC-003) — Duckable transient
```
FocusMirror: onAudioFocusChange: AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK
mdreader: FocusIntent::DuckableTransient -> gain=0.20
mdreader: transport: "playing" position_secs=... (continua avançando)
FocusMirror: onAudioFocusChange: AUDIOFOCUS_GAIN
mdreader: FocusIntent::FocusRegained -> gain=1.00
```

#### T-04 (SC-041) — Silence-demanding + rewind math
```
FocusMirror: onAudioFocusChange: AUDIOFOCUS_LOSS_TRANSIENT
mdreader: FocusIntent::SilenceDemanding -> Paused, gain=1.00
... (duração D segundos) ...
FocusMirror: onAudioFocusChange: AUDIOFOCUS_GAIN
mdreader: FocusIntent::FocusRegained -> resume at R (sentence snap)
mdreader: rewind: yield=P paused=D cursor=P-5% snap=R backward=P-R≤30
```

#### T-11 (SC-013) — Engine failure + fallback
```
mdreader: EngineError::Unreachable (or other)
mdreader: Failure offer: alternatives=[standin_b] window=3s
mdreader: Fallback to standin_b from position X
mdreader: transport: "playing" engine_id="standin_b"
```

#### SC-005 / SC-012 / SC-019 — Diagnostic record scan
```
diag: Writing record: {"v":1,"ts_ms":...,"kind":"transport_change","position_secs":...}
diag: Writing record: {"v":1,"ts_ms":...,"kind":"focus_change","position_secs":...}
```

---

## 3. Captura do Diagnostic Record (diag.jsonl)

### 3.1 Localização no device (debug builds)
```
/data/data/app.mdreader/files/diag.jsonl
```

### 3.2 Ler durante/ao final de cada trial
```bash
# Ler últimas N linhas
adb shell run-as app.mdreader cat files/diag.jsonl | tail -20

# Ler todo o arquivo (salvar localmente)
adb shell run-as app.mdreader cat files/diag.jsonl > ~/diag_trial_$(date +%Y%m%d_%H%M%S).jsonl

# Filtrar por kind específico
adb shell run-as app.mdreader cat files/diag.jsonl | grep '"kind":"focus_change"'
```

### 3.3 Estrutura do JSONL (conforme `diagnostic-record.schema.json`)
```json
{
  "v": 1,
  "ts_ms": 1728754321000,
  "kind": "transport_change",
  "session_id": "abc123...",
  "position_secs": 45.2,
  "transport": "playing",
  "engine": "standin_a",
  "engine_tier": "test_double"
}
{
  "v": 1,
  "ts_ms": 1728754322000,
  "kind": "focus_change",
  "position_secs": 45.2,
  "focus": "DuckableTransient",
  "gain": 0.20
}
{
  "v": 1,
  "ts_ms": 1728754325000,
  "kind": "rewind",
  "position_secs": 44.5,
  "yield_secs": 45.2,
  "paused_duration_secs": 12.0,
  "cursor_secs": 44.6,
  "resume_secs": 44.0,
  "backward_secs": 1.2
}
```

---

## 4. Scripts de Automação para Trials

### 4.1 Script: `capture_trial.sh` — Captura automática com timestamp
```bash
#!/bin/bash
# Uso: ./capture_trial.sh "T-01" "SC-001" "trial_1"

TRIAL_ID="$1"
SC_ID="$2"
TRIAL_NUM="$3"
TS=$(date -u +%Y%m%d_%H%M%S)
OUT_DIR=~/mdreader_evidence/${TRIAL_ID}_${SC_ID}_${TRIAL_NUM}_${TS}

mkdir -p "$OUT_DIR"

echo "=== Iniciando captura para $TRIAL_ID / $SC_ID trial $TRIAL_NUM ==="
echo "Output dir: $OUT_DIR"

# Limpar logcat
adb logcat -c

# Capturar logcat em background
adb logcat -v time -s mdreader:V RustStdoutStderr:V Tauri:V FocusMirror:V RoutingMirror:V MediaMirror:V Residency:V ShellHost:V PlaybackService:V *:S > "$OUT_DIR/logcat.log" &
LOGCAT_PID=$!

# Capturar diag.jsonl inicial
adb shell run-as app.mdreader cat files/diag.jsonl > "$OUT_DIR/diag_before.jsonl" 2>/dev/null || echo "diag.jsonl não existe ainda" > "$OUT_DIR/diag_before.jsonl"

echo "Captura iniciada. PID logcat: $LOGCAT_PID"
echo "Pressione ENTER quando o trial terminar..."
read

# Parar logcat
kill $LOGCAT_PID 2>/dev/null
wait $LOGCAT_PID 2>/dev/null

# Capturar diag.jsonl final
adb shell run-as app.mdreader cat files/diag.jsonl > "$OUT_DIR/diag_after.jsonl" 2>/dev/null

echo "=== Captura finalizada ==="
echo "Arquivos em: $OUT_DIR"
ls -la "$OUT_DIR"
```

### 4.2 Script: `extract_sc_evidence.sh` — Extrai evidência por SC
```bash
#!/bin/bash
# Uso: ./extract_sc_evidence.sh <logcat_file> <diag_file> <SC_ID>

LOGCAT="$1"
DIAG="$2"
SC_ID="$3"

case "$SC_ID" in
  "SC-001")
    grep 'transport.*playing' "$LOGCAT" | head -5
    grep 'position_secs' "$LOGCAT" | awk '{print $NF}' | sort -n | tail -1
    ;;
  "SC-002")
    grep -E '(BecomingNoisy|DeviceDisappeared)' "$LOGCAT"
    grep 'transport.*paused' "$LOGCAT"
    # Calcular latência entre sinal e pause
    ;;
  "SC-003"|"SC-041")
    grep 'focus.duck gain=' "$LOGCAT"
    grep 'gain=1.00' "$LOGCAT"
    grep 'rewind:' "$LOGCAT"
    ;;
  "SC-005"|"SC-012"|"SC-019")
    # Scan do diag.jsonl para playing sem advance
    python3 -c "
import json, sys
last_playing = None
for line in open('$DIAG'):
    rec = json.loads(line)
    if rec.get('kind') == 'transport_change' and rec.get('transport') == 'playing':
        if last_playing and rec['position_secs'] <= last_playing:
            print(f'VIOLAÇÃO SC-005: position não avançou: {last_playing} -> {rec[\"position_secs\"]}')
        last_playing = rec['position_secs']
"
    ;;
esac
```

---

## 5. Checklist de Evidência por Trial

Para **CADA trial**, salve:

| Arquivo | Descrição |
|---------|-----------|
| `logcat.log` | Logcat filtrado completo do trial |
| `diag_before.jsonl` | Estado do diagnostic record antes |
| `diag_after.jsonl` | Estado do diagnostic record depois |
| `notes.txt` | Observações manuais (timestamp, ação, resultado) |
| `screenshot_*.png` | (Opcional) Lock screen, notificação, etc. |

---

## 6. Integração com VERIFICATION.md

Ao preencher o `VERIFICATION.md`, cite a evidência assim:

```markdown
### SC-002 — Headphone pull pause ≤1s
**Verdict**: PASS (10/10)
**Evidência**:
- Logcat: `mdreader_evidence/T-02_SC-002_trial_1_20261012_143000/logcat.log` linhas 15-23
- Latências medidas: 0.32s, 0.41s, 0.28s, 0.35s, 0.39s, 0.31s, 0.44s, 0.29s, 0.37s, 0.33s (todas ≤1s)
- Diag: `diag_after.jsonl` entries kind=routing_change com latency_ms=250
- Speaker silence confirmado visual/auditivamente em todos os 10 trials
```

---

## 7. Comandos Rápidos de Referência

```bash
# Ver logcat em tempo real (filtro mdreader)
adb logcat -v time -s mdreader:V *:S

# Ver diagnostic record atual
adb shell run-as app.mdreader cat files/diag.jsonl | tail -10

# Ver media session state
adb shell dumpsys media_session | grep -A 20 "app.mdreader"

# Ver notification state
adb shell dumpsys notification --n app.mdreader

# Ver audio focus state
adb shell dumpsys audio | grep -A 10 "Audio Focus"

# Forçar stop do app
adb shell am force-stop app.mdreader

# Reiniciar app
adb shell am start -n app.mdreader/.MainActivity
```

---

## 8. Troubleshooting Comum

| Problema | Solução |
|----------|---------|
| `run-as app.mdreader` falha | App deve ser debuggable (debug build) |
| `diag.jsonl` vazio | App precisa ter escrito pelo menos um evento (inicie playback) |
| Logcat não mostra `mdreader` | Verifique se `tracing` está configurado no Rust (`RUST_LOG=mdreader=trace`) |
| Latência >1s no T-02 | Verifique se `RoutingMirror` está registrado corretamente; teste ambos sinais separadamente |
| Gain não muda no T-03 | Verifique se `FocusMirror` está enviando intent `DuckableTransient` para Rust |