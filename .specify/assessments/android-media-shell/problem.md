# Problem Definition: Android Media Shell

- **Slug**: android-media-shell
- **Created**: 2026-11-20 (backfilled)
- **Inputs used**: intake.md | research.md

## Problem Statement

There is no playback substrate for the driving-first reader: without a Rust-authority PCM shell and a non-authoritative Kotlin MediaSession mirror, position/buffering have no single owner, focus interruptions have no mandated duck/pause mapping, and locked-screen control, process-death recovery and device-removal pause are undefined. Every downstream "10 out of 10" criterion is therefore unmeasurable.

## Affected Users & Stakeholders

- **Users**: driver/listener using the phone locked, screen off, hands on the wheel.
- **Stakeholders**: maintainer (owns gates 1–8, reference-device trials), downstream features 002/004/005 (consume the shell contract).

## Goals

- Rust player owns position, buffering and format conversion; Kotlin is a strict mirror (FR-002/FR-003, SC-035).
- Screen-off playback with lock-screen play/pause/skip/rate; 5-minute continuity, ±5 s skip, one-step rate (FR-001/FR-018/FR-019, SC-001/SC-004/SC-016/SC-026).
- Focus classes: duck permitted, silence-demanding → pause ≤1 s, permanent → no auto-resume (FR-012–FR-014, SC-003/SC-041).
- Headphone removal pauses within 1 s via two named mechanisms (FR-016/FR-016a, SC-002).
- Process death → paused at last position with explicit resume; Stop discards position (FR-030–FR-032, SC-007/SC-011/SC-018).
- FGS manifest declaration + bootstrapping from Application/Activity incl. Android 14+ restrictions (FR-006–FR-010, SC-006/SC-014).
- Offline-only, closed sets, on-device diagnostics with zero document prose (FR-026/FR-039/FR-044/FR-045).

## Non-Goals

- Document parsing, synthesis, voice/engine implementation (FR-024 — separate spec 002).
- Android Auto (FR-025) and any network voice path (FR-026).
- Active mid-document engine fallback (`SC-044` DEFERRED); selector invariant only (`SC-051`).

## Success Metrics

- SC-001/SC-002/SC-003/SC-004: 10/10 for 5-min continuity, device-removal pause, duck gain, lock-screen transport.
- SC-006: automated FGS declaration + permission check green.
- SC-007/SC-011/SC-012: killed-service recovery pauses, never auto-resumes.
- SC-009: first audible ≤3 s (amended to ≤10 s cold-start by spec 003).
- SC-035: automated check finds zero playback logic in Android/Tauri surface.
- SC-057: every governed closed set enumerated.

## Cost of Inaction

The product has no audible path; specs 002/004/005 cannot be exercised and the driving-first promise is untestable.

## Open Questions

- Device-only trials remain [NV] until run on the reference phone by the gate owner.
- `T232/T233` (surface directory/chapter controls) and 11 other open tasks are explicit follow-ups, not blockers for this assessment.
