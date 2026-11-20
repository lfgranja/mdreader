# Implementation Plan: 020 PocketTTS Voice Cloning Spike

- **Feature**: 020-pockettts-voice-cloning-spike
- **Spec**: `specs/020-pockettts-voice-cloning-spike/spec.md`
- **Source plan**: `PLAN-80-20.md` (80/20, device-first; kept intact)
- **Evidence log**: `measurements.md` (kept intact)
- **Status**: Executed through Phase 1; gate result NO-GO → Phases 2/3 conditional
- **Created**: 2026-11-15 (artifact backfilled 2026-11-20)

## 1. Summary

Run a device-first go/no-go spike for offline pt-BR cloned voice (Ressoa) on `2201117TY` (Redmi Note 11, SD680, Android 13). Phase 0 unblocks measurement, Phase 1 measures PocketTTS pt synthesis with the Ressoa reference outside the app, Phase 2 integrates into `adapter-synthesis` **only on GO**, Phase 3 executes cheaper Piper fallbacks **on NO-GO**. The gate compares first-audible TTFA against the attested Piper baseline (1551 ms) and SC-002 (3000 ms), with sustained `RTF < 1` and acceptable RSS/stability.

## 2. Technical Context

- **Device**: `2201117TY` (spesn), SM6225/bengal (SD680), Android 13 SDK 33, aarch64; serials `192.168.100.102:39761` / `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp`.
- **Runtime**: prebuilt sherpa-onnx v1.13.8 arm64 C API (`libsherpa-onnx-c-api.so`, `libonnxruntime.so`), NDK r29 (`aarch64-linux-android29-clang`).
- **Reference**: `tts-work/voices/ressoa/reference_30s_24k.wav`, PCM16 mono 24 kHz, 29.88 s, SHA256 `ca471e2f…7e52a0`; manifest `voice.json`; device copy `/data/local/tmp/ressoa_ref_30s.wav`.
- **Model**: `haex-space/pocket-tts-sherpa-onnx/pt` (~140 MB, sherpa layout) at `tts-work/models/ressoa-pocket-pt/pt` and `/data/local/tmp/ressoa-pocket-pt/pt/`; on-demand direction only, never APK payload.
- **Harness**: `/tmp/ressoa_ttfa_device.c` → `/tmp/ressoa_ttfa_device` using `SherpaOnnxReadWave`, `SherpaOnnxCreateOfflineTts` (Pocket), `SherpaOnnxOfflineTtsGenerateWithConfig` with `reference_audio/ref_sr` and progress callback for `first_chunk_ms`; RSS/HWM from `/proc/self/status`.
- **Production context (untouched)**: Piper VITS-only adapter (`crates/adapter-synthesis/src/synth.rs`), mono f32 @ 22,050 Hz handoff, pinned `sherpa-onnx = "=1.13.8"`, closed `VoiceId`, no reference/embedding API, no callback streaming.
- **Measurement**: `first_chunk_ms`, `rtf`, `model_load_ms`, `read_ref_ms`, `synth_total_ms`, `audio_secs`, `VmRSS`/`VmHWM`; cold (new process) and warm (iter1 same process) across steps/threads.

## 3. Phases (from PLAN-80-20.md)

### Phase 0 — Desbloqueio de medição (pré-requisito)
- Reconectar o device via `adb` TLS.
- Tornar o debug APK debuggable com `DebugIngestActivity` empacotada (geração Tauri/Gradle, não overlay pós-build) para capturar `play_to_audio_ms` via `run-as` / `scripts/device-ttfa-validation.sh`.
- Revalidar baseline Piper no device.
- **Execução**: dispositivo conectado; app v0.1.0 não debuggable → `play_to_audio_ms` direto não capturado na data inicial; baseline atestado permanece 1551 ms, medição direta posterior registrou 14059 ms (ver `measurements.md`).

### Phase 1 — PoC TTFA fora do mdreader (maior valor, menor risco)
- Selecionar bundle pt compatível (haex/sherpa layout; verificar 1.13.8 vs ≥2.0.1/BOS).
- Rodar inferência no device com a referência Ressoa e medir a matriz completa; registrar em `measurements.md`.
- Comparar com 1551 ms / 3000 ms e aplicar o gate.
- **Execução**: concluído — 9 configs × cold/warm; resultados em `measurements.md`.

### Phase 2 — Spike de integração (somente se GO)
- Introduzir driver PocketTTS em `adapter-synthesis` com `reference_audio` + cache de embedding e `generate_with_callback` para streaming.
- Resample 24→22,050 Hz mono f32 no adapter (sem refatoração global do player).
- Estender `VoiceId`/contratos de forma controlada (docs + testes); empacotar modelo/referência sob demanda com manifesto/hash/licença.
- Diagnósticos: `play_to_audio_ms`, `voice_state_ms`, métricas de engine para voz clonada.
- **Estado**: **não autorizado** (gate NO-GO).

### Phase 3 — Fallback (se NO-GO)
- Otimizar TTFA Piper (prewarm, segmentação/handover, profiling no device).
- Avaliar bake/custom Piper 22.050 Hz somente com dados e direitos claros.
- **Estado**: trilha preservada; Piper permanece padrão.

## 4. Decision Gate

| Métrica | Alvo (GO) | Observado (Phase 1) | Resultado |
|---|---|---|---|
| `first_chunk_ms` cold | ≤ ~1551 ms | 9804–26113 ms | FAIL |
| `first_chunk_ms` warm | ≤ ~1551 ms | 6690–10599 ms | FAIL |
| `RTF` | < 1 sustentado | 2.035–6.565 (sempre > 1) | FAIL |
| `model_load_ms` | aceitável | 2994–3937 ms | risco |
| `VmHWM` pico | aceitável | ~1.18–1.30 GB | risco |
| `play_to_audio_ms` fim-a-fim | ≤ 3000 ms / ~1551 ms | [NÃO EXECUTADO] (pior por inferência) | [NV] |

**Gate: NO-GO** para integração neste perfil device/runtime/modelo. Piper pt-BR permanece padrão.

## 5. Constitution / Governance Check

| Gate | Status | Notes |
|------|--------|-------|
| R1 (assess → decision before specify) | PASS | `.specify/assessments/pockettts-voice-cloning-spike/decision.md` |
| R3 (clarifications ≤3) | PASS | Gate e métricas definidos em `PLAN-80-20.md`; incógnitas viram validações de implementação |
| Rust authority / handoff | PASS | Spike é harness externo; produção inalterada (FR-PKT-007) |
| Offline / zero INTERNET | PASS | Nenhuma permissão/capacidade adicionada (NFR-PKT-001) |
| Fail-closed / contratos | PASS | Nenhuma mudança em engine de produção (FR-PKT-007/008) |
| Gates 1–9 | N/A | Nenhuma mudança de produção autorizada; gates permanecem com o dono do gate |

## 6. Risks

- **Dispositivo**: SD680 pode não sustentar RTF < 1 / TTFA baixo (confirmado negativamente na medição).
- **Compatibilidade**: mismatch sherpa 1.13.8 ↔ bundles recentes (BOS/formato); binários termux-static falharam por libc++.
- **Licença/consentimento**: pesos Kyutai gated, voz de terceiros, áudio extraído de vídeo — pré-condição para qualquer distribuição.
- **Qualidade pt-BR**: aderência à referência e inteligibilidade em texto longo não validadas.

## 7. Reprodução

Comando por configuração (exemplo):
```
adb -s <serial> shell "cd /data/local/tmp && LD_LIBRARY_PATH=/data/local/tmp \
  /data/local/tmp/ressoa_ttfa_device /data/local/tmp/ressoa-pocket-pt/pt \
  /data/local/tmp/ressoa_ref_30s.wav \"<texto>\" <steps> <threads> 2 1.0"
```
Harness, libs e dados brutos permanecem conforme a seção "Artefatos e reprodutibilidade" de `measurements.md`.
