# SC-009 Gate-Check Record — FR-024 Synthesis Adapter

**Recorded**: 2026-10-09
**Recorded by**: T047 drafting agent (read-only subagent of feature `002-fr-024-synthesis-adapter`, branch `001-android-media-shell`). This record is the only file this agent created; no existing file was modified and no git command was run (another agent is committing concurrently).
**Mandate**: `tasks.md` T047 — the SC-009 measurement and its verdict, per `spec.md:115-116` ("a hard gate for v1 … PASS requires all four stages in order").
**Fail-closed authority**: `.specify/memory/constitution.md:296-297` — "Gates are fail-closed: a missing input fails its gate rather than being skipped. A gate that passes when its input is absent is not a gate."

**Engine dependency (stage 2 input)**: Stage 2 additionally depends on the working engine of T044, and this record **awaits the T044 engine-binding outcome** (tasks.md T044 unchecked; T055/V1'' attestation left unchecked). Status at recording time: the T044 binding is **present but in progress in the working tree** (another agent) — `crates/adapter-synthesis/Cargo.toml:14` declares `sherpa-onnx = "=1.13.8"`; `src/synth.rs:44-47,82-114` binds the sherpa-onnx `OfflineTts` API behind a once-loaded `ENGINE: OnceLock<Option<OfflineTts>>`; the Piper pt-BR `faber` voice is embedded at `crates/adapter-synthesis/assets/voices/pt_BR-faber-medium/` (`pt_BR-faber-medium.onnx` = 63,201,428 bytes; blake3 `feb0b4b613456f0b531c86744af18b2d9a8dbdb7accd34eab2c068a8ece26cbd` per `PROVENANCE.md:47-51`). On-host evidence the binding works: the test the T055 record left red — `audio_chunk::a_bounded_segment_delivers_a_conforming_non_empty_mono_buffer` — now **PASSES** (`PASS [103.424s]`, one-time model load + real inference). **This does not unblock stage 2**: the benchmark's endpoint is the transducer on the reference device, which no host build can provide (stage 2 below).

**Host**: x86_64 Linux, headless; no Android device; no speaker/transducer measurement path; no `libasound2-dev` (T036 B1); Android NDK toolchain absent (T036 B2).

**Observed test state (this record's evidence base)**:

- Full suite, 2026-10-09, `cargo nextest run -p adapter-synthesis --no-fail-fast` → **`Summary [ 280.309s] 106 tests run: 103 passed (3 slow), 3 failed, 0 skipped`**
  - `FAIL [ 14.934s] adapter-synthesis::cli_protocol synthesize_subcommand_fails_closed_with_nothing_on_stdout`
  - `FAIL [ 8.306s] adapter-synthesis::unit_failure synthesis_fails_closed_with_a_named_init_failure` (panic at `crates/adapter-synthesis/tests/unit_failure.rs:647:17`: "synthesize must fail closed with a named failure, not return Ok — an empty buffer would be a silent success (R2 / R7 / SC-035)")
  - `SIGTERM [279.797s] adapter-synthesis::cli_protocol synthesis_never_leaks_document_prose_into_stdout` (proptest still running when nextest cancelled; each case now performs real model inference)
  - All three failures are **T044-transition artifacts, not adapter defects**: the first two pin the obsolete design-phase premise "the Piper `faber` voice model is not bound in this design-phase crate" (`tests/unit_failure.rs:616-618`; `tests/cli_protocol.rs:1044-1048`) and are now red *because* the engine is bound and returns real audio (the `Ok`-arm of `unit_failure.rs:633-650` is hard-wired to fail: `let silent_success = buffer.len() < usize::MAX;` at `:646`); the third is a timeout-class abort, not an assertion failure. They are T044's to rewrite (the fail-closed contract moves from "unbound → `InitFailed`" to "bound → conforming non-empty mono f32 @ 22050 Hz buffer", which `audio_chunk::a_bounded_segment_delivers_a_conforming_non_empty_mono_buffer` already pins green).
  - Delta vs. the T055 record (`tasks.md:227`: 106 run, 105 passed, 1 failed): `a_bounded_segment_delivers_a_conforming_non_empty_mono_buffer` flipped red→green; the two stale-premise tests above flipped green→red; the no-leak proptest flipped green→timeout.
- Targeted evidence run, 2026-10-09 (stage-3 evidence tests, directly observed): `cargo nextest run -p adapter-synthesis --no-fail-fast <14 filters>` → **`Summary [ 203.411s] 14 tests run: 14 passed (1 slow), 92 skipped`**

---

## Stage 1 — Requirements review — **PASS** (discharged on host)

Requirement text, quoted verbatim from `spec.md:115`:

> **SC-009**: With the adapter active, a user can press Play on a parsed document and audible audio is produced within ≤3 seconds at the transducer (full pipeline: adapter mono f32 @ 22050 Hz → player device-format conversion → speaker; endpoint is transducer, not adapter output; reference device = `001/plan.md:50-53` mid-range arm64, 4x Cortex-A78-class, 6 GB RAM). Pre-roll (`FR-015a`) begins on the press; ≤3 s applies to first audible after Print [sic] (cold start), independently of any prior pre-roll; warm-resume (pre-roll buffered) governed by `SC-020` (≤4 s).

Gate text, quoted verbatim from `spec.md:116`:

> **Gate-check sequence (edit 5)**: Requirements review → `SC-009` benchmark on reference device → mechanism-preservation verification (`SC-051` preserved continuously, `SC-035` named failure, `SC-027` position verified, `FR-028` notification integrity) → discharge-readiness assessment (`SC-044` prerequisites explicit, consent deferred, `test_double` inadmissible with adapter registered). PASS requires all four stages in order; `SC-044` deferred is a valid PASS condition for v1.

Acceptance bound: **≤3 s press→first-audible**; **hard gate for v1** (`spec.md:17`: "Confirm ≤3 s achievable with basic adapter (benchmark on reference device before declaring PASS); `SC-009` is a hard gate for v1. If benchmark fails, adapt with pre-roll/watchdog rather than defer to full synthesis feature."); **PASS requires all four stages in order**.

Review findings:

- The requirement is **unambiguous**: the bound (≤3 s), the start event (Play press on a parsed document, cold start, independent of any prior pre-roll), the end event (first *audible* audio), the endpoint (the transducer — explicitly "not adapter output"), the measured chain (adapter mono f32 @ 22050 Hz → player device-format conversion → speaker), and the device class (`001/plan.md:50-53`: "mid-range arm64, 4x Cortex-A78-class cores, 6 GB RAM or equivalent, document already parsed … a faster device passing is conforming, a slower device failing is not evidence of failure") are all named.
- It is **testable only at the transducer endpoint**: measuring the adapter's output (e.g. the `synthesize` CLI subcommand) would skip the player device-format conversion stage and the speaker, violating `spec.md:115`'s explicit "endpoint is transducer, not adapter output". The measurement chain is corroborated by `checklists/audio-safety.md` CHK002 (RESOLVED 2026-10-07: "measurement chain (Play → first AudioChunk → player device-format conversion → speaker) explicit") and CHK008 (device-format conversion time included in the measurement, stated explicitly).
- Verdict: the requirements-review stage discharges on this host. **PASS.**

## Stage 2 — SC-009 benchmark — **BLOCKED (named blocker)**

The benchmark **cannot be executed on this host**. Named blockers:

1. **No reference device.** The reference device of `001/plan.md:50-53` (mid-range arm64, 4x Cortex-A78-class cores, 6 GB RAM or equivalent) does not exist in this environment, and it is *normative* for SC-009: "All '10 out of 10 trials' latency criteria (SC-009, SC-054, SC-050-idle-resume) are measured on this class of device" (`001/plan.md:51-53`; on-device 10/10 trials incl. "latency (≤3 s play-to-audio)" per `001/plan.md:43-44`; the 10/10-trial protocol is cross-referenced at `001/spec.md:1325-1326` per `checklists/audio-safety.md` CHK002).
2. **The Android aarch64 build cannot be produced on this host.** T036 B1: `alsa-sys v0.4.0` cannot build — `pkg-config exited with status code 1` (`libasound2-dev` absent), evidence `alsa-sys-0.4.0/build.rs:13:19` — which defeats `cargo nextest run --workspace` via `crates/player` and constitution gate 9 (the Android build). T036 B2: `ndk-sys 0.6.0` is Android-only — `compile_error!("ndk-sys only supports compiling for Android")`, evidence `ndk-sys-0.6.0+11769913/src/lib.rs:22:1` — defeating any workspace-wide build on this host (`tasks.md:100-120`).
3. **The transducer endpoint does not exist here.** This is a headless host with no speaker and no audio-output measurement path. Even with T044's engine demonstrably working on this host (stage-2 input observed: `audio_chunk::a_bounded_segment_delivers_a_conforming_non_empty_mono_buffer` PASS `[103.424s]`), that measures the **adapter's output**, which `spec.md:115` explicitly excludes as the endpoint. The player device-format conversion stage is unbuilt here (`crates/player` does not compile, B1) and the speaker stage is physically absent.

**What the harness WOULD measure** (so a future run on the reference device can execute it): wall-clock **press→first-audible** latency, from Play-press dispatch to the first audible sample at the **transducer**, across the full pipeline **in stage order** — adapter mono f32 @ 22050 Hz → player device-format conversion → speaker — on a parsed document, cold start, with pre-roll (`FR-015a`) beginning on the press but the ≤3 s bound applying to first audible after the press independently of any prior pre-roll (`spec.md:115`); 10/10 trials on the reference device class (`001/plan.md:43-44,50-53`); bound ≤3 s; warm-resume (pre-roll buffered) is **not** part of this benchmark — it is governed separately by `SC-020` (≤4 s).

**Where it would live**: an **on-device benchmark wired to the pipeline endpoint** — instrumented at the transducer side of the player output path on the reference device (e.g. an on-device trial harness that timestamps the Play dispatch and the first audible sample at the speaker), *not* at the adapter's `synthesize` CLI subcommand (which stops at the adapter's output; the CLI surface is a host-side smoke check of synthesis latency only, not the SC-009 measurement).

**Unblock condition (reference-device requirement stated explicitly)**: (a) the reference device of `001/plan.md:50-53` (mid-range arm64, 4× Cortex-A78-class cores, 6 GB RAM or equivalent, document already parsed); (b) the gate-9 Android build produced (`cargo tauri android build --target aarch64`) — currently impossible on this host (B1/B2); (c) T044 attested as the working engine (binding present in the working tree, attestation pending — see header); (d) the press→first-audible harness wired to the transducer endpoint on that device.

**Explicit non-classifications** (per the T047 mandate and `constitution.md:296-297`):

- **Not a PASS** — no measurement exists; a gate that passes when its input is absent is not a gate (`constitution.md:296-297`).
- **Not DEFERRED** — `spec.md:17` mandates pre-roll/watchdog adaptation only *if the benchmark runs and fails*; the benchmark has not run at all, so the adaptation spec does not exist and no DEFERRED reclassification is admissible. Reclassifying as DEFERRED without the pre-roll/watchdog adaptation `spec.md:17` mandates would launder a missing input into a schedule decision.

Verdict: **BLOCKED (named blocker)** — the missing input (reference device + transducer endpoint) fails the gate.

## Stage 3 — Mechanism-preservation verification — **PARTIAL** (adapter-side invariants: PASS-on-host-evidence; device-surface behaviors: NOT-HOST-VERIFIABLE)

Evidence base: the two nextest runs quoted in the header. nextest enumerates every failure; every test not listed among the 3 failures passed. The 14 stage-3 evidence tests below were additionally re-observed green in the targeted run (`Summary [ 203.411s] 14 tests run: 14 passed (1 slow), 92 skipped`).

### SC-051 preserved — **PASS-on-host-evidence** (selector invariant) / **NOT-HOST-VERIFIABLE** (on-device mechanism drill)

Host-verifiable, observed green:

- `adapter-synthesis::unit_failure sc_051_registration_makes_offline_default_the_sole_selectable_tier` — PASS `[0.075s]` (`tests/unit_failure.rs:529`): pins `REGISTERED_TIERS == [EngineTier::OfflineDefault]`, an exhaustive 5-arm `is_selectable` match over `EngineTier::ORDER` (no `_ =>`, so a sixth tier member fails to compile), `!is_selectable(EngineTier::TestDouble)` while the adapter is registered (`001/spec.md:89`), and the fallback chain collapsing to the single `offline_default` tier — active mid-document switch unavailable by construction.
- `adapter-synthesis::unit_failure tier_at_failure_is_always_the_registered_offline_default` — PASS `[0.080s]` (`tests/unit_failure.rs:416`).
- `adapter-synthesis::unit_failure engine_error_is_closed_at_seven_members_with_no_adapter_failed` — PASS `[0.103s]` (`tests/unit_failure.rs:251`): the 7-member `EngineError` closed set, no `AdapterFailed` variant.

NOT host-verifiable: the on-device mechanism drill — adapter fails mid-document → paused-with-stated-reason (`FR-023a`), 3-second offer expiry (`FR-041`), no auto-resume (`FR-014`), notification carries the failure reason (`FR-037`). These are Android-surface behaviors requiring the device and a compiling `crates/player` (B1).

### SC-035 named failure — **PASS-on-host-evidence** (named-failure surface) / **NOT-HOST-VERIFIABLE** (player-side sole-conversion behavior)

Host-verifiable, observed green:

- `adapter-synthesis::audio_chunk a_stereo_delivery_names_channel_mismatch` — PASS `[0.099s]` (`tests/audio_chunk.rs:386`)
- `adapter-synthesis::audio_chunk a_wrong_rate_delivery_names_rate_mismatch` — PASS `[0.142s]` (`tests/audio_chunk.rs:414`)
- `adapter-synthesis::audio_chunk a_wrong_width_delivery_names_rate_mismatch` — PASS `[0.185s]` (`tests/audio_chunk.rs:446`)
- `adapter-synthesis::audio_chunk a_non_finite_duration_fails_closed_as_a_rate_mismatch` — PASS `[0.120s]` (`tests/audio_chunk.rs:352`)
- `adapter-synthesis::prop_format the_engine_error_closed_set_is_exactly_the_seven_members` — PASS `[0.115s]` (`tests/prop_format.rs:113`)
- `adapter-synthesis::prop_format a_rate_deviation_alone_names_rate_mismatch` — PASS `[0.163s]` (`tests/prop_format.rs:174`)
- `adapter-synthesis::prop_format a_channel_deviation_alone_names_channel_mismatch` — PASS `[0.205s]` (`tests/prop_format.rs:191`)
- `adapter-synthesis::prop_format a_width_deviation_names_rate_mismatch_because_the_set_has_no_width_variant` — PASS `[0.126s]` (`tests/prop_format.rs:209`)
- `adapter-synthesis::unit_failure every_engine_error_member_is_reachable_through_the_classification_table` — PASS `[0.086s]` (`tests/unit_failure.rs:692`)

NOT host-verifiable: the player-side half of SC-035 — that the `player` device-format conversion is the *sole* conversion step and is never a silent resample/downmix (`spec.md:384-389`, `FR-024b`). `crates/player` does not compile on this host (B1).

### SC-027 position verified — **PASS-on-host-evidence** (echo + alignment anchors) / **NOT-HOST-VERIFIABLE** (transport advance, resume, replay)

Host-verifiable, observed green:

- `adapter-synthesis::unit_failure failure_session_position_echoes_the_segment_source_range` — PASS `[0.118s]` (`tests/unit_failure.rs:453`): `EngineFailure.session_position` echoes the segment's `resume_point` (falling back to `source_range`) — an echo, never adapter-computed (`spec.md:106`; segment-resume.md CHK030 closure).
- `adapter-synthesis::audio_chunk the_failure_session_position_echoes_the_segment_boundary` — PASS `[203.398s]` (`tests/audio_chunk.rs:631`; slow — real synthesis).
- Segmentation/resume-alignment properties, green in the full run (not among the 3 enumerated failures): `prop_chunk::max_chunk_bytes_is_exactly_5000` and `prop_chunk::every_segment_stays_within_the_5000_byte_budget` (`tests/prop_chunk.rs:56,79` — the byte-budget test `constitution.md:353-354` requires); `prop_chunk_refined::paragraph_boundary_takes_precedence_over_sentence_boundaries_within_it` (`tests/prop_chunk_refined.rs:375`); `prop_segmentation_shapes::atx_headings_are_headings_and_absorb_their_body_lines` (`tests/prop_segmentation_shapes.rs:365`); `prop_segmentation_shapes::budget_floor_at_max_utf8_char_width_keeps_bound_and_losslessness` (`tests/prop_segmentation_shapes.rs:492`). These make the resume anchors (nearest completed paragraph/heading boundary, never re-split) decidable from the document alone.

NOT host-verifiable: actual playback-position advance through delivered chunks, resume at the nearest completed paragraph/heading on regain, replay after unexpected end bounded ≤30 s (`FR-015b` / `FR-004` / `SC-007`), and `blake3` fingerprint invalidation on edit (`FR-036`) — these are `crates/player` + `crates/session` behaviors; `crates/player` does not compile on this host (B1), and the position verification named in `spec.md:116` is an on-device criterion.

### FR-028 notification integrity — **NOT-HOST-VERIFIABLE** (no host evidence exists)

The notification is an Android surface (Kotlin `MediaSession` mirror; title + current section title + progress only — no body text, elapsed/remaining, rate, or voice; determinate iff total known at session start, else indeterminate for the whole session — `AGENTS.md` "Constraints agents get wrong"). No test in the adapter suite covers it — the adapter crate has no notification surface — so there is **no host evidence at all** for this sub-item. What exists on this host is only the *requirements-quality* claim: `checklists/audio-safety.md` CHK005/CHK013/CHK030 are marked `[x]`, and that checklist's own Note states the `[x]` marks approve the requirement text, not the behavior ("It does not verify `SC-009` benchmark results … those are verification / gate activities"). The behavioral verification belongs to the on-device 10/10 trials (`001/plan.md:43-44`).

Stage-3 verdict: **PARTIAL** — the adapter-side invariants (SC-051 selector invariant, SC-035 named-failure surface, SC-027 echo/alignment anchors) PASS on host evidence; the device-surface half of every sub-item (and all of FR-028) is NOT-HOST-VERIFIABLE and is therefore not discharged.

## Stage 4 — Discharge-readiness assessment — **WRITTEN on host** (the assessment discharges as a written artifact); the gate as a whole is **NOT PASSED**

The seven SC-044 discharge prerequisites, quoted from `spec.md:120` and stated with their status at recording time:

> **SC-044 discharge prerequisites (explicit, edit 4)**: (1) second selectable real engine registered; (2) consent acquisition surface built (`writer` / `UI` / `verb` / `storage key` / `T075`); (3) `test_double` admissible only when *no* adapter registered (`001/spec.md:89`; `closed.rs:305-312`); (4) `SC-051` mechanism preserved continuously (not degraded); (5) `SC-027` / `FR-036` resume verified; (6) `SC-009` benchmark passes; (7) `SC-035` named-failure verified. All must be stated before PASS; with adapter registered, only preservation (not discharge) is verifiable in v1.

1. **Second selectable real engine registered — NOT MET.** Only `offline_default` is registered (`REGISTERED_TIERS == [EngineTier::OfflineDefault]`, pinned green by `sc_051_registration_makes_offline_default_the_sole_selectable_tier`).
2. **Consent acquisition surface built — NOT MET, and deliberately deferred.** Consent acquisition is out of Option A scope: "consent acquisition deferred (no writer/UI/verb/storage key; out of Option A scope per `spec.md:138`); SC-044 stays `DEFERRED` (not close on `test_double` …)" (`spec.md:22`, clarification C3-A split).
3. **`test_double` admissible only when no adapter registered — NOT MET (inadmissible).** The adapter is registered, so `test_double` is inadmissible per `001/spec.md:89`: "`test_double` is admissible only while no non-test_double adapter is registered, which is exactly the FR-024 condition that makes it admissible at all; it sorts after every real tier so a stand-in can never outrank a real engine…". Pinned green: `!is_selectable(EngineTier::TestDouble)` inside `sc_051_registration_makes_offline_default_the_sole_selectable_tier`.
4. **`SC-051` mechanism preserved continuously (not degraded) — MET on host evidence** (stage 3, SC-051 PASS-on-host-evidence); on-device drill outstanding.
5. **`SC-027` / `FR-036` resume verified — PARTIAL** (host evidence for the echo and alignment anchors; on-device transport/resume/replay verification outstanding).
6. **`SC-009` benchmark passes — NOT MET: BLOCKED** (stage 2).
7. **`SC-035` named-failure verified — MET on host evidence** (stage 3, SC-035 PASS-on-host-evidence); player-side conversion behavior outstanding.

Consent deferred: per `spec.md:22` (C3-A split) — SC-044 stays `DEFERRED`, not closed on the `test_double` branch; `test_double` inadmissible once the adapter is registered per `001/spec.md:89`. `SC-044` remains `DEFERRED` per `constitution.md:643` (`gate-8-full: DEFERRED, owner=synthesis`).

Verdict: the discharge-readiness assessment **can be written on this host** — it is a requirements-statement exercise, and all seven prerequisites are stated above with met/not-met status — but the overall SC-009 gate **remains BLOCKED at stage 2**, so the gate as a whole is **NOT PASSED**. Per `spec.md:116`, "PASS requires all four stages in order"; stage 2 did not execute, so no PASS exists to confer. "`SC-044` deferred is a valid PASS condition for v1" applies only when the other three stages pass; it does not rescue a blocked stage 2.

---

## Overall verdict

**SC-009: BLOCKED (named blocker — reference device required); stages 1, 3, 4 partially dischargeable on host; stage 2 not executable here. Not a pass, not a deferral.**

Consequences recorded for the next agent: (i) stage 2 unblocks only on the reference device of `001/plan.md:50-53` with a gate-9 Android build and the press→first-audible harness wired to the transducer endpoint; (ii) T044 owns the three stale-premise test rewrites (`unit_failure::synthesis_fails_closed_with_a_named_init_failure`, `cli_protocol::synthesize_subcommand_fails_closed_with_nothing_on_stdout`, and the `synthesis_never_leaks_document_prose_into_stdout` proptest's case count/timeout budget) before V1''/T055 can attest green; (iii) the T036 host blockers B1/B2 remain unrepaired and are 001/workspace-config owned.

---

## Addendum 2026-10-11 — Stage 2 EXECUTED on hardware (FAIL, named result)

**Recorder**: orchestrator session (user-operated device). Method: in-app Play press → first audible at the phone speaker, stopwatch-operated, 5 cold-start runs (app force-closed between runs, screen on, in-app button — lock-screen path not yet implemented, so the press event is the WebView button, not the notification).

**Device** (below reference class — see caveat):
- Model Redmi 2201117TY (Redmi Note 11), SoC Snapdragon 680 (`bengal`: 4× Cortex-A73 + 4× A53 — no A78 core), 5.8 GB RAM, Android 13 (API 33)
- Reference class per `001/plan.md:50-53`: 4× Cortex-A78-class, 6 GB RAM. This device is slower on both axes.

**Results** (press→first-audible, cold start):

| Run | 1 | 2 | 3 | 4 | 5 |
|-----|---|---|---|---|---|
| s   | 9 | 9 | 9 | 9 | 9 |

**Verdict: FAIL — bound ≤3 s exceeded 3×, 5/5 runs, zero variance.**

**Reading of the number**: the 9.0 s consistency across cold starts points at a one-time-per-launch cost (63 MB model load + first single-threaded ONNX inference on small cores), not run-to-run variance. The pipeline itself is proven working on device (audible pt-BR speech, correct segments); only the latency bound is unmet.

**Device-class caveat** (this record's own Stage-1 rule, line 38): "a faster device passing is conforming, a slower device failing is not evidence of failure". This device is below the reference class, so the FAIL does not by itself prove the gate unpassable on reference hardware — but it is a measured result, not an absent input, and per `spec.md:17` a failed benchmark mandates **pre-roll/watchdog adaptation, never silent deferral**.

**Adaptation pointer (not implemented)**: warm the engine before the press — load the model at app start and pre-synthesize the first segment at document open (pre-roll `FR-015a` already begins on the press; moving load+first-inference ahead of it leaves only stream start + device conversion in the measured window). Watchdog: named `InitFailed`-style timeout if first-audible exceeds the bound. Both are future tasks, not claimed here.

**Stage-2 status after this addendum**: EXECUTED — FAIL on below-reference hardware; gate as a whole remains NOT PASSED. Prior BLOCKED verdict is superseded by this measurement for the executed portion only; the reference-device measurement is still outstanding.
