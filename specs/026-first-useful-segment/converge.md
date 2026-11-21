# Converge: 026 Primeiro segmento útil para reduzir TTFA

Registro **append-only**: novas entradas devem ser acrescentadas ao final, sem editar ou renumerar as anteriores.

## 2026-11-21 — implementação e verificação documental local (ciclo SDD 026)

**Status**: implementado no working tree de `feature/026-first-useful-segment`; evidências locais medidas neste checkout e reportadas pelo orquestrador; gate completo, PR/merge/CI e dispositivo `[NV]`.

### O que convergiu

- Assess `.specify/assessments/first-useful-segment/{intake,research,problem,concept,decision}.md` — veredito **go** (Option A: primário apenas no início fresco; scorecard com evidência `adequate` e risco de device `[NV]`).
- Artefatos em `specs/026-first-useful-segment/`: `spec.md` (FR-026-001..006, SC-026-001..006, 7 clarificações), `plan.md` (Phase 0/1 embutidos, Fases A–D), `tasks.md` (T001..T014), `checklists/requirements.md` (23/23), `analyze.md` (PASS com observações; 5 divergências LOW resolvidas por leitura).
- `src-tauri/src/lib.rs` — único arquivo de código alterado: `PRIMER_BYTES = 160` (:5560) e `split_first_segment_primer` (:5570-5603) dividem o primeiro segmento no último whitespace da janela de 160 bytes, herdam `source_range`/`boundary_kind`/`resume_point` e reindexam a lista; `prepare_session_document` ganhou `primer: bool` (:6725-6729) e aplica o helper logo após `split_document` (:6757-6761), antes de título, totais, seções e instalação da estrutura.
- Chamadores: estacionamento a frio passa `false` (:3985); `play_document` passa `start_offset_secs == 0.0` (:6830). Não há terceiro chamador; retomada com offset e oferta de retomada preservam a segmentação original.
- Caminhos preservados e verificados por leitura, não alterados: `handover_pending_for_generation` (:5943), `produce_bounded` (:5773), `stream_synthesis` (:6134), `DocumentSegment`/`split_document` (`crates/adapter-synthesis/src/chunk.rs:123-142,163-165`), contrato PCM mono `f32` @ 22050 Hz.
- Testes no target já pinado: `mod ttfa_013_tests` (:10713) com `primer_segment` (:10718), `t013_primer_splits_long_opening_and_reindexes` (:10730 — primário, restante, índices 0..3 e metadados herdados) e `t013_primer_leaves_short_single_and_unbroken_openings_untouched` (:10751 — abertura curta, lista unitária e sem whitespace intactas). Nenhum target novo; `scripts/expected-tests.txt` intacto.
- Cadeia documental fechada: `validate.md` (R4) registra os checks de host e `[NV]`; `release.md` (R7) propõe `v0.1.0-rc5` com changelog FR/T e rollback.

### Evidência local (medida neste checkout / reportada pelo orquestrador)

- `git diff --stat` sobre `src-tauri/src/lib.rs`: **1 arquivo alterado, 115 inserções, 8 remoções** — confinado ao helper, à assinatura do preparador, aos dois chamadores e aos testes inline.
- `cargo fmt --all -- --check`: **PASS** (orquestrador, ciclo 026). `cargo clippy -p src-tauri --lib -D warnings`: **PASS** (orquestrador, ciclo 026). Nenhum desses comandos foi executado por esta entrada documental.
- Inspeção estrutural: janela de bytes com recuo até `is_char_boundary` (:5581-5584), `rfind` de whitespace ASCII com não-divisão quando ausente (:5585-5587), herança via `..first.clone()` e reindexação por `enumerate().skip(2)` (:5588-5601).
- Checklist de requisitos: 23/23 itens revisados e marcados `[x]`; nenhum `[NEEDS CLARIFICATION]` permanece na spec.
- Nenhum arquivo de outra spec, bug, `scripts/verify.sh`, `converge.md` de outro ciclo, HTML, capabilities ou workflow foi alterado.

### [NV] — não verificado neste ciclo

- TTFA efetivamente observado em `2201117TY` (baseline histórico `play_to_audio_ms=183089`, ~183 s com "Hamlet trecho.md"): **[NV]**; nenhuma captura pareada antes/depois existe neste checkout. A latência de 183 s permanece **DEFER**, sem tarefa neste ciclo.
- Melhoria real de TTFA em dispositivo (SC-026-001) e dominância da síntese da primeira unidade sobre o custo fixo de inicialização do modelo: **[NV]** — a evidência em host é estrutural (unidade de ~3,9 KB → ≤160 bytes).
- `verify.sh` completo (gates 1–9) e seu exit code: **[NV]** até leitura pelo orquestrador; nenhuma afirmação de `VERIFY_EXIT=0` é feita aqui.
- Builds Android, pacote aarch64 (APK/AAB), CI, PR e merge remotos: **[NV]** nesta entrada.
- Comportamento de whitespace, UTF-8 e reindexação em dispositivo real: **[NV]**; coberto apenas por inspeção estrutural e testes de host no módulo pinado.

### Dívidas remanescentes

1. Captura pareada de `play_to_audio_ms` em dispositivo para transformar SC-026-001 em medição (owner natural: ciclo futuro com device; T232 continua DEFER).
2. Alinhamento de redação entre "primário terminado em whitespace" (spec) e o corte que descarta o whitespace de ambas as unidades (F3 em `analyze.md`) — sem edição neste ciclo.
3. Confirmação em device de que a parcela fixa de inicialização do modelo não anula o ganho da unidade curta — `[NV]`.

### Rollback

Reverter as alterações de código em `src-tauri/src/lib.rs` (remover `PRIMER_BYTES`, `split_first_segment_primer`, o parâmetro `primer`, os valores `false`/`start_offset_secs == 0.0` nos chamadores e os dois testes) e remover os artefatos documentais criados neste ciclo em `specs/026-first-useful-segment/`, `.specify/assessments/first-useful-segment/`, `.specify/validations/first-useful-segment/` e `.specify/releases/first-useful-segment/`. Nenhuma configuração remota, tag, assinatura, schema persistido ou arquivo de outra spec é tocado; nada foi comitado ou mesclado, portanto o rollback é local e reversível por restauração do working tree.
