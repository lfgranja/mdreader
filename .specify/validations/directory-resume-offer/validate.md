# Validation (R4): 024 Retomada de Diretório na Reabertura

- **Date**: 2026-11-20
- **Branch**: `024-directory-resume-offer`
- **Spec**: `specs/024-directory-resume-offer/spec.md` (FR-024-001..006, SC-024-001..006)
- **Status**: **PASS nos checks locais/host**; itens dependentes de device, CI remoto, janela visual e acessibilidade real estão `[NV]`. Este artefato é documental — nenhum gate, teste ou formatador foi executado por ele; a evidência abaixo foi obtida pelo agente principal no branch alvo.

---

## 1. Evidência local obtida (host)

| Check | Comando / evidência LOCAL | Resultado |
|---|---|---|
| Formatação | `cargo fmt --all --check` | **exit 0** — PASS |
| Lint pedantic | `cargo clippy --workspace --all-targets -- -D warnings` | **exit 0** — PASS |
| Teste comportamental pinado | `cargo test -p src-tauri --test test_directory_resume` | **4 passed** — PASS (target `src-tauri::test_directory_resume`, pin em `scripts/expected-tests.txt:113`) |
| Byte-identity dos bundles | `cmp -s` entre os 5 HTML (`src-tauri/src/status_view.html`, `dist/index.html`, `src-tauri/src/assets/index.html`, `src-tauri/dist/index.html`, `src-tauri/gen/android/app/src/main/assets/index.html`) | **byte-idêntico** (5/5) — PASS |
| Checklist de qualidade | `specs/024-directory-resume-offer/checklists/requirements.md` — CHK001..CHK023 | **23/23** revisadas e marcadas `[x]` — PASS |
| Qualidade da spec | Sem marcadores `[NEEDS CLARIFICATION]`; 5 clarificações registradas; FR/SC rastreáveis | PASS (revisão documental) |
| Gate de avaliação | `.specify/assessments/directory-resume-offer/decision.md` — veredito `go` | PASS |
| Imutabilidade do pin | `git status --short` não lista `scripts/expected-tests.txt` como modificado; nenhum novo target criado | PASS (inspeção local) |

---

## 2. Success Criteria — SC-024-001..006

| SC | Critério (spec) | Evidência LOCAL | Resultado |
|---|---|---|---|
| **SC-024-001** | 100% dos bookmarks `Paused` na reabertura resultam em rota de diálogo publicada e janela de 3.000 ms (baseline: rota `none`) | Teste pinado (`4 passed`) afirma a rota `in_app_dialog` e a constante de 3.000 ms; inspeção de `lib.rs`/`shell_jni.rs` confirma publicação no caminho de reabertura | **PASS em host**; a experiência visual da janela em dispositivo é `[NV]` |
| **SC-024-002** | 100% dos payloads contêm `filename` não vazio e `offset_secs` igual ao bookmark (baseline: campos ausentes) | Teste pinado afirma `filename` não vazio e `offset_secs = 45.0` derivado do bookmark (T003/T006) | **PASS em host** |
| **SC-024-003** | Exatamente 1 resolução da máquina de oferta por reabertura; zero re-derivações de rota em poll | Teste pinado afirma contagem = 1 (T013); `route_offer` mutata preservada, sem re-derivação em poll (T004/T007) | **PASS em host** |
| **SC-024-004** | 5 de 5 superfícies HTML byte-idênticas após a mudança | `cmp -s` dos 5 bundles → idênticos byte a byte (T009/T010/T014) | **PASS** |
| **SC-024-005** | Zero linhas adicionadas/removidas em `scripts/expected-tests.txt` e zero novos targets | `git status --short` sem alteração no pin; teste reescrito no target existente (T002/T012/T013/T014) | **PASS** |
| **SC-024-006** | 100% das atribuições de nome/offset na interface ocorrem antes da atualização da oferta | Inspeção estrutural de `src-tauri/src/status_view.html`: `state.chapterFilename`/`state.saved_offset_secs` atribuídos antes de `refreshOffer()` (T008/T011) | **PASS (inspeção estrutural em host)** |

> Todas as SCs acima são mensuráveis em host e foram satisfeitas neste ciclo. Nenhuma afirmação de comportamento visual em dispositivo é reivindicada.

---

## 3. [NV] — Não verificado neste ciclo

- **`verify.sh` completo**: `PROPTEST_CASES=32 bash scripts/verify.sh` (gates 1–8) está **em execução em background**; resultado pendente de verificação pelo orquestrador. Nenhum resultado de gate é afirmado aqui.
- **CI remoto / PR**: pipelines remotos, status de PR e merge não executados nem observados (`[NV]`).
- **Dispositivo Android**: nenhuma execução em aparelho real; toda a evidência é de host (`[NV]`).
- **Janela de 3 s visual**: o cronômetro de 3.000 ms e o auto-continue em primeiro plano foram validados por constantes e transições da máquina em host; a percepção visual em device é `[NV]`.
- **TalkBack / foco**: navegação por leitor de tela e ordem de foco do diálogo não foram aferidas (`[NV]`).
- **Gate 9**: build/pacote Android (APK/AAB, compile para aarch64) não executado neste ciclo (`[NV]`).

---

## 4. Limites do ciclo

- Nenhum outro arquivo foi editado por este artefato além de `validate.md` e `release.md`.
- `specs/010` não foi editado e T232/T233 não foram renumerados.
- `scripts/expected-tests.txt` permanece intacto; nenhum novo target de teste foi criado.
- Nenhum `git add`/`git commit` realizado; o branch permanece com as mudanças no working tree e **sem merge** (ver `.specify/releases/directory-resume-offer/release.md`).
