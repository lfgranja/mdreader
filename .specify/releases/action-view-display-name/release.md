# Release (R7): 025 Nome de exibição no ACTION_VIEW via MediaStore

- **Date**: 2026-11-21
- **Branch**: `025-action-view-display-name`
- **SemVer proposto**: **v0.1.0-rc4** (pre-release incremental sobre a proposta `v0.1.0-rc3` do ciclo 024; sem bump de aplicativo independente — a linha `v0.1.0-rc` permanece a trilha de release). Proposta, não aplicada. Tags presentes no checkout: `v0.1.0-rc1`, `v0.1.0-rc2`.
- **Validation**: R4 PASS nos checks documentais/host realizados (`.specify/validations/action-view-display-name/validate.md`); dispositivo, `verify.sh`, builds, testes, CI remoto e gate 9 `[NV]`.
- **Status de merge**: **NÃO houve merge.** As mudanças permanecem no working tree do branch `025-action-view-display-name`; nenhum `git add`/`git commit` foi realizado por este ciclo de artefatos.

---

## 1. Resumo

Corrige a titulação de documentos abertos via `ACTION_VIEW` com URI de item do MediaStore: quando a consulta direta a `OpenableColumns.DISPLAY_NAME` falha, o nome é resolvido na coleção `MediaStore.Files` por `_id=?`, filtrado por um predicado que rejeita valores vazios e numéricos, e codificado na URI sintética `content://display_name/<nome>` que o Rust já decodifica. O título `document:1000848919.md` dá lugar ao nome real do arquivo. SAF, `ACTION_SEND`, fingerprint BLAKE3 e ponte JNI permanecem intactos. A latência de 183 s até o primeiro áudio é **DEFER**, fora deste ciclo.

---

## 2. Changelog (IDs)

- **FR-025-001 / T003, T004, T005**: `queryName(resolver, uri)` (`DocIngest.kt:643-670`) passa a consultar a coleção `MediaStore.Files.getContentUri(volume)` com selection `"${BaseColumns._ID}=?"` e argumento do id extraído do último segmento, restrita a URIs `content://` com authority do MediaStore; imports `BaseColumns`/`MediaStore` adicionados.
- **FR-025-002 / T002, T006, T007, T008**: o ramo `ACTION_VIEW` (:768-784) constrói incondicionalmente `effectiveUri = "content://display_name/${Uri.encode(name)}"`, eliminando o fallback para `uri.toString()`. O consumidor Rust `picked_document_name` (`shell_jni.rs:1103`) decodifica o último segmento — nenhuma mudança em Rust ou na assinatura JNI.
- **FR-025-003 / T001, T005, T006**: predicado único `isUsableName` (:640-641) rejeita nulo, em branco e somente dígitos nos três candidatos (consulta direta, coleção e último segmento); quando nenhum é utilizável, o nome é `document.md`.
- **FR-025-004 / T009, T010, T011**: preservados `queryName(resolver, treeUri, docId)` (SAF/diretório, :634-638), o ramo `ACTION_SEND` (:785+), `blake3_256`/`writeSnapshot`/`onDocumentIngested` (:734-751) e a assinatura JNI (`shell_jni.rs:849-893`). Diff restrito a 1 arquivo, 36 inserções e 10 remoções.
- **FR-025-005 / T012**: verificação em host por inspeção estrutural, rastreabilidade 10/10 em `analyze.md`, `[NV]` para dispositivo e **DEFER** registrado para a latência de 183 s.
- **Correção (regressão reaberta, sem ID formal de spec)**: sintoma `document:1000848919.md` no lugar de "Hamlet trecho.md" ao abrir via app de arquivos (`content://media/external/file/1000848919`). Nenhum ID de bug dedicado foi emitido neste ciclo; a referência é o bug `playback-not-starting-after-file-selection`, sem edição daquele artefato.
- **Polish / T012**: revisão de que os IDs são T001..T012, nenhum ID de outro ciclo foi reintroduzido e nenhum arquivo fora do escopo foi tocado.

Evidência de host local: `git diff --stat` (1 arquivo, 36+/10-); inspeção estrutural da consulta à coleção, do predicado e da URI sintética; leitura de `picked_document_name`; 23/23 itens do checklist revisados. Nenhum gate, build, teste ou formatador é reivindicado como executado por esta cadeia documental.

---

## 3. Rollback

Reverter a alteração de código em `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` e remover os artefatos documentais criados neste ciclo, todos no working tree — como nada foi comitado nem mesclado, o rollback é uma operação local e não afeta nenhuma tag/release publicada:

**Código (1):**
1. `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`

**Artefatos centrais (5):** `specs/025-action-view-display-name/spec.md`, `plan.md`, `tasks.md`, `analyze.md`, `converge.md`.

**Documentos de apoio:** `specs/025-action-view-display-name/checklists/requirements.md`, `.specify/assessments/action-view-display-name/{intake,research,problem,concept,decision}.md`, `.specify/validations/action-view-display-name/validate.md` e este `release.md`.

Procedimento: `git checkout -- android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` e remover os quatro diretórios de artefatos listados acima. Nenhuma configuração remota, migração de schema, surface de consentimento, tag, assinatura ou build assinado está envolvida. `specs/019`, `specs/009`, bugs, `scripts/verify.sh` e `converge.md` de outras specs **não** fazem parte do rollback (nunca foram alterados). Restaurar os 4 blocos do Kotlin retorna o estado anterior ao branch: consulta direta única e fallback para a URI bruta no `ACTION_VIEW`.

---

## 4. Riscos residuais / [NV]

- **Dispositivo (`2201117TY`)**: resolução real do `_display_name` e título final exibido — **[NV]**; a evidência ADB do bug permanece como dado de contexto, não como verificação deste ciclo.
- **`verify.sh` completo (gates 1–8)**: não executado por esta cadeia; resultado pendente de observação pelo orquestrador — **[NV]**.
- **Builds, formatadores e testes**: não executados neste ciclo documental — **[NV]**.
- **CI remoto / merge de PR**: não executado; nenhuma verificação remota é reivindicada — **[NV]**.
- **Gate 9 / aparelho**: build/pacote aarch64 (APK/AAB) e medição de áudio — **[NV]**.
- **Volume**: inferido do primeiro segmento com default `external`; volumes não-padrão degradam para `document.md` — risco mitigado, sem verificação em device.
- **Codificação do nome**: `Uri.encode`/`percent_decode` verificados por leitura estrutural; nomes com caracteres especiais em device — **[NV]**.
- **Latência de 183 s (pré-roll 60 s)**: **DEFER**, dono natural T232; nenhuma tarefa deste ciclo a endereça.

---

## 5. Confirmação de estado

- Nenhum merge realizado; branch `025-action-view-display-name` com mudanças locais, sem commit deste ciclo.
- Nenhuma tag, assinatura, publicação em loja ou configuração remota alterada.
- `specs/019`, `specs/009`, bugs, `scripts/verify.sh` e `converge.md` de outras specs intactos.
- Cadeia documental completa: assess (5) → spec/plan/tasks/analyze/converge (5) + checklist → validate (R4) → release (R7).
