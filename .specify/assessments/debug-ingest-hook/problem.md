# Problem Definition: Debug-Only Ingest Validation Hook

- **Slug**: debug-ingest-hook
- **Created**: 2026-11-20
- **Inputs used**: intake.md | research.md

## Problem Statement

Deterministic validation of Spec 009 T184 and Spec 010 T226 ingest behavior is blocked by the native SAF picker UI, and existing programmatic results are hard to reproduce or trust: they lack consistent FR-008 evidence, durable debug artifact placement across `src-tauri/gen` regeneration, combined proof of release non-exposure, and a strict split between covered non-UI paths and residual native picker/a11y cases.

## Affected Users & Stakeholders

- **Users**: maintainers/QA running ingest validation campaigns on device/emulator.
- **Stakeholders**: Android/shell maintainer, security/release owner (FR-010), evidence reviewer (FR-008/FR-012).

## Goals

- Provide a debug/test-only automation entry routing through the production `DocIngest`/JNI path (FR-001..FR-005).
- Emit explicit parseable outcomes and authoritative FR-008 evidence for every run (FR-006..FR-008).
- Keep the entry absent/inert in release and durable across generated-project refresh (FR-010..FR-011, FR-013).
- Split T184/T226 reporting into programmatic coverage and residual native UI/a11y cases (FR-012).

## Non-Goals

- No release exposure, no fake/parallel ingest path, no playback/position/rate authority change.
- No SAF picker automation, accessibility certification, gate 9 release claim, or full T184/T226 completion from hook evidence alone.

## Success Metrics

- SC-001..006: runs execute without human picker interaction; ≥95% explicit results within 2 min; T184/T226 summaries declare `full_parent_completion_claimed=false`; release exposure records zero externally invocable entries with `mode=full` D1–D4; host gates remain green; overlay apply/check 100% visible.

## Cost of Inaction

T184/T226 remain manual, fragile and easy to overclaim; debug artifacts silently disappear after regeneration and release exposure is unproven.

## Open Questions

- FR-010 `mode=full` evidence (T050) and full FR-008 device wiring (T049/T067) remain tracked verification tasks, not design blockers.
