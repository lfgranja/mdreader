# Validation (R4): 009 Abrir Arquivo Pelo Título e Iniciar Reprodução

- **Date**: 2026-11-20
- **Status**: PARTIAL PASS (host verified; native SAF UI/a11y residuals open; live CI/PR [NV])

| Check | Command / evidence | Result |
|---|---|---|
| Spec quality | FR-001..FR-017, SC-001..004; clarification (`.meta-box` full-area affordance) recorded | PASS |
| TDD RED/GREEN | US1 tests T162/T163 before T164–T167; US2 T168/T169 before T170/T171; US3 T172/T173 before T174–T176 | PASS (prior cycles) |
| Host gates | T177 fmt, T178 nextest, T179 census, T180 clippy, T181 purity scans, T182 `./scripts/verify.sh` | PASS (verified in prior cycles; not re-run here) |
| Ingest authority | `ACTION_OPEN_DOCUMENT` → `DocIngest.pickDocument` → JNI → `open_document_bytes_named`; Blake3 identity; `offerDialog` 3 s | PASS (T158–T171) |
| Refusal isolation | `FormatUndeliverable`/`InvalidUtf8`, cancel preserves transport, `MAX_DOC_BYTES` 93.636 | PASS (T172–T176) |
| Programmatic evidence | T184-PROG via Spec 012 hook: non-UI `executed-pass`, `full_parent_completion_claimed=false` | PARTIAL (non-UI only) |
| Native UI/a11y | T184 full on-device scenarios 1–4; T184-RES-SAF-CANCEL; T184-RES-A11Y | OPEN (residual) |
| Task-ID uniqueness | suffixed T184-* subtasks de-checkboxed (kept as bullets) to resolve duplicate T184 IDs | PASS (edit-only) |

## Success criteria

- SC-001 ≤2 touches; SC-002 <1 s to speech; SC-003 100% cancel isolation; SC-004 100% resume-offer for deliberate pauses — verified for non-UI paths in prior cycles.
- Full SC-003/SC-004 UI demonstration remains tied to T184 residuals.

## [NV]

- Native SAF cancel/back and TalkBack/keyboard traversal on device; live CI/PR merge. No gate/test executed by this artifact.
