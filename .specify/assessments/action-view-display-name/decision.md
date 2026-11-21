# Decision: Nome de exibição no ACTION_VIEW via MediaStore

- **Slug**: action-view-display-name
- **Decided**: 2026-11-21
- **Verdict**: go
- **Artifacts reviewed**: intake.md, research.md, problem.md, concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | `problem.md`: o título numérico é reproduzível a partir da forma da URI e da consulta direta que retorna `null`; o contraste com `ACTION_SEND` isola o caminho `ACTION_VIEW`. |
| Evidence strength | adequate | `research.md`: diff, predicado, consumidor Rust e contrato JNI foram lidos em host com linha; a resolução real em dispositivo é `[NV]` e a latência de 183 s é DEFER, o que impede `strong` mas não derruba o `go`. |
| Value vs. inaction | strong | `problem.md` "Cost of Inaction": a regressão permanece aberta e o usuário não consegue associar o título ao documento nem ao bookmark. |
| Feasibility / appetite | strong | `concept.md` Option A é `small`: 36 inserções/10 remoções em um único arquivo Kotlin; nenhuma mudança em Rust, manifesto, JNI ou fingerprint. |
| Strategic fit | strong | Mantém a fronteira existente (Kotlin transporta bytes, Rust deriva o título da URI), preserva SAF/`ACTION_SEND` e não edita specs ou bugs de outros ciclos. |
| Risk posture | adequate | Riscos de volume, codificação e permissão estão identificados com mitigação concreta: default `external`, par `Uri.encode`/`percent_decode` verificado por leitura e fallback determinístico quando a consulta falha. |

## Verdict & Rationale

**GO — Option A (fallback MediaStore localizado no Kotlin).**

O problema é real e verificável em host pela forma da consulta e pelo diff; a
evidência é majoritariamente código lido e citado com linha; a solução é a menor
que corrige a causa sem cruzar a fronteira JNI. A condição de `go` é satisfeita:
problema `strong`, evidência `adequate` (não `weak`/`unknown`) e opção conceitual
recomendada. As duas incertezas restantes — validade do `volume` e fallback
quando nenhum nome é obtido — são resolvidas na especificação como default
`external` e `document.md`, e não bloqueiam. O comportamento em dispositivo
segue marcado `[NV]`; a latência de 183 s é explicitamente **DEFER** e não entra
neste ciclo.

## If needs-clarification

Não aplicável — o veredito é `go`.

## If go — Handoff to `/speckit-specify`

- **Problem**: abrir um `.md` via `ACTION_VIEW` com URI de item do MediaStore
  exibe `document:<id>.md` em vez do nome do arquivo, porque `queryName`
  consulta o URI do item em vez da coleção `MediaStore.Files`.
- **Chosen approach**: Option A — tentar `OpenableColumns.DISPLAY_NAME` no URI;
  se falhar e o URI for `content://` com authority MediaStore, consultar a
  coleção por `_id=?`; filtrar com `isUsableName`; montar
  `content://display_name/<Uri.encode(nome)>` incondicionalmente no
  `ACTION_VIEW`, com fallback `document.md`.
- **In scope / out of scope**:
  - *In scope*: imports `BaseColumns`/`MediaStore`, predicado `isUsableName`,
    `queryName(resolver, uri)`, ramo `ACTION_VIEW`, verificação estrutural em
    host e rastreabilidade FR→T→código.
  - *Out of scope*: SAF/diretório, `ACTION_SEND`, BLAKE3/snapshot, assinatura
    JNI, Rust, specs 009/019, bugs, `converge.md` de outras specs e a latência
    de 183 s (**DEFER**).
- **Success metrics**: nome legível em 100% dos itens MediaStore com
  `_display_name` exposto; 100% dos ramos `ACTION_VIEW` com URI sintético e zero
  `uri.toString()`; 100% dos nomes filtrados por `isUsableName`; zero mudanças
  nos caminhos preservados; 100% dos FRs rastreáveis em host com `[NV]` para
  device.
- **Carried-forward open questions**:
  - Validar `volume` contra volumes disponíveis ou manter o default `external`.
  - Manter `document.md` como fallback fixo quando nenhum nome é utilizável.
  - `[NV]` Nome resolvido e título final em dispositivo — fora do escopo deste
    ciclo.
  - **DEFER**: latência de 183 s até o primeiro áudio (pré-roll 60 s), dono
    natural T232.
