# Decision: Carregamento de Documentos e Início Direto de Reprodução pelo Título

- **Slug**: open-file-from-title
- **Decided**: 2026-11-02
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | A impossibilidade de abrir e alternar arquivos diretamente pela interface gráfica da Lente Sinestésica torna o aplicativo dependente de automações externas para uso real. |
| Evidence strength | strong | 90% da infraestrutura de seleção e ingestão via SAF (`DocIngest.kt`) e ponte JNI (`shell_jni.rs`) já existe e está testada no repositório. |
| Value vs. inaction | strong | Transforma o Ressoa de um player de arquivos pré-carregados em um leitor de audiolivros/markdown plenamente utilizável no Android pelo usuário final. |
| Feasibility / appetite | strong | Apetite `small` (1 a 2 dias). Requer apenas a integração do evento de clique na UI com a chamada nativa já construída e o gatilho de auto-play. |
| Strategic fit | strong | Preserva integralmente a autoridade exclusiva de playback do Rust, o modelo offline-only e os contratos de integridade de bookmarks do projeto. |
| Risk posture | strong | Riscos mitigados: cancelamento do SAF não afeta a reprodução atual, limites de formato/tamanho são garantidos por `DocIngest` e regras de acessibilidade WCAG 1.4.2 são mantidas. |

## Verdict & Rationale

**Veredito: GO.**
A iniciativa soluciona a principal lacuna funcional de usabilidade diária do aplicativo móvel, viabilizando que o ouvinte selecione documentos de texto (`.md`, `.txt`) diretamente pela interface da Lente Sinestésica e comece a ouvir de forma imediata. O trabalho aproveita quase na totalidade a infraestrutura já implementada e testada de ingestão (`DocIngest.kt` e `shell_jni.rs`), exigindo esforço reduzido (`small`), sem dependências externas adicionais e com baixíssimo risco arquitetural.

## If go — Handoff to `/speckit-specify`

- **Problem**: Ouvintes não conseguem carregar e alternar arquivos de texto locais diretamente pela interface gráfica da Lente Sinestésica, enfrentando atrito para iniciar a leitura de novos documentos.
- **Chosen approach**: Option A — Toque na caixa de título (`.meta-box` / `#documentTitle`) com affordance e semântica acessível para acionar o SAF (`ACTION_OPEN_DOCUMENT`) nativo do Android e inicializar imediatamente a síntese/reprodução após o carregamento bem-sucedido.
- **In scope / out of scope**:
  - *In scope*:
    - Adicionar evento de clique e acessibilidade (`role="button"`, `tabindex="0"`, `aria-label`) na caixa de título da Lente Sinestésica (`status_view.html`).
    - Expor e conectar o comando Tauri / JNI para invocar `DocIngest.pickDocument(activity)`.
    - Ao receber o callback de documento carregado com sucesso, disparar a reprodução automática (`play`), a menos que o arquivo possua bookmark salvo com parada deliberada (onde deve ser exibido o diálogo padrão de oferta `offerDialog` / FR-009).
    - Manter feedback de erro silencioso/seguro caso a seleção seja cancelada no SAF ou o arquivo seja recusado por formato/tamanho.
  - *Out of scope*:
    - Gerenciador de arquivos ou explorador de pastas customizado dentro do aplicativo.
    - Suporte a formatos além de `text/markdown` e `text/plain`.
    - Edição ou exclusão de documentos.
- **Success metrics**:
  - Fluxo de abrir novo documento e iniciar escuta reduzido para 1 toque e 1 confirmação no SAF.
  - Reprodução iniciando em menos de 1s após a seleção do arquivo.
  - 100% de preservação de bookmarks e conformidade com o contrato FR-009.
- **Carried-forward open questions**:
  - Confirmação final da área de clique (`.meta-box` inteira vs `#documentTitle` apenas).
  - Feedback visual sutil (hover/active state e cursor) para evidenciar a interatividade da área de título sem poluir a estética visual minimalista da Lente Sinestésica.
