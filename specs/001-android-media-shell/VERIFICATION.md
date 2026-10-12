# VERIFICATION — `001-android-media-shell`

Attestation report (T087). Only the Phase-8 verification tasks
(T084–T087) attest success; no implementation task attests itself.

- **Feature**: `001-android-media-shell` (branch `001-android-media-shell`,
  tracked on `main`)
- **Host**: Linux x86_64, JDK 25, Rust 1.85+ (Edition 2024), cargo-nextest
  0.9.133, Android SDK (NDK 29.0.13846066, platform android-36,
  build-tools 36.0.0), `aarch64-linux-android` target installed
- **Pinned**: `tauri = 2.12.1` (host CLI is tauri-cli 3.0.0-alpha.3 — the
  generated `gen/` toolchain was realigned to the 2.12.1 template; see §8)
- **Attestation rule**: a criterion is PASS only where the named evidence was
  executed on this host and is cited below; anything not executed here is
  DEFERRED with a named owner, never claimed as PASS.

Evidence logs (this session): full gate run
`/tmp/opencode/verify-t084.log` (exit 0), gate-9 build `/tmp/opencode/gate9b.log`
(exit 0), purity inventory `/tmp/opencode/purity-t086.log`.

---

## 1 · Gate summary (T084)

| Gate | Verdict | Evidence |
|---|---|---|
| 1 format | PASS | `cargo fmt --all -- --check` exit 0 |
| 2 tests | PASS | `cargo nextest run --workspace`: 525 run, 525 passed (7 slow), 1 skipped |
| 3 documentation | PASS | `cargo doc --workspace --no-deps` exit 0 |
| 4 lint-drift | PASS | two-way drift check against the pinned T001 set |
| 5 supply-chain | PASS | `cargo deny check advisories licenses bans sources` ok; `cargo audit` exit 0 (2 quality advisories as allowed warnings — Exceptions Log entry 10 + the Linux-gtk-chain-only glib advisory, both absent from the Android artifact) |
| 6 deny-list | PASS | no denied crate in any manifest or the lockfile |
| 7 closed-set | PASS | no `_ =>` in `crates/*/src` (comment-stripped scan) |
| 8 shell | PASS | `check-shell-purity.sh` exit 0 — conformance mapping documented in-script |
| 8 full | **DEFERRED** (owner = synthesis feature) | the full on-device conformance suite; reported DEFERRED, never claimed as PASS — constitution gate 8 remains OPEN |
| 9 device build | PASS | `cargo tauri android build --target aarch64` exit 0 → `app-universal-debug.apk` (270 MB; `lib/arm64-v8a/libsrc_tauri.so` 154 MB + `libonnxruntime.so` + `libsherpa-onnx-*.so`) and `app-universal-debug.aab` |

---

## 2 · Host-decided criteria — PASS (gate references)

| SC | Requirement | Verdict | Evidence |
|---|---|---|---|
| SC-006 | Foreground service declaration + permissions | **PASS** | `scripts/check-manifest.sh` exit 0 — asserts `<service android:foregroundServiceType="mediaPlayback">` + `FOREGROUND_SERVICE` + `FOREGROUND_SERVICE_MEDIA_PLAYBACK` + `minSdk(Version) 29` across all three declaration forms (Kotlin DSL `app/build.gradle.kts`, JSON `tauri.conf.json`, XML `AndroidManifest.xml`) |
| SC-015 | Pitch preserved at all 7 rungs incl. 2.0×→4.0× | **PASS** | `engine-core::prop_pitch` 9/9 (stand-in A and B each: declared shape at every rung, pitch held across all seven rungs, pitch held across the 2→4 pair; pre-roll bound; spectral-peak and zero-crossing fundamental recovery) — inside the 525 |
| SC-032 | minSdk floor declared at API 29 | **PASS** (declaration) | `check-manifest.sh` SC-032 floor assertion, both committed (`tauri.conf.json`) and generated (`gen/android/app/build.gradle.kts`) forms. Device floor behavior (`SC-032-floor`, trial T-28) is in §5 |
| SC-035 | No transport/rate/position/buffering/focus-response logic in the platform surface | **PASS** | `scripts/check-shell-purity.sh` exit 0 — forbidden call shapes (`\.position\s*=`, `\.rate\s*=`, `setPlaybackState\(.*position|state`, `seekTo\(`, `gain\s*[*/+-]?=`, `duck\w*\s*=`, `skip(Forward\|Backward)\s*\(`) absent everywhere; vocabulary confined to the 10-file allowlist in type/getter/delegation positions; the 149-occurrence residue is adjudicated in §7 (T086) |
| SC-056 | Zero document text in the record; `detail` absent or ≤200 chars non-prose; every `failure`/`refusal` carries `error` | **PASS** | `diag::prop_no_prose` (invariants S1–S3) — inside `cargo nextest run -p diag` 26/26 |
| SC-057 | Closed sets have no catch-all/other/internal/unknown; engine ids resolve to the constitution registry; `test_double` restricted to `standin_a`/`standin_b` and admissible only while no non-`test_double` adapter is registered | **PASS** | `check-shell-purity.sh` SC-057 section exit 0 — closed-set scan over comment-stripped sources; engine-id registry resolution; the single documented exemption is `rust-player` (`crates/player/src/cli.rs` — a player-side playback-backend self-identifier, not a TTS synthesis engine: never in the fallback order, no `engine_tier`, never in the diagnostic record's engine field) |

---

## 3 · Zero-occurrence criteria — PASS (record scans + canary evidence)

Executed as `cargo nextest run -p diag` (26/26 passed, 0 skipped) and
`cargo nextest run --workspace` (525/525). T068b's canary suite
(`crates/diag/tests/prop_scanner_canary.rs`) is the evidence that the scans
can fail at all — each property plants exactly one non-conforming record in
an otherwise conforming proptest-generated background and asserts a NON-EMPTY
report in which every violation is the canary's own kind; the control asserts
conforming records report ZERO. A scan that has never been shown to reject a
planted violation is not evidence of zero occurrences.

| SC | Requirement | Verdict | Evidence |
|---|---|---|---|
| SC-005 | Zero `transport:"playing"` entries with no strictly-greater `position_secs` follow-up | **PASS** | Scan: `diag::scan` `PlayingWithoutAdvance` (S4), executed one-pass over the JSONL by `diag::scan::scan_record`. One-directional property: `diag::prop_no_prose::every_playing_entry_is_followed_by_strictly_greater_position` (green). Canary C1: `diag::prop_scanner_canary::canary_playing_without_advance_is_detected` — planted `transport:"playing"` with no greater follow-up reports non-zero, classified exactly `PlayingWithoutAdvance`. Control: `diag::prop_scanner_canary::conforming_records_report_zero_violations` (zero). Writer-side guarantee: every transport change carries `position_secs` (schema-enforced, `diag::contract_schema`), and the player's transport math only emits playing transitions that advance (the rewind/snap/transport properties inside the 525). On-device record pull (trials T-34/T-35/T-45/T-46) is in §5 |
| SC-012 | Zero audio starts without a user action, in any recovery path | **PASS** | Decided behaviorally by the offer machine, which is the only path to `Playing`: `session::prop_offer::given_an_unexpected_end_…_never_auto_audio` (unexpected end → paused with an explicit resume, never auto audio); `…_with_no_foreground_latch_…` and `…_foregrounded_then_backgrounded_…` (the 3-second offer converts to an explicit resume when the foreground latch is absent or withdrawn); `…_with_the_application_foregrounded_…` (the single permitted auto-start — the FR-032c deliberate-pause reopen after its 3-second offer expires with the application foregrounded — a latch read at expiry only, never an early conversion). The FR-023a fallback continuation is a continuation of an already-started session, not a session start. Record form: every `session_start` conforms to the contract schema (`diag::contract_schema`) and every recorded value is closed-set-constrained; the record scanner's canary suite (C1–C7) proves the scanner rejects planted violations. Provenance is a property of the event chain, not a record field, so the property tests above are the scan that can fail — they assert the negative (never auto audio) |
| SC-019 | Zero resume offers for Stopped or completed documents | **PASS** | Behavior: `session::prop_offer::given_an_open_offer_when_the_document_completes_…_no_offer_and_no_residency`; `…_when_a_new_session_ends_terminally_…_no_offer_remains`; `session::prop_resume::a_terminal_end_reopens_at_the_first_spoken_position_with_no_offer` and `…_the_entry_is_deleted_and_the_reopen_starts_at_zero_with_no_offer` — the offer form resolves solely from the stored `end_kind`, and `stopped`/`completed` never resolve to an offer. Record scan: `diag::prop_no_prose::no_session_end_carries_playing` (S5 — a `session_end` never carries `transport:"playing"`, so a Stopped/completed document can never be recorded as mid-playback); canary C6: `diag::prop_scanner_canary::canary_session_end_with_playing_is_detected`; control C8 zero |

---

## 4 · Surface criteria — host legs PASS, device leg DEFERRED

| SC | Requirement | Verdict | Evidence |
|---|---|---|---|
| SC-010 | Zero document body text in the notification / lock screen / shade; only title + section title + progress (+ controls) | **PARTIAL** — host legs PASS; device leg DEFERRED | Host leg: `cargo nextest run -p player --test prop_surface` 11/11 — the surface carries exactly title + section + progress + controls, never body/elapsed/remaining/rate/voice; section = most-recent-passed else absent; determinacy fixed at session start; title from metadata else the stable filename placeholder, never the first sentence. Static leg: body-text render-path grep (`BigTextStyle\|InboxStyle\|setBigText\|setBigContentTitle\|bodyText\|documentText\|docBody\|prose\|spokenText\|currentSentence\|paragraphText`) over `android-shell/` + `src-tauri/src/` returns **zero hits**. Device leg (locked visual check 10/10 across a boundary crossing, a resume, a skip in each direction, an engine change) is in §5 |
| SC-037 | Zero instances of the application presenting, listing, or exporting the diagnostic record | **PARTIAL** — static + cross-reference legs PASS; manual-absence walk DEFERRED | Static leg: record-read/export grep (`diag(record)?\|session\.record\|export\|share\|read.*record\|record.*(read\|view\|list)`) over `android-shell/` + `src-tauri/src/` returns zero hits that present, list, read, or export the record — every match is a false positive: "diagnostic *intents*" (the `control_*_unmapped` / `lifecycle_unmapped` intent vocabulary, a different meaning of "diagnostic"), `Context.RECEIVER_NOT_EXPORTED` (a non-export flag — the opposite of export), "shared" (matched by `share`), Rust code re-exports of declarations, and the `diagnose` command (see §9). Cross-reference leg: the only record reader is the developer tooling in `crates/diag` (T074 reader CLI + scanner), reachable via `crates/diag` CLI or `adb shell run-as`; the T081 allowlist contains no record-read or export path. Manual-absence walk (every Tauri view, settings surface, lock-screen/shade affordance) is in §5 |

---

## 5 · On-device trials — DEFERRED (no API-29+ hardware on this host)

The 48 trial-runnable procedures (T-01–T-48, each 10/10 with exact bounds)
and the FR-026 offline-start procedure are written and execution-ready in
`tests/device/trials.md` but cannot be executed here — no API-29+ device is
attached. Carried forward as hardware debt to a CI host with an API-29+
device. SC-051 (two-stand-in failover drill) is included in this deferral;
SC-044 is explicitly excluded (see §6). The host-side mechanisms these
trials exercise are green (the 525: focus duck/pause, routing, recovery,
offer machine, failover selection, pitch, transport math, fingerprint,
trim bounds, closed-set exhaustiveness).

| SC | Trial | SC | Trial | SC | Trial |
|---|---|---|---|---|---|
| SC-001 | T-01 | SC-023 | T-19 | SC-045 | T-38 |
| SC-002 | T-02 | SC-024 | T-20 | SC-046 | T-39 |
| SC-003 | T-03 | SC-025 | T-21 | SC-047 | T-40 |
| SC-041 | T-04 | SC-026 | T-22 | SC-048 | T-41 |
| SC-055 | T-05 | SC-027 | T-23 | SC-049 | T-42 |
| SC-004 | T-06 | SC-028 | T-24 | SC-050 | T-43 |
| SC-007 | T-07 | SC-029 | T-25 | SC-050b | T-44 |
| SC-008 | T-08 | SC-030 | T-26 | SC-051 | T-45 |
| SC-009 | T-09 | SC-031 | T-27 | SC-052 | T-46 |
| SC-011 | T-10 | SC-032 (floor) | T-28 | SC-053 | T-47 |
| SC-013 | T-11 | SC-033 | T-29 | SC-054 | T-48 |
| SC-014 | T-12 | SC-033b | T-30 | FR-026 | S-3 |
| SC-016 | T-13 | SC-034 | T-31 | | |
| SC-017 | T-14 | SC-036 | T-32 | | |
| SC-018 | T-15 | SC-038 | T-33 | | |
| SC-020 | T-16 | SC-039 | T-34 | | |
| SC-021 | T-17 | SC-040 | T-35 | | |
| SC-022 | T-18 | SC-042 | T-36 | | |
| | | SC-043 | T-37 | | |

Also deferred with the device legs: the SC-010 locked visual check (§4),
the SC-037 manual-absence walk (§4), and the on-device record pull that
re-confirms the §3 zero-occurrence scans over a real device record
(trials T-34/T-35/T-45/T-46).

---

## 6 · Carried forward — DEFERRED with named owners

| Item | Verdict | Owner | Note |
|---|---|---|---|
| SC-044 (FR-023b) | **DEFERRED** | synthesis feature (`002-fr-024-synthesis-adapter`) | Real engines do not exist in this feature — FR-024 places synthesis in the 002 feature, so the hardware evidence (one offline + one cloud engine, mid-document failover with no restart, second failure → paused with a stated reason) cannot be produced here. The recovery *mechanism* is discharged now against the two contract-honouring stand-ins by SC-051 (T-45, itself device-deferred). Logged as carried-forward hardware debt per T086's task text |
| Constitution gate 8 (full) | **DEFERRED** | synthesis feature | Reported by `verify.sh` as `gate-8-full: DEFERRED (owner=synthesis)` — never claimed as PASS; gate 8 remains OPEN. Only gate-8-shell (the conformance-mapping subset) is green |

---

## 7 · T086 human-review residue adjudication (SC-035)

The T081 purity gate flags 149 occurrences of
`transport|position_secs|rate|gain|skip` in the 10-file allowlist. No grep
can decide whether an occurrence is a type/getter/delegation position or a
computation, so each was read individually (Principle V — a gate that passes
on judgement is not a gate; the judgement is recorded here, never automated).

**Verdict: all 149 occurrences are type, getter, delegation, doc-comment, or
test-assertion positions — zero computations.** No `mut` state, no arithmetic
on position/rate/transport, no gain application, no skip math. The single
arithmetic in the residue is the sanctioned seconds→millis unit conversion of
the published `position_secs` in `MediaMirror.kt`'s mirror output (the
platform's display unit), not a computation on playback position.

| File | Occurrences | Classification |
|---|---|---|
| `android-shell/…/MediaMirror.kt` | 64 | `MirrorState` field declarations (type positions), constructor parameters, the sanctioned seconds→millis unit conversion of the published `position_secs`, test assertions |
| `src-tauri/src/status_view.rs` | 46 | Struct field declarations, constructor parameters, test assertions, JSON wire-form literals in test fixtures — all mirror positions over the read-only view |
| `src-tauri/src/lib.rs` | 13 | 7 doc-comment prose + 6 delegation positions (`"position_secs": snap.position_secs`, `"transport": snap.transport.to_string()` in the three mirror payload constructors) |
| `android-shell/…/Residency.kt` | 10 | All doc-comment prose |
| `src-tauri/src/failure.rs` | 6 | 3 doc-comment prose + 3 test call sites (`render_failure(&EngineError::RateMismatch, …)`, `assert_ne!` on rendered statements) |
| `src-tauri/src/surface.rs` | 4 | Doc-comment prose on the `EngineUnreachable`/`EngineRateMismatch` variants of both reason enums |
| `android-shell/…/PlaybackService.kt` | 4 | 1 doc-comment prose + 1 intent-extra key constant (`EXTRA_POSITION = "position"`) + 1 delegation read (`intent.getDoubleExtra(EXTRA_POSITION, 0.0)` — reads a Rust-supplied extra and only logs it) + 1 log statement |
| `android-shell/…/RoutingMirror.kt` | 2 | Doc-comment prose |
| `android-shell/…/FocusMirror.kt` | 0 | — |
| `src-tauri/src/permissions.rs` | 0 | — |

---

## 8 · Gate-9 toolchain note (host-local, git-ignored `gen/` only)

The `gen/android` project on this host was generated by the alpha CLI
(tauri-cli 3.0.0-alpha.3) with **AGP 9.0.0 + KGP 2.1.0 + Gradle 9.8.0** — an
internally inconsistent toolchain: KGP 2.1.0 cannot coexist with AGP 9's
built-in Kotlin extension, so applying `org.jetbrains.kotlin.android` in
tauri 2.12.1's own `mobile/android/build.gradle.kts` failed with
`Cannot add extension with name 'kotlin', as there is an extension already
registered with that name`.

Fix: realigned the git-ignored host build config to the **tauri-cli 2.12.1
template's tested combination** — AGP 9.3.1 + KGP 2.2.10 + Gradle 9.6.1 +
`android.builtInKotlin=false` + `android.newDsl=false` (the template's
`gradle.properties` flags that disable AGP 9's built-in Kotlin so the
explicit KGP application is the single registration). The template's own
library module ships `compileSdk = 36`, proving AGP 9.3.1 accepts the
installed SDK 36 — no platform download was required. The pinned
`tauri = 2.12.1` dependency, `src-tauri/tauri.conf.json` (minSdk 29), and
the manifest are untouched; `gen/` is git-ignored build output. Gate 9 then
passed (exit 0, device artifact produced).

**Gate-9 regression found and fixed (2026-10-12, `ff6da8b`)**: inspecting
the produced `app-universal-debug.apk` showed its dex carried only
`R`/`BuildConfig` — **no app classes at all** (no `MainActivity`, no
`PlaybackService`, none of the five mirrors): the app would crash on
launch, so the exit-0 artifact was hollow. Two causes, both latent
because `android-shell/` had never been part of any Gradle source set
(the exact dead-code hazard T028b names):

1. The alpha-CLI-generated `app/build.gradle.kts` applied no Kotlin
   plugin — it was written for AGP 9's built-in Kotlin, which the
   toolchain fix disabled (`android.builtInKotlin=false`), silently
   leaving the `:app` module without a Kotlin compiler. (The Oct 11
   release APK, built before the realignment, still carried the
   classes; the post-realignment debug APK did not.) Fixed by matching
   the tauri-cli 2.12.1 template: `import
   org.jetbrains.kotlin.gradle.dsl.JvmTarget`,
   `id("org.jetbrains.kotlin.android")` in the plugins block,
   `compileOptions` Java 1.8, and
   `kotlin { compilerOptions { jvmTarget = JvmTarget.JVM_1_8 } }`
   (host-local gen edit).
2. T028b's wiring had never been applied. `app/build.gradle.kts` now
   adds `android-shell/src/main/kotlin` as a `main` sourceSet
   (host-local gen edit), and the stray duplicate copy of
   `PlaybackService.kt` under `gen/.../java/app/mdreader/` (identical
   to `android-shell`'s, from the 347a493 experiment) was removed —
   with both roots in the source set it would be a duplicate class.
3. The first real compile of the mirrors caught a latent defect:
   `MediaMirror.kt:714` referenced `Notification.CATEGORY_MEDIA`, which
   does not exist in the Android framework (verified against the
   android-36 platform `android.jar`); MediaStyle notifications use
   `CATEGORY_TRANSPORT` (API 21+, minSdk 29 satisfied).

Re-verified after the fix: `compileUniversalDebugKotlin` and
`compileUniversalReleaseKotlin` both BUILD SUCCESSFUL; the gate-9 re-run
(`cargo tauri android build --target aarch64 --debug`) exited 0 and the
universalDebug dex now contains `MainActivity`, `PlaybackService`,
`FocusMirror`, `MediaMirror`, `Residency`, `RoutingMirror` plus the
generated Tauri plumbing, with `lib/arm64-v8a/libsrc_tauri.so`
unchanged; `check-shell-purity.sh` and `check-manifest.sh` both exit 0;
`verify.sh` re-run exit 0 (525 run / 525 passed, 1 skipped). The
tracked-file part of the fix is the `MediaMirror.kt` one-line category
correction; the gen wiring is git-ignored host-local build config,
recorded here and in the commit message.

---

## 9 · Observation (non-blocking, recorded for the 002 feature)

The Tauri bridge (`src-tauri/src/lib.rs`, `run_mobile_plugin`) dispatches
one frontend-facing command outside the shell-IPC closed verb set:
`"diagnose"` — a read-only voice-asset inventory (resolved resource dir;
model/tokens/espeak presence; file counts; model byte size). It never touches
playback, position, rate, or transport, and it does not read the diagnostic
record (verified for SC-037, §4). The shell-IPC contract's closed verb set
(`Play | Pause | Stop | SkipForward5s | SkipBack5s | RateUp | RateDown |
CycleVoiceNext | CycleVoicePrev | ResumeAtStored | StartOver |
ChooseEngine(id)`) governs the notification/MediaSession → Rust command
surface and is unchanged; the bridge currently exposes `play`/`pause`/`stop`
as literal arms, with the remaining verbs parsed in `player::cli::ShellVerb`
(the player CLI) per the one-code-path principle. Noted here so the 002
feature either documents the frontend command surface in the ipc contract or
folds the asset inventory into it.

---

## 10 · Attestation summary

- **PASS (host)**: SC-006, SC-015, SC-032 (declaration), SC-035, SC-056,
  SC-057 — 6 criteria, gate-referenced.
- **PASS (record scan + canary)**: SC-005, SC-012, SC-019 — 3 criteria,
  zero-occurrence scans executed via `cargo nextest run -p diag`, each backed
  by T068b canary evidence that the scans reject planted violations.
- **PARTIAL (host legs PASS; device leg DEFERRED)**: SC-010, SC-037 — 2
  criteria.
- **DEFERRED (hardware, procedures ready)**: the 47 remaining trial-runnable
  SCs (T-01–T-48 mapping in §5) + FR-026 + the device legs of SC-010,
  SC-037, SC-032-floor, and the §3 record pulls.
- **DEFERRED (owner = synthesis feature)**: SC-044; constitution gate 8
  (full).

Gates 1–7, 8-shell, and 9 are green on this host (exit 0). No criterion
outside the executed evidence above is claimed as PASS.
