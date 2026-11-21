# Converge: 025 Action View Display Name

Registro **append-only**: novas entradas devem ser acrescentadas ao final, sem editar ou renumerar as anteriores.

## 2026-11-21 — implementação e verificação documental local (ciclo SDD 025)

**Status**: implementado no working tree de `025-action-view-display-name`; evidências locais medidas neste checkout; PR/merge/CI, gate completo e dispositivo `[NV]`.

### O que convergiu

- Assess `.specify/assessments/action-view-display-name/{intake,research,problem,concept,decision}.md` — veredito **go** (Option A, fallback MediaStore localizado no Kotlin).
- Artefatos em `specs/025-action-view-display-name/`: `spec.md` (FR-025-001..005, SC-025-001..005, 6 clarificações), `plan.md` (Phase 0/1 embutidos, Fases A–D), `tasks.md` (T001..T012), `checklists/requirements.md` (23/23) e `analyze.md` (PASS com observações; 4 divergências LOW resolvidas).
- `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`: imports `BaseColumns` e `MediaStore`; predicado `isUsableName` (:640-641) aplicado aos três candidatos; `queryName(resolver, uri)` (:643-670) com consulta direta e fallback à coleção `MediaStore.Files` por `_id=?` no volume extraído; ramo `ACTION_VIEW` (:768-784) com cascata de nome e `effectiveUri = "content://display_name/${Uri.encode(name)}"` incondicional.
- Consumidor Rust verificado por leitura, não alterado: `picked_document_name` (`src-tauri/src/shell_jni.rs:1103`) isola o último segmento, aplica `percent_decode` e acrescenta a extensão do snapshot — o URI sintético já produz o título sem mudança na ponte JNI.
- Caminhos preservados: `queryName(resolver, treeUri, docId)` (SAF/diretório, :634-638), ramo `ACTION_SEND` (:785+), `blake3_256`/`writeSnapshot`/`onDocumentIngested` (:734-751) e a assinatura JNI em `shell_jni.rs:849-893`.
- Rastreabilidade completa FR/SC ↔ T001..T012 ↔ símbolos reais em `analyze.md` (10/10 requisitos cobertos, 0 CRITICAL/HIGH).

### Evidência local (medida neste checkout)

- `git diff --stat` sobre `DocIngest.kt`: **1 arquivo alterado, 36 inserções, 10 remoções** — confinado a imports, predicado/`queryName` e ramo `ACTION_VIEW`.
- Inspeção estrutural: consulta a `MediaStore.Files.getContentUri(volume)` com `"${BaseColumns._ID}=?"` e argumento do id (:659-664); filtro `isUsableName` (:650, :669, :780); fallback `document.md` (:781); nenhum `uri.toString()` entre :768 e :784.
- Consumidor Rust: `percent_decode(last)` em `shell_jni.rs:1108`, confirmado por leitura.
- Checklist de requisitos: 23/23 itens revisados e marcados `[x]`.
- Nenhum arquivo de outra spec, bug, `scripts/verify.sh` ou `converge.md` alterado por este ciclo.

### [NV] — não verificado neste ciclo

- Resolução real do `_display_name` na coleção `MediaStore.Files` em dispositivo `2201117TY` e o título final exibido na interface: **[NV]**.
- `verify.sh` completo (gates 1–8) e seu exit code: **[NV]** até leitura pelo orquestrador — nenhuma afirmação de `VERIFY_EXIT=0` é feita aqui.
- Builds, formatadores, testes e CI/PR/merge remotos: **[NV]** nesta entrada.
- Latência de 183 s até o primeiro áudio (`preroll_secs=60.000`): registrada como **DEFER**, fora do escopo e sem tarefa neste ciclo.

### Dívidas remanescentes

1. Validação do `volume` contra a lista de volumes disponíveis — premissa assumida como primeiro segmento com default `external`.
2. Substituição de `document.md` por um nome derivado de metadados do item quando a coleção não responder — candidata a ciclo próprio.
3. Comprovação em dispositivo do título "Hamlet trecho.md" no lugar de `document:1000848919.md` — `[NV]`.
4. Redução da latência de 183 s (pré-roll 60 s) — **DEFER**, dono natural T232.

### Rollback

Reverter as alterações de código em `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` e remover os diretórios/artefatos documentais criados neste ciclo em `specs/025-action-view-display-name/`, `.specify/assessments/action-view-display-name/`, `.specify/validations/action-view-display-name/` e `.specify/releases/action-view-display-name/`. Nenhuma configuração remota, tag ou assinatura foi alterada; nenhum arquivo de outra spec é tocado.
