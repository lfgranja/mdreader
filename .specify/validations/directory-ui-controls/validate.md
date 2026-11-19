# Validation (R4): 023 Directory UI Controls

- **Date**: 2026-11-18
- **Status**: PASS (local deterministic; PR/CI [NV])

| Check | Command | Result |
|---|---|---|
| Spec quality | checklist all items | PASS |
| TDD RED | nextest target (before implementation) | 2 failed / 1 passed / 2 skipped |
| TDD GREEN | same target after implementation | 5/5 PASS |
| Host gates | `PROPTEST_CASES=32 bash scripts/verify.sh` | `VERIFY_EXIT=0` |
| Census | live 137 vs expected 137 | diff empty |
| Bundle parity | sha256 of 5 HTML files | all `dcefad7e…` |
| Surface invariants | C-H4 corpse check, role=status count, data-elements, pinned aria labels | unchanged |
| Rust authority | `git diff --name-only` | no `.rs` bridge changes |

## Success criteria

- SC-DUI-001: structural tests + host gates PASS (measured).
- SC-DUI-002: five bundles byte-identical (measured).
- SC-DUI-003: spec 010 T229/T230/T234 closed; T232/T233 open (measured).

## [NV]

- Device visual/interaction; live CI/PR merge.
