# Tasks: 020 PocketTTS Voice Cloning Spike

- **Spec**: `specs/020-pockettts-voice-cloning-spike/spec.md`
- **Plan**: `specs/020-pockettts-voice-cloning-spike/plan.md`
- **Legend**: `[x]` executed/recorded · `[ ]` pending · `[~]` partial/blocked

## Phase 0 — Desbloqueio de medição (pré-requisito)

- [x] T001 Reconectar device `2201117TY` via `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp` / `192.168.100.102:39761` (FR-PKT-003)
- [x] T002 Fixar identidade e hash da referência Ressoa (`tts-work/voices/ressoa/reference_30s_24k.wav`, SHA256 `ca471e2f…7e52a0`) e manifesto `voice.json` (FR-PKT-001)
- [~] T003 Garantir debug APK debuggable com `DebugIngestActivity` empacotada para captura de `play_to_audio_ms` via `run-as` / `scripts/device-ttfa-validation.sh` (FR-PKT-006) — app v0.1.0 não debuggable na execução inicial; bloqueio registrado
- [x] T004 Registrar baseline atestado Piper (1551 ms, T051/T052) e medição direta do app (14059 ms) em `measurements.md` (FR-PKT-005)

## Phase 1 — PoC TTFA fora do mdreader

- [x] T005 Selecionar bundle PocketTTS pt em layout sherpa (`haex-space/pocket-tts-sherpa-onnx/pt`, ~140 MB), fora do APK, com direção de download sob demanda (FR-PKT-002)
- [ ] T006 Confirmar compatibilidade exata bundle ↔ runtime sherpa 1.13.8 vs ≥2.0.1 (fix BOS) — `[NÃO VERIFICADO]` além do PoC executado (FR-PKT-002)
- [x] T007 Compilar harness C-API arm64 (`/tmp/ressoa_ttfa_device.c` → `/tmp/ressoa_ttfa_device`, NDK r29, libs v1.13.8 em `/data/local/tmp`) (FR-PKT-003)
- [x] T008 Medir matriz `num_steps ∈ {1,2,4}` × threads `∈ {1,2,4}`, cold/warm, com `speed=1.0` (excite smoke flagrado), registrando `model_load_ms`, `read_ref_ms`, `first_chunk_ms`, `synth_total_ms`, `rtf`, `VmRSS`/`VmHWM` (FR-PKT-004)
- [x] T009 Registrar matriz bruta, comandos por configuração e limitações em `measurements.md` (FR-PKT-011)
- [x] T010 Aplicar gate 80/20: `first_chunk_ms` ≤ ~1551 ms cold e warm, `RTF < 1`, RSS/estabilidade aceitáveis → **NO-GO** com evidência (FR-PKT-005, SC-PKT-002/003)
- [x] T011 Explicitar que `first_chunk_ms` é síntese (callback), não `play_to_audio_ms` fim-a-fim; não reivindicar TTFA de app (FR-PKT-006, SC-PKT-004)
- [x] T012 Registrar proxy host x86_64 (`num_steps 5/2/1`) como evidência complementar, não substituto do device (FR-PKT-004)
- [ ] T013 [NÃO EXECUTADO] Medir `play_to_audio_ms` fim-a-fim no mdreader com Ressoa integrado — pré-requisito: GO + integração (FR-PKT-006/008)

## Phase 2 — Spike de integração (somente se GO) — NÃO AUTORIZADO

- [ ] T014 Introduzir backend/driver PocketTTS em `adapter-synthesis` com `reference_audio` + cache de embedding e `generate_with_callback` (FR-PKT-008)
- [ ] T015 Resolver formato: resample 24→22,050 Hz mono f32 no adapter, sem refatoração global do player (FR-PKT-008)
- [ ] T016 Representar voz clonada sem quebrar `VoiceId` fechado (extensão controlada + docs + testes) (FR-PKT-008)
- [ ] T017 Empacotar modelo/referência sob demanda com manifesto, hash, licença e consentimento; nunca embutir no APK (FR-PKT-002/010)
- [ ] T018 Adicionar diagnósticos `play_to_audio_ms`, `voice_state_ms` e métricas de engine para voz clonada (FR-PKT-008)
- [ ] T019 Rodar gates 1–9 com a mudança de produção (somente após GO) (NFR-PKT-003)

## Phase 3 — Fallback (NO-GO) — TRILHA PRESERVADA

- [x] T020 Manter Piper pt-BR como padrão; registrar NO-GO e racional em `measurements.md` (FR-PKT-009, SC-PKT-003)
- [ ] T021 Otimizar TTFA Piper (prewarm, segmentação/handover, profiling no device) (FR-PKT-009)
- [ ] T022 Avaliar Piper custom/bake 22.050 Hz apenas com dados e direitos confirmados (FR-PKT-009/010)
- [x] T023 Registrar pré-condições de licença/consentimento (Kyutai/HF gated, voz de terceiros, áudio de vídeo) para eventual reavaliação (FR-PKT-010, SC-PKT-006)
- [x] T024 Criar artefatos SDD de apoio (spec/plan/tasks/checklist/assessment/validate/release) sem alterar `measurements.md` nem `PLAN-80-20.md` (SC-PKT-001/005)
