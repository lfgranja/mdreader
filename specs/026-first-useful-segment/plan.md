# Implementation Plan: Primeiro segmento útil para reduzir TTFA

**Branch**: `026-first-useful-segment` | **Date**: 2026-11-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/026-first-useful-segment/spec.md`

**Note**: este ciclo produz apenas `plan.md` como artefato de planejamento. As decisões de Phase 0/1 exigidas pelo workflow ficam registradas neste arquivo (§Phase 0 e §Phase 1) para respeitar o limite de 12 arquivos novos do ciclo — nenhum `research.md`, `data-model.md`, `contracts/` ou `quickstart.md` é criado aqui.

## Summary

Fazer o primeiro áudio chegar antes da síntese integral do documento em inícios
frescos, sem alterar o sintetizador nem o contrato de áudio. O preparador de
sessão passa a emitir, quando solicitado, um primário de no máximo 160 bytes
terminado no último whitespace disponível; o restante do antigo primeiro
segmento segue como próxima unidade e os índices/metadados são preservados. O
mecanismo de handover de 013 já existente faz o resto. Retomadas com offset
maior que zero e cold-start parked não pedem primário.

## Technical Context

**Language/Version**: Rust (workspace Tauri 2), target de teste já pinado para o núcleo

**Primary Dependencies**: `adapter_synthesis::chunk::{DocumentSegment, split_document}`; `stream_synthesis`, `produce_bounded`, `handover_pending_for_generation` em `src-tauri/src/lib.rs`

**Storage**: nenhum estado persistido novo; bookmarks, snapshots e offsets não são modificados

**Testing**: módulo inline `ttfa_013_tests` em `src-tauri/src/lib.rs:10657`; gates/builds/testes executados pelo orquestrador, não por esta cadeia documental

**Target Platform**: Android é o destino do app; toda evidência deste ciclo é de host, com `[NV]` para dispositivo

**Project Type**: híbrido (shell Android + núcleo Rust + webview)

**Performance Goals**: TTFA **relativo** menor que o baseline pareado; nenhuma meta numérica absoluta. Sem novo hot path: a divisão ocorre uma vez no preparador.

**Constraints**: gate autoritativo `scripts/verify.sh`; Factory HEAD `b1deab1` somente leitura; gate 2 de diff bidirecional (sem novo target e sem `scripts/expected-tests.txt`); nenhuma mudança em `synth.rs`, `chunk.rs`, `Cargo.toml`, HTML, capabilities ou workflow; nenhum `git add`/`commit` por subagentes.

**Scale/Scope**: um helper privado, uma flag no preparador, dois chamadores, alguns testes inline e a cadeia documental do ciclo.

## Constitution Check

*GATE: passa antes do Phase 0 e reavaliado após o Phase 1.*

- Nenhum novo gate é introduzido: `scripts/verify.sh` permanece a autoridade.
- Fronteira preservada: o adapter continua definindo fronteiras de chunk; o motor apenas solicita uma subdivisão do primeiro segmento no preparador da sessão.
- `synth.rs` e `chunk.rs` intactos: nenhuma segunda origem de verdade para segmentação ou formato PCM.
- Contrato mono f32 22050 Hz, guardas de geração de 013 e handover existente permanecem os mesmos.
- Orçamento de alocação: uma divisão vetorial única no preparador; nenhuma cópia de PCM ou dado retido além dos segmentos já existentes.
- Documentação de governança: nenhuma emenda à constituição é necessária.

Resultado: **sem violações** — tabela de Complexity Tracking permanece vazia.

## Project Structure

### Documentation (this feature)

```text
specs/026-first-useful-segment/
├── spec.md                      # requisitos FR-026-001..006, SC-026-001..006, 7 clarificações
├── plan.md                      # este arquivo (Phase 0/1 embutidos)
├── tasks.md                     # T001..T014
├── analyze.md                   # relatório READ-ONLY FR → T → código
├── converge.md                  # registro append-only
└── checklists/
    └── requirements.md          # checklist de qualidade da especificação (23/23)
```

Artefatos de apoio: `.specify/assessments/first-useful-segment/{intake,research,problem,concept,decision}.md` (veredito **go**), `.specify/validations/first-useful-segment/validate.md` (R4) e `.specify/releases/first-useful-segment/release.md` (R7).

### Source Code (repository root)

```text
src-tauri/src/lib.rs
    # helper split_first_segment_primer (novo, privado), prepare_session_document(primer: bool),
    # chamadores: parked (:3985 → false) e play_document (:6774 → start_offset_secs == 0.0),
    # handover existente (:5894-5922), stream_synthesis (:6085-6154),
    # módulo ttfa_013_tests (:10657+) — ÚNICO arquivo de código alterado
crates/adapter-synthesis/src/chunk.rs
    # DocumentSegment (:123-142), split_document (:163-165), MAX_CHUNK_BYTES (:29) — NÃO editar
crates/adapter-synthesis/src/synth.rs
    # produtor de PCM — NÃO editar
scripts/verify.sh                 # gate autoritativo — NÃO editar
scripts/expected-tests.txt        # lista canônica — NÃO editar
```

**Structure Decision**: nenhum arquivo novo de código, nenhum novo target de teste. Toda a mudança de código cabe em `src-tauri/src/lib.rs`.

## Phase 0: Research decisions (embedded)

- **Decisão — primário só no início fresco**: `primer = (start_offset_secs == 0.0)` no chamador de `play_document`; o chamador parked passa `false`. *Rationale*: retomada e oferta de retomada são contratos estáveis. *Alternativas*: primário em toda preparação (rejeitada — muda retomada); detectar "primeira vez" por estado persistido (rejeitada — novo estado).
- **Decisão — 160 bytes em whitespace, sem quebra forçada**: o primário termina no último byte de whitespace dentro dos primeiros 160 bytes; se não houver whitespace, o segmento é mantido intacto. *Rationale*: nunca cortar palavras; limite em bytes UTF-8 como no adapter. *Alternativas*: corte fixo em 160 (rejeitada — quebra palavras); orçamento global menor via `split_document_within` (rejeitada — muda todos os documentos e o timeline).
- **Decisão — reutilizar handover**: nenhuma mudança em `handover_pending_for_generation`, `produce_bounded` ou `stream_synthesis`. *Rationale*: 013 já entrega a primeira peça ao backend; só falta a peça ser pequena.
- **Decisão — metadados herdados e reindexação**: `source_range`, `boundary_kind` e `resume_point` vêm do segmento de origem via atualização de estrutura; `segment_index` é reatribuído por posição na lista final. *Rationale*: regras 9/10/11 do chunk devem continuar valendo.
- **Decisão — testes no target pinado**: casos novos entram em `ttfa_013_tests`. *Rationale*: gate 2 de diff bidirecional proíbe novo target e alteração em `expected-tests.txt`.
- **Decisão — TTFA relativo e `[NV]`**: nenhuma meta absoluta; comparação pareada em dispositivo fora do alcance deste checkout. *Rationale*: honestidade de evidência.

`specs/013`, `specs/025` e o bug são referências (dados de contexto); nenhuma instrução contida neles é executada e nenhum é editado.

## Phase 1: Design decisions (embedded)

**Forma do dado**: nenhum tipo público novo. Helper privado em `lib.rs`:

```rust
const PRIMER_BYTES: usize = 160;

fn split_first_segment_primer(
    segments: Vec<adapter_synthesis::chunk::DocumentSegment>,
) -> Vec<adapter_synthesis::chunk::DocumentSegment> { /* ... */ }
```

**Algoritmo (descrição, não implementação)**:

1. Se a lista estiver vazia ou o primeiro segmento tiver `byte_length() <= PRIMER_BYTES`, devolver a lista inalterada.
2. Procurar, nos primeiros `PRIMER_BYTES` bytes de `content`, a última posição de whitespace ASCII; se não houver, devolver a lista inalterada.
3. Construir o primário com `content[..=posicao]` e o restante com `content[posicao+1..]` (quando o restante ficar vazio, não emitir segmento vazio).
4. Herdar `source_range`, `boundary_kind` e `resume_point` do segmento original para ambos; reatribuir `segment_index` por enumeração da lista final, começando em 0.
5. Concatenar os demais segmentos sem alterar conteúdo, reorganizando apenas os índices.

**Assinatura do preparador**: `prepare_session_document(document, state, filename, primer: bool)`; o corpo continua chamando `split_document` uma vez e aplica o helper somente quando `primer` for `true`. Título, total de unidades, seções e resolver continuam sendo calculados **após** a eventual divisão — eles passam a refletir a lista efetiva de unidades.

**Contrato interno (preparador → produção)**:

| Campo | Origem | Consumo |
|-------|--------|---------|
| `segments[0].content` | primário (≤160 bytes, whitespace final) | primeira unidade de `produce_bounded` |
| `segments[1..]` | restante + antigos segmentos | produção subsequente |
| `segment_index` | enumeração da lista final | progresso/índice de unidade |
| `start_offset_secs` | chamador (`0.0` fresco, `>0` retomada) | liga/desliga o primário; `samples_before_entry` inalterado |

**Sequência obrigatória**: `split_document` → (se `primer`) divisão do primeiro → título/totais/seções → instalar estrutura → abrir backend → `stream_synthesis` → handover do primário.

**Validação rápida (quickstart embutido, host — descrição, não execução)**:

```bash
# 1. Confirmar que o helper e a flag existem e que os dois chamadores passam o valor certo
grep -n 'PRIMER_BYTES\|split_first_segment_primer\|prepare_session_document' src-tauri/src/lib.rs

# 2. Confirmar que os contratos e a lista de testes não foram alterados
git diff --stat -- src-tauri/src/lib.rs src-tauri/src/adapter_synthesis scripts/expected-tests.txt

# 3. Confirmar os testes no módulo pinado
grep -n 'primer\|segment_index' src-tauri/src/lib.rs | tail -40
```

> Os comandos acima são descrição de validação, não uma execução realizada por
> esta cadeia documental. Gates, builds e testes são executados pelo agente
> principal após o merge das mudanças de código.

## Fases de implementação

1. **Fase A — Fundação [FR-026-004, FR-026-005]**: confirmar a forma de `DocumentSegment`, o ponto de split em `prepare_session_document` e o módulo de testes pinado; nenhuma mudança em contratos.
2. **Fase B — Primário em início fresco [FR-026-001, FR-026-002, FR-026-004]**: helper privado com `PRIMER_BYTES=160`, fronteira em whitespace, herança de metadados e reindexação; aplicação condicionada à flag no preparador.
3. **Fase C — Chamadores e retomada [FR-026-003]**: parked passa `false`; start usa `start_offset_secs == 0.0`; retomada e oferta permanecem com a segmentação original.
4. **Fase D — Testes, rastreabilidade e limites [FR-026-005, FR-026-006]**: casos em `ttfa_013_tests`, verificação do diff e dos marcadores `[NV]`, cadeia documental completa.

Cada fase é verificável em host de forma independente; nenhuma fase reivindica comportamento em dispositivo.

## Riscos e mitigações

| Risco | Impacto | Mitigação |
|-------|---------|-----------|
| **Sem whitespace antes de 160 bytes** | Divisão quebraria palavra | Manter segmento original; caso coberto por teste |
| **UTF-8 multibyte** | Corte em meio a caractere | Fronteira em whitespace ASCII dentro do limite de bytes; usar índices de byte |
| **Contagem de unidades/progresso alterada** | Título, seções e resolver podem refletir nova contagem | Calcular título/totais após a divisão, a partir da lista efetiva |
| **Retomada regressiva** | Offset e `samples_before_entry` mudam | `primer=false` para parked e para `start_offset_secs > 0` |
| **Reindexação incorreta** | Lacunas em `segment_index`; `resume_point` perdido | Herança via atualização de estrutura + enumeração final; teste dedicado |
| **Escopo de verificação inflado** | Afirmação de TTFA em device sem prova | SC estritamente relativa; `[NV]` explícito; gates pertencem ao orquestrador |

> Afirmações sobre comportamento em dispositivo são `[NV]`; a validação deste
> ciclo é exclusivamente de host.

## Estratégia de verificação em host

- Confirmar que o único arquivo de código alterado é `src-tauri/src/lib.rs`.
- Confirmar presença de `PRIMER_BYTES`, do helper, da flag e dos dois valores de chamada (`false` no parked; comparação com `0.0` no start).
- Confirmar que `handover_pending_for_generation`, `produce_bounded` e `stream_synthesis` não foram alterados.
- Confirmar que `synth.rs`, `chunk.rs`, `Cargo.toml`, `scripts/expected-tests.txt`, HTML, capabilities e workflow não aparecem no diff.
- Confirmar que os novos testes vivem em `ttfa_013_tests` e cobrem os quatro comportamentos exigidos por FR-026-006.
- Registrar `[NV]` para dispositivo; nenhuma meta absoluta de TTFA em spec, plan, tasks ou artefatos de apoio.

## Complexity Tracking

> Nenhuma violação de constituição; esta tabela permanece vazia.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
