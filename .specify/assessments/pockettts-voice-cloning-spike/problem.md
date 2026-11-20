# Problem Definition: PocketTTS Voice Cloning Spike (020)

- **Slug**: pockettts-voice-cloning-spike
- **Created**: 2026-11-15 (artifact backfilled 2026-11-20)
- **Inputs used**: intake.md | research.md

## Problem Statement

It is unknown whether an offline cloned pt-BR voice (Ressoa) can meet Ressoa's TTFA/RTF/memory bar on the reference low-end device (`2201117TY`, SD680). Integrating a neural cloning model without a device-first gate risks regressing the core offline reading latency (1551 ms attested baseline, 3000 ms bound) and introducing packaging/licensing exposure. A decision must be made from measured evidence, not from host benchmarks or vendor claims.

## Affected Users & Stakeholders

- **Users**: low-end Android readers whose first-audio latency must stay competitive.
- **Stakeholders**: maintainer (TTFA/RTF/memory contracts, adapter architecture), reviewers/legal (gated weights, reference voice consent), product (Ressoa voice identity).

## Goals

- Produce a reproducible on-device measurement matrix with the canonical Ressoa reference and PocketTTS pt bundle (FR-PKT-001..004, 011).
- Apply an explicit go/no-go gate against 1551 ms / 3000 ms, `RTF < 1`, RSS/stability (FR-PKT-005).
- Keep production contracts, permissions and Piper default unchanged (FR-PKT-007/009).
- Record the decision, caveats, fallbacks and license preconditions (FR-PKT-006/010, SC-PKT-003/004/006).

## Non-Goals

- No shipping/bundling a cloned voice; no user-facing cloning; no cloud inference.
- No modification of the production adapter, handoff format or closed `VoiceId` vocabulary within the spike.
- No claim of end-to-end `play_to_audio_ms` from synthesis-only measurements.
- No distribution of gated weights or third-party voice audio without explicit rights.

## Success Metrics

- SC-PKT-001: complete, reproducible matrix in `measurements.md`.
- SC-PKT-002/003: gate applied and decision consistent with evidence (recorded NO-GO).
- SC-PKT-004: synthesis-vs-playback metric distinction explicit.
- SC-PKT-005: production contracts/permissions unchanged.
- SC-PKT-006: fallbacks and license/consent preconditions recorded.

## Cost of Inaction

Without the spike, a slow cloning model could be integrated on vendor promises, directly harming first-audio latency on the reference device; or a viable voice option could be dismissed without evidence.

## Open Questions

- Would a compatible quantized bundle/runtime on stronger hardware change the gate? [NV]
- Can license/consent for gated weights and the reference voice ever be cleared? Unresolved.
