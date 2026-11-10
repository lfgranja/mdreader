# Plan: 016-voz-lula

## Overview

Adicionar suporte à seleção de voz de síntese no adaptador `offline_default`, incluindo uma segunda voz (`lula`) que homenageia o Presidente Luiz Inácio Lula da Silva, mantendo a voz `faber` como padrão.

## Steps

### 1. Estrutura de assets da voz Lula

Criar `crates/adapter-synthesis/assets/voices/pt_BR-lula-medium/` com:
- `PROVENANCE.md` — origem, licença, integridade do modelo
- `MODEL_CARD` — metadados do modelo (idioma, sample rate, speakers)
- `pt_BR-lula-medium.onnx.json` — config do modelo sherpa-onnx (placeholder/documentação)
- `espeak-ng-data/` — árvore de fonemização (referência para a da voz faber, ou cópia)
- **Não inclui** `pt_BR-lula-medium.onnx` nem `tokens.txt` (obtidos separadamente)

### 2. Código — suporte multi-voz em `synth.rs`

- Adicionar enum `Voice { Faber, Lula }` com método `dir_name()`
- Adicionar `static SELECTED_VOICE: OnceLock<Voice>` + `pub fn set_voice(Voice)`
- Modificar `asset_dir_for(voice: Voice) -> PathBuf` — retorna diretório da voz selecionada
- Modificar `model_path(voice)`, `tokens_path(voice)`, `espeak_data_dir(voice)` — voice-aware
- Duplicar `OnceLock` de engine: `ENGINE_FABER` e `ENGINE_LULA` (lazy load por voz)
- Modificar `engine_for(voice: Voice) -> Option<&'static OfflineTts>`
- Modificar `synthesize(segment: &DocumentSegment, voice: Voice)` — aceita parâmetro de voz
- Manter `synthesize(segment)` como wrapper que usa `SELECTED_VOICE` (compatibilidade)

### 3. CLI — `--voice` option

- Adicionar `Voice` como `ValueEnum` no CLI
- Adicionar `--voice` global ao subcommand `Synthesize`
- Chamar `set_voice()` antes de `synthesize_payload()`
- Atualizar `synthesize_payload` para passar voice para `synthesize()`

### 4. Ponte Tauri — `src-tauri/src/lib.rs`

- Chamar `adapter_synthesis::synth::set_voice()` quando configurado (ex.: via comando ou settings)
- Atualizar `synthesize_segment` para passar voice

### 5. Testes

- Atualizar chamadas de `synthesize()` nos testes existentes para passar `Voice::Faber`
- Adicionar teste: selecionar `Voice::Lula` sem .onnx → `InitFailed`
- Adicionar teste: `set_voice(Voice::Lula)` + `synthesize(segment)` usa assets da voz Lula

### 6. Documentação

- Atualizar `PROVENANCE.md` da voz Lula com instruções de como obter/traçar o modelo
- Atualizar `ROADMAP.md` se necessário

## Acceptance

- `cargo build --workspace` compila
- `cargo nextest run --workspace` passa
- `cargo clippy --workspace --all-targets -- -D warnings` limpo
- `cargo fmt --all -- --check` passa