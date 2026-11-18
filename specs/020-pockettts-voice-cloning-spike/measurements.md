# Spike 020 — PocketTTS + voz “Ressoa” (PoC) — Medições

Data: 2026-11-15 (UTC). Status: **validação direta on-device (item 1) concluída**; integração mdreader/fim-a-fim (item 2) **não realizada**; gate 80/20 (item 3) **NO-GO** com evidência direta atual.

## Contexto e referências canônicas
- Dispositivo de referência: Xiaomi Redmi Note 11 `2201117TY` (spesn), Android 13 SDK 33, Qualcomm SM6225/bengal (SD680), aarch64. Seriais: `192.168.100.102:39761`, `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp` (execução da matriz via TLS).
- Voz “Ressoa” (referência de clonagem, canônica): `tts-work/voices/ressoa/reference_30s_24k.wav` (origem `tts-work/voz_mdreader_30s.wav`) — PCM16 mono **24000 Hz, 29,88 s**, 1.434.284 B, SHA256 `ca471e2fe598aa146a38f80a5976ca51cc84a80434ae92368bfe1fd1477e52a0`. Manifesto `tts-work/voices/ressoa/voice.json`. Cópia no device: `/data/local/tmp/ressoa_ref_30s.wav`.
- Bundle modelo PocketTTS pt (haex/sherpa layout, ~140 MB, **não embarcado no APK**, download on-demand simulado): `haex-space/pocket-tts-sherpa-onnx/pt` em `tts-work/models/ressoa-pocket-pt/pt` e `/data/local/tmp/ressoa-pocket-pt/pt/` (`decoder.int8.onnx`, `encoder.onnx`, `lm_flow.int8.onnx`, `lm_main.int8.onnx`, `text_conditioner.onnx`, `token_scores.json`, `vocab.json`). Compatibilidade exata bundle↔runtime `[NÃO VERIFICADO]` além do PoC executado.
- Bounds/gates: SC-002 **3000 ms** (`play_to_audio_ms`); referência atestada Piper **1551 ms** (T051/T052).

## Baseline Piper (app mdreader, direto no device, automatizado)
- Script: `mdreader/scripts/device-ttfa-validation.sh` (APK debug debuggable, `DebugIngestActivity`, `diagnostic-record.jsonl`, extração `play_to_audio_ms`).
- Execução 2026-11-15T173301Z, relatório `mdreader/device-ttfa-20261115T173301Z/` (exit 1): ingest OK (`ok:document:/data/local/tmp/ttfa-validation.md`), diag capturado (11 eventos), sem crash/ANR, media session ativa.
  - **`play_to_audio_ms = 14059 ms` — FAIL (> 3000 ms)**. Sonda consistente: 14368 ms.
- Histórico atestado: **1551 ms** (T051/T052, PASS).

## Item 1 — PoC PocketTTS + Ressoa on-device (2201117TY) [CONCLUÍDO]

**Runtime/harness compatível:** `ressoa_ttfa_device.c` compilado com NDK r29 `aarch64-linux-android29-clang` (Android 29), linkado dinamicamente às libs arm64 do prebuilt sherpa-onnx v1.13.8 em `/data/local/tmp/` (`libsherpa-onnx-c-api.so`, `libonnxruntime.so`, `LD_LIBRARY_PATH=/data/local/tmp`). C API 1.13.8: `SherpaOnnxReadWave`, `SherpaOnnxCreateOfflineTts` (Pocket), `SherpaOnnxOfflineTtsGenerateWithConfig` com `reference_audio/ref_sr` e callback `SherpaOnnxGeneratedAudioProgressCallbackWithArg` para **`first_chunk_ms`**. Mede `read_ref_ms`, `model_load_ms`, `synth_total_ms`, `audio_secs` (amostras/sr), `rtf`, `VmRSS/VmHWM` (`/proc/self/status`). Binário: `/data/local/tmp/ressoa_ttfa_device`. Texto: `Olá, esta é uma validação da voz Ressoa no dispositivo de referência.` (74 chars), `max_num_sentences=1`.

**Caveat de métrica:** `first_chunk_ms` = tempo até 1º callback de geração (síntese). **Não é** `play_to_audio_ms` fim-a-fim do mdreader (exclui pipeline app, enqueue/pre-roll/player, decodificação de pacote on-demand, saída audível). `[NÃO VERIFICADO em integração mdreader]`

**Matriz (processo novo por config = cold load medido; 2 iters/processo: iter0 pós-load, iter1 warm/reuso no mesmo processo; `speed=1.0` salvo nota; `rc=0` em todas).** Fonte: `/tmp/ressoa_device_matrix.txt`.

| steps | threads | speed | model_load_ms | read_ref_ms | iter | first_chunk_ms | synth_total_ms | audio_s | rtf | VmHWM pós-load (kB) | VmHWM pico (kB) |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1 | 2.0* | 3937.021 | 24.844 | 0 (cold) | 25107.982 | 29934.113 | 4.560 | 6.564498 | — | — |
| 1 | 1 | 2.0* | — | — | 1 (warm) | — | — | — | — | — | — |
| 1 | 2 | 1.0 | 3690.518 | 23.518 | 0 (cold) | 15727.870 | 18687.562 | 4.467 | 4.183470 | 284692 | 1253616 |
| 1 | 2 | 1.0 | — | — | 1 (warm) | 6690.010 | 9328.686 | 4.320 | 2.159418 | — | 1253616 |
| 1 | 4 | 1.0 | 3766.831 | 24.653 | 0 (cold) | 16259.563 | 19342.800 | 5.110 | 3.785160 | 283684 | 1227480 |
| 1 | 4 | 1.0 | — | — | 1 (warm) | 8659.699 | 10577.413 | 4.240 | 2.494673 | — | 1227480 |
| 2 | 1 | 1.0 | 3501.629 | 20.937 | 0 (cold) | 24616.334 | 29869.960 | 4.880 | 6.120893 | 291332 | 1294332 |
| 2 | 1 | 1.0 | — | — | 1 (warm) | 8784.423 | 12858.817 | 3.867 | 3.325126 | — | 1294332 |
| 2 | 2 | 1.0 | 3539.737 | 29.163 | 0 (cold) | 14920.498 | 17827.643 | 4.387 | 4.064129 | 279008 | 1218680 |
| 2 | 2 | 1.0 | — | — | 1 (warm) | 7021.321 | 9654.712 | 4.153 | 2.324570 | — | 1218680 |
| 2 | 4 | 1.0 | 3414.873 | 17.119 | 0 (cold) | 14745.896 | 17108.578 | 4.800 | 3.564287 | 279880 | 1205240 |
| 2 | 4 | 1.0 | — | — | 1 (warm) | 10462.162 | 13026.305 | 4.640 | 2.807393 | — | 1205240 |
| 4 | 1 | 1.0 | 3332.378 | 16.574 | 0 (cold) | 26113.020 | 32056.249 | 5.280 | 6.071259 | 289112 | 1295416 |
| 4 | 1 | 1.0 | — | — | 1 (warm) | 10599.692 | 16112.832 | 4.960 | 3.248555 | — | 1295416 |
| 4 | 2 | 1.0 | 3750.712 | 24.581 | 0 (cold) | 16145.862 | 19411.065 | 4.960 | 3.913521 | 279688 | 1287504 |
| 4 | 2 | 1.0 | — | — | 1 (warm) | 7210.259 | 9921.371 | 4.400 | 2.254857 | — | 1287504 |
| 4 | 4 | 1.0 | 2993.671 | 16.806 | 0 (cold) | 9803.797 | 11464.918 | 4.320 | 2.653916 | 276248 | 1222076 |
| 4 | 4 | 1.0 | — | — | 1 (warm) | 8983.705 | 11071.999 | 5.440 | 2.035294 | — | 1222076 |

\* `steps=1,threads=1,iter=0` veio de execução smoke anterior com harness que registrou `speed=2.000` (sem RSS por iteração, sem iter1 instrumentado); mantido por rastreabilidade — não é o melhor caso.

**Sumário direto (todas as configs medidas, speed=1.0 onde aplicável):**
- `model_load_ms`: **2994–3937 ms** (bundle ~140 MB + estado/ref).
- `read_ref_ms`: **16,6–29,2 ms**.
- Cold `first_chunk_ms` (síntese, iter0): **9804–26113 ms** (melhor: s=4,t=4; pior: s=4,t=1).
- Warm `first_chunk_ms` (mesmo processo, iter1): **6690–10599 ms** (melhor: s=1,t=2).
- `RTF`: **2,035–6,565** (sempre **> 1**; melhor warm: s=4,t=4, RTF 2,035).
- `VmHWM` pico: **~1,18–1,30 GB** durante/após síntese (retorna ~34–55 MB RSS após destroy/free; HWM permanece alto no processo).

## PoC proxy host (x86_64, NÃO substitui device/app) [EXECUTADO]
Harness Rust `/tmp/ressoa-ttfa-probe` (`sherpa-onnx=1.13.8`, callback), mesmo bundle/ref, 2 threads, 69 chars:
- `num_steps=5`: load 1047,6 ms, first_chunk 4649,5 ms, RTF 1,1995 (24 kHz).
- `num_steps=2`: load 1046,1 ms, first_chunk 4324,4 ms, RTF 1,1419.
- `num_steps=1`: load 963,1 ms, first_chunk 4381,8 ms, RTF 1,1539.

## Item 2 — `play_to_audio_ms` fim-a-fim no mdreader [NÃO EXECUTADO]
- **Motivo:** Ressoa/PocketTTS não integrado ao pipeline do mdreader (adapter VITS-only, sem `reference_audio`/streaming por callback, contrato/handoff 22.05 kHz vs 24 kHz, sem pacote/manifesto/hash/licença on-demand). Com `RTF > 1` e `first_chunk_ms` de síntese mínimo observado **~6,7 s** (warm) / **~9,8 s** (cold) no SD680, o TTFA audível fim-a-fim (`play_to_audio_ms`, inclui pipeline/enqueue/pre-roll/player) seria necessariamente **pior** `[INFERÊNCIA]` e **incompatível** com 3000 ms/1551 ms — inviabilizando spike de integração sob o gate atual. `[NÃO VERIFICADO em integração mdreader]`

## Item 3 — Gate 80/20 (decisão final do PoC) [NO-GO]
- **Evidência direta on-device (2201117TY/SD680):** em todas as 9 configs × cold/warm medidas, **`RTF > 1`** (2,04–6,56) e **`first_chunk_ms` de síntese > 6,6 s** (melhor warm) / **> 9,8 s** (melhor cold), com `model_load_ms` ~3–4 s e pico de memória ~1,2–1,3 GB. Isso **não atende** SC-002 (3000 ms) nem 1551 ms, mesmo antes do overhead fim-a-fim do app.
- **Decisão:** **NO-GO** para uso/integração de Ressoa via PocketTTS neste perfil de device/runtime/modelo medido. **Manter Piper pt-BR como padrão.**
- **Ressoa (voz opcional, baixada sob demanda):** somente se PoC futuro demonstrar viabilidade on-device (TTFA/RTF/RSS/estabilidade) em runtime/modelo/quantização compatíveis **e** houver clearing de licenças/uso de voz e pesos (Kyutai/HF gated/termos, consentimento/distribuição da referência) — **requer autorização explícita; não concluído**. Base/modelo **nunca** embutido como payload multi-GB no APK.
- **Fallbacks 80/20 preservados:** otimizar TTFA/prewarm/handover do Piper; avaliar engine Ressoa-compatível opcional baixada ou Piper custom/bake 22.05 kHz apenas com recursos/direitos confirmados.

## Artefatos e reprodutibilidade
- Matriz bruta: `/tmp/ressoa_device_matrix.txt` (host). Harness: `/tmp/ressoa_ttfa_device.c` + `/tmp/ressoa_ttfa_device` (aarch64 Android 29). Libs runtime device: `/data/local/tmp/libsherpa-onnx-c-api.so`, `/data/local/tmp/libonnxruntime.so` (prebuilt sherpa-onnx v1.13.8). Dados device: `/data/local/tmp/ressoa_ref_30s.wav`, `/data/local/tmp/ressoa-pocket-pt/pt/*`.
- Execução por config (ex.): `adb -s <serial> shell "cd /data/local/tmp && LD_LIBRARY_PATH=/data/local/tmp /data/local/tmp/ressoa_ttfa_device /data/local/tmp/ressoa-pocket-pt/pt /data/local/tmp/ressoa_ref_30s.wav \"<texto>\" <steps> <threads> 2 1.0"`
- Limitações: métricas de síntese (callback), não playback audível fim-a-fim; qualidade prosódica/aderência da ref 30 s e compat bundle↔runtime não validadas; binários sherpa `offline-tts` termux-static 1.13.8/2.0.1 falharam no device por símbolo libc++ ausente (contornado via harness C-API próprio).
