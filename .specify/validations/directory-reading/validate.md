# Validation (R4): 010 Leitura Contínua de Diretórios de Markdown

- **Date**: 2026-11-20
- **Status**: PASS (host + physical-device attestation in prior cycles; live CI/PR [NV])

| Check | Command / evidence | Result |
|---|---|---|
| Spec quality | FR-001..FR-013, SC-001..004; assessed under `directory-reading` assessment (go) | PASS |
| TDD RED/GREEN | US1 T195–T197 before T198–T203; US2 T204/T205 before T206–T210; US3 T211–T213 before T214–T218 | PASS (prior cycles) |
| Host gates | T219 fmt, T220 nextest, T221 census, T222 clippy, T223 purity, T224 `cargo deny/audit`, T225 `./scripts/verify.sh` | PASS (verified in prior cycles; not re-run here) |
| Directory contract | SAF `ACTION_OPEN_DOCUMENT_TREE`, natural sort, `ChapterQueue`, gapless advance, `DIRECTORY_BOOKMARKS` composite Blake3, 20 s heartbeat | PASS (T185–T218) |
| Hardware E2E | T226 on Xiaomi Redmi Note 11 (2201117TY, SDK 33, wireless ADB `adb-53cf9aa7`): SAF tree, TTFA 1,551 ms (<3 s), transitions, resume, contextual skip, invalid entries, a11y | PASS (1,551 ms measured) |
| Programmatic evidence | T226-PROG via Spec 012 hook: 3 chapters ingested, first auto-opened | PASS (non-UI complement) |
| Residual subtasks | T226-RES-* (SAF tree/transition/resume/hardware/invalid/a11y) verified on hardware | PASS |
| Task-ID uniqueness | suffixed T226-* subtasks de-checkboxed (kept as bullets) to resolve duplicate T226 IDs | PASS (edit-only) |

## Success criteria

- SC-001 100% automatic N→N+1 transition; SC-002 TTFA <3 s (1.551 s measured); SC-003 100% resume precision; SC-004 <1 s silence gap — attested in prior cycles.
- 49/49 tasks checked; ROADMAP §2.2 records CONVERGED (hardware attested).

## [NV]

- Fresh gate run and live CI/PR merge; no gate/test executed by this artifact.
