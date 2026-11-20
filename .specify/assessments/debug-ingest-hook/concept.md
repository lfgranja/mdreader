# Concept: Debug-Only Ingest Validation Hook

- **Slug**: debug-ingest-hook
- **Created**: 2026-11-20
- **Recommended option**: Option A

## Options

### Option A — Harden the existing debug-only seam through the production ingest path
- **Sketch**: Keep a debug/test build-variant entry (`DebugIngestActivity`/`DebugIngestHook`, `app.mdreader.action.DEBUG_INGEST`) that accepts a fixture reference, validates input, and calls the same production `DocIngest` snapshot/fingerprint + JNI callbacks; emit `ok:*|error:*|refused:*` with FR-008 evidence; store canonical overlay sources under `specs/012-debug-ingest-hook/android-overlays/app/src/` with idempotent fail-closed apply/check; verify release non-exposure with static + runtime + repeatable checks; report T184/T226 programmatic vs residual separately.
- **Appetite**: small-to-medium hardening of an already implemented seam
- **Trade-offs**: Gives repeatable non-UI evidence without weakening release or overstating coverage. Cost is evidence plumbing, overlay durability and release-exposure verification.
- **Rabbit holes**: treating `src-tauri/gen/**` as canonical, `static`/`runtime`-only FR-010 claims, catch-all error variants, dropping native picker/a11y residuals from reports, silent pass on missing evidence.

### Option B — Fake test-only ingest path
- **Sketch**: Load fixtures directly in a test harness bypassing production ingest.
- **Appetite**: small
- **Trade-offs**: Proves nothing about the production route; rejected by FR-004/FR-005.

### Option C — Manual-only SAF UI validation
- **Sketch**: No seam; keep clicking through the picker.
- **Appetite**: none for tooling
- **Trade-offs**: Fragile, non-deterministic, expensive; rejected.

## Recommendation

Option A. It is already partially implemented and exercised on device; remaining work is bounded hardening and verification, with explicit boundaries against overclaim.

## Out of Scope (recommended option)

- Release exposure, new engine/ingest authority, SAF picker automation, a11y certification, gate 9 claims, full T184/T226 completion from hook-only evidence.

## Assumptions to Validate

- Overlay version detection must be fail-closed (CHK024) — spec requires; apply/check exercised in T051.
- `mode=full` release evidence needs an installable release artifact and bound identities (T050) — environment-gated, never substituted by partial modes.
