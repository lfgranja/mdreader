# Converge: 027 Exaustão da produção não finaliza o documento

Registro **append-only**: novas entradas devem ser acrescentadas ao final, sem editar ou renumerar as anteriores.

## 2026-11-21 — cadeia documental do ciclo SDD 027 (pré-implementação)

**Status**: cadeia documental produzida no working tree de `feature/027-production-exhaustion`, criada de `dev @ 61c86200`. A correção de código, gates, builds, testes e dispositivo permanecem `[NV]` nesta entrada; Factory HEAD `b1deab1` é somente leitura.

### O que convergiu

- Assess `.specify/assessments/production-exhaustion/{intake,research,problem,concept,decision}.md` — veredito **go** (Option A: detecção de exaustão no topo do `while`; scorecard com evidência `adequate` em host e risco de device `[NV]`). Origem declarada: bug `playback-stall-at-end-of-document`, achado no smoke de longa duração do ciclo 026.
- Artefatos em `specs/027-production-exhaustion/`: `spec.md` (FR-027-001..006, SC-027-001..006, 7 clarificações), `plan.md` (Phase 0/1 embutidos, Fases A–D), `tasks.md` (T001..T014), `checklists/requirements.md` (23/23 `[x]`), `analyze.md` (PASS com observações, pré-implementação; 5 divergências LOW resolvidas por leitura).
- Diagnóstico de host confirmado nos símbolos atuais: `produce_bounded` (`src-tauri/src/lib.rs:5773-5863`) só chama `finish_production_if_current` após `commit_produced_unit` (:5834-5841) e handover (:5842); o ramo `segments.get(next_unit) == None` (:5817-5819) com `pending` vazio executa `return Ok(())` sem finalizar. `state.production` permanece `Some`, `production_running` (:6360-6365) continua `true` e o gate `drained && !production_running` (:2669) nunca chama `signal_document_end` (:3215) — sem `action=complete` nem `session_end`, PLAYING travado em 199,203 s no smoke do ciclo 026.
- Caminho de correção especificado (não aplicado nesta entrada): checagem de exaustão no topo do `while` (:5782), em bloco curto com `MutexGuard` solto; chamada a `finish_production_if_current` (:6004-6018), que já é generation-safe (:6009-6013) e instala a timeline medida (:6016); `break` subsequente. Checagem de `done` pós-commit (:5847-5860) preservada para o tick da última unidade.
- Fronteiras fixadas: nenhuma chamada de `signal_document_end`/`dispatch_document_end`/`ShellVerb::Complete` no produtor; nenhum enum novo; `production_running`, `advance_production`, handover, backpressure, primário de 026 e contrato PCM mono `f32` @ 22050 Hz inalterados; teste no módulo `ttfa_013_tests` do target pinado, ao lado de `t007` (:11007-11061).
- Cadeia documental fechada com `validate.md` (R4) e `release.md` (R7): SemVer **v0.1.0-rc6 proposto**, não aplicado, com rollback local.

### Evidência local (medida neste checkout)

- Leitura estrutural de `src-tauri/src/lib.rs` (:2628-2692, :3215-3226, :5708-5726, :5773-5863, :5943-5971, :6004-6026, :6305-6365, :6390-6405, :6473-6492, :11007-11061) e de `crates/player/src/backend.rs:778-780`.
- Bug de origem lido em `.specify/bugs/playback-stall-at-end-of-document/{assessment,fix,test}.md`: veredito "likely valid, needs reproduction in clean session", severidade medium-high, fix e test ainda `pending`.
- Rastreabilidade 12/12 (FR+SC), checklist 23/23 `[x]`, 14 tarefas sem órfãs, 0 marcadores de ambiguidade, 0 violações de constituição.
- Nenhum arquivo de outra spec, bug, `scripts/verify.sh`, `scripts/expected-tests.txt`, `converge.md` de outro ciclo, HTML, capabilities ou workflow foi alterado por esta entrada.

### [NV] — não verificado neste ciclo

- Correção de código efetivamente aplicada e diff resultante: **[NV]** nesta entrada; a implementação cabe ao agente principal.
- `verify.sh` completo (gates 1–9) e seu exit code: **[NV]**; nenhuma afirmação de `VERIFY_EXIT=0` é feita aqui.
- `cargo fmt`, `cargo clippy`, builds, suíte de testes e CI: **[NV]**; não executados por esta cadeia documental.
- Dispositivo `2201117TY`: `action=complete` + `session_end` após >220 s, com diag zerado, e A/B com/sem primário — **[NV]**. O bug de origem permanece `pending` até essa reprodução.
- PR, merge e publicação: **[NV]**; nada foi comitado ou mesclado por esta entrada.

### Dívidas remanescentes

1. Aplicar a checagem de exaustão em `produce_bounded` e o teste em `ttfa_013_tests` conforme T003–T005 e T012 (owner natural: implementação do ciclo 027).
2. Reprodução limpa em device (diag zerado, ACTION_VIEW com `-f 0x1`, observação >220 s) para fechar `.specify/bugs/playback-stall-at-end-of-document/fix.md` e `test.md`.
3. A/B com/sem primário para descartar participação do ciclo 026 no travamento — `[NV]`.

### Rollback

Reverter a alteração de código em `src-tauri/src/lib.rs` (remover a checagem de exaustão, a chamada a `finish_production_if_current`, o `break` associado e o teste de regressão) e remover os artefatos documentais criados neste ciclo em `specs/027-production-exhaustion/`, `.specify/assessments/production-exhaustion/`, `.specify/validations/production-exhaustion/` e `.specify/releases/production-exhaustion/`. Nenhuma configuração remota, tag, assinatura, schema persistido ou arquivo de outra spec é tocado; como nada foi comitado ou mesclado, o rollback é local e reversível por restauração do working tree.
