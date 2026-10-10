# Research: Android Media Shell

**Feature**: `001-android-media-shell` | **Date**: 2026-10-02 | **Status**: all unknowns resolved

No NEEDS CLARIFICATION remains in the spec (checklist confirms zero markers); the unknowns
below are plan-level technical decisions. Each follows Decision / Rationale / Alternatives.

## R1. Tauri 2.12.1 Android plugin bridge

- **Decision**: Rust↔Kotlin over `PluginApi::register_android_plugin` +
  `PluginHandle::run_mobile_plugin`. Kotlin plugin class constructed `(Landroid/app/Activity;)V`.
- **Rationale**: Sanctioned Tauri 2 path; `PluginHandle` is the only Rust-side token for
  calling into Kotlin. `registerService` and `mobile::update_manifest` do not exist in
  Tauri 2 (only iOS `update_info_plist`), so no plugin API can declare the service.
- **Alternatives considered**: raw JNI without the plugin layer (rejected — bypasses
  `PluginManager` lifecycle); `tauri-plugin-background-service` (rejected — generic
  background service, not `mediaPlayback`-typed; manifest edit still required).

## R2. Foreground-service manifest declaration

- **Decision**: Manual `<service android:foregroundServiceType="mediaPlayback">` in the
  generated `AndroidManifest.xml` + `FOREGROUND_SERVICE`,
  `FOREGROUND_SERVICE_MEDIA_PLAYBACK` (+ `POST_NOTIFICATIONS` on API 33+) permissions;
  SC-006 automated check re-run on every Tauri upgrade.
- **Rationale**: Android 10+ treats background audio as user-visible work only under this
  declaration; Tauri regenerates the manifest, so only an automated check (not inspection)
  keeps the declaration from regressing (FR-007).
- **Alternatives considered**: declaring the `<service>` in a plugin manifest (rejected —
  the typed service belongs in the app manifest via merger/script, permissions may ride
  the plugin manifest).

## R3. Resident service + Android 14 start restrictions

- **Decision**: The playback `Service` itself stays resident paused between sessions
  (FR-009a: exactly one resident component, one notification); cold-start refusal on
  API 34+ (`ForegroundServiceStartNotAllowedException`) surfaces a stated reason with no
  playing state, and audio begins only after an app open (FR-010).
- **Rationale**: API 34 refuses `startForegroundService()` from the background; a live
  resident process is the only way a lock-screen Play press works with the app closed.
  Teardown at document end (FR-032f) bounds the battery cost.
- **Alternatives considered**: `WorkManager`/`AlarmManager`/`BOOT_COMPLETED` restart
  (all rejected — deferrable-work schedulers or alarm-restricted paths that cannot take
  a foreground service from the background on API 34+).

## R4. Rust PCM playback: cpal over oboe

- **Decision**: `cpal` 0.18.2, AAudio backend, mono f32 stream at the declared rate;
  `supported_output_configs()` exposes the device format; conversion to device format in
  the Rust playback module; resampling is the engine's job, never the module's
  (FR-033/FR-034/FR-043).
- **Rationale**: Production AAudio backend on `aarch64-linux-android`; matches the handoff
  contract exactly (engine resamples, module converts format). Mismatch (rate, channels,
  width) surfaces as a named engine failure (FR-035 + stereo edge case).
- **Alternatives considered**: `oboe` 0.6.x (rejected for now — built-in conversion is
  convenient but adds a C++ dependency and a less-mature Rust binding; revisit only if
  Rust-side conversion proves complex).

## R5. Focus classification + routing

- **Decision**: `AudioFocusRequest.Builder(AUDIOFOCUS_GAIN)` (API 26+); classify by the
  interrupter's request — `…_MAY_DUCK` → duck to ~20% and keep reading (FR-013),
  demand-silence (call/alarm) or permanent loss → pause, never auto-resume (FR-013a/FR-014);
  `ACTION_AUDIO_BECOMING_NOISY` → pause (FR-016); new-device-appears → follow switch at
  unchanged position + record output-device event (FR-017a), never private→speaker.
- **Rationale**: Classification by request (not by our reading) is what keeps audio out
  from under phone calls while leaving navigation prompts ducked; direction-of-travel
  (appearing vs disappearing device) separates deliberate acts from privacy events.
- **Alternatives considered**: single duck-everything policy (rejected — superseded by
  FR-013a: 20%-under-a-call is unintelligible and socially wrong); pause-everything
  (rejected — chops the document on every navigation prompt).

## R6. MediaSession as mirror + notification contract

- **Decision**: Rust is sole authority; `MediaSession.setPlaybackState()` reflects engine
  state, never computes position (FR-002/FR-003). `MediaStyle` notification shows title +
  current section title + progress only; `setOngoing(true)` while playing (FR-028e);
  transient reason surfaces expire in 3 s; resident paused surface becomes dismissable.
  Progress determinate only when the total is known at session start, else indeterminate
  for the whole session (FR-028a). Audible confirmation (FR-018a/SC-008): short earcon on
  every lock-screen press plus spoken state on resume/rate-change, emitted through the same
  Rust path the mirror reports; blind trials pass on earcon-or-speech, never on visual.
- **Rationale**: Two position authorities desynchronise notification and audio invisibly
  in tests and audibly in the car — the named defect FR-003 exists to prevent. Determinacy
  fixed at session start keeps a guess from ever presenting as fact.
- **Alternatives considered**: Media3-owned playback (rejected — constitution v1.2.0
  moved authority to Rust deliberately); teleprompter body text on lock screen
  (rejected — FR-028, driving hazard).

## R7. Position store: redb + blake3

- **Decision**: `redb` — `bookmarks` table (32-byte `blake3` content hash → offset,
  paused-at, sentence index); `settings` table (app-wide last rate, default 1.0x).
  Pause/close/kill/power-loss/network-loss preserve; Stop and document-end delete
  (FR-032/FR-032a/FR-032b); edited file = different hash = no resume (FR-036a).
- **Rationale**: ACID + crash-safe + pure Rust suits "survive process death" (FR-004);
  content hash survives move/rename and invalidates on edit by construction.
- **Alternatives considered**: `sled` (larger binary, less durability control); `rusqlite`
  (overkill for two KV tables, heavier on Android).

## R8. Rewind, snap, re-anchor, pre-roll

- **Decision**: Transient resume = cursor back `min(5% × paused-duration, 30 s)`, audio
  from that sentence's start, total backward movement hard-capped at 30 s (FR-015).
  Engine/voice change re-anchors to nearest sentence/node boundary, else document start
  (FR-038). Pre-roll ≤60 s ring, empty while idle, refilled on Play press; idle pause
  synthesises/bills nothing (FR-015a/FR-015c/FR-015b pipeline property).
- **Rationale**: Proportional term scales context with interruption length; sentence snap
  keeps resumes intelligible; the cap bounds both long pauses and long sentences; idle
  drain removes unbilled synthesis cost.
- **Alternatives considered**: fixed-second rewind (rejected — over-replays short ducks,
  under-replays long interruptions); continuous pre-roll while paused (rejected — bills
  per-character engines for unheard audio).

## R9. Rate ladder + pitch-preserving stretch

- **Decision**: Closed const ladder `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`, one step per
  press, saturate at ends (FR-019); stretch is an engine-contract operation the Rust
  module invokes, never platform/buffer stretching (FR-043); delivered rate invariant
  across rungs (SC-034); rate never shown on lock screen (FR-028).
- **Rationale**: Closed ladder is glance-operable without looking; engine-side stretch
  keeps the pitch guarantee (FR-019a) testable per engine and replaceable with the engine.
- **Alternatives considered**: free/continuous rate (out of scope — unglanceable);
  platform-layer stretching (rejected — untestable without an engine, wrong side of the
  boundary). Contingency: if a real engine cannot stretch, evaluate `soundtouch`
  (LGPL-2.1 → Exceptions Log entry) in the tasks phase; `rubato` is resampling only and
  `rubberband` bindings are too immature.

## R10. Diagnostic record

- **Decision**: Append-only JSONL in app-private storage, schema in
  `contracts/diagnostic-record.schema.json`; dual bound (30 days + 10 MB ≈ 100k entries),
  oldest-first trim emitted as a trim event; best-effort writes (storage-full drops a gap
  marker, never interrupts playback); readable via `adb`/tooling; no in-app surface
  (FR-039a deferral); never leaves the device (FR-040).
- **Rationale**: JSONL is directly tooling-readable with exact control over rotation and
  trim-visibility; private storage needs no permission and is unreadable by other apps
  (FR-039b); the record is the only evidence for the zero-occurrence criteria.
- **Alternatives considered**: `sled`/SQLite store (overkill, less directly readable);
  `log`-crate file logging (unstructured, no rotation/trim semantics); device-wide log
  (rejected — wider surface than entitled, readable by other apps).

## R11. Engine-failure recovery mechanism (SC-051 vs SC-044)

- **Decision**: Closed declared fallback order (remaining same-tier → higher tiers in
  registry order → Edge last, never default; consent-bounded — unconsented engines never
  selected, FR-023c); 3 s bounded offer naming failure + alternatives, then automatic
  fallback from last spoken position; no-alternative expires to paused-with-reason.
  Verified here against two stand-in engines honouring the handoff contract (SC-051);
  SC-044 re-verified on one-offline + one-cloud real hardware when synthesis lands.
- **Rationale**: Two engines is the smallest configuration where takeover is observable;
  stand-ins make FR-023a testable now instead of schedule-blocked on FR-024; closed order
  keeps unattended switches auditable and consent-bounded.
- **Alternatives considered**: single-engine reporting only (rejected — reduces recovery
  to failure display); three-engine provisioning (rejected — forced-second-failure gives
  the same evidence cheaper).

## R12. Engine-error taxonomy: the closed 7-variant set (precedent survey)

- **Decision**: Keep `EngineError` at exactly 7 variants —
  `Unreachable | InitFailed | RateMismatch | ChannelMismatch | DeviceRefused |
  DeviceLost | StartRefused` (wire form already normative in
  `contracts/diagnostic-record.schema.json`: `engine_unreachable … start_refused`,
  required on every `failure`/`refusal` entry) — add the condition→variant table
  below to `contracts/engine-handoff.md`, and admit a new variant only when a
  condition's recovery class differs from every existing variant's. No
  `*_UNSPECIFIED`/`Other`/`Unknown` catch-all member.
- **Rationale**: five production taxonomies converge on the same two rules —
  keep the set closed, and choose the member by recovery class, not by cause:
  1. **Yuan et al., OSDI'14** ("Simple Testing Can Prevent Most Critical
     Failures", USENIX OSDI'14, pp. 249–265 — usenix.org/conference/osdi14/
     technical-sessions/presentation/yuan): of 198 production failures in
     Cassandra/HBase/HDFS/MapReduce/Redis, 48 catastrophic; **92% of the
     catastrophic failures came from incorrect handling of non-fatal errors
     explicitly signaled in software** (only 25% of non-catastrophic ones did).
     35% were trivial mistakes: 25% errors ignored (a log-only handler counts
     as ignoring), 8% abort-in-over-catch (a high-level exception — e.g.
     `Throwable` — caught where a specific one was intended, bringing down the
     whole cluster; the fix was "handle `RemoteException` explicitly"), 2%
     TODO/FIXME in the handler. Over-broad buckets create exactly those two
     modes: an unnamed condition is either swallowed or over-caught.
     *Correction to the brief*: the paper says "non-fatal errors explicitly
     signaled in software", not "signed errors".
  2. **ExoPlayer/Media3 `PlaybackException`** (developer.android.com/reference/
     androidx/media3/common/PlaybackException): 53 codes in 8 numeric families —
     policy −1..−999, general 1000s, IO 2xxx, parsing 3xxx, decoder 4xxx,
     AudioTrack 5xxx, DRM 6xxx, VideoFrameProcessor 7xxx — each family carrying
     an `*_UNSPECIFIED` member, plus `CUSTOM_ERROR_CODE_BASE = 1000000`:
     "Player implementations that want to surface custom errors can use error
     codes greater than this value, so as to avoid collision." The canonical
     list is closed; extension happens out-of-band. Even Google's set shows the
     cost of the unspecified bucket: androidx/media#1126 (HLS stream dying with
     bare `ERROR_CODE_IO_UNSPECIFIED`) needed source archaeology and a PR
     rollback to diagnose — the bucket hid the real condition.
  3. **gRPC** (grpc.github.io/grpc/core/md_doc_statuscodes.html): 16 canonical
     error codes (+OK). "Applications should only use values defined above.
     gRPC libraries that encounter values outside this range must either
     propagate them directly or convert them to UNKNOWN." Member choice is by
     recovery semantics, not cause: `UNAVAILABLE` iff the client may retry the
     call; `ABORTED` iff it should retry at a higher level;
     `FAILED_PRECONDITION` iff it must not retry until system state is fixed;
     `OUT_OF_RANGE` (the more specific) preferred over `INVALID_ARGUMENT` for
     iteration. This is the "recovery class" admission test in production.
  4. **POSIX errno** (man7.org/linux/man-pages/man3/errno.3.html): POSIX.1
     defines ~85 symbolic names (Linux implements all; ~130 with Linux-only
     names — the brief's "~50" is the classic core subset). All POSIX.1 names
     have distinct values (sole alias: `EAGAIN`/`EWOULDBLOCK`); numeric values
     vary across UNIX systems and even across Linux architectures, so **names
     are the portable contract, not numbers**. Failure is signaled out-of-band
     by the return value (−1/NULL); errno is consulted only after that signal
     and is never zero — the taxonomy is never conflated with success values
     (`getpriority`: −1 is a valid success, so callers zero errno first).
     POSIX threads return the error number directly instead of setting errno —
     an explicit channel, never an implicit one. Families cluster naturally:
     network (`EADDR*/ECONN*/EHOST*/ENET*/EPROTO*`), filesystem
     (`EACCES/EEXIST/ENOENT/ENOTDIR/EISDIR/ENOTEMPTY/EROFS`), resource limits
     (`EMFILE/ENFILE/ENOMEM/ENOSPC/EDQUOT/EAGAIN`), math (`EDOM/ERANGE`),
     process (`ESRCH/ECHILD/EINTR`), IPC/STREAMS, keys, locale (`EILSEQ`).
  5. **Postman** (blog.postman.com/what-are-http-status-codes/; Postman Team
     Collections "Errors"): "Be specific within each class: instead of always
     returning 200 or 400, select the most accurate status code… Specific codes
     make debugging easier and provide better client guidance"; "Don't return
     500 Internal Server Error when the client sends invalid data. Reserve 5xx
     codes for actual server problems"; 500 is reserved "for unexpected
     conditions that don't fit other categories" — map to the best existing
     member; the catch-all is the last resort, never the default. Standardized
     machine-readable error code + human message in one body.
  6. **SSML 1.1 + Android TTS** (W3C REC speech-synthesis11-20100907;
     developer.android.com/reference/android/speech/tts): report-then-degrade,
     never silent. SSML §1.5: "A conforming synthesis processor MAY detect and
     report an error and MAY recover from it"; §3.1.13/§3.2.1
     (`onlangfailure`/`onvoicefailure`): the processor **MUST report** the
     failure under every policy value (`changevoice`/`ignoretext`/
     `ignorelang`/`processorchoice`) before degrading; lexicon fetch/parse
     errors MUST notify the hosting environment, SHOULD include the specific
     cause ("if the lexicon couldn't be fetched due to an http 404 error, that
     error code could be included with the notification"), then MUST continue
     with a defined recovery. SSML 1.1 Requirements: "SSML should provide error
     messages and include detail… URI itself, protocol response code and a
     reason." Android TTS: generic `ERROR`(−1) plus a closed set of 7 named
     codes (`ERROR_SYNTHESIS` −3, `ERROR_NETWORK` −6, `ERROR_NETWORK_TIMEOUT`
     −7, and `ERROR_SERVICE`, `ERROR_OUTPUT`, `ERROR_INVALID_REQUEST`,
     `ERROR_NOT_INSTALLED_YET`); `SynthesisCallback.error(errorCode)` — "The
     service should call this method if the speech synthesis fails" — with
     `done` required "at the end of synthesis, regardless of errors", and
     `UtteranceProgressListener` never calling both `onDone` and `onError`
     for one utterance. An engine that fails silently (never terminating the
     callback) violates the contract and hangs the client — the exact
     "silent fallback" this project's FR-023/FR-035 forbid.
- **Condition→variant table** (candidate for `contracts/engine-handoff.md`):

  | Condition | Variant | Recovery class |
  |---|---|---|
  | Engine endpoint not reachable (process/host down, network path down) | `Unreachable` | closed fallback order (FR-023c): next same-tier → higher tiers → Edge last, behind 3 s offer |
  | Engine initialization failed (init error, voice data missing/unfinished) | `InitFailed` | same closed fallback order; no in-session retry of the failed engine |
  | Delivered rate ≠ declared 22050 Hz, or pitch-preserving stretch failed (deviation > ±3 cents at 2.0×) | `RateMismatch` | handoff-contract violation → failover; never substitute a format (FR-043, SC-034) |
  | Delivered channel count ≠ 1 | `ChannelMismatch` | handoff-contract violation → failover; never downmix |
  | Output device refuses the declared shape at stream open (AAudio open fails at 22050 Hz / PCM-float / mono) | `DeviceRefused` | named failure with stated reason under FR-037; paused-with-reason; never silent resample (FR-033/FR-035) |
  | Output device lost mid-session (`ACTION_AUDIO_BECOMING_NOISY`, `onAudioDevicesRemoved`) | `DeviceLost` | pause within the ≤1 s bound; transient reason expires in 3 s; no auto-resume (FR-016/FR-017) |
  | Engine refuses to start (API 34+ FGS start refusal, consent missing) | `StartRefused` | stated reason, no playing state; audio begins only after app open (FR-010) |

  `RateMismatch`/`ChannelMismatch` share a recovery class but name distinct
  handoff invariants whose distinction the diagnostic record must preserve
  (FR-045: every `failure` entry carries `error`); that is the existing set's
  diagnostics-identity split, not a precedent for adding members freely.
- **Why no 8th "unspecified" member**: Media3 needs per-family `UNSPECIFIED`
  because its codes cross version and process boundaries and must round-trip
  conditions other implementations emit. `EngineError` is a same-workspace,
  compiler-checked closed set: gate 7 forbids `_ =>` in `crates/*/src`, SC-057
  enumerates the sets, and FR-044 already bans "catch-all, 'other', 'internal',
  or 'unknown' arms" because "a catch-all converts an unhandled condition into a
  silently wrong one". Rust match exhaustiveness makes an unhandled condition a
  compile error — the strongest possible answer to Yuan's 92%. An unclassifiable
  condition is evidence of a missing recovery class: add the variant through the
  coordinated-edit + semver-bump process the spec already mandates (FR-044),
  never a catch-all arm.
- **Admission test for any future variant** (gRPC/Postman rule): a new condition
  earns a new variant iff its recovery differs from every existing variant's
  recovery; otherwise map it to the closest existing variant and add the row to
  the table. If a genuinely new family appears (e.g. a DRM-like subsystem),
  prefer Media3's out-of-band range pattern (a separate enum or reserved
  numeric space) over growing this one.
- **Alternatives considered**: adding `Unspecified` (rejected — over-broad
  bucket, the exact Yuan ignore/over-catch failure mode); per-engine error enums
  (rejected — a second list, violates closed-set governance and SC-057);
  free-form error strings (rejected — unparseable in the diagnostic record,
  whose `detail` field is bounded to 200 chars of identifier-pattern `k=v`
  pairs, FR-045).

## R13. Error-kind taxonomy: where parse errors live, hierarchy vs flat, domain vs system

- **Decision**: No new `kind` and no new `EngineError` variant for parse
  errors. Parse errors stay owned by the layer whose invariant they violate
  (document validation / SSML assembly, upstream of synthesis). The synthesis
  layer reports a violation of its input contract through the existing
  `failure` kind with the closest-matching `error` member (or the closed
  fallback order when the condition is engine-attributable). The diagnostic
  record keeps its two-axis structure — `kind` (event taxonomy) × `error`
  (condition taxonomy) — and any future condition earns an `error` member
  only via the R12 admission test (recovery class differs) plus the WebIDL
  discrimination test below.
- **Rationale**:
  1. **IEEE 1044-2009** ("IEEE Standard Classification for Software
     Anomalies", IEEE Reliability Society Standards Committee; supersedes
     1044-1993; status now *Inactive-Reserved*, inactivated 2020-03-05 —
     cite it as the classification precedent, not as living maintenance):
     "a uniform approach to the classification of software anomalies,
     **regardless of when they originate or when they are encountered**",
     with classification data used for "defect causal analysis". Origin and
     detection are separate dimensions: a parse error *detected* at
     synthesis may *originate* at document validation, and classifying it
     as a synthesis error misattributes origin — corrupting the causal
     analysis the taxonomy exists to serve.
  2. **End-to-end argument** (Saltzer, Reed, Clark, ACM TOCS 2(4):277–288,
     1984, DOI 10.1145/800164.804050): a correctness function "can
     completely and correctly be implemented only with the knowledge and
     help of the application standing at the end points of the communications
     system". Document-structure validation belongs to the document pipeline
     that holds the invariant; synthesis is a downstream boundary that must
     fail loudly, not re-classify.
  3. **OWASP Input Validation Cheat Sheet**: "Validate data from internal
     APIs, partner feeds, queues, and stored records when it crosses a
     trust boundary. An internal transport does not establish that a value
     meets the receiving component's rules (CWE-20)." The SSML chunk
     crossing into an adapter is such a boundary: the adapter asserts the
     contract and fails fast; the owning validation is upstream.
  4. **W3C SSML 1.1 §3.1.13** (see R12): lexicon fetch/**parse** errors
     "MUST notify the hosting environment, SHOULD include the specific
     cause … then MUST continue with a defined recovery". W3C's own
     synthesis standard treats parse errors as first-class *reportable*
     conditions with mandatory notification and a defined recovery — not a
     silent fallback, and not a new top-level event kind.
  5. **WebIDL Living Standard §2.8.1 ("Base DOMException error names")**:
     the web platform's error vocabulary is a **flat closed set**, and
     `SyntaxError` is explicitly "used to report parsing errors in web
     APIs" — a parse condition *is* a legitimate enum member. But the
     extension rule is the discrimination test: "specifications must use
     one of these names … [else] must file an issue to discuss adding a new
     name to the shared namespace … adding new use-case-specific names is
     only important if you believe web developers will discriminate multiple
     error conditions arising from a single API." A new member is earned
     only when a consumer must *act differently* on it.
  6. **Hierarchy vs flat — standards evidence**: classification *models*
     are hierarchical (ISO/IEC 25010:2011 SQuaRE: characteristic →
     sub-characteristic → attribute; reliability = maturity, availability,
     fault tolerance, recoverability; IEEE 1044 classifies on independent
     axes type/severity/cause), but production *error enums* are flat
     closed sets with family encoding, never nested types: POSIX errno
     (~85 symbolic names clustered by family), gRPC (16 codes; "Applications
     should only use values defined above"), Media3 (53 codes in 8 numeric
     families, 3xxx = parsing), HTTP (RFC 9110 §15: three-digit classes),
     WebIDL (flat names; derived interfaces only when extra introspectable
     state is needed, §2.8.2). The schema already implements the multi-axis
     model: `kind` × `error` × `engine_tier` × `focus`, each axis a flat
     closed set.
  7. **Domain vs system — OpenTelemetry** ("Recording errors", development
     status; "Semantic conventions for exceptions in logs", stable):
     "Semantic conventions that define domain-specific status codes SHOULD
     specify which status codes should be reported as errors by a
     general-purpose instrumentation" — domain codes are not system errors
     by default. HTTP spans: 4xx on a SERVER span MUST leave span status
     unset; 5xx SHOULD be Error; classification is context-dependent
     ("a 404 … indicates an error if the application expected the resource
     to be available. However, it is not an error when the application is
     simply checking whether the resource exists"); intentional cancellation
     "SHOULD NOT be treated as an error". "Errors that were retried or
     handled (allowing an operation to complete gracefully) SHOULD NOT be
     recorded"; exceptions handled by the instrumented library are NOT
     RECOMMENDED for recording; record once, at the point of propagation.
     Severity "reflects the expected impact of the exception, not just its
     presence" (FATAL = shutdown; ERROR = unhandled; WARN =
     expected-to-be-handled; DEBUG = not an actual issue, e.g. client
     cancellation detected server-side). `error.type` "SHOULD be predictable
     and SHOULD have low cardinality. Instrumentations SHOULD document the
     list of errors they report" — a closed, documented set is the
     recommended form.
  8. **W3C has no general error taxonomy.** The W3C artifacts bearing on
     this are SSML 1.1's report-then-degrade rule (R12) and the WebIDL
     DOMException vocabulary (§2.8.1–2.8.2); W3C Trace Context carries no
     error classification. The domain/system split in this codebase is the
     schema's existing `failure` (system/transport condition) vs `refusal`
     (policy/consent decision); per OTel, the error-ness of a domain code
     must be defined by the domain convention — which FR-045 does by
     requiring `error` on both `failure` and `refusal` entries.
- **Application to the parse question**: a synthesis failure caused by
  unparseable input has three distinct recovery classes, and only the third
  is engine-attributable: (a) malformed Markdown/segmentation output — a
  document-pipeline defect (deterministic bad input; no failover fixes it;
  fail fast as a programming error); (b) SSML assembly produced ill-formed
  SSML — an internal invariant violation (a bug; the closed set's job is
  to make such conditions exhaustively visible, not to bucket them); (c)
  the engine rejects SSML our declared capability says it accepts — an
  adapter capability-declaration defect (Principle VI: degradation must be
  explicit and observable), recovery class = closed fallback order, same as
  `Unreachable`/`InitFailed`. None earns a new variant today: (a)/(b) are
  programming errors surfaced by Rust exhaustiveness, and (c) maps to the
  existing failover class. If a fourth class ever emerges (e.g. "engine's
  SSML dialect is narrower than declared, user-visibly"), it enters the
  `error` enum — never the `kind` enum — via coordinated edit + semver
  bump, and only if a consumer must discriminate it (WebIDL test) with a
  different recovery (gRPC test).
- **Alternatives considered**: a third `kind` value `parse` (rejected —
  conflates the event axis with the condition axis, the exact conflation
  IEEE 1044's multi-axis model separates; breaks the schema's
  `failure`/`refusal` → `error`-required symmetry); a `parse_failed`
  member in `EngineError` (rejected until a distinct recovery class is
  demonstrated — premature granularity violates the R12/WebIDL admission
  tests); re-classifying upstream parse errors at the synthesis layer
  (rejected — origin misattribution per IEEE 1044, and hides
  document-pipeline defects behind engine-flavoured errors).

## R14. Failure-log content safety under GDPR (Principle XI): synthesis failures on bad input

- **Decision**: Keep the diagnostic record content-free at every layer,
  including synthesis-layer failures triggered by bad input. A synthesis
  failure is logged as the typed `error` name + position + engine
  identifier + bounded `k=v` detail (identifiers/positions only, e.g.
  `chunk_index=N`, `byte_len=N`) — never the offending text, an exception
  message carrying input, or a stack trace. Where content-level diagnosis
  is needed, record the existing `blake3` content hash + length, not text.
- **Rationale**:
  1. **The biometric hook is not the operative one.** GDPR Art. 4(14):
     "'biometric data' means personal data resulting from specific technical
     processing relating to the physical, physiological or behavioural
     characteristics of a natural person, which allow or confirm the **unique
     identification** of that natural person"; Art. 9(1) prohibits processing
     such data "for the purpose of uniquely identifying a natural person"
     absent explicit consent (EDPB Guidelines 05/2020 v1.1 interpret the
     "explicit consent" standard). A synthesized reading voice does not
     allow unique identification of any natural person — no registry engine
     clones a data subject's voice (voices are trained models; per-voice
     weight licences are recorded in the constitution) — so Art. 9's
     biometric prohibition is not engaged by the *voice*. Art. 9 *is*
     engaged by the *input* if the document itself reveals special-category
     data (health, beliefs): the content, not the waveform.
  2. **The operative constraints are the general principles.** If a
     "synthesis failed on bad input" entry captured the offending text, the
     log would: process document content for a purpose (diagnostics)
     incompatible with its collection purpose (reading) — Art. 5(1)(b)
     purpose limitation; store content beyond necessity for 30 days —
     Art. 5(1)(c) data minimisation + Art. 5(1)(e) storage limitation; and
     fail Art. 25's by-design/by-default duty ("by default, only personal
     data which are necessary for each specific purpose of the processing
     are processed. That obligation applies to the amount of personal data
     collected, the extent of their processing, the period of their
     storage"). Consent (Art. 4(11): "freely given, **specific**, informed
     and unambiguous") for a cloud provider covers *transmission* to that
     provider — it does not cover creating a persistent local store, so
     logging input text would exceed even a consented provider's processing
     scope. The record is never transmitted (constitution), so the exposure
     would be local storage; Art. 5/25 apply to *all* processing, not just
     egress.
  3. **Standards corroborate content-minimal error records.** OTel
     exceptions semconv: `exception.message` carries the WARNING "This
     attribute may contain sensitive information" — the reason the record
     uses a closed enum for `error` instead. RFC 7807 §3.1: `detail`
     "ought to focus on helping the client correct the problem, rather than
     giving debugging information"; §5: "Risks include leaking information
     that can be exploited to compromise the system, access to the system,
     or the privacy of users of the system", and generators are
     "encouraged to avoid making implementation details such as a stack
     dump available". NIST SP 800-92 (Kent & Souppaya, Sept 2006,
     DOI 10.6028/NIST.SP.800-92) is the US federal log-management
     guidance; ISO/IEC 27001:2022 control 8.15 (Logging) requires event
     logging with protected log information. The schema's existing
     invariants — no document text in any entry, `detail` ≤200 chars of
     identifier-pattern `k=v`, `error` required on `failure` *and*
     `refusal` (FR-045) — are the Art. 25 by-design implementation: the
     failure is a typed, content-free identifier.
  4. **Position identifiers are not content.** `position_secs`,
     `chunk_index`, `byte_len`, and the `blake3` fingerprint (already the
     resume-bookmark key) are derivable metadata; they preserve
     diagnosability (a failed chunk is reproducible from hash + document,
     both on-device) without storing the text. The hash does link the entry
     to a specific document — by design, like a fingerprint — but the
     record alone discloses no content, and it is never transmitted.
  5. **Principle XI scope**: the principle forbids document content reaching
     a third-party synthesis provider without explicit, per-provider,
     revocable consent, and forbids default telemetry/analytics/crash
     reporting. A local-only, content-free record satisfies both: no egress
     occurs, and no consent scope is exceeded. Logging *content* at the
     synthesis layer would not violate XI's egress rule (nothing leaves
     the device) but would violate the GDPR analysis in point 2 — and if
     the record were ever transmitted, that would breach XI directly (new
     recipient, new purpose, no consent).
- **Application**: a synthesis adapter that rejects a chunk because of its
  content MUST NOT echo the chunk into `detail` or any field; it reports
  the typed error + identifiers. An adapter whose native error message
  embeds input text must be wrapped at the adapter boundary so only the
  typed name crosses into the record — the same seam Principle VI uses to
  keep vendor types out of the core now enforces the privacy boundary.
  (Adjacent, not engaged: the EU AI Act (Regulation (EU) 2024/1689)
  regulates synthetic-content transparency to recipients and prohibits
  certain biometric processing; neither changes this local-log analysis.)
- **Alternatives considered**: logging a truncated excerpt for diagnosability
  (rejected — any excerpt is document text, the exact thing the schema bans
  and Art. 5(1)(c) minimises); logging content hashes only, with no error
  name (rejected — the failure class is the diagnostically necessary field;
  the hash is supplementary); moving failure logging out of the synthesis
  layer entirely (rejected — OTel records once at the point of propagation,
  the synthesis layer is where the condition becomes observable, and FR-045
  requires `error` on every `failure` entry wherever raised).
