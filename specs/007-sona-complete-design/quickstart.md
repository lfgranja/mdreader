# Quickstart: SONA Complete Design (007) — Validation Guide

**Feature**: `007-sona-complete-design` | **Date**: 2026-10-28

## Prerequisites

- Rust 1.85+ (edition 2024), `cargo-nextest`, Android SDK + API-29+ device (for on-device legs).
- Workspace builds: `cargo build --workspace`.
- Feature branch `007-sona-complete-design` checked out.

## Host-checkable (no device)

1. **Format/lint**: `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
2. **Unit tests**: `cargo nextest run --workspace`.
3. **Doc tests**: `cargo test --doc`.
4. **Supply chain**: `cargo deny check advisories licenses bans sources && cargo audit`.
5. **Hex census (C-H1)**: run chroma audit over `src-tauri/src/status_view.html`, `status_view.rs`, `MediaMirror.kt`, `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg`; verify celeste/jade appear only in sanctioned scopes (waveform ribbon draw code + decorative wave CSS/SVG).
6. **Surface audit (C-H4/C-H5/C-H6)**: run surface-audit checks; verify 10-element inventory, aria inventory, ≤1 continuous loop, reduced-motion guard.
7. **Byte-identity (C-H8)**: served bundle HTML == `src-tauri/src/status_view.html`.

## On-device (gate 9)

1. Build: `cargo tauri android build --target aarch64`.
2. Install on device; open app with a document loaded.
3. Verify all proposal elements present: wave ribbons (teal→jade→violet), glassmorphism card, waveform visualizer (reactive ribbons + playhead), numeric rate `1.25×` in speed area, repeat button, glow on logo + waveform baseline.
4. Verify reduced-motion: enable "Remove animations" (Android) → decorative animations freeze; no new loops.
5. Verify empty title state: `—` glyph + `aria-label="Sem título"`.
6. Verify notification purity: lock screen shows only accent `#F2F0E6`, no body/rate/voice/mark.
7. Verify repeat: press repeat → current section restarts; earcon = skip-back sound.
8. Verify waveform reacts to playback (RMS-driven ribbons move).
9. Verify glassmorphism fallback on unsupported webview (solid `--sona-stage`).
10. Verify canvas pauses on background (`visibilitychange`); freezes under reduced motion.

## Validation scenarios (acceptance)

- **SC-001**: All proposal elements present (visual inspection + census).
- **SC-002**: No accessibility regressions (progressbar aria, empty slot aria-label, focus ring, no alert misuse).
- **SC-003**: Notification accent-only.
- **SC-004**: Reduced-motion respected.
- **SC-005**: Surface inventory valid; no dead corpses; new elements documented.
- **SC-006**: Device trial matches proposal.

## References

- Spec: [spec.md](spec.md)
- Plan: [plan.md](plan.md)
- Research: [research.md](research.md)
- Data model: [data-model.md](data-model.md)
- Contracts: [contracts/status-surface.md](contracts/status-surface.md), [contracts/surface-audit.md](contracts/surface-audit.md)
- Clarification docs: `clarification-*.md` in this directory
