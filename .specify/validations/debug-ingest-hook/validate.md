# Validation (R4): 012 Debug-Only Ingest Validation Hook

- **Date**: 2026-11-20
- **Status**: PARTIAL PASS (host contracts/overlays green; FR-010 `mode=full` and full device wiring open; [NV] where noted)

| Check | Command / evidence | Result |
|---|---|---|
| Spec quality | FR-001..FR-013, NFR-012-MAINT-001..005, SC-001..006; clarifications recorded in `spec.md` | PASS |
| TDD RED/GREEN | Host contract scaffolds T001–T012/T014/T021/T026–T044 RED→GREEN in prior cycles | PASS (host) |
| Production path | FR-004/FR-005 route via `DocIngest` snapshot/fingerprint + `onDocumentIngested`/`onDirectorySelected` JNI callbacks | PASS (no fake path) |
| Result grammar / evidence | FR-006/FR-008 `ok:*|error:*|refused:*`, FR-008 minimum fields, channel precedence/corroboration | PARTIAL (wiring gaps T049/T067) |
| Overlay durability | FR-011 canonical root `android-overlays/app/src/`, version-detectable apply/check; T051 exercised check→apply→check | PASS (host) |
| Residual split | FR-012 T184-PROG/T226-PROG non-UI evidence with `full_parent_completion_claimed=false`; native SAF/a11y residuals separately tracked | PASS (reporting contract) |
| Release exposure | FR-010/SC-004 D1–D4 with bound artifact identity, `mode=full` (T050) | OPEN (partial modes do not pass) |
| Host contract suite | ~86 passed in recorded T048/T045 contexts; gates 1–8 not re-run here | PASS (prior cycles) |
| Ledger | 54/69 tasks checked; ROADMAP §2.2 CONVERGED / Suportado | PARTIAL |

## Success criteria

- SC-001 automation without picker; SC-002 ≥95% explicit results ≤2 min — design met, device denominators incomplete.
- SC-003 residual-split reporting — PASS; SC-004 release exposure — [NV] (T050 pending); SC-005 host gates — prior-cycle evidence; SC-006 overlay durability — PASS (T051).

## [NV]

- `mode=full` release-exposure evidence, complete FR-008 device-run classification (cold-start/busy/corroboration), and live CI/PR merge. No gate/test executed by this artifact.
