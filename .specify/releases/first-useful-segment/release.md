# Release (R7): 026 Primeiro segmento útil para reduzir TTFA

- **Date**: 2026-11-21
- **Branch**: `feature/026-first-useful-segment`
- **SemVer proposto**: **v0.1.0-rc5** (pre-release incremental sobre a proposta `v0.1.0-rc4` do ciclo 025; sem bump de aplicativo independente — a trilha `v0.1.0-rc` permanece). **Proposta, não aplicada.** Tags presentes no checkout: `v0.1.0-rc1`, `v0.1.0-rc2`.
- **Validation**: R4 PASS nos checks documentais/host realizados (`.specify/validations/first-useful-segment/validate.md`); dispositivo, `verify.sh` completo, builds, testes, CI e merge `[NV]`.
- **Status de merge**: **NÃO houve merge.** As mudanças permanecem no working tree do branch; nenhum `git add`/`git commit` foi realizado por este ciclo de artefatos.

---

## 1. Resumo

Em um início fresco de reprodução (`start_offset_secs == 0.0`), o primeiro segmento enviado à síntese passa a ser um primário de no máximo 160 bytes, cortado no último whitespace da janela, seguido pelo restante do antigo primeiro segmento. O mecanismo de handover do ciclo 013, já existente, enfileira e inicia o backend com essa primeira peça — o ouvinte ouve áudio antes da síntese integral de um documento curto (~3,9 KB, 183 s observados em `2201117TY`). Retomadas com offset e estacionamento a frio continuam com a segmentação original. Sintetizador, fronteiras de chunk, contrato PCM mono `f32` @ 22050 Hz e configurações de teste permanecem intactos. A métrica de TTFA é estritamente relativa ao baseline pareado; nenhuma meta numérica absoluta é definida e a medição em dispositivo é `[NV]` (183 s registrado como **DEFER**).

---

## 2. Changelog (FR/T)

- **FR-026-001 / T003, T004**: `PRIMER_BYTES = 160` (`src-tauri/src/lib.rs:5560`) e `split_first_segment_primer` (:5570-5603). A janela é contada em bytes UTF-8, recua até `is_char_boundary` e corta no último whitespace ASCII; se não houver whitespace antes do limite, a lista volta inalterada. Nenhuma palavra ou caractere é quebrado.
- **FR-026-002 / T002, T006**: o primário ocupa `segments[0]` antes de `install_session_structure`, tornando-se a primeira unidade de `produce_bounded`/`stream_synthesis`. `handover_pending_for_generation` (:5943) é reaproveitado: nenhuma lógica nova de sincronização, geração ou start de backend.
- **FR-026-003 / T007, T008, T009**: `prepare_session_document` ganha `primer: bool` (:6725); o chamador parked passa `false` (:3985) e `play_document` passa `start_offset_secs == 0.0` (:6830). Retomada, bookmarks, `stream_anchor_secs` e `samples_before_entry` não são recalculados.
- **FR-026-004 / T001, T005**: primário e restante herdam `source_range`, `boundary_kind` e `resume_point` via atualização de estrutura; `segment_index` é reatribuído por posição (0, 1, 2…), sem lacunas. Título, total de unidades e seções são calculados após a divisão, refletindo a lista efetiva.
- **FR-026-005 / T010, T011, T014**: diff confinado a `src-tauri/src/lib.rs` — 1 arquivo, 115 inserções, 8 remoções. `synth.rs`, `chunk.rs`, `Cargo.toml`, `scripts/expected-tests.txt`, HTML, capabilities, workflow e `converge.md` de outras specs intactos; nenhum target de teste novo.
- **FR-026-006 / T012, T013, T014**: dois casos em `ttfa_013_tests` (:10713): `t013_primer_splits_long_opening_and_reindexes` (:10730) e `t013_primer_leaves_short_single_and_unbroken_openings_untouched` (:10751). Rastreabilidade 12/12 em `analyze.md`, checklist 23/23, `[NV]` para dispositivo.
- **Polish / T013, T014**: revisão de IDs, limites de evidência e ausência de `git add`/`commit` por subagente. `cargo fmt --all -- --check` e `cargo clippy -p src-tauri --lib -D warnings`: **PASS**, conforme reportado pelo orquestrador do ciclo (não executados por esta cadeia documental).

Evidência de host local: `git diff --stat`, inspeção estrutural do helper e dos dois chamadores, leitura dos caminhos preservados (`handover`, `produce_bounded`, `stream_synthesis`, `chunk.rs`), `analyze.md` com 5 divergências LOW resolvidas e 23/23 itens do checklist revisados.

---

## 3. Rollback

Reverter as alterações de código em `src-tauri/src/lib.rs` (remover `PRIMER_BYTES`, `split_first_segment_primer`, o parâmetro `primer` de `prepare_session_document`, os valores `false`/`start_offset_secs == 0.0` nos chamadores e os dois testes inline) e remover os artefatos documentais criados neste ciclo, todos no working tree — como nada foi comitado nem mesclado, o rollback é local e não afeta nenhuma tag/release publicada:

**Código (1):** `src-tauri/src/lib.rs`.

**Artefatos centrais (5):** `specs/026-first-useful-segment/spec.md`, `plan.md`, `tasks.md`, `analyze.md`, `converge.md`.

**Documentos de apoio:** `specs/026-first-useful-segment/checklists/requirements.md`, `.specify/assessments/first-useful-segment/{intake,research,problem,concept,decision}.md`, `.specify/validations/first-useful-segment/validate.md` e este `release.md`.

Procedimento: restaurar `src-tauri/src/lib.rs` ao estado anterior ao branch e remover os diretórios de artefatos listados. Nenhuma configuração remota, migração de schema, surface de consentimento, tag, assinatura ou build assinado está envolvida. `specs/013`, `specs/025`, bugs, `scripts/verify.sh`, `scripts/expected-tests.txt` e `converge.md` de outras specs **não** fazem parte do rollback (nunca foram alterados). Restaurar o preparador devolve o comportamento anterior: um único segmento ~3,9 KB como primeira unidade e nenhuma divisão no início fresco.

---

## 4. Riscos residuais / [NV]

- **Dispositivo (`2201117TY`)**: TTFA pareado após o primário e o tamanho real da redução — **[NV]**; baseline histórico `play_to_audio_ms=183089` registrado como **DEFER**, não como medição deste ciclo.
- **Custo fixo de inicialização do modelo**: pode limitar o ganho da unidade curta; confirmação depende de captura pareada — **[NV]**.
- **`verify.sh` completo (gates 1–9)**: não executado por esta cadeia; resultado pendente de observação pelo orquestrador — **[NV]**.
- **Builds, formatadores e testes**: não executados nesta cadeia documental; `fmt`/`clippy` PASS e os dois testes em `ttfa_013_tests` são evidências reportadas/estruturais — **[NV]** para a suíte completa.
- **CI remoto / merge de PR**: não executado; nenhuma verificação remota é reivindicada — **[NV]**.
- **Gate 9 / aparelho**: build/pacote aarch64 (APK/AAB) e medição de áudio — **[NV]**.
- **Fronteira de whitespace (F3 em `analyze.md`)**: o corte descarta o whitespace de ambas as unidades; a redação da spec diz "terminado em whitespace". Comportamento verificado por teste (palavras não quebradas); alinhamento de redação ficou para ciclo futuro, sem edição neste ciclo.
- **T232 / worker de pré-síntese**: continua **DEFER**, sem tarefa neste ciclo.

---

## 5. Confirmação de estado

- Nenhum merge realizado; branch `feature/026-first-useful-segment` com mudanças locais, sem commit deste ciclo.
- Nenhuma tag, assinatura, publicação em loja ou configuração remota alterada.
- `specs/013`, `specs/025`, bugs, `scripts/verify.sh`, `scripts/expected-tests.txt` e `converge.md` de outras specs intactos.
- Cadeia documental completa: assess (5) → spec/plan/tasks/analyze/converge (5) + checklist → validate (R4) → release (R7).
