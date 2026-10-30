# Implementation Plan: SONA Complete Design — Real and Exact as Proposed

**Branch**: `007-sona-complete-design` | **Date**: 2026-10-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/007-sona-complete-design/spec.md`

## Summary

Implement the complete SONA design exactly as proposed (`DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png`) on the single StatusView surface: decorative wave ribbons (teal→jade→violet), glassmorphism title card, reactive RMS waveform visualizer (canvas, smooth mirrored ribbons with integrated playhead), numeric rate in the speed-control area, repeat control, and glow/neon accents. Builds on 005/006 and exercises the constitution v2.0.0 amendments (design-token expansion for jade/celeste + supplementary visual elements). Rust owns RMS instrumentation + throttled IPC; Kotlin MediaMirror stays a declarative mirror (no new logic); the frontend renders canvas. Contracts are amended per §VII for token sanctioned scopes and the new inventory element.

## Technical Context

**Language/Version**: Rust (edition 2024, MSRV 1.85, tracks `stable`), Kotlin (android-shell, Tauri 2.12.1), HTML/CSS/JS (status_view.html surface of record)

**Primary Dependencies**: tauri 2.12.1 (`tauri::ipc::Channel`), cpal 0.18.2, `crates/player` (SampleQueue, CpalBackend, PositionClock), `crates/session`, serde, tracing

**Storage**: N/A for this feature — RMS is an in-memory `AtomicF32` (no persistence); position/bookmarks remain in `crates/session` (redb)

**Testing**: cargo nextest (unit + integration), host census (chroma C-H1, surface-audit C-H4/C-H5/C-H6, byte-identity C-H8), device trials (gate 9, on-device 10/10)

**Target Platform**: Android (primary, API 29+), desktop webview (dev/CI)

**Project Type**: Tauri mobile+desktop app (modular Cargo workspace)

**Performance Goals**: canvas ≤30 fps (33.3 ms gate); RMS IPC ~20 Hz (50 ms); no allocation on audio hot path; pause redraw on `visibilitychange`; freeze under reduced motion

**Constraints**: ≤1 continuous CSS/JS loop (S5, continuous set = {indeterminate pill}); reduced-motion freeze (not hide); notification accent-only (`#F2F0E6`); value-blind except painted numeric rate in speed-control area; surface count = 2; no scripts/`foreignObject`; glassmorphism fallback to solid `--sona-stage`; glow on 2 elements max; jade/celeste painted only in sanctioned scopes

**Scale/Scope**: single StatusView surface; element inventory 9→10 (canvas added); token set 10→12+ (jade, celeste promoted); repeat control added to transport grid

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle II (Rust owns playback)**: PASS — RMS computed in CPAL callback in `crates/player`; frontend only visualizes. No playback logic in Kotlin.
- **Principle III (Closed sets)**: PASS with amendment — token set expands 10→12+ per §VII (coordinated edit + justification + census update); rate ladder, transport, focus outcomes unchanged; new element (canvas) requires inventory update.
- **Principle V (Deterministic gates)**: PASS — host census (chroma, surface-audit) extended, not weakened; C-H1 amended to sanctioned-scopes, not deleted.
- **Principle VII (Closed Sets, No Catch-Alls)**: PASS with amendment — design-token expansion clause exercised; sanctioned paint scopes defined; no `Other`/`Unknown` arms.
- **Principle VIII (Driving-First, NON-NEGOTIABLE)**: PASS — all new visuals are supplementary (non-required); required interactions unchanged; notification remains accent-only.
- **Principle IX (Pure-offline)**: PASS — no external fonts/assets/network; SVG/CSS only.
- **SC-035 (shell purity)**: PASS — no new Kotlin playback logic; MediaMirror stays declarative mirror.
- **No secrets in repo**: PASS — no new secrets.

**Violations requiring justification**: None. Token expansion and supplementary visuals are explicitly permitted by constitution v2.0.0 amendments.

## Project Structure

### Documentation (this feature)

```text
specs/007-sona-complete-design/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── status-surface.md   # amended (S3, S4, S8, S9, S10, S11)
│   └── surface-audit.md    # amended (C-H1 scoped, C-H4/C-H5 inventory)
├── spec.md
└── checklists/requirements.md
```

### Source Code (repository root)

```text
crates/player/src/
├── backend.rs           # CpalBackend; add AtomicF32 RMS in consume_into path
├── earcon.rs            # unchanged (reuse SkipBack for repeat)
└── lib.rs

src-tauri/src/
├── status_view.html     # surface of record: add canvas, repeat button, glassmorphism, glow, alpha overlays
├── status_view.rs       # StatusView mirror; no new aria fields (HTML-side)
└── lib.rs               # mirror publish channel; add throttled RMS channel

android-shell/src/main/kotlin/app/mdreader/
└── MediaMirror.kt       # unchanged (declarative mirror; accent-only)

DESIGN-SONA/
├── tokens.md            # version bump + sanctioned scope notes
└── assets/              # static SVGs (waves, logo)
```

**Structure Decision**: Existing modular Cargo workspace + Tauri app. No new crates. Changes are localized to `crates/player` (RMS instrumentation), `src-tauri` (surface HTML + mirror channel), and contract/docs. Android shell untouched (SC-035 purity).

## Complexity Tracking

No violations requiring justification — all changes operate within amended constitution clauses (token expansion §VII, supplementary visuals §VIII).
