# Data Model: SONA Complete Design (007)

**Feature**: `007-sona-complete-design` | **Source**: [spec.md](spec.md) | **Extends**: `specs/005-adopt-design-sona/data-model.md`

Closed-set discipline (constitution §VII): every table here is a closed set; adding/removing a member is a coordinated edit with version bump and docs.

## SonaToken (expanded — 12+ members)

Original 10 (005) + jade + celeste promoted to painted with sanctioned scopes (per C1 Option A).

| Token | Hex | Role (paint span) | 007 paint scope |
|---|---|---|---|
| `--sona-void` | `#0B0F1A` | page background | unchanged |
| `--sona-stage` | `#141B2D` | raised surface (title card, canvas backdrop) | unchanged |
| `--sona-voice` | `#F2F0E6` | primary text + progress fill + notification accent | unchanged (accent-only notification) |
| `--sona-etwahl` | `#8FD8E8` | focus ring only (enumerated controls) + glow on header mark | unchanged |
| `--sona-hush` | `#A9B8CE` | secondary text (section, metadata) | unchanged |
| `--sona-harp` | ledger value | positional rate dot + glow on waveform baseline | unchanged (rate stays gold) |
| `--sona-celeste` | `#B9A6F5` | **PROMOTED to painted** (was reserved) | waveform-ribbon violet layers + decorative-wave violet stops ONLY |
| `--sona-boundary` | `#55688F` | progress track + canvas axis | unchanged |
| `--signal-caution` | `#FFD400` | 3 s pause warning only | unchanged |
| `--signal-failure` | `#FF4D63` | permanent failure only | unchanged |
| `--sona-aria` (jade) | `#5AD6A8` | **PROMOTED to painted** (was brand-only) | waveform-ribbon middle layers + brand-layer assets ONLY |

- **Banned paints remain**: jade `#5FD0A0` erratum variant never; celeste/jade outside sanctioned scopes = C-H1 violation.
- Glassmorphism uses translucent overlays (rgba/rgb alpha) of existing tokens; base colors verbatim SonaToken members.

## SurfaceInventory (in-app driving surface — 9 → 10 elements)

| Element | Binding | Status |
|---|---|---|
| Header mark | Concha Harmônica "R" SVG | restyle + glow (FR-010) |
| Metadata card | glassmorphism title card (title/section) | NEW styling (FR-009) |
| **Waveform canvas** | `<canvas id="waveform" aria-hidden="true">` smooth ribbons + integrated playhead | **NEW (007)** |
| Progress (hidden) | `#progress` sr-only `role="progressbar"` + live region | retained for aria (C-H5) |
| Transport controls | prev/play/next + **repeat** + stop | repeat NEW (FR-011) |
| Rate ladder | positional dot `--sona-harp`, 7 rungs, aria value only | numeric rate painted in speed area (FR-003) |
| Reason line | 3 s expiry + signal colors | unchanged |
| Resume offer | 004 contract | unchanged |
| Focus ring | `:focus-visible` `--sona-etwahl` | unchanged |
| Live region | visually hidden `role="status"` | now populated (C5) |

- Waveform canvas counts as a new inventory element → C-H4/C-H5 contract update (S11).
- Repeat control counts as a new interactive element → C-H4 update + earcon reuse.

## WaveformVisualizer

| Field | Type | Rule |
|---|---|---|
| `rms` | `AtomicF32` | lock-free store updated in CPAL callback (`consume_into`); no allocation on hot path |
| `layers` | 4–6 translucent bezier curves | teal→jade→violet, mirrored around center axis |
| `playhead` | white center axis + circular knob | drawn from progress percentage; none in indeterminate state |
| `aria` | `aria-hidden="true"` | decorative; progress exposed via hidden `#progress` |
| `cadence` | IPC-event-driven | ~20 Hz RMS IPC; 30 fps cap; pause on `visibilitychange`; freeze under reduced motion |
| `dpr` | `min(devicePixelRatio, 2)` | backing store cap |
| `fallback` | static SVG/CSS placeholder | if RMS/canvas unavailable |

## RepeatControl

| Field | Type | Rule |
|---|---|---|
| `action` | restart current section | reuses skip/section logic |
| `earcon` | `EarconKind::SkipBack` | reuse Allegro-trás (880→660 Hz, 40 ms); no new variant |
| `haptic` | id 4 (SkipBack) | existing haptic pattern |
| `inventory` | new interactive element | C-H4 update required |

## RMS channel (IPC)

| Field | Type | Rule |
|---|---|---|
| `channel` | `tauri::ipc::Channel` (dedicated) | ~20 Hz throttled; skip publishes when unchanged/backgrounded |
| `payload` | `f32` (latest RMS) | single value; frontend redraws on receipt |
| `source` | `AtomicF32` shared with CpalBackend | lock-free read |

## State transitions (additions)

- Waveform redraw: IPC event → canvas redraw (no rAF loop); reduced motion → static snapshot; `visibilitychange` hidden → pause; visible → one fresh frame.
- Progress: determinate → playhead knob drawn; indeterminate → no knob; omit `aria-valuenow`; announce "carregando" once.
- Repeat: press → restart section + SkipBack earcon + haptic id 4.

## Validation rules

- C-H1 (chroma): jade/celeste paint only in sanctioned scopes; banned elsewhere.
- C-H4 (inventory): 10 elements; no `display:none` corpses.
- C-H5 (aria): canvas `aria-hidden`; `#progress` progressbar + live region; no painted numeric rate except speed area.
- C-H6 (movement): ≤1 continuous loop (pill only); no new `@keyframes` for waveform.
- C-H8 (byte-identity): served HTML = `status_view.html`.
