# Converge: 012 Debug-Only Ingest Validation Hook

Status: **CONVERGED / Suportado** (54/69 tasks) — seam de automação debug-only para ingestão de documento/diretório (rota de produção DocIngest/JNI) implementado e amplamente exercitado em host/contratos, com cobertura programática parcial em dispositivo; exposições de release, durabilidade de overlays e validações opt-in de dispositivo/release permanecem com itens abertos.

## Implementation Summary

Endurecido o hook de validação de ingestão disponível apenas em builds debug/test para acionar, sem interação manual com o picker SAF, os fluxos reais de ingestão de documento e diretório (`DocIngest` snapshot/fingerprint + callbacks JNI `onDocumentIngested`/`onDirectorySelected` → autoridade Rust), produzindo resultados parseáveis (`ok:*|error:*|refused:*`) e evidência alinhada a FR-008. A spec estabelece: fontes/overlays canônicos versionados fora de `src-tauri/gen/**` com materialização idempotente e checks fail-closed (FR-011/SC-006), verificação combinada de não-exposição em release (FR-010), e rastreio residual separado para T184/T226 (FR-012) distinguindo cobertura programática não-UI de casos nativos de picker/a11y. Foram adicionados contratos, scripts de automação/checagem, testes de contrato em host e atualizações de rastreio residual em specs dependentes.

### Evidence (tasks, artifacts, git)

- `specs/012-debug-ingest-hook/tasks.md`: **54 checked / 15 unchecked (69 total)** (excluindo cabeçalhos/separadores; inclui itens concluídos de setup/fundação/US1–US4/FR-011/FR-010 e verificações parciais). Concluídos incluem scaffolds/contratos RED→GREEN em host (T001–T012, T014, T021, T026–T044, T045–T046, T051–T052, T069 etc.), materialização/check de overlay, scripts de exposição/release e split de residuals T184/T226. Em aberto: itens de hardening/execução e verificação opt-in/dispositivo (ex.: **T013, T015–T020, T022–T025, T047, T048, T049, T050, T067** no ledger atual) — incluindo wiring/classificação completa de runs de dispositivo, cenários cold-start/busy/corroboration, e atestação `--mode full` de FR-010/SC-004 em artefato release + target verificado.
- Artifacts/spec: `spec.md`, `plan.md`, `tasks.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/debug-ingest-automation.md`, `contracts/durable-overlay-materialization.md`, `contracts/release-exposure-evidence.md`, `contracts/t184-t226-residual-tracking.md`, `android-overlays/`, `checklists/`, notas de decisão/harness presentes.
- Git evidence (commits relevantes, amostra): `095137d3 fix(spec-012): add aggregation/packaging contract modes for T067/T049`, `c3252f85 fix(spec-012): wire FR-008 evidence emission into normal hook device runs (T018/T024/T067)`, `5c86593e fix(spec-012): add device-run evidence wiring contract modes (T018/T024/T067)`, `2c40a52d docs(spec-012): record T049 hook run 20261106T234046Z with FR-008 anomalies/gaps`, `812b8b5e docs(spec-012): attest T051 check-apply-check and T052 residual/evidence integrity`, `107e02eb docs(spec-012): mark T045 cross-artifact reconciliation complete (host)`, `f0ecef3f docs(spec-012): add host contract modes and integrity notes to debug-ingest contract`, `50225161 test(spec-012): add T067/T049 RED PBT aggregation/packaging contract`, `8eb951ae docs(spec-012): record full Spec 012 host suite 86 passed`.
- Device probe (evidência parcial registrada): run T049 em `adb-53cf9aa7` (2201117TY, SDK 33, `app.mdreader` debuggable) com cenários `executed-pass` não-UI (documento/diretório) e anomalias/gaps documentados (correlação/cache/build-identity/denominadores/SC, cenários bloqueados/inconclusivos não exercitados) — suporte a T184-PROG/T226-PROG sem fechar residuals UI nativos.
- ROADMAP alignment: **CONVERGED / Suportado (54/69 tasks)** — hooks de teste via ADB.

## Gate results

- Host/script/contratos (Spec 012): **[verified in prior cycles / parcial]** — suíte de contratos host registrada como verde (ex.: 86 passed em registro de task T048/T045 contextos) e checks `check→apply→check` de overlay exercitados (T051) em cenário documentado; **não** constitui atestação completa de todos os itens de verificação opt-in.
- Gates 1–8 (workspace): **[not re-run in this pass]** — sem nova execução de `./scripts/verify.sh` neste registro; alterações relacionadas foram validadas por checks direcionados em ciclos anteriores conforme tasks.
- Gate 9 (Android build): **[not re-run in this pass]**.
- FR-010/SC-004 (release exposure, `mode=full`): **pendente** (T050 opt-in) — sem evidência D1–D4 vinculada a artefato release + instalação verificada neste registro.
- Validação on-device completa de automação (T049/T067 wiring): **pendente/parcial** — evidência programática exercitada, porém com lacunas/anomalias e cenários não cobertos explicitamente rastreados.

## Non-Changes

- Nenhuma modificação de código, execução de gates, formatters, builds ou testes realizada neste registro de convergência (edit-only).
- Sem expansão de escopo além do consolidado em `spec.md`/`plan.md`/`tasks.md`, contratos e estado ROADMAP.

## Unresolved / Deferred

- **T049 / T067 / T018 / T024 (device evidence wiring & execution)**: completar classificação FR-008 em execuções reais (blocked/inconclusive/busy/cold-start/corroboration), agregação por operação, retenção/correlação/build-identity e fechamento de anomalias registradas; manter `full_parent_completion_claimed=false` para cobertura via hook.
- **T050 (FR-010/SC-004)**: executar `check-debug-ingest-release-exposure.sh --mode full` com insumos exigidos; resultados parciais/bloqueados não satisfazem FR-010/SC-004.
- **T013, T015–T017, T019, T020, T022–T023, T025, T047, T048** (conforme ledger): hardening/verificação incremental remanescente no escopo de harness/testes e atestação formal de subconjuntos relevantes.
- **FR-012 residuals em T184/T226**: casos nativos SAF picker cancel, TalkBack/teclado/travessia e outros fluxos UI/hardware permanecem rastreados nas specs 009/010 (hook fornece apenas evidência programática não-UI).

## Acceptance Criteria Status

- [x] Seam debug-only roteando ingestão por caminho de produção com contratos/resultados parseáveis e infraestrutura de contratos/scripts em host [verified in prior cycles].
- [x] Durabilidade/materialização de overlays canônicos (FR-011) e rastreio residual T184/T226 separado (FR-012) estabelecidos [verified in prior cycles, com checks exercitados].
- [x] Evidência programática não-UI parcial em dispositivo registrada via hook (apoio a T184-PROG/T226-PROG) — **suportado, não exaustivo**.
- [ ] Comprovação `mode=full` de ausência de exposição em release (FR-010/SC-004) com identidades vinculadas — **pendente** (T050).
- [ ] Cobertura/atestação completa de cenários de dispositivo, wiring FR-008 em todas as execuções relevantes e fechamento de itens de verificação opt-in — **pendente** (T049/T067 e correlatos).
