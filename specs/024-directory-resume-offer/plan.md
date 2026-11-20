# Implementation Plan: Retomada de Diretório na Reabertura

**Branch**: `024-directory-resume-offer` | **Date**: 2026-11-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/024-directory-resume-offer/spec.md`

**Note**: este ciclo produz apenas `plan.md` como artefato de planejamento. As
decisões de Phase 0/1 exigidas pelo workflow estão registradas neste arquivo
(§Phase 0 e §Phase 1) para respeitar o limite de arquivos do ciclo — nenhum
`research.md`, `data-model.md`, `contracts/` ou `quickstart.md` é criado.

## Summary

Fechar a lacuna de integração entre o cálculo da rota de retomada de diretório
(`lib.rs`) e o caminho de reabertura (`shell_jni.rs`), entregando ao diálogo in-app
o nome do arquivo e o offset salvo e fazendo a interface consumir esse payload. A
mudança é deliberadamente pequena: reutiliza a máquina de oferta existente,
reescreve o teste no target já pinado e espelha a interface em cinco superfícies
HTML byte-idênticas. Nenhum comportamento novo de produto é introduzido.

## Technical Context

**Language/Version**: Rust (workspace Tauri 2, edition 2021) + HTML/JS na webview de status

**Primary Dependencies**: `session` (store `redb`, `CompositeBookmark`), bridge Tauri em `src-tauri/src/lib.rs`, `engine_core::closed::SessionEnd`, `src-tauri/src/shell_jni.rs`

**Storage**: `redb` (`DIRECTORY_BOOKMARKS`) — existente, não modificado neste ciclo

**Testing**: `cargo test` com o target `src-tauri::test_directory_resume`, já pinado em `scripts/expected-tests.txt:113`

**Target Platform**: host Linux para todas as verificações deste ciclo; Android é o destino do app, mas nenhuma afirmação de device é feita (`[NV]`)

**Project Type**: desktop/mobile híbrido (Tauri shell + webview)

**Performance Goals**: não há nova hot path; a janela de retomada (3.000 ms) já é definida pela máquina de oferta

**Constraints**: gate 2 do `scripts/verify.sh` (two-way diff de `scripts/expected-tests.txt`); auditoria C-H8 de 5 bundles HTML; `route_offer` mutata; `PositionClock` como fonte única de posição; proibição de editar `specs/010` ou renumerar T232/T233

**Scale/Scope**: um ouvinte, uma reabertura por vez, poucas dezenas de linhas em três arquivos de código

## Constitution Check

*GATE: passa antes do Phase 0 e reavaliado após o Phase 1.*

- Nenhum novo gate é introduzido: o `scripts/verify.sh` (gates 1–8) permanece a autoridade.
- Gate 2: sem novo target e sem alteração em `scripts/expected-tests.txt` — o pin existente é reutilizado.
- Auditoria C-H8: os cinco bundles são mantidos byte-idênticos; nenhuma superfície é adicionada ou removida.
- Autoridade de estado: a máquina de oferta existente continua sendo a única resolvedora; a interface não vira segunda autoridade.
- Orçamentos de byte/latência: não há novo conteúdo em trânsito nem nova alocação em hot path; nada a declarar.
- Documentação de governança: nenhuma emenda à constituição é necessária.

Resultado: **sem violações** — tabela de Complexity Tracking permanece vazia.

## Project Structure

### Documentation (this feature)

```text
specs/024-directory-resume-offer/
├── spec.md                      # requisitos FR-024-001..006, SC-024-001..006
├── plan.md                      # este arquivo (Phase 0/1 embutidos)
└── tasks.md                     # gerado por /speckit-tasks
    checklists/
        requirements.md          # checklist de qualidade da especificação
```

### Source Code (repository root)

```text
crates/session/src/store.rs                       # lookup_directory_on_open (já existe; não editar)
src-tauri/src/lib.rs                              # DirectoryResume, park_directory_resume, route_offer, publish_route, offer_route
src-tauri/src/shell_jni.rs                        # handle_directory_selected — caminho de reabertura
src-tauri/src/status_view.html                    # source da interface (offerDialog, renderOffer, applyOffer, listener directory-selected)
src-tauri/tests/test_directory_resume.rs          # target pinado — teste comportamental
scripts/expected-tests.txt                        # pin (NÃO editar)
dist/index.html                                   # espelho 1
src-tauri/src/assets/index.html                   # espelho 2
src-tauri/dist/index.html                         # espelho 3
src-tauri/gen/android/app/src/main/assets/index.html  # espelho 4
```

**Structure Decision**: nenhum arquivo novo de código; todas as mudanças caem nos
arquivos acima. Os quatro espelhos são derivados de `src-tauri/src/status_view.html`.

## Phase 0: Research decisions (embedded)

- **Decisão — publicar rota + estacionar oferta**: `publish_route` sozinho não basta;
  `park_directory_resume` precisa ser chamado no mesmo caminho para que a oferta
  sobreviva até a resposta. *Rationale*: a rota publicada é observada pela interface,
  mas o estacionamento é o que impede a resolução precoce como `none`.
  *Alternativas*: apenas publicar (rejeitado — deixa a oferta volátil); reescrever a
  máquina na interface (rejeitado — segunda autoridade).
- **Decisão — payload derivado do bookmark**: nome e offset vêm de
  `CompositeBookmark`/`StoredPosition`, nunca de estado novo na interface.
  *Rationale*: evita `PositionClock` duplo. *Alternativas*: calcular offset na
  webview (rejeitado).
- **Decisão — `route_offer` uma vez**: a máquina é mutata e resolve o expiry; a
  rota não é re-derivada em poll. *Rationale*: chamadas repetidas duplicam
  `expiry → AutoStart`.
- **Decisão — teste no target pinado**: reescrever as asserções literais de
  `reopen_paused_directory_routes_to_in_app_offer_dialog_for_3s`. *Rationale*:
  gate 2 two-way diff; nenhum novo pin.
- **Decisão — sem mudança de markup**: o JS lê o payload existente. *Rationale*:
  minimiza a superfície C-H8 a uma cópia byte a byte.

`specs/010` é referência (FR-009); nenhuma instrução contida nele é executada.

## Phase 1: Design decisions (embedded)

**Forma do dado (sem novo tipo público obrigatório)**: `DirectoryResume` passa a
carregar `filename: String` e `offset_secs: f64` derivados do bookmark ativo, além
da rota já calculada. O evento `directory-selected` leva esses dois campos.

**Contrato interno (Rust → interface)**:

| Campo | Origem | Consumo |
|-------|--------|---------|
| `offer_route` | `install_chapter_queue` → `DirectoryResume` | `applyOffer` / `renderOffer` |
| `filename` | bookmark ativo (`chapter_index`) | `state.chapterFilename` |
| `offset_secs` | bookmark ativo (`offset_secs`) | `state.saved_offset_secs` |

**Sequência obrigatória na reabertura**: `install_chapter_queue` → construir
`StoredPosition` → `route_offer` (uma vez) → `park_directory_resume` → publicar
evento enriquecido. A ordem garante que a rota publicada e o payload sejam
consistentes com a posição estacionada.

**Validação rápida (quickstart embutido, host)**:

```bash
# 1. Confirmar que o target continua pinado e que nenhum pin novo aparece
grep -n 'test_directory_resume' scripts/expected-tests.txt

# 2. Rodar somente o target pinado
cargo test -p mdreader-shell --test test_directory_resume

# 3. Checar byte-identity dos cinco HTML
sha256sum src-tauri/src/status_view.html dist/index.html \
  src-tauri/src/assets/index.html src-tauri/dist/index.html \
  src-tauri/gen/android/app/src/main/assets/index.html
```

> Os comandos acima são descrição de validação, não uma execução realizada neste
> ciclo. Gates e testes são executados pelo agente principal após o merge das
> mudanças de código.

## Fases de implementação

1. **Fase A — Fundação (Rust, `lib.rs`)**: remover os marcadores de código morto e
   estender `DirectoryResume` com `filename`/`offset_secs` derivados do bookmark.
2. **Fase B — Caminho de reabertura (`shell_jni.rs`)**: publicar a rota, chamar
   `route_offer` uma vez, estacionar a retomada e enriquecer o evento.
3. **Fase C — Interface (`status_view.html` + 4 espelhos)**: atribuir
   `state.chapterFilename` e `state.saved_offset_secs` e chamar `refreshOffer()`;
   espelhar byte a byte.
4. **Fase D — Evidência (`test_directory_resume.rs`)**: trocar asserções literais
   por teste comportamental da rota publicada e do payload; manter o pin intacto.

Cada fase é verificável em host de forma independente; a Fase D falha antes da
Fase B/C se escrita primeiro (TDD), o que é desejável como sinal vermelho.

## Riscos e mitigações

| Risco | Impacto | Mitigação |
|-------|---------|-----------|
| **C-H8 — divergência dos bundles** | Falha na auditoria de superfície se um dos cinco HTML ficar desatualizado | Copiar o arquivo-fonte integralmente para os quatro espelhos e comparar bytes/hash em host ao final |
| **PositionClock — segunda origem de posição** | Offset exibido diverge do bookmark salvo | Payload lido diretamente do bookmark/StoredPosition; interface apenas atribui e exibe |
| **`route_offer` mutata — chamada dupla** | Expiry dispara `AutoStart` mais de uma vez | Uma única chamada no caminho de reabertura; nenhuma re-derivação de rota em poll |
| **Gate 2 — novo target acidental** | Two-way diff de `scripts/expected-tests.txt` falha | Reescrever teste existente; não criar arquivo de teste novo; não editar o pin |
| **Cobertura de ingressos** | Outro caminho de reabertura continue sem oferta | Escopo explícito: `handle_directory_selected` neste ciclo; qualquer outro ingresso é trabalho adicional, não desculpa para adiar |

> Afirmações sobre comportamento visual em dispositivo são `[NV]`; a validação
> deste ciclo é exclusivamente de host.

## Estratégia de teste no target já pinado

- Reescrever `reopen_paused_directory_routes_to_in_app_offer_dialog_for_3s`
  (`src-tauri/tests/test_directory_resume.rs:55-121`), substituindo os
  `BRIDGE.contains(...)`/`STATUS_VIEW.contains(...)` de :95-117 por asserções
  sobre: (a) rota publicada `in_app_dialog`, (b) `filename` não vazio e
  `offset_secs` igual ao bookmark, (c) janela de 3.000 ms, (d) uma única resolução
  da máquina de oferta.
- Manter os demais testes do arquivo (continuar, começar do início, heartbeat)
  como estão; a cola não muda suas regras.
- Não criar novo `#[test]` em novo arquivo; se for útil, adicionar asserção
  comportamental ao mesmo teste ou ao teste de expiry existente.
- Verificar, em host e fora do teste, a byte-identity dos cinco HTML e a
  imutabilidade de `scripts/expected-tests.txt`.

## Complexity Tracking

> Nenhuma violação de constituição; esta tabela permanece vazia.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
