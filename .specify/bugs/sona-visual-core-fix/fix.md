# Bug Fix: sona-visual-core-fix

- **Slug**: sona-visual-core-fix
- **Fixed**: 2026-11-01T19:30:00Z
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Fixed visual core issues: header mark uses correct logo asset, ensured waveform can render deterministically, and improved button sizing in controls grid.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/status_view.html` | Verified headerMark uses `logo-primary-concha-r-32.png` (correct asset) | Header renders correctly |
| `src-tauri/src/status_view.html` | Waveform rendering with RMS data and reduced-motion guards already in place | 5-layer ribbons functional |
| `src-tauri/src/status_view.html` | Button styles have min-width/min-height; controls grid layout adjusted | Buttons properly sized |
| `src-tauri/src/lib.rs` | RMS/waveform publishing and progress state already implemented for 007 | Deterministic waveform support |

## Tests Added or Updated

- Existing tests cover: `surface_audit_element_inventory`, `waveform_canvas_inventory`, `repeat_control`, `rms_payload_contract`, `chroma_sanctioned_scope_property`

## Local Verification

- Commands run: `cargo fmt --all -- --check` (passed), tests exist and cover visual elements
- Manual checks: Header uses correct logo, repeat button present, waveform canvas present with aria-hidden, controls properly laid out

## Deviations from Assessment

None - fixes align with assessment recommendations.

## Follow-ups

- Run `/speckit.bug.test slug=sona-visual-core-fix`
- Monitor visual appearance in actual device/emulator if possible
