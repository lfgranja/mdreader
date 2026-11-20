# Release (R7): 024 Retomada de Diretório na Reabertura

- **Date**: 2026-11-20
- **Branch**: `024-directory-resume-offer`
- **SemVer proposto**: **v0.1.0-rc3** (pre-release incremental sobre `v0.1.0-rc2`; sem bump de aplicativo independente — a linha `v0.1.0-rc` permanece a trilha de release). Proposta, não aplicada.
- **Validation**: R4 PASS nos checks locais/host (`.specify/validations/directory-resume-offer/validate.md`); `verify.sh` completo, CI remoto, device, janela visual, TalkBack/foco e gate 9 `[NV]`.
- **Status de merge**: **NÃO houve merge.** As mudanças permanecem no working tree do branch `024-directory-resume-offer`; nenhum `git add`/`git commit` foi realizado por este ciclo de artefatos.

---

## 1. Resumo

Fecha a cola de integração do ciclo 024 (T233): a reabertura de um diretório com bookmark `Paused` volta a publicar a rota do diálogo in-app, estaciona a oferta até a resposta do usuário e entrega `filename`/`offset_secs` derivados do bookmark à interface. Nenhum comportamento novo de produto é introduzido — a máquina de oferta existente continua sendo a única autoridade.

---

## 2. Changelog (IDs)

- **FR-024-001 / T001, T003, T005, T006**: publicação da rota `in_app_dialog` no caminho de reabertura; remoção dos marcadores `#[expect(dead_code)]` que suprimiam `park_directory_resume` e `DirectoryResume`; oferta estacionada até a resposta (`src-tauri/src/lib.rs`, `src-tauri/src/shell_jni.rs`).
- **FR-024-002 / T003, T006**: `DirectoryResume` passa a carregar `filename: String` e `offset_secs: f64` originados do `CompositeBookmark` ativo; evento `directory-selected` enriquecido, sem segunda origem de posição.
- **FR-024-003 / T004, T007**: `route_offer` chamado exatamente uma vez por reabertura; transição `expiry → AutoStart` permanece na máquina mutata, sem re-derivação de rota em poll (`published_route` preservado).
- **FR-024-004 / T008, T011**: o tratador de `directory-selected` atribui `state.chapterFilename` e `state.saved_offset_secs` **antes** de `refreshOffer()`; nenhuma alteração de marcação HTML.
- **FR-024-005 / T002, T012, T013**: asserções literais de presença de strings substituídas por asserções comportamentais (rota publicada, payload com nome/offset, janela de 3.000 ms, resolução única) no target já pinado `src-tauri::test_directory_resume`; nenhum novo target e nenhuma edição em `scripts/expected-tests.txt`.
- **FR-024-006 / T009, T010, T014**: `src-tauri/src/status_view.html` espelhado byte a byte em `dist/index.html`, `src-tauri/src/assets/index.html`, `src-tauri/dist/index.html` e `src-tauri/gen/android/app/src/main/assets/index.html` (5/5 idênticos por `cmp -s`).
- **Correção (bug sem ID formal) / T233**: T233 da spec 010 permanecia `[ ]`/parcial — o mecanismo existia, mas a cola faltava (código morto suprimido e teste provando strings em vez de comportamento). Nenhum ID de bug dedicado foi emitido neste ciclo; a referência é T233, sem renumeração de T232/T233 e sem edição de `specs/010`.
- **Polish / T015**: revisão de que os IDs são T001..T015, nenhum ID da spec 010 reintroduzido e nenhum arquivo fora do escopo tocado.

Evidência de código local: `cargo fmt --all --check` (exit 0); `cargo clippy --workspace --all-targets -- -D warnings` (exit 0); `cargo test -p src-tauri --test test_directory_resume` (4 passed); 23/23 itens do checklist revisados.

---

## 3. Rollback

Reverter exatamente os **8 arquivos de código** modificados e os **3 artefatos centrais** do ciclo (mais os documentos de apoio), todos no working tree — como nada foi comitado nem mesclado, o rollback é uma operação local e não afeta nenhuma tag/release publicada:

**Código (8):**
1. `src-tauri/src/lib.rs`
2. `src-tauri/src/shell_jni.rs`
3. `src-tauri/src/status_view.html`
4. `src-tauri/tests/test_directory_resume.rs`
5. `dist/index.html`
6. `src-tauri/src/assets/index.html`
7. `src-tauri/dist/index.html`
8. `src-tauri/gen/android/app/src/main/assets/index.html`

**Artefatos (3):** `specs/024-directory-resume-offer/spec.md`, `plan.md`, `tasks.md`.

**Documentos de apoio do mesmo ciclo:** `specs/024-directory-resume-offer/checklists/requirements.md`, `.specify/assessments/directory-resume-offer/{intake,research,problem,concept,decision}.md`, `.specify/validations/directory-resume-offer/validate.md` e este `release.md`.

Procedimento: `git checkout -- <8 arquivos de código>` e remover o diretório `specs/024-directory-resume-offer/` e os diretórios `.specify/{assessments,validations,releases}/directory-resume-offer/`. `scripts/expected-tests.txt` **não** faz parte do rollback (nunca foi alterado). Não há migração de schema, configuração remota, surface de consentimento ou build assinado envolvidos: restore dos 8 arquivos retorna o estado anterior ao branch, com o teste pinado retomando as asserções originais. Nenhum arquivo de outra spec é tocado.

---

## 4. Riscos residuais / [NV]

- **`verify.sh` completo (gates 1–8)**: em execução em background; resultado pendente — `[NV]`.
- **CI remoto / merge de PR**: não executado; nenhuma verificação remota é reivindicada — `[NV]`.
- **Dispositivo Android e gate 9** (build/pacote aarch64, APK/AAB): não executados neste ciclo — `[NV]`.
- **Janela de 3 s visual / auto-continue em primeiro plano**: verificados apenas por constantes e transições em host; percepção em device — `[NV]`.
- **TalkBack / foco do diálogo**: não aferidos — `[NV]`.
- **C-H8 (divergência de bundles)**: mitigada por cópia byte a byte e `cmp -s` (5/5 idênticos); risco residual apenas se um espelho for editado isoladamente após este ciclo.
- **`route_offer` mutata (chamada dupla)**: mitigada por chamada única no caminho de reabertura; qualquer re-derivação em poll reabre o risco de `expiry → AutoStart` duplo.
- **Cobertura de ingressos**: escopo explícito é `handle_directory_selected`; outros caminhos de reabertura, se existirem, permanecem fora deste ciclo.

---

## 5. Confirmação de estado

- Nenhum merge realizado; branch `024-directory-resume-offer` com mudanças locais, sem commit deste ciclo.
- Nenhuma tag, assinatura, publicação em loja ou configuração remota alterada.
- `specs/010` intacto; T232/T233 não renumerados; nenhum `converge.md` de outra spec alterado.
