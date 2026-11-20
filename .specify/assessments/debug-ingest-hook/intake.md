# Idea Intake: Debug-Only Ingest Validation Hook

- **Slug**: debug-ingest-hook
- **Created**: 2026-11-20
- **Source**: `specs/012-debug-ingest-hook/spec.md` (user request: debug-only/test hook accepting URI/path and routing through the same production document-ingest/directory-open flow) + assessment handoff `.specify/assessments/debug-ingest-hook-automation/decision.md` (go, Option A)
- **Type**: validation tooling / hardening

## Idea (as captured)

> "Add a debug-only/test hook (intent/exported test activity or command protected by debug/build flag) that accepts a URI/path and routes through the same document-ingest/directory-opening production flow, plus instrumented/Espresso tests if desired." — `specs/012-debug-ingest-hook/spec.md:9`

## Restated

Harden the small debug/test-only automation seam so maintainers can validate Spec 009 T184 and Spec 010 T226 ingest behavior deterministically without touching the native SAF picker: accept a caller-supplied fixture, route through the production `DocIngest` snapshot/fingerprint + JNI callback path, emit explicit parseable outcomes and FR-008 evidence, keep the entry absent/inert in release, and keep native picker/a11y residuals explicitly tracked.

## Origin & Context

- **Raised by**: validation campaign need, 2026-11-05; prior assessment `debug-ingest-hook-automation` already returned `go` for Option A.
- **Trigger**: manual SAF interaction makes T184/T226 validation fragile; debug runs on `adb-53cf9aa7` (2201117TY, SDK 33) already produced partial non-UI evidence.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: how to keep the hook durable across `src-tauri/gen` regeneration and prove release non-exposure — resolved in spec FR-010/FR-011 via canonical overlays + fail-closed apply/check and combined static/runtime evidence]
