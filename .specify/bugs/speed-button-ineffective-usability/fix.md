# Bug Fix: Botão de velocidade de reprodução e usabilidade aprimorada

- **Slug**: `speed-button-ineffective-usability`
- **Fixed**: 2026-11-11T01:50:00-04:00
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

1. **Multiplicador de velocidade no sintetizador TTS (`adapter-synthesis`)**:
   - `crates/adapter-synthesis/src/synth.rs`: integrado multiplicador atômico de taxa (`ACTIVE_RATE_MULTIPLIER_CENTIS`, clamp [0.5, 4.0]) e funções públicas `set_playback_rate` e `playback_rate`. No método `synthesize()`, `GenerationConfig.speed` agora combina a velocidade do plano prosódico com o multiplicador de taxa de reprodução (`effective_speed = (prosody_speed * rate_multiplier).clamp(0.5, 4.0)`).
2. **Propagação imediata e re-ancoragem em tempo real (`src-tauri`)**:
   - Em `src-tauri/src/lib.rs`: criada a conversão `rate_to_f32(rate: PlaybackRate)`.
   - Na inicialização (`seed_session_rate`) e a cada evento de taxa (`execute_live_control`), `adapter_synthesis::set_playback_rate()` é chamado com a taxa selecionada.
   - Em reprodução ativa (`Playing`), dispara `reanchor_at(state, snapshot.position_secs)` para re-sintetizar e reiniciar o stream de áudio na nova velocidade de forma imediata e perceptível.
   - Permitida a persistência e ajuste de taxa mesmo em estados de finalização (`Completed` e `Stopped`), evitando que cliques de taxa sejam rejeitados nesses estados.
   - Ajustado `resumable_transport` para admitir retomadas e recomeços adequados.
3. **Novo ribbon `wave-04-aurora-breeze`**:
   - Integrado o novo design de ondas suaves atrás do monograma "R" no topo do aplicativo (`DESIGN-SONA/assets/waves/waves-top-ribbons.png` e SVG correspondente).
4. **Usabilidade e Layout da Ilha Flutuante (`.bottom-island`)**:
   - Refatorado o layout da ilha flutuante para coluna hierárquica em dois níveis (Tier 1: controles de transporte `[↻] [⏮] [▶] [⏭]`; Tier 2: controles de velocidade `[−] [1.25×] [+]`), eliminando o problema de botões quebrando para fora da ilha ou da tela em dispositivos móveis.
   - Controles mantidos responsivos e acessíveis.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `crates/adapter-synthesis/src/synth.rs` | modified | Multiplicador atômico de velocidade aplicado a `GenerationConfig.speed`. |
| `crates/adapter-synthesis/src/lib.rs` | modified | Re-exportadas `set_playback_rate` e `playback_rate`. |
| `src-tauri/src/lib.rs` | modified | Conversão `rate_to_f32`, sincronização em `seed_session_rate` e `execute_live_control`, reancoragem em `Playing`, suporte a taxa em `Completed`/`Stopped`. |
| `dist/index.html` | modified | Ilha flutuante em 2 níveis, botões com áreas de toque adequadas, sem quebra de tela. |
| `src-tauri/src/status_view.html` | modified | Sincronizado com `dist/index.html`. |
| `src-tauri/src/assets/index.html` | modified | Sincronizado para empacotamento webview. |
| `src-tauri/gen/android/app/src/main/assets/index.html` | modified | Atualizado para build do Android. |

## Verification

1. **Gates 1 a 8 (`./scripts/verify.sh`)**:
   - Gate 1 (format check): PASS
   - Gate 2 (nextest suite + expected-tests.txt diff): PASS (642/1058 testes executados com 0 falhas)
   - Gate Surface-Audit (C-H1..C-H9): PASS
   - Gate 3 (docs): PASS
   - Gate 4 (clippy -D warnings): PASS
   - Gate 5 (supply chain / deny): PASS
   - Gate 6 (deny-list scan): PASS
   - Gate 7 (closed-set): PASS
   - Gate 8 (pipeline & shell purity): PASS
2. **Gate 9 & Hardware Validation**:
   - `cargo tauri android build --target aarch64 --debug`: Build do APK concluído com sucesso.
   - Instalação e execução no dispositivo físico Xiaomi Redmi Note 11 (Android 13 / arm64-v8a) via ADB.
   - Carregamento de documento e reprodução de áudio TTS natural confirmada.
   - Mudança de velocidade em tempo real (1.0x -> 1.25x -> 1.5x -> 1.75x -> 2.0x -> 1.25x) verificada acusticamente e validada no log diagnóstico (`action=rate rate_from=... rate_to=...` e avanço acelerado do clock de áudio).
   - Comportamento de layout da ilha e botões verificado sem quebras.
