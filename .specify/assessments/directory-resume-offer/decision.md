# Decision: Cola da oferta de retomada de diretório

- **Slug**: directory-resume-offer
- **Decided**: 2026-11-20
- **Verdict**: go
- **Artifacts reviewed**: intake.md, research.md, problem.md, concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | `problem.md`: a retomada de diretório está persistida mas inalcançável na reabertura; FR-009 da spec 010 define explicitamente o comportamento esperado. |
| Evidence strength | adequate | `research.md`: todas as peças e linhas foram verificadas em host (mapa ScoutT233, teste pinado, bundles); ausência de telemetria de usuário e comportamento em device `[NV]` impedem `strong`, mas não derrubam o `go`. |
| Value vs. inaction | strong | `problem.md` "Cost of Inaction": T233 fica aberto, código morto permanece suprimido e o teste continua provando strings em vez de comportamento. |
| Feasibility / appetite | strong | `concept.md` Option A é `small`: poucas dezenas de linhas em `lib.rs`, `shell_jni.rs`, `status_view.html` + espelhos e no teste já pinado. |
| Strategic fit | strong | Não altera a spec 010, não cria novo target, não modifica `scripts/expected-tests.txt` e mantém a máquina de oferta existente como única autoridade. |
| Risk posture | adequate | Riscos C-H8 (5 bundles), `PositionClock` (segunda origem de posição) e `route_offer` mutata (chamada dupla) estão identificados com mitigação concreta: espelhar byte a byte, derivar o payload do bookmark e chamar a máquina uma única vez. |

## Verdict & Rationale

**GO — Option A (cola mínima sobre o mecanismo existente).**

O problema é real e verificável em host, a evidência é majoritariamente código
lido e citado com linha, e a solução recomendada é a menor que fecha a lacuna sem
criar segunda autoridade de oferta. A condição de `go` do gate é satisfeita:
problema `strong`, evidência `adequate` (não `weak`/`unknown`) e opção conceitual
recomendada. As duas incertezas restantes — único ingresso de reabertura e seam de
teste — são validáveis durante `/speckit-specify`/`/speckit-clarify` e não
bloqueiam a especificação; o comportamento em dispositivo segue marcado `[NV]` e
não é reivindicado neste ciclo.

## If needs-clarification

Não aplicável — o veredito é `go`.

## If go — Handoff to `/speckit-specify`

- **Problem**: reabrir uma pasta com leitura em andamento não apresenta a oferta
  de retomada com nome do arquivo e posição salva, embora bookmarks e máquina de
  oferta já existam.
- **Chosen approach**: Option A — publicar a rota `InAppDialog` no caminho de
  reabertura, estacionar a retomada, enriquecer o payload do evento com
  `filename`/`offset_secs`, fazer a interface atribuir esses valores e chamar a
  atualização da oferta, e trocar asserções literais por teste comportamental no
  target já pinado.
- **In scope / out of scope**:
  - *In scope*: remoção dos marcadores de código morto, payload enriquecido,
    `route_offer` chamado uma vez, `park_directory_resume`, ligação JS, teste
    comportamental no target pinado e byte-identity dos cinco bundles HTML.
  - *Out of scope*: edição de `specs/010`, renumeração de T232/T233, novo target
    de teste, alteração em `scripts/expected-tests.txt`, redesign da máquina de
    oferta e verificação em dispositivo.
- **Success metrics**: rota publicada em 100% dos bookmarks `Paused` com janela
  de 3.000 ms; payload com nome não vazio e offset igual ao bookmark; exatamente
  uma resolução de oferta por reabertura; 5/5 bundles byte-idênticos; zero linhas
  alteradas no pin de testes.
- **Carried-forward open questions**:
  - Confirmar se `handle_directory_selected` é o único ingresso de reabertura a
    publicar a rota neste ciclo.
  - Confirmar se a seam de teste para observar rota publicada e payload já existe
    no bridge ou precisa ser criada sem novo target.
  - `[NV]` Janela de 3 s e auto-continue em foreground em dispositivo real —
    fora do escopo de verificação deste ciclo.
