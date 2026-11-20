# Decision: Leitura Contínua de Diretórios de Markdown

- **Slug**: directory-reading
- **Decided**: 2026-11-03
- **Verdict**: go
- **Artifacts reviewed**: intake.md, research.md, problem.md, concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | O atrito de interagir com a tela para cada nota unitária inviabiliza a escuta passiva de vaults no Obsidian e documentações por capítulos. |
| Evidence strength | strong | Evidências extraídas de ecossistemas reais (plugins Obsidian, leitores Evie e Voice Dream, dados da API SAF do Android e medições de corpora reais). |
| Value vs. inaction | strong | Desbloqueia a proposta central de valor do Ressoa para além de arquivos isolados, capturando o público de PKM e documentação técnica. |
| Feasibility / appetite | strong | A Option B (Streaming Chapter Queue) cabe no orçamento de 2 a 3 semanas (medium), aproveitando o motor de áudio e desacoplando a indexação em background. |
| Strategic fit | strong | 100% alinhado com a Constituição v2.0.0 (Rust como autoridade exclusiva, orçamento de TTFA ≤ 3s, limite orçado de 93KB por arquivo e operação estritamente offline). |
| Risk posture | strong | Riscos principais mapeados (sobrecarga de IPC no SAF, estouro de memória e poluição por rascunhos) com mitigações concretas (indexação assíncrona, arquivos individuais e ordenação natural). |

## Verdict & Rationale

**Veredito: GO**

A iniciativa de leitura de diretórios inteiros (`directory-reading`) é validada com nota máxima em todos os critérios. A dor do usuário é evidente e comprovada no mercado de ferramentas de notas pessoais (Obsidian/Logseq), a viabilidade técnica está assegurada através da **Option B (Streaming Chapter Queue)** sem comprometer os pilares constitucionais do Ressoa, e os riscos de I/O na plataforma Android estão devidamente endereçados. A funcionalidade está aprovada para transição para o ciclo formal de Especificação (Spec-Driven Development via `/speckit-specify`).

## If go — Handoff to `/speckit-specify`

- **Problem**: Ouvintes com vaults ou coleções de notas Markdown sofrem com interrupções frequentes e atrito ao serem forçados a selecionar manualmente cada arquivo a cada poucos minutos, impossibilitando a escuta contínua e mãos-livres.
- **Chosen approach**: Option B — Playlist Linear de Capítulos com Início Antecipado (Streaming Chapter Queue), disparando o primeiro arquivo imediatamente em < 1s enquanto a árvore é indexada em background pelo Android e consumida com transição gapless pelo Rust.
- **In scope / out of scope**:
  - *In scope*: Suporte a `ACTION_OPEN_DOCUMENT_TREE` no Android SAF; ordenação natural por convenção numérica (`01-`, `02-`) e fallback alfabético; transição automática entre arquivos consecutivos; persistência de bookmark composta por diretório e arquivo individual; preservação do orçamento de TTFA ≤ 3s e teto de 93KB por arquivo individual.
  - *Out of scope*: Gerenciador de arquivos completo com drag-and-drop; edição/exclusão de notas no disco; travessia de subpastas além do 1º nível na v1; file-watcher reativo com áudio desligado.
- **Success metrics**: Zero toques manuais entre arquivos consecutivos; TTFA ≤ 3,0 segundos para início da reprodução após seleção da pasta; 100% de restauração precisa de progresso no arquivo e segundo corretos ao retomar.
- **Carried-forward open questions**:
  - Definição exata da chave composta de bookmark no `redb` (`<dir_uri>:<file_content_hash>`).
  - Formato dos metadados de capítulo exibidos no `#titleCard` da Lente Sinestésica ("Capítulo X de Y: Nome do Arquivo").
  - Mapeamento das ações de controle de mídia (se os botões de avançar/retroceder na notificação e fones de ouvido saltam 30s ou saltam de capítulo).
