# Tasks: 016-voz-lula

## T1 — Estrutura de assets da voz Lula
- [ ] Criar `crates/adapter-synthesis/assets/voices/pt_BR-lula-medium/PROVENANCE.md`
- [ ] Criar `crates/adapter-synthesis/assets/voices/pt_BR-lula-medium/MODEL_CARD`
- [ ] Criar `crates/adapter-synthesis/assets/voices/pt_BR-lula-medium/pt_BR-lula-medium.onnx.json` (placeholder)
- [ ] Criar `crates/adapter-synthesis/assets/voices/pt_BR-lula-medium/espeak-ng-data/` (referência/cópia da voz faber)
- [ ] Documentar como obter `pt_BR-lula-medium.onnx` e `tokens.txt`

## T2 — Código: enum Voice e set_voice em synth.rs
- [ ] Adicionar enum `Voice { Faber, Lula }` com `dir_name()`
- [ ] Adicionar `static SELECTED_VOICE: OnceLock<Voice>` + `pub fn set_voice(Voice)`
- [ ] Modificar `asset_dir_for(voice)`, `model_path(voice)`, `tokens_path(voice)`, `espeak_data_dir(voice)`
- [ ] Adicionar `ENGINE_FABER` e `ENGINE_LULA` OnceLocks
- [ ] Adicionar `engine_for(voice: Voice)`
- [ ] Modificar `synthesize(segment, voice)` com parâmetro voice
- [ ] Manter `synthesize(segment)` wrapper compatível

## T3 — CLI: --voice option
- [ ] Adicionar `Voice` como `ValueEnum` em cli.rs
- [ ] Adicionar `--voice` ao subcommand `Synthesize`
- [ ] Chamar `set_voice()` antes de `synthesize_payload()`
- [ ] Atualizar `synthesize_payload` para passar voice

## T4 — Ponte Tauri: suporte a seleção de voz
- [ ] Chamar `set_voice()` em src-tauri/src/lib.rs quando configurado
- [ ] Atualizar `synthesize_segment` para passar voice para `synthesize()`

## T5 — Testes: multi-voz
- [ ] Atualizar chamadas de `synthesize()` nos testes existentes para `Voice::Faber`
- [ ] Adicionar teste: voz Lula sem .onnx → `InitFailed`
- [ ] Adicionar teste: `set_voice(Voice::Lula)` + synthesize usa assets da voz Lula

## T6 — Verificação: gates 1-4
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo nextest run --workspace`
- [ ] `cargo doc --workspace --no-deps`