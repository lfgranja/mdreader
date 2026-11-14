# Converge: 010 Leitura Contínua de Diretórios de Markdown

Status: **CONVERGED** (Atestado em hardware real, 49/49 tasks) — leitura contínua de diretórios/vaults via SAF `ACTION_OPEN_DOCUMENT_TREE`, fila de capítulos (`ChapterQueue`), transições gapless, retomada composta em `redb` e navegação contextual; evidência de hardware físico registrada em ciclos anteriores.

## Implementation Summary

Implementada a leitura contínua hands-free de diretórios e vaults Markdown/texto no Android. O seletor nativo `ACTION_OPEN_DOCUMENT_TREE` com permissões persistidas indexa arquivos elegíveis (`.md`, `.txt`) em background, aplica ordenação natural/hierárquica com precedência de índice quando aplicável, e inicia reprodução do primeiro capítulo com TTFA orçado (< 3,0 s). A autoridade de transporte no Rust gerencia `ChapterQueue`, transições gapless entre capítulos, pré-síntese cooperativa, saltos contextuais (seção/capítulo) e resiliência a entradas inválidas (vazio/não-UTF8/> 93.636 bytes) sem interromper o transporte. O progresso de diretório é durável em tabela dedicada `DIRECTORY_BOOKMARKS` (chave por fingerprint composto Blake3 da coleção ordenada), com heartbeat e oferta de retomada na reabertura.

### Evidence (tasks, artifacts, git)

- `specs/010-directory-reading/tasks.md`: **49 checked / 0 unchecked (49 total)**. Fases completas: setup/infra (T185–T189), fundação/armazenamento/queue (T190–T194), US1 seleção+TTFA+gapless+UI (T195–T203), US2 retomada precisa/heartbeat/oferta (T204–T210), US3 navegação contextual/resiliência/pré-roll (T211–T218), verificação e hardware (T219–T226). Subtarefas de evidência: **T226-PROG** concluído (ingest/manifesto/dispatch não-UI via hook Spec 012/JNI), **T226-RES-SAF-TREE/TRANSITION/RESUME-UI/HARDWARE-CONTROLS/INVALID-ENTRIES-UI/A11Y-PICKER** concluídos com validação E2E em hardware.
- Artifacts/spec: `spec.md`, `plan.md`, `tasks.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/directory-playback.md`, `checklists/` presentes no diretório da spec.
- Git evidence (commits relevantes): `1aa568d4 docs(governance): attest specs 010 and 013 hardware convergence and update roadmap to 98%`; suporte/hardening de automação e evidência programática relacionados: `c3252f85`, `2c40a52d`, `5c86593e` (Spec 012 hook runs/wiring usados para T226-PROG). Nenhum commit isolado com prefixo `spec-010` no recorte de 120 commits, mas a atestação de convergência em hardware e o ledger de tasks/ROADMAP registram a entrega.
- ROADMAP alignment: **CONVERGED (Atestado em hardware real, 49/49 tasks)** — `ChapterQueue`, tree SAF, gapless.

## Gate results

- Gates 1–8 / suíte de verificação de host: **[verified in prior cycles]** — tasks T219–T225 registram fmt, nextest workspace, census de testes, clippy, scans de pureza/shell, `cargo deny/audit` e `./scripts/verify.sh` como concluídos; nenhuma execução de gate fabricada neste registro.
- Gate 9 (Android build): **[not re-run in this pass]** — integração Android validada em ciclos anteriores.
- Validação em hardware real (E2E): **[verified in prior cycles]** — T226 registra execução em dispositivo físico Xiaomi Redmi Note 11 (2201117TY, Android 13 SDK 33, wireless ADB `adb-53cf9aa7`): seleção SAF de árvore, TTFA medido (1.551 ms < 3,0 s), transições/capítulos, persistência de bookmarks, controles `next/previous` e filtragem de entradas inválidas.

## Non-Changes

- Nenhuma modificação de código, execução de gates, formatters, builds ou testes realizada neste registro de convergência (edit-only).
- Sem expansão de escopo além do consolidado em `spec.md`/`plan.md`/`tasks.md` e do estado ROADMAP.

## Unresolved / Deferred

- **Nenhum bloqueador aberto** no ledger de tasks (49/49 concluídas, incluindo subtarefas T226-*). Residualidades históricas de escopo foram cobertas por evidência programática e atestação em hardware físico conforme tasks.
- Observação: medições de desempenho dependem das condições do dispositivo/execução registradas; este registro não repete medições.

## Acceptance Criteria Status

- [x] Seleção de diretório via SAF, indexação/ordenação e início de reprodução contínua com TTFA orçado [verified in prior cycles / hardware attested].
- [x] Transição automática gapless entre capítulos, persistência de `CompositeBookmark`/`DIRECTORY_BOOKMARKS` e oferta de retomada precisa [verified in prior cycles].
- [x] Navegação contextual (seção/capítulo), resiliência a arquivos inválidos/oversize e controles de transporte integrados [verified in prior cycles].
- [x] Evidência end-to-end em dispositivo físico e subtarefas de escopo residual (SAF/transição/resume/controles/a11y/entradas inválidas) concluídas (T226 + T226-*).
