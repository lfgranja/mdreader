# Converge: 024 Directory Resume Offer

Registro **append-only**: novas entradas devem ser acrescentadas ao final, sem editar ou renumerar as anteriores.

## 2026-11-20 — implementação e verificação local (ciclo SDD 024 / T233)

**Status**: implementado em `024-directory-resume-offer`; evidências locais medidas neste checkout; PR/merge/CI e dispositivo `[NV]`.

### O que convergiu

- Assess `.specify/assessments/directory-resume-offer/{intake,research,problem,concept,decision}.md` — veredito **go** (Option A, cola mínima sobre a máquina de oferta existente).
- Artefatos em `specs/024-directory-resume-offer/`: `spec.md` (FR-024-001..006, SC-024-001..006), `plan.md`, `tasks.md` (T001..T015), `checklists/requirements.md` (23/23) e `analyze.md` (PASS com observações; divergências resolvidas).
- `src-tauri/src/lib.rs`: `DirectoryResume` enriquecido (`chapter_filename`, `stored_position`) em `install_chapter_queue`; `park_directory_resume` publica `InAppDialog`; `route_offer` mutata resolvida uma vez; `offer_route` apenas lê a rota publicada; marcadores de código morto removidos dos dois símbolos.
- `src-tauri/src/shell_jni.rs`: `handle_directory_selected` estaciona a retomada e publica o evento `directory-selected` com `chapterFilename`, `storedOffsetSecs`, `offerRoute`, `windowMs`, `pendingColdStart`; seam `directory_resume_open` para o teste pinado.
- `src-tauri/src/status_view.html`: listener `directory-selected` atribui `state.chapterFilename` e `state.saved_offset_secs` antes de `refreshOffer()`; `renderOffer` consome os valores sem mudança de markup.
- `src-tauri/tests/test_directory_resume.rs`: teste comportamental reescrito no target já pinado (`reopen_paused_directory_routes_to_in_app_offer_dialog_for_3s`), verificando rota publicada, payload e janela de 3.000 ms.
- Pin preservado: `scripts/expected-tests.txt:113` = `src-tauri::test_directory_resume`, sem linhas adicionadas ou removidas; nenhum novo target; `specs/010` e T232/T233 intactos.
- Cinco superfícies HTML espelhadas byte a byte: `src-tauri/src/status_view.html`, `dist/index.html`, `src-tauri/src/assets/index.html`, `src-tauri/dist/index.html`, `src-tauri/gen/android/app/src/main/assets/index.html` (auditoria C-H8).

### Evidência local (medida neste checkout)

- `cargo fmt --all --check` → exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo test -p src-tauri --test test_directory_resume` → **4 passed**.
- Byte-identity das cinco superfícies HTML confirmada por `cmp` → 5/5 idênticas.
- Rastreabilidade completa FR/SC ↔ T001..T015 ↔ símbolos reais em `analyze.md` (12/12 requisitos cobertos, 0 CRITICAL/HIGH).

### [NV] — não verificado neste ciclo

- Janela visual de 3 s e auto-continue em primeiro plano em dispositivo real: **[NV]** (host é a única autoridade aqui).
- CI/PR/merge e qualquer efeito remoto: **[NV]** até abertura e observação green.
- Resultado final do gate `scripts/verify.sh` (iniciado em background pelo agente principal): **[NV]** até leitura do exit code pelo próprio orquestrador — nenhuma afirmação de `VERIFY_EXIT=0` é feita nesta entrada.

### Dívidas remanescentes

1. `scripts/verify.sh` — desfecho pendente de observação pelo agente principal (gates 1–8 + supply chain).
2. PR/merge/CI e validação em device — `[NV]`, fora do escopo de host deste ciclo.
3. Outros ingressos de reabertura além de `handle_directory_selected` — explicitamente fora do escopo pela premissa da spec; eventual cobertura exige ciclo próprio.
4. Nomenclatura/âncoras documentadas em `analyze.md` (F1–F5): plan com `-p mdreader-shell`, nomes de campos e números de linha derivados. Não corrigidas aqui para preservar `spec.md`/`plan.md`/`tasks.md`; candidatas a alinhamento oportunista futuro.
5. T013 usa garantia estrutural de unicidade de `route_offer` em vez de contador literal; reforço opcional.

### Rollback

Reverter as alterações de código do ciclo em `lib.rs`, `shell_jni.rs`, `status_view.html` + espelhos e no teste pinado; nenhuma configuração remota foi alterada.
