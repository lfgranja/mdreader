# On-device trial harness — `001-android-media-shell`

> **Attestation rule.** This document is the **procedure set only** — it records no
> verdicts. **T086** executes every procedure in `trials.md` 10/10 on API-29+
> hardware. **T087** alone attests results, per criterion, in
> `specs/001-android-media-shell/VERIFICATION.md`. No agent self-attests; a
> verdict is valid only if it traces to a completed recording sheet (§5).

## 1. Purpose

This harness is the operator-executable procedure set for every success criterion
of the feature that can only be discharged on hardware: lock-screen playback,
audio-focus response, output routing, latency bounds, resume/recovery offers,
engine failover, notification content, and residency. It exists because
gates 1–8 are host-runnable and gate 9 only proves the artifact compiles for
the device — none of them can show that audio keeps playing in a locked phone in
a moving car (constitution Principle VIII, Principle V: every decision that can be
made deterministically is, and everything that cannot is a recorded procedure,
not a judgement call).

## 2. Required device

- **Floor**: Android 10 (API 29) — the SC-032 floor. No requirement in the
  specification is verified on an older release; older releases are out of scope.
- **Normative latency reference device** (plan.md §Technical Context, normative
  for FR-027 / FR-019c): Redmi 2201117TY (Redmi Note 11), Snapdragon 680
  (`bengal`), 4x Cortex-A73 + 4x Cortex-A53, ~5.8 GB RAM, Android 13 / API 33,
  document already parsed. SC-009, SC-054, and SC-050's post-idle start are
  measured on this phone; every verdict counts as-is against the amended bound,
  with no device-class asterisk.
- **Build**: `aarch64-linux-android` artifact from gate 9
  (`cargo tauri android build --target aarch64`).

## 3. Prerequisites

1. Gate-9 build produced the device artifact; gates 1–8 already green.
2. T028a Tauri init complete; the test document corpus is already parsed; the
   audio engine stand-ins are registered (`standin_a`, `standin_b`,
   `engine_tier: test_double` — admissible only while no non-`test_double`
   adapter is registered, per the 2026-10-05 clarification).
3. **Debug/harness build** with `tracing` enabled on logcat tag `mdreader`. The
   harness's declared observation channels (fixture contract of the debug build):
   - Position/transport: the `MediaSession` mirror via
     `adb shell dumpsys media_session` — the mirror reflects the Rust playback
     module's state by construction (FR-002/FR-003); a mirror/audio divergence
     is itself a defect.
   - Tracing lines on tag `mdreader`: `focus.duck gain=<factor>`,
     `preroll.buffered_secs=<n>`, `audio.stream rate=<Hz> channels=<n>
     format=<f32>`, `audio.first_sample_ts_ms=<ts>`,
     `engine.failure error=<kind>`, `offer.shown engines=<ids>`,
     `boundary.resolved at=<secs>`, `engine.billed_bytes=<n>`.
4. **Harness fixtures** (debug builds only — never in a release artifact):
   - **Focus-injector APK**: issues audio-focus requests on demand —
     duckable transient (`AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK`),
     silence-demanding transient (`AUDIOFOCUS_GAIN_TRANSIENT`), permanent
     (`AUDIOFOCUS_GAIN`), and the deferred-grant scenario (a transient holder
     active while the app's `setAcceptsDelayedFocusGain(true)` request is
     accepted but not yet granted; the grant arrives when the holder abandons).
   - **Fault-injection broadcast** `mdreader.TEST_FAULT` with extra
     `fault=<device_refused|engine_unreachable|engine_init_failed|
     rate_mismatch|disable_noisy_receiver|inject_standin_a_failure|
     inject_standin_b_failure>`.
   - **Known-gap synthetic audio**: the stand-ins deliver synthetic PCM with a
     tone/gap map known to the harness, so a resume landing is checkable against
     energy peaks (FR-015 verification rule: the resume MUST land in a gap,
     never in an energy peak).
5. **Test document corpus**: (a) a titled document with ≥2 headings and
   metadata title, longer than 5 minutes; (b) a document with no metadata and
   no headings; (c) a document with content before its first heading; (d) a
   document with a pre-computed total spoken length; (e) the same document at
   two paths (move/rename corpus); (f) a content-edited copy of (a).
6. **Host tooling**: `cargo nextest run -p diag` (record scanners, T087),
   `crates/diag` reader CLI (`--format json` default), `crates/session` CLI
   (`get | put | clear | rate get|set`).

## 4. How to run

1. `adb install -r` the gate-9 debug artifact; grant notification permission on
   API 33+ (`adb shell pm grant <pkg> android.permission.POST_NOTIFICATION`) —
   except for SC-028, which revokes it.
2. Execute `trials.md` top to bottom. Every section is 10 trials unless the
   section states otherwise; a section may not be skipped or reordered without a
   recorded reason on the sheet.
3. Record **every trial** on the sheet in §5: trial number, observation,
   pass/fail, timestamp (ISO-8601, device clock).
4. **Any single trial failure fails its criterion.** Stop the section, capture
   evidence (logcat excerpt around the trial, record excerpt, screenshot where a
   surface is in play), and report the partial count (e.g. "SC-009: 7/10").

## 5. Recording sheet template

One row per trial; copy the block per criterion.

```
SC-___  criterion: <name>                     device: <model>  build: <sha>
| Trial | Observation | Pass/Fail | Timestamp (ISO-8601) |
|-----|-------------|-----------|----------------------|
| 1   |             |           |                      |
| ... |             |           |                      |
| 10  |             |           |                      |
Verdict: ___/10   Operator:          Evidence ref:
```

## 6. Routing table

| Criterion | Category | Discharged by |
|---|---|---|
| SC-001, SC-002, SC-003, SC-041, SC-055, SC-004, SC-007, SC-008, SC-009, SC-011, SC-013, SC-014, SC-016, SC-017, SC-018, SC-020, SC-021, SC-022, SC-023, SC-024, SC-025, SC-026, SC-027, SC-028, SC-029, SC-030, SC-031, SC-032, SC-033, SC-033b, SC-034, SC-036, SC-038, SC-039, SC-040, SC-042, SC-043, SC-045, SC-046, SC-047, SC-048, SC-049, SC-050, SC-050b, SC-051, SC-052, SC-053, SC-054 | **On-device trial, 10/10** (48 sections T-01–T-48) | `trials.md`, executed by T086 |
| SC-010 | **Surface procedure** (3 legs) | `trials.md` §S-2: T051 `prop_surface` green + body-text render-path grep over `android-shell/` + `src-tauri/` (zero hits) + locked visual check 10/10 |
| SC-037 | **Static-scan + manual-absence procedure** | `trials.md` §S-1: grep app surfaces for record-read/export strings (zero hits) + manual absence walk (refs T074/T081) |
| FR-026 | **Offline-start procedure** | `trials.md` §S-3: airplane-mode start + 60 s continue with no network |
| SC-005, SC-012, SC-019 | **Record-scan (zero-occurrence)** — NOT trials | `trials.md` §S-5: T087 record scans (`cargo nextest run -p diag`, cited by target name, T068b canary evidence) |
| SC-006 | **N/A on-device** — host gate | T081 `scripts/check-manifest.sh` |
| SC-015 | **N/A on-device** — host gate | T055 `crates/engine-core/tests/prop_pitch.rs` |
| SC-035 | **N/A on-device** — host gate | T081 `scripts/check-shell-purity.sh` (+ T086 human-review residue) |
| SC-056 | **N/A on-device** — host gate | T068 `crates/diag/tests/prop_no_prose.rs` |
| SC-057 | **N/A on-device** — host gate | T081 closed-set/registry-reference check |
| SC-044 | **CARRIED-FORWARD** — not runnable here | owner: synthesis feature (`002-fr-024-synthesis-adapter`); SC-051 discharges the mechanism meanwhile |

**Feature 004 (`004-ressoa-dogfood-build`)** — the dogfood build's own
acceptance set lives in `trials-004.md` (executed by T051, attested by the
Phase 8 tasks T048–T052, reference phone Redmi 2201117TY only, fixture
`fixtures/fixture-pt-2h.md`, every section 10/10):

| Criterion | Category | Discharged by |
|---|---|---|
| SC-001 (commute loop, per rung) · SC-002 (first audible ≤ 3 s) · SC-003 (interruptions per class) · SC-004 (blind confirmations, sounds on/off) · SC-005 (resume matrix + replay caps) · SC-006 (lock-screen purity + dismissal refusal) · SC-007 (rate persistence + unpitched) · SC-008 (record bounds + zero prose) · SC-009 (offline) · SC-010 (in-app purity + Stop clears) | **On-device trial, 10/10** | `trials-004.md` §T051-01…T051-10, executed by T051 |
| Lock-screen privacy (hide-contents) | **Manual inspection** | `trials-004.md` §T051-11 |

Coverage check: 57 SCs + FR-026 = 58 entries; 48 trials + 9 routed/non-trial
procedures = 58. Nothing is unrouted.

## 7. Bounds the pass criteria carry (exact values — a procedure that does not
assert the number is not the procedure)

| Bound | Value | Asserted by |
|---|---|---|
| Headphone / A2DP-loss pause | ≤ 1 s | SC-002 |
| Play-to-audio (document already parsed) | ≤ 3 s | SC-009, SC-050 (post-idle) |
| Rate-change audible effect | ≤ 2 s | SC-054 |
| Resume-offer window | 3 s; reading begins ≤ 4 s after offer appears (3 s + 1 s post-expiry budget) | SC-020 |
| Engine-failure offer window | 3 s | SC-039/040/051/052 |
| Replay after kill/power loss | ≤ 30 s | SC-007, SC-017 |
| Transient rewind | 5 % of paused duration, capped at 30 s; total backward movement ≤ 30 s | SC-041 |
| Pre-roll bound | ≤ 60 s; drains to empty while paused-idle | SC-013, SC-050 |
| Duck depth | 15–25 % of normal gain (app-applied gain factor) | SC-003 |
| Declared format | mono f32 @ 22050 Hz, exact-requested, never device default | SC-034 |
| Rate ladder | 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0 — one step per press, saturating ends | SC-026 |
| Pitch invariance | ≤ ±50 cents F0 across all rungs (host-measured, SC-015); on-device: zero audible pitch change | SC-027-adjacent, T-23 |
| Diagnostic record | 30-day window + 10 MB cap, oldest-first, trim visible as `trim` entries; `detail` ≤ 200 chars, non-prose pattern; `failure`/`refusal` carry `error` | SC-025 (device), SC-056 (host) |
| Focus-loss pause reason | shown ≤ 3 s, expires everywhere after 3 s | SC-031 |
| Failure reason | persists, no expiry | SC-048 |
| Notification content | title + current section title + progress + controls only — no body text, no elapsed/remaining, no rate, no voice | SC-010, SC-046, SC-027 |
