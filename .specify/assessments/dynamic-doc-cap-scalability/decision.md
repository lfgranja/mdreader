# Decision: dynamic-doc-cap-scalability

- **Slug**: `dynamic-doc-cap-scalability`
- **Date**: 2026-11-11
- **Verdict**: `go`

## 1. Decision Rationale
1. **High Impact / Low Effort (80/20)**: Directly eliminates the primary runtime failure mode for real-world long-form documents.
2. **Zero Architecture Drift**: Does not alter playback authority, audio handoff format (mono f32 @ 22050 Hz), or closed sets.
3. **Safety & Containment**: 1 MiB is safe for mobile RAM and within Android IPC / Binder limits for filesystem-backed descriptors.

## 2. Transition
Promoted directly into Spec 020 (`specs/020-dynamic-doc-cap-scalability/`).
