# Idea Research: Debug-Only Ingest Validation Hook

- **Slug**: debug-ingest-hook
- **Created**: 2026-11-20
- **Evidence confidence (overall)**: high for local code/device evidence; medium for official-external re-fetch (prior session recorded allowlisted sources, not live re-verified)

## Users & Demand

- Maintainers need deterministic, non-interactive evidence for Spec 009 T184 / Spec 010 T226 ingest behavior; manual SAF picker interaction is fragile and expensive — [source: `.specify/assessments/debug-ingest-hook-automation/problem.md`, `specs/012-debug-ingest-hook/spec.md:9`] (confidence: high)

## Prior Art

- Production ingest path already exists: `DocIngest` snapshot/fingerprint and JNI callbacks `onDocumentIngested` / `onDirectorySelected` → Rust `open_document_bytes_named` / directory manifest handling; FR-004/FR-005 require the hook to reuse it, not fork it — [source: `spec.md:114-115`, `plan.md:Summary`] (confidence: high)
- Prior device probe (T049, `adb-53cf9aa7`, 2201117TY SDK 33, debuggable `app.mdreader`) produced partial `executed-pass` non-UI results with documented anomalies (correlation, cache, build-identity, SC denominators) — [source: `specs/012-debug-ingest-hook/converge.md`, `specs/009-.../tasks.md:108`] (confidence: high)
- Earlier assessment `debug-ingest-hook-automation` returned `go`; its research/decision are the audit trail for this spec — [source: `.specify/assessments/debug-ingest-hook-automation/decision.md`] (confidence: high)

## Market & Context

- Cost of doing nothing: T184/T226 remain dependent on manual UI validation and programmatic results are hard to reproduce or trust.

## Data & Constraints

- FR-001/FR-002/FR-010: hook must be unavailable outside debug/test and absent/inert in release; combined evidence required (build-variant placement + release artifact static inspection + runtime negative invocation + repeatable check) — [source: `spec.md:111-112,120`] (confidence: high)
- FR-008 is authoritative for evidence minimum (fixture/reference, operation, parseable outcome, build+device identity, diagnostics, crash/ANR status, residual scope, evidence disposition) — [source: `spec.md:118`] (confidence: high)
- FR-011/FR-013/SC-006: canonical overlay root `specs/012-debug-ingest-hook/android-overlays/app/src/` with version-detectable idempotent apply/check; `src-tauri/gen/**` is never canonical — [source: `spec.md:121,123,151`] (confidence: high)
- FR-012/SC-003: T184/T226 summaries must distinguish programmatic coverage from native SAF picker cancel and TalkBack/keyboard residuals with `full_parent_completion_claimed=false` — [source: `spec.md:122,148`] (confidence: high)
- Ledger: 54/69 tasks complete; FR-010 `mode=full` (T050) and complete device wiring (T049/T067) remain open; no gate re-run in this assessment — [source: `converge.md`] (confidence: high)

## Evidence Against the Idea

- Debug backdoor/security risk, generated-project drift, and evidence overclaim are real; mitigations are debug-only guards, durable canonical overlays with fail-closed apply/check, and explicit split residuals. — [source: spec FR-010..FR-012, CHK002] (confidence: high)

## Gaps & Open Questions

- `mode=full` FR-010 evidence and complete FR-008 device-run wiring remain pending (T050, T049/T067); non-blocking for the go decision because the seam is already implemented and the gaps are tracked verification tasks.

## Sources

- `specs/012-debug-ingest-hook/{spec.md,plan.md,tasks.md,converge.md,contracts/*}` (host: local repo)
- `.specify/assessments/debug-ingest-hook-automation/{decision.md,research.md}` (host: local repo)
- `specs/009-open-file-from-title/tasks.md`, `specs/010-directory-reading/tasks.md` (host: local repo)
