# ADR-002: §VII Diagnostic-Kind Extension (EarconTruncation / EarconEmissionFailure)

- **Status**: Coordinated / planned (release-level; not yet released)
- **Date**: 2026-10-26
- **Feature**: `005-adopt-design-sona`
- **Branch**: `005-adopt-design-sona`
- **Author**: Design-dir / spec-level (coordinated by T012)
- **Constitution reference**: `.specify/memory/constitution.md` §VII (Closed sets, no catch-alls); plan.md l17 / l41; data-model.md §DiagnosticEvent (l89–96)

## Context

The burst-substitute policy (`latest wins`, `short_mix` single-channel) and the emission-failure path (synthesis error / channel unavailable) require typed diagnostic records, not silent branches. The existing `EventKind` (9 members, `crates/diag/src/writer.rs`, `contract_schema`) has no branch for truncation or emission failure.

Per constitution §VII, any new member in a closed set must be:
1. Declared explicitly (no `Other` / `Unknown` / `_ =>`)
2. Coordinated across all consuming surfaces (writer, scan, contract schema, docs)
3. Documented as a §VII change at release (not a silent edit)

`data-model.md` §DiagnosticEvent defines exactly two additions: `EarconTruncation` (truncation counter) and `EarconEmissionFailure` (typed emission-failure with required `error` field). `tasks.md` T012 defines the coordinated edit (writer.rs + scan.rs + schema 9→11 + semver + docs).

The feature is defensible ONLY if this amendment is visible in the release artifact (docs / version / changelog), not buried in source.

## Decision

**Adopt the coordinated §VII extension with explicit release-level documentation.**

The two new kinds are sanctioned; the rule is: one truncation → one `EarconTruncation`; one failure → one `EarconEmissionFailure` (with `error`); never more than one re-emission; never a silent `Other`; the `short_mix` channel stays single (E2.5) — no second channel introduced.

No new engine trait / SDK identifier is added; the extension is player-side diagnostic only (VI engine-neutral).

The amendment artifact is this ADR + the coordinated source edits (T012) + `crates/diag/CHANGELOG.md` entry + `VERSION` tag update (`design-sona-v2.2.0` + diag-release note) — never a silent source-only change.

## Consequences

- **Positive**: Closed-set discipline preserved; burst/failure policies are observable; audit trail exists; constitution §VII satisfied explicitly.
- **Risk if omitted**: A release without this ADR + version note makes the §VII claim unverified — a future auditor could treat the kind additions as a silent branch (violating MUST — no catch-all, no silent deletion/omission of coordination).
- **Mitigation**: T012 must carry the docs edit; T027 (`verify.sh` wire) must confirm `contract_schema` reads `exactly eleven`; T031 (`STATE.md` resume) must record the amendment.

## References

- `specs/005-adopt-design-sona/tasks.md` T012 (diag extension), T027 (verify), T031 (attestation)
- `specs/005-adopt-design-sona/data-model.md` §DiagnosticEvent (l89–96)
- `specs/005-adopt-design-sona/contracts/earcon-mapping.md` E3 (system interruption), E4 (failure / emission-failure)
- `.specify/memory/constitution.md` §VII (Closed sets; coordinated edition; no catch-all)
- `crates/diag/src/writer.rs` (current 9-kind `EventKind`); `crates/diag/src/scan.rs`; `crates/diag/contract_schema`
- `specs/005-adopt-design-sona/plan.md` l17 (coordinated extension claim) — now cross-ref to this ADR
- `specs/005-adopt-design-sona/research/clarify-e3-sc002-haptic-side-channel.md` (related side-channel disposition; not this extension)

## Verification (only Phase 8 attests; this ADR is documentation only)

- Host check: `crates/diag/src/writer.rs` + `scan.rs` must contain exactly 11 named kinds, zero `_ =>` / `Unknown` / `Other`; `contract_schema` document updated; `cargo deny` / `cargo audit` pass; `verify.sh` exits 0.
- Device check: not applicable (diag is host-side typed record; no device trial for this kind set; device legs covered by SC-002/SC-003/SC-006/SC-007 separately).
