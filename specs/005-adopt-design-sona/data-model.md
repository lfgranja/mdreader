# Data Model: Adotar o DESIGN-SONA (Option A subset)

**Feature**: `005-adopt-design-sona` | **Source**: [spec.md](spec.md) | **Extends**: `specs/004-ressoa-dogfood-build/data-model.md` (NotificationSurface, Earcon, Interruption, PlaybackRate, DiagnosticRecord shapes — referenced, not redefined) · `specs/001-android-media-shell/data-model.md` (PlaybackSession/Position base)

Closed-set discipline (constitution §VII): every table here is a closed set; adding or removing a member is a coordinated edit with version bump and docs — never a local convenience. The audited-hex lexicon, the earcon confirmation set, the rate ladder, the element inventory and the haptic disposition set carry NO catch-alls.

## SonaToken (audited hex lexicon — closed, 10 members)

| Token | Hex | Role (paint span) | Provenance |
|---|---|---|---|
| `--sona-void` | `#0B0F1A` | page background (in-app) | design dir; shipped |
| `--sona-stage` | `#141B2D` | raised surface structural (metadata card + mark container) — never text/signal/notification | batch C2/96 |
| `--sona-voice` | `#F2F0E6` | primary text (título) + progress fill (both states, flat) + **MediaStyle accent (the app's single notification color)** | batches 91/C1; T044 ships accent |
| `--sona-etwahl` | `#8FD8E8` | focus ring only (`:focus-visible` on enumerated interactive controls); «fluxo ativo» role reserved unpainted | batch E1; recipe `tokens.md:20` |
| `--sona-hush` | `#A9B8CE` | current-section text (metadado secundário) | shipped bit-exact |
| `--sona-harp` | value recorded in `DESIGN-SONA/tokens.md` (ledger; census consumes the ledger constant — drift fails the check) | positional rate dot on the ladder (in-app only; never value) | design dir |
| `--sona-celeste` | `#B9A6F5` | **RESERVED, unpainted this feature** — zero paint in audited files = INVENTORY VIOLATION | batch C2 |
| `--sona-boundary` | `#55688F` | progress track | field C1 |
| `--signal-caution` | `#FFD400` | 3 s pause warning ONLY (banner + in-app pulsing dot) — never adjacent to failure (≥2.25:1 separation) | batch D2; shipped `status_view.html:29`/`:179` |
| `--signal-failure` | `#FF4D63` | permanent failure ONLY (static paint, no pulse) | batch D2; shipped `:30`/`:180` |

- **Banned paints** (closed violation list): jade `#5AD6A8` and its erratum `#5FD0A0` — NEVER on any audited surface (the player-color prohibition; the one approved green lives only in dir errata notes as a provenance marker).
- Off-role paint of any member is the same violation class (batch D2 exclusivity).

## SurfaceInventory (in-app driving surface — closed enumeration)

| Element | Binding | Status |
|---|---|---|
| Header mark | Concha Harmônica "R" (`DESIGN-SONA/assets/logo/logo-primary-concha-r.svg`; triad `logo-canonical.svg` = host faces ONLY) | restyle stamp (C4) |
| Metadata card | static title + current section on `--sona-stage` | restyle stamp (C2/96) |
| Progress | `role="progressbar"`, track `--sona-boundary`, fill/pill `--sona-voice` | restyled (C1/C5) |
| Transport controls | existing 004 controls, single real grid stays | kept + tokens (C6/B4) |
| Rate ladder | positional dot `--sona-harp`, 7 rungs, aria value only | kept + restyle (value-blind) |
| Reason line | 3 s expiry behavior + `--signal-*` colors (004 contract) | kept + tokens (D2) |
| Resume offer | 004 contract | kept |
| Focus ring | `:focus-visible` `--sona-etwahl` on every enumerated interactive control | NEW (E1) |
| Live region | visually hidden `role="status"` announcing ladder value + reason/failure states (`aria-hidden` dot strip, `progressbar` + valuetext) | NEW (D5 → SC-001 item (iv), SC-007) |

- **Dead removals (structural, not cosmetic)**: empty version footer; unrendered state fields; the numeric-rate bench leak («Taxa: 1.0×») — the wiring kill is the evidence, not a feature (batch B4/104).
- **Excluded forever here**: sumário/TOC, elapsed/remaining time, numeric rate, RMS strip, buffer ring, any excluded screen (Cockpit Eyes-Free, Console Braun & Rams, Sumário Editorial Suíço, Matriz de Telemetria).
- **Any element not listed here** (visible, hidden, or dead) = inventory violation (SC-001 item (ii) fails).

## NotificationSurface (extends 004)

| Field | Rule |
|---|---|
| Accent | exactly one value = `--sona-voice` `#F2F0E6` (the platform's single accent channel); nothing else app-painted |
| Content | título + current section + progress only (004 FR-028 bank unchanged) |
| Warning state | static reason line (TTL 3 s) + Ostinato earcon at trigger; NO pulsing dot, NO extra color |
| Failure state | reason line expiry + Sforzando; permanent = static red **only** on in-app (`--signal-failure`) |
| Mark/logo | none on lock-screen/notification (FR-028/FR-029) |

## EarconConfirmation (closed, 7 confirmations across 6 names)

| Action/State | Earcon | Acoustic identity (AUTHOR = 004) | Class |
|---|---|---|---|
| Play | Adagio | C5→E5 ascending, 120 ms | user |
| Pause | Andante | E5→C5 descending, 120 ms | user |
| Rate step | Unison | 880 Hz tick, 60 ms | user |
| Skip forward | Allegro (frente) | chirp 880→1108 Hz, 40 ms | user |
| Skip back | Allegro (trás) | chirp 880→660 Hz, 40 ms | user |
| Warning (pause 3 s) | Ostinato | 004 contract | system |
| Permanent failure | Sforzando | E4 ≈330 Hz square + minor-2nd, ≈450 ms | system |

Policy fields: additive mix earcon↔narração ONLY (full gain, duck-exempt short channel); burst = truncate/substitute «latest wins» (in-flight user earcon tail dropped pre-append in `enqueue_earcon`); system earcons interrupt in-flight user confirmations and fire AT state change (never queued); pause drop-and-refire = user-initiated set only; latency ≤100 ms (first consumed sample); emission failure → action proceeds + AT MOST one re-emission at the next render space → typed diag event; rapid-sequence sub-test (2–3 heterogeneous in 100–150 ms) is the SC-002 evidence leg; systematic confusion on the risky pairs (Allegro-trás×Andante; Unison×Allegro-frente) authorizes a 004 amendment — never a silent edit; ≥24 h replication.

## RateLadder (extends 004; value-blind preserved)

Closed 7 rungs `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`; in-app + notification expose it positionally only (`--sona-harp` dot / rate-step action without value); the NUMERIC value exists solely inside `aria-valuetext`/live region («velocidade 1,25») and TalkBack announcements — painted numerals on the driving surface = value-blind violation (SC-001 item (iv)).

## ProgressIndication (extends 004 Progress)

Content-offset fraction (consumed/total parsed units); determinate iff both totals known at session start (else indeterminate for the whole session — no mid-session flip); determinate render = flat `--sona-voice` fill over `--sona-boundary`; indeterminate render = solid sliding pill `--sona-voice` over the boundary track (004's fading shimmer REMOVED; no gradient exemption registered); census covers BOTH states.

## MovementSpec (closed motion set)

| Motion | Bound |
|---|---|
| Sliding pill (indeterminate) | the ONLY continuous motion; freezes at a VISIBLE track position under `prefers-reduced-motion: reduce` |
| Yellow pulse (caution) | strictly inside the 3 s TTL; then stop (2.2.2 strict satisfaction) |
| Failure paint | static — never loops |
| Banner entry | one-shot ≤200 ms (`slideIn` kept) |
| Anything else | prohibited by the census (`animation`/`transform` count minus the freeze-guard exception, item (v)) |

## HapticDisposition (E3, closed)

Disposition set = {`sound-only-gate`, `004-owner (T051-04 sounds-off)`}. SC-002 attribution = SOUND ONLY; app haptics inert during the trial (`haptic=None` shipped — `lib.rs:6047`; `MediaMirror.kt:540-583` never fires; no app off-switch; system «feedback háptico» does NOT gate direct `VibratorManager` calls); trial sheet carries the mandatory haptic-disposition line (rigid mount, hands off body, admin buzz-check at pilot); if 004's haptic wiring EVER lands before the trial, this feature owns the deactivation route (build flag or publish `None`) — the disposition itself is never dropped silently.

## DiagnosticEvent (coordinated §VII extension of the 004 kind set)

| Kind | Trigger | Bound |
|---|---|---|
| truncation counter | `enqueue_earcon` drops an in-flight user earcon tail (bursts) | temporal consequence registered — never suppression policy |
| emission-failure event | synthesis error / short channel unavailable | action proceeds; narration untouched; AT MOST one re-emission at next render space; failure = typed event, never unregistered silence |

No `Other`/`Unknown` arm is added; the existing diag-record privacy/trim rules are untouched.

## DesignDirState (post-alignment target)

Declared version = single **`v2.2.0`** in every audited header + `DESIGN-SONA/VERSION` + `DESIGN-SONA/CHANGELOG.md` + git tag `design-sona-v2.2.0`; stale rows fixed (`tokens.md:16/:28/:47/:65`, `components.md:17`, `DESIGN-SONA.md:60/:85`, `preview.html:238`), failure row = shipped E4, ONE banned green `#5AD6A8` (surface.md `#5FD0A0` = erratum), yellow-dot erratum recorded (`tokens.md:48`, `components.md:18`).

## ContrastPair (SC-001 item (i) — closed calculation table)

Pairs = every painted text/non-text over its effective background (void `#0B0F1A` / stage `#141B2D`), thresholds text ≥4.5:1 / non-text ≥3:1 (WCAG 2.1 AA), parametrized per background (future light mode reuses the calculator). Measured shipped margins: fill×track 4.88; fill over void/stage 16.75/15.01; `--sona-aria` 11.20→10.59 on recompute; `--sona-boundary` 3.43/3.07 (narrow non-text passes); any failing pair resolves via the ledger's own recipe — never an off-set hex.

## State transitions (prose)

Transport state machine, interruption classes, dismissal, session-end reasons: UNCHANGED from 001/004 (this feature adds no state). Added transitions here: warning pulse `pulse → stop` inside 3 s TTL → banner expiry; failure `static from trigger` (no pulse path); reduced-motion `pill running → pill frozen` (one-way while the setting holds); progress `indeterminate ⇄ determinate` only across session start rules (never mid-session flip); earcon channel `idle → sounding` (1 user earcon bounded, burst = replace); haptic disposition constant within a gate session.

## Validation rules (traceability quick-tab)

FR-001→SonaToken/SurfaceInventory · FR-002→NotificationSurface · FR-003/FR-009→EarconConfirmation (+DiagnosticEvent) · FR-004→header mark/host-only triad · FR-005/FR-006→SonaToken + MovementSpec/HapticDisposition set · FR-007→NotificationSurface (no mark) · FR-008→SurfaceInventory (single-column preserved; stamps + dead removals) · FR-010→EarconConfirmation latency · SC-001→census/contrast/inventory/aria/movement · SC-002→EarconConfirmation protocol + HapticDisposition · SC-005→latency oracle · SC-006→census + triad placement · SC-007→live region + freeze leg.
