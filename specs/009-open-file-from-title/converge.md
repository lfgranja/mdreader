# Converge: 009 Abrir Arquivo Pelo Título e Iniciar Reprodução

Status: **CONVERGED** (0 findings, 26/30 tasks) — seleção de arquivo pelo título via SAF/JNI com identidade por Blake3 e política de retomada/`offerDialog`; resíduos de validação UI/a11y em dispositivo permanecem rastreados.

## Implementation Summary

Implementada a abertura interativa de documentos compatíveis (`.md`, `.txt`) a partir do bloco de título (`.meta-box`) da Lente Sinestésica no Android. O toque/ativação por teclado dispara o seletor nativo SAF (`ACTION_OPEN_DOCUMENT`) por meio da ponte Kotlin/JNI (`DocIngest.kt`, `ShellHost`, `shell_jni.rs`), com ingestão sob autoridade exclusiva do Rust (`open_document_bytes_named`), snapshot privado, hash de conteúdo Blake3-256 como identidade de documento/bookmark, início automático de reprodução para documentos novos e preservação do contrato de retomada/oferta (`offerDialog` / `AwaitOffer`, janela de 3,0 s) para pausas deliberadas. Cancelamentos, recusas (`FormatUndeliverable`/arquivos inválidos/vazios) e falhas de disponibilidade do SAF são tratados sem corromper o transporte nem o documento atual.

### Evidence (tasks, artifacts, git)

- `specs/009-open-file-from-title/tasks.md`: **26 checked / 4 unchecked (30 total)**. Concluídos T158–T183 (infra/JNI/ação `pick_document`, MVP de seleção e auto-play, retomada/oferta, cancelamento/refusas, formatação/testes/gates/quickstart ADB em ciclos anteriores). Em aberto: **T184** (validação manual/ADB on-device dos cenários 1–4), **T184-PROG** (cobertura programática não-UI exercitada parcialmente via hook de debug Spec 012, sem fechar escopo UI completo), **T184-RES-SAF-CANCEL** (cancel/back nativo SAF com reprodução ativa — residual UI), **T184-RES-A11Y** (teclado/focus-visible/TalkBack/restauração de foco no fluxo título↔picker — residual acessibilidade/UI).
- Artifacts/spec: `spec.md`, `plan.md`, `tasks.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/ipc-and-jni.md`, `checklists/` presentes no diretório da spec.
- Git evidence (commits relevantes): `bb271cf1 fix(spec-009): wire title-box click handler, propagate picker errors, initialize JVM on IPC path`, `703066b7 fix(spec-009): fix shell_jni.rs compilation for Android target`, `2ca2c8af fix(spec-009): surface title-box picker failures and harden debounce`, `8b1a14a3 fix(spec-009): update fix.md and test.md with verified device results`; suporte de validação programática ligado à Spec 012: `2c40a52d docs(spec-012): record T049 hook run ...`, `c3252f85 fix(spec-012): wire FR-008 evidence emission into normal hook device runs (T018/T024/T067)`.
- ROADMAP alignment: **CONVERGED (0 findings, 26/30 tasks)** — SAF JNI, Blake3 hash, `offerDialog`.

## Gate results

- Gates 1–8 / suíte de verificação de host: **[verified in prior cycles]** — tasks T177–T182 registram `cargo fmt --check`, `cargo nextest --workspace`, census de testes, `cargo clippy -D warnings`, scans de pureza/shell e `./scripts/verify.sh` como concluídos; nenhuma execução de gate fabricada neste registro.
- Gate 9 (Android build): **[not re-run in this pass]** — escopo Android coberto por ciclos anteriores/integração do app; sem novo artefato atestado aqui.
- Validação on-device UI completa (SAF real/cancelamento/a11y): **pendente** (T184/T184-RES-*).

## Non-Changes

- Nenhuma modificação de código, execução de gates, formatters, builds ou testes realizada neste registro de convergência (edit-only).
- Sem expansão de escopo além do consolidado em `spec.md`/`plan.md`/`tasks.md` e do estado ROADMAP.

## Unresolved / Deferred

- **T184**: validação manual/ADB end-to-end em dispositivo/emulador para cenários 1–4 do `quickstart.md` permanece aberta.
- **T184-PROG**: evidência programática não-UI exercitada parcialmente via hook de debug (Spec 012) — **não** satisfaz fluxos UI nativos (SAF cancel, oferta visual completa, acessibilidade/foco); `full_parent_completion_claimed=false`.
- **T184-RES-SAF-CANCEL** (open): cancel/back no seletor SAF nativo com reprodução ativa deve preservar playback/metadados — residual UI.
- **T184-RES-A11Y** (open): travessia teclado/TalkBack, `:focus-visible`, anúncio e restauração de foco no título/picker — residual acessibilidade/UI.
- Dependência cruzada: fechamento integral do escopo residual depende de verificação UI/acessibilidade em ambiente com dispositivo/harness adequado (apoio parcial por Spec 012, sem substituir validação nativa).

## Acceptance Criteria Status

- [x] Seleção de documento compatível pelo título com acionamento SAF/JNI e ingestão pela rota de produção Rust (`open_document_bytes_named`) [verified in prior cycles].
- [x] Identidade de documento/bookmark por Blake3-256 e preservação da política de retomada/`offerDialog` para pausas deliberadas [verified in prior cycles].
- [x] Tratamento de cancelamento/refusas/formato/tamanho sem corromper sessão ativa, conforme contratos e tasks concluídas [verified in prior cycles].
- [ ] Cobertura completa de cenários UI nativos on-device (SAF cancel/back, acessibilidade/talkback/foco, oferta visual end-to-end) — **pendente/deferida** (T184 residuals).
