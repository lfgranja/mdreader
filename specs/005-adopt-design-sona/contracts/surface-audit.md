# Contract — Surface Audit (host checks for 005 acceptance)

**Feature**: `005-adopt-design-sona` | **Date**: 2026-10-24 | Constitution: gates stay deterministic and fail-closed (Principles III–V). This document is THE check specification; `verify.sh` wires each check as a named step (004-style fail-closed), and the cargo-test twins live in `scripts/expected-tests.txt` (two-way sync, T157 precedent).

## A1. Auditee list (CLOSED)

1. `src-tauri/src/status_view.html`
2. `src-tauri/src/status_view.rs` (relevant constants)
3. `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (constants block `:728-743` only)
4. `DESIGN-SONA/{tokens.md, components.md, DESIGN-SONA.md, preview.html, CHANGELOG.md, VERSION}` (post-alignment — a missing CHANGELOG/VERSION fails the alignment task's audit)
5. `src-tauri/tauri.conf.json` (S2 wiring invariant: the served surface identity)
6. `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg` (in-app header asset; hex census applies)

Files must NOT be silently added: the list itself fails if an audited location disappears (missing input = failed gate, never skip).

## A2. Checks

| id | Name | What fails it |
|---|---|---|
| C-H1 | **hex census** | any hex value outside `data-model.md` §SonaToken inside auditees; `#5AD6A8`/`#5FD0A0` paint anywhere; `#B9A6F5` paint anywhere (celeste vacancy violation); slate hexes/shipped color leftovers (`#0f172a`, `#1e293b`, off-set progress white `#FFFFFF`) |
| C-H2 | **role exclusivity** | `#8FD8E8` outside `:focus-visible`; `#FFD400` outside {banner, in-app 3 s caution dot}; `#FF4D63` outside permanent-failure; caution×failure adjacency without ≥2.25:1 separation |
| C-H3 | **accent-only** (MediaStyle) | any app-painted value in the MediaStyle block other than `#F2F0E6`; or a structural change (body text, mark, rate value entered) |
| C-H4 | **element inventory** | the StatusView contains anything outside §SurfaceInventory (visible, hidden, dead), or is missing one of the 9 entries; structural-dead removals must be removals (no `display:none` corpses) |
| C-H5 | **aria inventory** | missing `aria-label`/state attribute on an enumerated control; missing `role="progressbar"`/valuenow/valuetext; missing the visually-hidden `role="status"` live region; legacy alert+polite pair; painted numeric rate anywhere |
| C-H6 | **movement census** | ≥1 loop outside {pill}; missing reduced-motion freeze guard; unbounded yellow pulse (pulse must respect iteration bound ≤3 s TTL); pulse on the failure color; banner entry not one-shot (looping slide) |
| C-H7 | **contrast solver** | any painted pair under WCAG thresholds (text ≥4.5:1, non-text ≥3:1) over void/stage — computed, same table both modes |
| C-H8 | **served-identity** | served bundle HTML ≠ `src-tauri/src/status_view.html` (byte-identity), or auditee 5's `frontendDist` pointing at an unverified surface |
| C-H9 | **short-mix behavior** | burst truncate/replace violations (earcon↔earcon mixed slots; tail appended before the cut; system earcon queued; FIFO residual serialization vs the shipped `backend.rs:266,285-290` shape) — plus the ≤100 ms first-consumed-sample oracle |

- Dual-state census (H1–H7 run over BOTH progress states): the indeterminate state is reachable per `specs/004-ressoa-dogfood-build/contracts/surface.md` (`duration_known_at_start = !sections.is_empty()`), so the check instantiates determinate and indeterminate renders and audits both.
- Every failure is a NAMED failure (check id + file + line estimate) so a human can act — not a bare non-zero.

## A3. Where they run

- `cargo nextest run --workspace` — the rust twins (H1–H9 as pure functions over the auditee bytes, fixed parsers — no unguarded regex over documents).
- `scripts/verify.sh` — a new fail-closed gate step (`surface-audit`) running the same checks; for H8, a missing built bundle is reported `DEFERRED (owner=device-build)` (mirroring gate-8-full posture) — never silently passed — and the device session re-covers it.
- Standalone run: every check is invocable without a device (Principles I–II; `--format json` where applicable).

## A4. Whitelist discipline (no-weakening rule)

The auditee list and the allowed hex set are data owned by this contract + `data-model.md`. Loosening them is a coordinated contract edit (same discipline as closed sets), not a script tweak. A failure is never silenced by deleting an auditee or an assertion — the rule resolves in the audited file: never weaken the gate to go green.

## F. FR-006 / FR-007 — Design-SONA surface contract (remediated 2026-10-26, commit 81a9987 / follow-up)
- FR-006 (sound accessibility): Pre = audio path available / mute state indeterminate; Post = sound plays regardless of mute; Invariant = earcon never blocks access (redundancy, not barrier).
- FR-007 (notification purity): Pre = MediaStyle notification / lock-screen render; Post = NO brand logo / sound-wave glyph shown; Invariant = accent-only (`#F2F0E6`) + title + progress; logo waves vetoed (FR-028/FR-029).
- Verification: host census of hex values (10 members); device leg deferred (gate-9 / SC-003 mirror).
