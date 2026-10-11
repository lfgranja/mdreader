# Implementation Plan: Android Media Shell

**Branch**: `001-android-media-shell` | **Date**: 2026-10-02 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-android-media-shell/spec.md`

## Summary

Build the driving-first audio playback shell: Rust owns PCM playback (cpal, AAudio backend),
position, buffering, and transport as the single authority; a thin Kotlin surface owns only
what Android structurally requires (foreground `Service`, `MediaSession` mirror, notification,
audio-focus request). Behaviours: lock-screen transport (±5s skip, closed 7-step rate ladder,
Stop≠Pause), classified focus response (FR-013 DuckableTransient → 15–25% duck and keep
reading; FR-013a SilenceDemanding call/alarm → pause; FR-014 PermanentLoss → pause, never
auto-resume, latest-event-wins), headphone-pull pause with
private→speaker privacy, content-fingerprinted resume with 3s bounded offers, engine-failure
recovery against two contract-honouring stand-in engines (SC-051; SC-044 re-verified on real
engines later), and a 30-day/10 MB on-device diagnostic record with no in-app surface.

## Technical Context

**Language/Version**: Rust, Edition 2024, MSRV 1.85 (constitution-fixed)

**Primary Dependencies**: `tauri` 2.12.1 (stable 2.x; 3.0.0-alpha rejected per Principle IV);
`cpal` (latest stable at add-time, with the AAudio backend via `ndk`, mono f32 playback,
device-format conversion in Rust; pinned only if the AAudio/`aarch64-linux-android` requirement
forces a version, recorded with version + reason + risk per Principle IV);
`redb` (bookmark + settings store); `blake3` (content fingerprint); `serde`/`serde_json`
(JSONL diagnostic record, IPC payloads); `thiserror` (closed-set errors); `tracing`
(observability); `clap` (CLI per Principle II). Kotlin side: `MediaSession`,
`AudioFocusRequest` (`AUDIOFOCUS_GAIN`), `MediaStyle` notification — mirror only, no logic.
Engine side: two stand-in engines honouring the engine handoff contract (FR-024 places real
synthesis in another feature; SC-051 verifies the mechanism here, SC-044 the hardware later).

**Storage**: `redb` — `bookmarks` table (blake3 fingerprint → time-offset, paused-at, sentence
index), `settings` table (app-wide last rate, default 1.0x). Diagnostic record: append-only
JSONL in app-private storage (`/data/data/<pkg>/files/`), 30-day window + 10 MB cap,
oldest-first trim recorded as a trim event, best-effort writes that never interrupt playback.

**Testing**: `cargo nextest run --workspace` (unit + integration, host-runnable logic:
rewind math, rate ladder, fingerprinting, trim bounds, closed-set exhaustiveness); automated
manifest check (SC-006: `<service mediaPlayback>` + permissions present in built manifest);
automated no-logic-in-shell check (SC-035); on-device 10/10 trials for lock-screen, focus,
routing, latency (≤10 s cold-start play-to-audio at the transducer), and two-stand-in-engine failover (SC-051).

**Target Platform**: Android 10 (API 29) minimum through Android 14+ (API 34, FGS-start
restrictions); `aarch64-linux-android`; headless Linux CI plus on-device gate 9
(`cargo tauri android build --target aarch64`).

**Reference device** (normative for FR-027, FR-019c): Redmi 2201117TY (Redmi Note 11),
Snapdragon 680 (`bengal`), 4x Cortex-A73 + 4x Cortex-A53, ~5.8 GB RAM, Android 13 / API 33,
document already parsed. All "10 out of 10 trials" latency criteria (SC-009, SC-054,
SC-050-idle-resume) are measured on this phone; every verdict counts as-is against the
amended bound, with no device-class asterisk. (Amended 2026-10-11 per `003-reference-device`; prior class text superseded, preserved in git history.)

**Project Type**: Mobile app (Tauri) backed by a Cargo-workspace modular monolith of focused
library crates (`player`, `session`, `diag`, `engine-core`, `doc-structure`), each CLI-drivable per
Principles I–II.

**Performance Goals**: Audible audio ≤10 s after Play on the reference phone (cold start);
headphone-pull pause ≤1 s; skip displacement exactly ±5 s; rate change effective ≤2 s;
pre-roll ≤60 s and drained while idle; replay after kill ≤30 s; transient rewind
5%-of-pause capped at 30 s with sentence snap never exceeding the cap.

**Constraints**: Offline-capable (no network to start/continue); no paid tier; closed sets
everywhere (no `_ =>`, no `Other`/`Unknown`); deny-list + supply-chain gates fail closed;
Rust rigor lints (`unsafe deny`, pedantic clippy, no unwrap/expect/println);
mono-f32-at-declared-rate handoff (mismatch = named engine failure, never silent resample);
diagnostic record never transmitted, never in shared storage, no in-app surface (FR-039a);
lock-screen shows title + section title + progress only (no body text, no elapsed/remaining,
no rate/voice); notification non-dismissable while playing, pause-is-the-route-to-dismissal.

**Scale/Scope**: Single-document sessions only (no queue/playlist); one stored position per
document fingerprint; one resident component + one notification; diagnostic record ~100k
entries max; seven closed rate rungs (0.75x–4.0x).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I (Library-First)**: PASS — new crates (`player`, `session`, `diag`) are standalone with
  headless Linux CLI surfaces; Tauri app is one consumer. No organiser-only crates.
- **II (Text I/O)**: PASS — each crate exposes args/stdin→stdout/stderr with `--format json`;
  notification/IPC paths call the same code.
- **III (Test-First)**: PASS — host-testable logic (rewind/snap math, ladder, fingerprint,
  trim, focus classification) gets red-green tests; on-device trials cover the rest; no
  implementation before failing tests in tasks phase.
- **IV (Stable + supply-chain)**: PASS — pinned versions verified at add-time against
  crates.io; `cargo deny` + `cargo audit` + `--locked` gates; no new exception (the `ort`
  RC exception is pre-existing and untouched by this feature).
- **V (Deterministic gates)**: PASS — SC-006/SC-035 are automated checks; verify.sh ordering
  kept; missing input fails, never skips.
- **VI (Engine-neutral)**: PASS — stand-in engines implement the same trait; SSML capability
  + alignment granularity declared per adapter; engine failure surfaces named, never silent.
- **VII (Closed sets)**: PASS — rate ladder, focus outcomes (`DuckableTransient | SilenceDemanding |
  PermanentLoss`, plus the `regained` direction event per FR-044), transport states, engine ids,
  error enums all closed; `_ =>` banned by gate 7.
- **VIII (Driving-first)**: PASS — the feature IS this principle: lock-screen everything,
  screen-off playback, 3 s bounded offers, no look-demanding interaction.
- **IX (Offline-first)**: PASS — no network to start/continue; paused-idle synthesises
  nothing (FR-015c).
- **X (Free-tier budgeted)**: PASS — idle pause bills nothing; pre-roll bounded and inside
  granted budget; no paid tier required.
- **XI (Sovereignty)**: PASS — no secrets in repo/binary; diagnostic record device-local,
  private storage, never transmitted; fallback order consent-bounded, Edge last, never
  default, never selected without consent.
- **Tauri Platform Contract**: PASS — Kotlin confined to Service/MediaSession/notification/
  focus-request; Rust single playback authority, MediaSession is a mirror; manual
  `<service mediaPlayback>` manifest edit re-verified every Tauri upgrade via SC-006.
- **Audio Pipeline Contracts**: PASS — sentence/node granularity as designed precision;
  adapters declare deliverable granularity; degradation observable.
- **Driving-Safety / Licence**: PASS — no new licences introduced at plan level (LGPL
  `soundtouch` fallback is a tasks-phase contingency only if the engine contract cannot
  stretch; would enter Exceptions Log then, not now).

No violations. No Complexity Tracking entry required.

**Development workflow note (per user operating constraint 6 / tasks.md L11, L311):** Every file edit is followed by an atomic conventional commit with a detailed edit-summary (deterministic hook). This applies to all implementation phases and to any clarification edits (e.g., A1/B1/B3 above).

## Project Structure

### Documentation (this feature)

```text
specs/001-android-media-shell/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── engine-handoff.md
│   ├── shell-ipc.md
│   ├── storage-schema.md
│   └── diagnostic-record.schema.json
├── checklists/          # Reviewer-owned requirements-quality reviews
│   ├── requirements.md
│   ├── playback-authority.md
│   ├── notification-surface.md
│   ├── diagnosability-nfr.md
│   └── session-recovery.md
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)

VERIFICATION.md          # Phase 8 output: per-SC attestation report (created by tasks)
```

### Source Code (repository root)

```text
crates/
├── player/          # Rust playback authority: transport, position, buffering,
│   └── src/         #   pre-roll ring (≤60 s), rate ladder, rewind/snap math,
│       └── ...      #   device-format conversion, engine-failure recovery
│   └── tests/       #   prop_* property suites, integration/ + contract/ suites
├── session/         # Stored positions (redb: bookmarks + settings), content
│   └── src/         #   fingerprint (blake3), resume-offer state machine
│   └── tests/       #   prop_fingerprint, prop_resume, prop_offer
├── diag/            # Diagnostic record: JSONL append, 30 d + 10 MB trim,
│   └── src/         #   private-storage writer, tooling-readable reader CLI
│   └── tests/       #   schema conformance (+negative control), prop_record,
│                     #   prop_no_prose, prop_scanner_canary
├── engine-core/     # Engine-neutral trait: handoff contract, capability levels,
│   └── src/         #   pitch-preserving stretch op, stand-in engines (SC-051)
│   └── tests/       #   prop_closed_sets, standin_shape, prop_pitch,
│                     #   prop_fallback, contract/ (handoff + stand-in conformance)
└── doc-structure/   # Ordered document headings + boundary re-anchor, the runtime
    └── src/         #   stand-in for the parser FR-024 defers: headings_before(pos),
        └── ...      #   nearest-boundary re-anchor with 0.0 fallback, CLI-drivable
    └── tests/       #   prop_headings

src-tauri/
├── src/             # Tauri app: plugin registration, Channel streaming (delegates only, no playback logic)
│   ├── lib.rs            # register_android_plugin + run_mobile_plugin (T028)
│   ├── permissions.rs    # FGS/notification permission gate, no logic (T030b)
│   ├── surface.rs        # ShortReason/FullReason shapes + expiry (T065 owns)
│   ├── status_view.rs    # read-only rate/transport/position/reason view (T065b)
│   └── failure.rs        # failure strings conforming to surface.rs shapes (T078)
└── gen/android/     # Manual <service mediaPlayback> manifest declaration (incl. minSdk 29 per SC-032)
    └── app/src/main/AndroidManifest.xml

android-shell/      # Thin Kotlin, forward-only mirrors (SC-035 allowlist, zero computation):
└── src/main/kotlin/app/mdreader/  # PlaybackService.kt, FocusMirror.kt (T040),
                        #   RoutingMirror.kt (T047), MediaMirror.kt (T062), Residency.kt (T064)

tests/
├── device/          # 10/10 on-device trial procedures (T083); documentation only,
│   ├── README.md    #   no cargo target
│   └── trials.md

scripts/
├── verify.sh        # gates 1-9, single entrypoint, fail-closed on missing input
├── check-manifest.sh
├── check-shell-purity.sh
└── expected-tests.txt  # committed target list; verify.sh gate 2 diffs against it

docs/
├── architecture.md
├── ipc.md
├── fallback-order.md
├── diagnostic-record.md
└── byte-budget.md   # versioned Principle X table (per-request bytes, per-character price)

# Tests live in the crate that owns the behaviour, not in a root tests/ directory: the
# workspace root is a virtual manifest and therefore owns no targets, and Cargo only compiles
# `tests/**` inside a package. Cross-crate integration and contract suites sit under
# `crates/player/tests/{integration,contract}/` with the other crates as dev-dependencies.
```

**Test-harness placement** (constitution Principle I + gate 2 fail-closed): property suites
live beside the crate whose invariant they test (`crates/*/tests/prop_*.rs`); cross-crate
suites live in `crates/player/tests/integration/` and `crates/player/tests/contract/`, which
declare `session`, `diag`, and `engine-core` as dev-dependencies; `scripts/expected-tests.txt`
records the full `cargo nextest list --workspace-type all` target set so verify.sh fails closed
if a target is missing *or* unexpected.

**Structure Decision**: Cargo-workspace modular monolith (constitution-fixed). Four focused
library crates under `crates/` (each independently testable + CLI-drivable), Tauri app as one
consumer, Kotlin confined to `android-shell/` with zero playback logic (enforced by SC-035
check). No web `api/` — Svelte UI remains a later full surface, except for one minimal read-only
status view (`rate`, `transport`, `position`, current `reason?`) that renders only the
strings `surface.rs`/`failure.rs` produce. It adds no controls and no playback logic;
notification and diagnostic CLI remain the primary surfaces.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations — table intentionally empty.

## Post-Design Re-check (Phase 1 complete)

- All Phase 1 artifacts (data-model.md, contracts/, quickstart.md) resolve behaviour into
  the crates above without moving logic into the Kotlin shell or the Tauri app layer.
- Stand-in engines are test doubles behind `engine-core`, not a second engine set — SC-051
  verifies the mechanism; SC-044 remains the hardware obligation carried forward.
- `soundtouch`/`rubato` remain unevaluated contingencies: if the engine contract's
  pitch-preserving stretch cannot be honoured by real engines, the tasks phase records the
  licence implication (LGPL) in the Exceptions Log. No plan-level exception taken.
- Gates: no ERROR. Gate 8 split recorded in constitution Exceptions Log item 8; T080/T084 wording matches it (`gate-8-shell: PASS` + `gate-8-full: DEFERRED (owner=synthesis)`). Proceed to `/speckit.tasks`.
