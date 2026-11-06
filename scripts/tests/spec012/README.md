# Spec 012 contract tests

Host-side contract/invariant checks for `specs/012-debug-ingest-hook/`.

Normative references:
- `specs/012-debug-ingest-hook/contracts/debug-ingest-automation.md`
- `specs/012-debug-ingest-hook/contracts/durable-overlay-materialization.md`
- `specs/012-debug-ingest-hook/contracts/release-exposure-evidence.md`
- `specs/012-debug-ingest-hook/contracts/t184-t226-residual-tracking.md`
- `specs/012-debug-ingest-hook/spec.md` (§FR-008 source of truth)

Tests in this directory should prefer property-based/invariant cases where practical and must fail closed on ambiguous/missing required evidence.
