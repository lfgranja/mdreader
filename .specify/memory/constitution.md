<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-01
  Version change: 1.1.0 -> 1.2.0

  Bump rationale: **MINOR.** One constraint section is materially rewritten (Tauri Platform
  Contract) to record a decision that changes who owns playback, and one additional constraint
  is added to it. No principle is added, removed, or redefined, and no MUST is weakened, so
  this is not MAJOR; it is not PATCH either, because the amendment reverses a v1.0.0
  architecture decision rather than clarifying its wording.

  Modified constraints:
    - **Tauri Platform Contract** — rewritten. Three changes, in order of weight:
      (a) **The application is now declared a modular monolith**: one Cargo workspace of focused
          crates with the Tauri app as one consumer, making Principle I structural rather than
          aspirational, and the Rust player is one crate in that workspace.
      (b) **Playback authority moves from Kotlin/Media3 to Rust.** At v1.0.0 the contract said
          "Media3 (Kotlin) owns playback, media session, notification, and audio focus. Rust owns
          synthesis." That is now reversed: Rust owns synthesis *and* PCM playback, via `cpal`
          (AAudio backend) or `oboe`. Media3 is no longer in the audio path.
      (c) **Kotlin is redefined as a thin platform shell** — the `Service`, the `MediaSession`,
          the notification, and the audio-focus request, and nothing else.

  Corrected framing, because v1.1.0 asserted more than the evidence supported. The v1.0.0
  contract implied that a Kotlin plugin was the only way to get a media layer. It is not. Two
  corrections were established by checking primary sources:

    - **Rust can drive Android audio directly.** `cpal` 0.18.2 ships an `aaudio` backend
      (`src/host/aaudio`, with `java_interface.rs` and the `ndk` crate) and lists
      `aarch64-linux-android` as a supported platform on docs.rs. `oboe` 0.6.1 (10.8M downloads)
      is the lower-level alternative. PCM playback in Rust is a supported path, not a workaround.
    - **Only the `Service` class is structurally forced to be Kotlin.** The Android
      `ActivityManager` instantiates a `Service` by reflecting on the class named in the
      manifest, so a pure-Rust service cannot exist. That single fact is the entire language
      boundary; everything else in the Kotlin surface is a design decision, and this amendment
      now labels it as one.

  The decision to move playback authority to Rust carries a named risk, recorded because it is
  the reason this rule is written as it is: Media3 expects to be the playback owner, and driving
  the audio track from Rust while a Kotlin `MediaSession` mirrors state creates two authorities
  over position and buffering unless the rule below is enforced.

  Added constraint, in the same section:
    - **Single playback authority.** Rust owns playback state and is the sole authority for
      position, buffering, and transport. The Kotlin `MediaSession` is a *mirror*, never a second
      source of truth. Two authorities over playback position produces a desynchronisation between
      the notification and the audio that is invisible in unit tests and audible only in a car —
      which is precisely the failure Principle VIII exists to prevent.

  Principles modified: none. Principle I is reinforced by this amendment, not changed by it.
  Sections added: none. Sections removed: none.

  Scope discipline: this amendment touches this file only.

  Deferred items: none.
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-01
  Version change: 1.0.0 -> 1.1.0
  Ratified: 2026-10-01 (unchanged — this amends an adopted constitution, it is not a new adoption)

  Bump rationale: **MINOR.** One principle gains a clause (XI), one new constraint section is
  added (TTS Engine Registry), and one existing constraint is materially extended (Audio Pipeline
  Contracts, alignment granularity). No principle is removed or redefined, and no MUST is
  weakened, so this is not MAJOR; it is not PATCH either, because it widens the set of engines
  the system may reach at runtime, which is a governance expansion rather than a wording fix.

  Modified principles:
    - **XI (Credential and Content Sovereignty)** — one clause added. The rule gains a named,
      revocable exception for one specific endpoint. Rationale below.
    - **VI (Engine-Neutral Core, Adapters at the Edges)** — *not modified*. The Edge adapter fits
      the existing trait contract as written; no carve-out was needed and none was taken. Recorded
      here because the fact that it fit without amendment is itself the finding.

  Modified constraints:
    - **Audio Pipeline Contracts** — the sentence-alignment paragraph is materially extended.
      It previously named the hard problem and settled on sentence-level as designed precision.
      It now records which engines can actually *deliver* that precision, which turns an
      aspiration into a testable requirement, and it corrects a prior uncertainty.

  Added sections:
    - **TTS Engine Registry** (Additional Constraints). A closed table of admissible engines with
      the evidence for each. This exists because the engine set was being chosen by conversation
      rather than by document, and an engine arriving or leaving was not currently a governance
      event.

  Removed sections: none.  Renamed principles: none.

  The XI exception, stated plainly because it is the substantive change. Principle XI forbids
  sending document content to a third-party synthesis provider without explicit per-provider
  revocable consent. Microsoft's Edge "read aloud" endpoint is **not a contracted provider**: it is
  undocumented, has no SLA, and is reached with a synthesised authentication token rather than a
  credential the user holds. An Edge adapter therefore does egress under a provider the user never
  chose. This is admitted as an exception rather than left to be discovered, for three reasons:

    1. It is a **fallback, not a default**. The constitution does not rank Edge as the primary
       cloud engine, so the user is never silently moved onto it.
    2. It is **consent-gated by the same mechanism** as every other cloud provider: explicit,
       per-provider, revocable. The exception waives the *contract*, not the *consent*.
    3. Its **failure mode is visible**. Principle VI already requires that a degradation of
       capability be observable; an adapter that cannot be reached MUST surface as an engine
       failure, not as silence, because silence is the failure this whole project exists to fix.

  Trigger: audit of two third-party TTS projects against the adopted constitution. The first,
  `pgmichael/wavenet-for-chrome`, confirmed the free-tier economics and the 5,000-byte limit and
  changed nothing. The second, `travisvn/openai-edge-tts`, surfaced the sentence-boundary fact
  below and forced this amendment. Both audits are recorded rather than discarded, because an
  audit that changes nothing is evidence too.

  Corrections to research recorded at v1.0.0, from the same audit:
    - **Google Cloud TTS `timepoints` are UNVERIFIED.** v1.0.0 left the word-offset capability
      unconfirmed. It remains unconfirmed; this document now says so explicitly instead of
      implying symmetry between providers.
    - **Edge emits `WordBoundary` and `SentenceBoundary`.** Verified in `edge-tts` source
      (`src/edge_tts/submaker.py`), which enumerates exactly those two event types. This is the
      only engine in the registry with a documented *sentence*-granularity boundary, and it is why
      the alignment contract below is written as a requirement rather than a hope.
    - **`msedge-tts` 0.4.0 (MIT OR Apache-2.0)** exists and is maintained (27.8k downloads,
      updated 2026-05-20). The Rust-friendliness of the Edge path is therefore not the liability
      it was assessed as. `edge-tts-rs` 0.1.3 has been dead since 2024-01-19 and is banned.

  Scope discipline: this amendment touches this file only. No template layer was written back,
  and no crate, contract, or gate script was modified by it.

  Deferred items: none. No placeholder tokens remain. The engine ranking within the registry, and
  the measured startup-time bound, remain feature-level decisions for `/speckit.specify`.
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Version change: template (unresolved) -> 1.0.0
  Ratified: 2026-10-01 (initial adoption; the project had no constitution before this)

  Bump rationale: this is an **initial adoption**, not an amendment, so MAJOR/MINOR/PATCH
  does not apply. Every principle and section is new. The previous file was the unresolved
  Spec Kit scaffold, in which every value was an `[ALL_CAPS_IDENTIFIER]` placeholder.

  Principles added (11, all new):
    I.   Library-First, Thin Tauri Shell
    II.  Text I/O Protocol (CLI-First)
    III. Test-First (NON-NEGOTIABLE)
    IV.  Latest Stable Versions, Supply-Chain Gated
    V.   Deterministic Gates Over Judgment
    VI.  Engine-Neutral Core, Adapters at the Edges
    VII. Closed Sets, No Catch-Alls
    VIII.Driving-First (NON-NEGOTIABLE)
    IX.  Offline-First
    X.   Free-Tier Budgeted, Not Free-Hopeful
    XI.  Credential and Content Sovereignty

  Sections added:
    - All eleven Core Principles
    - Additional Constraints (Technology Stack, Dependency Deny-Lists, Licence Policy,
      Rust Code Rigor, Tauri Platform Contract, Driving-Safety Requirements,
      Audio Pipeline Contracts)
    - Development Workflow (Spec-Driven Development, Verification Gates, Exceptions Log)
    - Governance (Amendment Procedure, Versioning Policy, Compliance Review)

  Removed sections: none. This is the first constitution; the template carried no
  project-specific content to preserve.

  Deviations from the crsdd-fabro reference stack, each authorized by the sole maintainer
  (see Additional Constraints -> Exceptions Log). The reference stack is a **default, not an
  obligation**. The four that matter:

    - **Copyleft is permitted.** The reference bans GPL/AGPL/LGPL/EUPL because it ships a
      proprietary product to enterprises. This project is non-commercial and intended to be
      100% open source, and GPL-3.0 is the licence of `espeak-ng` (linked by sherpa-onnx) and
      of `piper1-gpl`. Excluding copyleft would have excluded the working offline engine.
      Revisiting this for adoption is a future amendment, not a current constraint.
    - **`ort` / `onnxruntime` are not banned.** The reference bans them to keep an LLM out of
      a code-migration pipeline. Here they are the inference runtime; banning them would ban
      the product. No LLM is in this system at all.
    - **Network clients are not banned.** The reference bans `reqwest`/`hyper`/`ureq`/`curl`
      to guarantee zero egress from an agent sandbox. There is no agent and no sandbox; an
      HTTPS client is this product's core capability. Egress control moves to Principle XI.
    - **`reqwest-tls` was removed from the deny-list draft.** It does not exist on crates.io.
      A named entry that does not exist is a fiction the gate cannot enforce.

  Research basis (verified 2026-10-01; every load-bearing claim below is sourced):
    - Google Cloud TTS: WaveNet is GA with no deprecation announcement, 4M chars/month free,
      USD 4/M after; the per-request content limit is **5,000 bytes** (not characters);
      Chirp3 is capped at 200 req/min; 100 concurrent streaming sessions per project.
    - `ort` `dist.tsv` ships a prebuilt `aarch64-linux-android` ONNX Runtime 1.30.0 with the
      NNAPI execution provider, SHA-256 verified and statically linked. No source build needed.
    - sherpa-onnx is Apache-2.0 and publishes arm64-v8a APKs for five Piper pt-BR voices.
      Piper pt-BR voice data is CC0 (`faber`, `cadu`, `jeff`); `edresson` is CC-BY-4.0 and is
      excluded on attribution grounds.
    - Kokoro-82M has three pt-BR voices, but sherpa-onnx's Kokoro build ships zh+en only, and
      Kokoro's pt-BR G2P requires espeak-ng. Hence **Piper forward** for the offline default.
    - Tauri v2 has **no** `mobile::update_manifest` API, so a foreground media service cannot be
      declared from a plugin. `tauri::ipc::Channel` does exist and is the sanctioned streaming path.

  Deferred items: none. No placeholder tokens remain. The `specs/` artifacts referenced by
  Development Workflow are created by `/speckit.specify`, not by this document.
-->
# mdreader Constitution

## Core Principles

### I. Library-First, Thin Tauri Shell
Every capability is a standalone Rust crate with one clear purpose, independently testable and
independently documented. The Tauri application is **one** consumer of those crates, never the
only one, and no crate may exist purely to organise other crates.

The engine MUST be drivable headless on Linux. Any behaviour reachable from the phone MUST also
be reachable from a command line, because that is what makes it testable in CI without an
emulator — a phone-only surface is a surface CI cannot see.

### II. Text I/O Protocol (CLI-First)
Every crate exposes its capability over text: arguments and stdin in, stdout out, errors to
stderr, never mixed. Machine-readable output is available via an explicit `--format json` flag
and MUST be the default for scripting; a human-readable format is offered alongside it.

The same protocol is how a crate is driven from the Tauri frontend over IPC, so the IPC path
and the CLI path exercise identical code. A capability that exists only behind IPC fails this
principle.

### III. Test-First (NON-NEGOTIABLE)
Tests are written, shown to fail, then implemented. Red-Green-Refactor is enforced by review,
not by tooling alone.

Test-first is non-negotiable rather than preferred because this project's value is entirely in
behaviour that is hard to verify by inspection: prosody boundaries, byte-budget chunking,
offset alignment between synthesised audio and Markdown source nodes, and audio-focus
transitions while a phone is locked. Reading the code does not tell you whether the reader stops
at a sentence boundary in a tunnel. The test suite is the only evidence that it does.

### IV. Latest Stable Versions, Supply-Chain Gated
Every dependency MUST be the **most recent stable release** available at the time it is added or
bumped. Stability means a non-pre-release version: not alpha, beta, release-candidate, nightly,
canary, or dev.

Departure from this rule is an exception, permitted only when a stated compatibility constraint
forces it, and each exception MUST record in the commit message: the exact version chosen, the
technical reason the stable alternative does not work, and the accepted risk. A preference,
scheduling convenience, or "it worked last time" is not a technical reason.

A supply-chain gate MUST fail closed. It MUST reject an active security advisory
(`cargo deny check advisories` plus `cargo audit`), MUST reject any source outside crates.io,
MUST reject a yanked version, and MUST verify the committed `Cargo.lock` on every build
(`--locked`). The gate MUST NOT auto-substitute one crate for another of similar name; a
failing crate is surfaced for human decision, because that substitution is the slopsquatting
pattern itself.

Rationale: pinning the newest stable keeps the known-CVE surface as small as the ecosystem
allows, and a fail-closed gate is what makes that policy safe to apply mechanically rather
than aspirationally.

### V. Deterministic Gates Over Judgment
Every decision that can be made deterministically MUST be made deterministically. Formatting,
lint level, test outcome, licence compliance, advisory status, dependency policy, and manifest
structure are decided by commands that exit zero or non-zero. No model and no reviewer opinion
is an input to whether the build is green.

Gates are fail-closed: a missing input fails its gate rather than being skipped. A gate that
passes when its input is absent is not a gate.

### VI. Engine-Neutral Core, Adapters at the Edges
TTS engines are reached through a Rust trait. No vendor SDK type, no HTTP response shape, no
SDK error code, and no provider-specific identifier may cross into the core. Implementations are
swappable and every provider-specific decision is confined to its adapter crate.

The intermediate spoken representation is engine-neutral **SSML**. Every adapter MUST declare its
SSML capability level, and degradation of prosody MUST be explicit and observable — never silent.
A heading that was rendered with a `<break>` and pitch adjustment under one engine, and flattened
to plain text under another, is a defect the user must be able to see, because the difference is
audible and it changes whether a document is pleasant to listen to.

SSML is chosen as the neutral form because the cloud engines honour it and the offline engines do
not, which makes the capability boundary unavoidable and therefore explicit rather than a
surprise discovered in the car.

### VII. Closed Sets, No Catch-Alls
Error types, provider identifiers, stage verbs, capability levels, and policy enums are closed
sets. Every path is explicit and auditable. No `Other`, `Internal`, `Unknown`, or catch-all arm
may appear on a closed-set enum.

Adding a variant is a breaking change requiring coordinated edits across affected files, a
semver bump, and a documentation update. The rationale is that a catch-all arm converts an
unhandled condition into a silently wrong one, and in an app whose failure mode is "went quiet
in the tunnel", a silent branch is worse than a failed build.

### VIII. Driving-First (NON-NEGOTIABLE)
The primary context is a moving vehicle. **No required interaction may demand looking at the
screen.** Every operation reachable while driving — play, pause, resume, next section, previous
section, adjust rate, change voice — MUST be reachable from the lock screen or notification
without unlocking the device and without a tap sequence that requires visual confirmation.

Playback MUST continue with the screen off, MUST survive the device being locked, and MUST
survive an interrupted network connection without stopping mid-sentence.

This principle exists because it is the failure the project was created to fix. The tools
replaced so far either have no Android version, cannot read local files, or take long enough to
start that they are already a distraction when they finally do. Driving-safety is therefore a
functional requirement here, not a UX preference, and it is the reason the notification and
media-session surface is built first rather than last.

### IX. Offline-First
The engine MUST produce audible speech with no network connectivity. A design that degrades to
silence when the network drops is non-conforming.

The offline path is `sherpa-onnx` running Piper pt-BR voice models. Cloud synthesis is an
enhancement layered on top, never the sole path. Rationale: tunnels, rural roads, and parking
structures are exactly where a car-commute reader is used, and a dependency that needs signal
cannot serve that case at all.

### X. Free-Tier Budgeted, Not Free-Hopeful
No build and no run may require a paid tier. Quotas, per-request content limits, and
per-character prices are recorded in a versioned table in the repository, re-verified against
provider documentation, and changed only by amendment.

Requests MUST be chunked below the smallest documented per-request limit, measured in **bytes**
rather than characters, and a test MUST enforce that budget. The 5,000-byte Google limit is
exceeded by roughly 2,000 bytes by a single mid-sized Portuguese paragraph, so this is not a
theoretical concern.

The default cloud voice MUST be Google Cloud TTS WaveNet while that family exists and its free
quota holds. When the quota ends or the family is retired, the system MUST escalate through the
remaining free options in order of quality-per-free-character, and finally MUST choose explicitly
between low-quality local synthesis and a paid high-quality API — surfacing that choice to the
user as a decision, never making it silently on their behalf and never spending money without
explicit opt-in.

Rationale: "free" is a property that decays. WaveNet is GA today with no deprecation announced
and the largest free tier of any option, but the constitution states a trigger and an escalation
order rather than a permanent preference, so that its eventual retirement is a planned state
transition instead of a break.

### XI. Credential and Content Sovereignty
No API key, token, or secret may appear in the repository, in a committed file, or in the
application binary. Credentials enter through the platform keystore under user control, or
through a user-operated proxy. This is non-negotiable because a key compiled into a distributed
APK is extractable in minutes, which is why the Chrome extension being replaced needed a backend
project of its own.

Document content MUST NOT reach a third-party synthesis provider without explicit, per-provider,
revocable user consent. No telemetry, no usage analytics, and no crash reporting to any external
service ships enabled by default.

**One named exception, and it is narrow.** The Microsoft Edge "read aloud" endpoint is not a
contracted provider: it is undocumented, has no SLA, and authenticates with a synthesised token
rather than a credential the user holds. An Edge adapter is therefore admissible as an **evaluated
fallback** under the same consent gate as any other cloud engine — explicit, per-provider,
revocable — while never being ranked as a default. The exception waives the *contract*, not the
*consent*, and it does not permit silent substitution: an Edge adapter that cannot be reached MUST
surface as an engine failure rather than as silence.

Rationale: this records a real property of that endpoint instead of leaving it to be discovered
mid-car-commute, and it keeps the consent boundary intact where it matters — the user's choice of
provider — while admitting that one endpoint has no contract behind it.

## Additional Constraints

### Technology Stack
- **Language**: Rust, Edition 2024, MSRV 1.85 (inherited from the reference stack)
- **Build**: Cargo workspace; every member crate declares `[lints] workspace = true`
- **App shell**: Tauri **2.12.1** (latest stable on the 2.x line as of 2026-10-01).
  `3.0.0-alpha.3` is available and is **not** to be taken: Principle IV permits pre-release only
  for compatibility, and stability here is a policy choice, not a forced exception.
- **Frontend**: Svelte 5 (Runes mode)
- **IPC**: `tauri::ipc::Channel` for Rust-to-frontend streaming. Note that
  `InvokeBody::Streaming` does not exist in Tauri 2, so streaming responses are built on
  channels or successive invocations, never assumed.
- **Native bridge**: custom Kotlin plugins via `PluginApi::register_android_plugin` and
  `PluginHandle::run_mobile_plugin`. There is no `registerService`; see the Tauri Platform
  Contract below.
- **Markdown**: `pulldown-cmark` (0.13.x). Its event stream carries source ranges, which is what
  makes boundary-to-source offset mapping possible.
- **Segmentation**: `unicode-segmentation` (UAX #29), with a pt-BR abbreviation guard list,
  because UAX #29 splits on the dot in `Dr.` and on the dot in `1.5`.
- **Inference**: `ort` (2.0.0-rc.13) with the NNAPI execution provider, plus `sherpa-onnx`
  (1.13.8) for the TTS pipeline. `ort` being an RC is a recorded exception under Principle IV:
  the Android arm64 prebuilt with NNAPI is only published on the RC line.
- **Errors**: `thiserror`, closed-set enums. **Observability**: `tracing`. **CLI**: `clap`
  derive. **Serialisation**: `serde`. **State**: `redb` for the read position bookmark.

Extending this list is an amendment, not a dependency-resolution decision.

### Dependency Deny-Lists
This project's deny-list is **not** the reference project's. Each entry is a named crate with a
reason that is still true here, and every name was verified to exist on crates.io on 2026-10-01.
A name that does not exist is removed rather than left as fiction.

**Deny (risk-based, not technology-based):**

| Crate | Reason |
|---|---|
| `tts-rs` | Slopsquat profile: generic name, created 2026-02, 755 downloads, never updated. Legitimate functionality, no adoption. |
| `onnx` | Frozen since 2018 (5.4k downloads), predates the `ort` ecosystem. |
| `sherpa` | Frozen since 2017 (12k downloads), unrelated tool predating `sherpa-onnx`. |
| `candle`, `burn`, `tch` | ML training/inference frameworks outside scope; large supply-chain surface. |
| `llm`, `llama-cpp-2`, `llama-rs`, `rust-bert` | Generic LLM shims and runtimes; no LLM in this system. |
| `openai`, `async-openai`, `openai-api` | LLM provider clients; Principle XI forbids unconsented egress. |
| `langchain`, `langchain-rust` | Orchestration frameworks outside scope. |
| `edge-tts-rs` | Dead since 2024-01-19 (0.1.3, 5.4k downloads); superseded by `msedge-tts`. |

The list is closed. Adding an entry requires a reason that survives the "is this true for this
project" question, and removal requires the same test. `sherpa-onnx`, `ort`, `espeak-rs`, and
`msedge-tts` are explicitly **not** banned: each is a verified, actively maintained dependency or
the sanctioned protocol client for this project.

### Licence Policy
Copyleft is **permitted**. `deny.toml` allows MIT, Apache-2.0 (with LLVM exception), BSD-2/3,
ISC, Zlib, MPL-2.0, Unicode-3.0, GPL-3.0, LGPL-3.0, AGPL-3.0, CC0-1.0, and CC-BY-4.0. Copyleft is
admitted because `espeak-ng` is GPL-3.0 and is linked by `sherpa-onnx`; excluding it would have
excluded the working offline engine.

**Two axes, not one.** Crate licence and model-weight licence are audited separately, because
they are different obligations. A permissive engine does not make its weights distributable:
Piper pt-BR voice `edresson` is CC-BY-4.0 and carries an attribution obligation that the CC0
voices (`faber`, `cadu`, `jeff`) do not, so the default set is the CC0 three. Weight licences MUST
be recorded per voice, not per engine, because the Piper repository holds both kinds.

Because the project is non-commercial and intended to be fully open source, copyleft
contamination of the shipped app is acceptable rather than a risk to be mitigated. Distribution
stays sideloaded and GitHub-Released for now; F-Droid (own repo first, public submission only if
wider adoption is wanted) and the Play Store are later amendments. F-Droid accepts GPL-3.0 and
AGPL-3.0, so the licence choice does not block that path.

### Rust Code Rigor
- `unsafe_code = "deny"`, `missing_docs = "deny"`, `missing_debug_implementations = "deny"`,
  `unreachable_pub = "deny"`, `unused_results = "deny"`
- `clippy::all = deny`, `clippy::pedantic = deny`, `clippy::nursery = warn`, `clippy::cargo = warn`
- `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg_macro`,
  `print_stdout`, `print_stderr` — all deny. No `println!`/`dbg!`; use `tracing`
- `allow_attributes` and `allow_attributes_without_reason` — deny. The only sanctioned
  suppression form is `#[expect(lint, reason = "auditable rationale")]`
- `wildcard_dependencies` and `multiple_crate_versions` — deny
- Panics in library code are defects. Failure is a typed error, not an unwind.

Never weaken `Cargo.toml` or `clippy.toml` to silence a lint. A lint that cannot be satisfied is
resolved in code or, if genuinely unavoidable, via `#[expect]` with a written reason.

### Tauri Platform Contract
The application is a **modular monolith**: one Cargo workspace of focused crates, and the Tauri
application is one consumer of them rather than the only one (Principle I). The audio path is
split so that **Rust owns synthesis and playback**, and a deliberately thin Kotlin surface owns
only what the Android platform requires a Java or Kotlin type for.

- **Kotlin is confined to the platform shell.** The Android `Service`, the `MediaSession`, the
  notification, and the audio-focus request live in Kotlin. Everything else — parsing, segmentation,
  SSML assembly, synthesis, and **PCM playback** — lives in Rust. This is a recorded architectural
  decision, not a platform impossibility: Kotlin calls into Rust over `PluginHandle::run_mobile_plugin`
  and Rust drives playback through `cpal` (AAudio backend) or `oboe`.
- **Single playback authority.** Rust owns the playback state and is the sole authority for
  position, buffering, and transport. The Kotlin `MediaSession` is a *mirror* of that state, never a
  second source of truth. Two authorities over playback position is the specific failure this rule
  exists to prevent: it produces a desynchronisation between the notification and the audio that
  is invisible in testing and audible only in a car.
- **Foreground service**: `tauri-plugin` exposes **no** `mobile::update_manifest` API — only
  `update_info_plist` for iOS. A `<service>` with `android:foregroundServiceType="mediaPlayback"`
  therefore **cannot** be declared from a plugin and MUST be added by editing
  `AndroidManifest.xml`/Gradle directly. That edit MUST be re-verified on every Tauri upgrade.
- **The Service is Kotlin for a structural reason.** The Android `ActivityManager` instantiates a
  `Service` by reflecting on the class named in the manifest, so a pure-Rust service cannot exist.
  This is the only language boundary in the audio path that the platform forces.
- **Plugin lifecycle**: `register_android_plugin` binds to the Activity and its Kotlin constructor
  takes `(Landroid/app/Activity;)V`. There is no `registerService`. The `Service` MUST
  self-bootstrap from the `Application` or the Activity, which is a non-standard lifecycle and
  interacts with Android 14+ restrictions on starting a foreground service from the background.
- **Required components**: `FOREGROUND_SERVICE` and `FOREGROUND_SERVICE_MEDIA_PLAYBACK`
  permissions; a `MediaSession`; `AudioFocusRequest` with `AUDIOFOCUS_GAIN`, ducking on transient
  loss and pausing on permanent loss; handling of `ACTION_AUDIO_BECOMING_NOISY` so that unplugging
  headphones pauses rather than playing out of a speaker.
- **Android Auto is out of scope for now.** Auto requires a browsing-capable
  `MediaLibraryService` and has no TTS category, so a Markdown reader does not qualify. This is
  recorded as a deliberate deferral, not an oversight.

### TTS Engine Registry
The admissible engine set is closed. Adding or removing an engine is an amendment, not a
dependency-resolution decision, because an engine arriving or leaving changes what the user hears
and what leaves the device.

| Engine | Tier | Boundary events | SSML | Admissible |
|---|---|---|---|---|
| `sherpa-onnx` + Piper pt-BR (`faber`, `cadu`, `jeff`) | offline default | none | none | **Yes** — Principle IX |
| Google Cloud TTS WaveNet | cloud default | `timepoints` UNVERIFIED | full | **Yes** — Principle X |
| Google Cloud TTS Neural2 / Chirp 3 HD / Studio | cloud escalation | UNVERIFIED | full / partial | Yes, ranked by free quota |
| Amazon Polly Neural (speech marks) | cloud escalation | word-level | full | Yes, ranked by free quota |
| Azure Speech Neural | cloud escalation | word-level | full | Yes, ranked by free quota |
| Microsoft Edge read-aloud (via `msedge-tts`) | evaluated fallback | **`WordBoundary` + `SentenceBoundary`** | none | **Yes, under the XI exception** |
| Kokoro-82M pt-BR | offline alternative | none | none | Deferred — sherpa-onnx ships zh+en only |

Ranking within the cloud tiers belongs to `/speckit.specify`. What this document fixes is the
*closed set* and the rule that capability differences are declared rather than assumed.

Two findings that changed the registry and are recorded so they are not re-litigated:

- **Sentence-granularity alignment is achievable today, but only via Edge.** Verified in
  `edge-tts` source (`src/edge_tts/submaker.py`), which accepts exactly `WordBoundary` and
  `SentenceBoundary`. Google's `timepoints` field is **UNVERIFIED** — not confirmed to exist in the
  REST response, and absent from the Gemini TTS docs. The alignment contract below therefore
  states sentence-level as a requirement of the *pipeline*, not as an assumption that every engine
  can satisfy it.
- **The Rust path to Edge is maintained.** `msedge-tts` 0.4.0 (MIT OR Apache-2.0, 27.8k downloads,
  updated 2026-05-20). `edge-tts-rs` 0.1.3 has been dead since 2024-01-19 and is banned. Note that
  the widely-starred `travisvn/openai-edge-tts` is a **Python/Docker wrapper around** `edge-tts`,
  not a protocol implementation: it exposes the OpenAI-compatible `/v1/audio/speech` shape and its
  SSE stream carries only `speech.audio.delta` and `speech.audio.done`, discarding the boundary
  events. An adapter MUST talk to the endpoint directly and MUST NOT proxy through that wrapper,
  because proxying loses the only capability the engine uniquely offers.

### Driving-Safety Requirements
Playback MUST continue with the screen off and while the device is locked. All driving-time
controls MUST be reachable from the lock screen or notification (Principle VIII). Rate and voice
changes MUST be reachable without unlocking. The app MUST NOT require a network connection to
begin or continue playback (Principle IX).

Audio MUST start within a bounded time from the control action, so that a tunnel does not turn a
play button into a guessing game. That bound is a measured requirement to be fixed by the first
`/speckit.specify`, not an open question.

### Audio Pipeline Contracts
Markdown is parsed to an AST carrying source ranges; filtered by a spoken-content policy (code
blocks, images, and tables are skipped or announced, not read verbatim); segmented into sentences
with a pt-BR abbreviation guard; normalised for spoken numbers and currency; assembled into
engine-neutral SSML; synthesised per chunk under the byte budget; and aligned back to source
nodes for highlighting.

Alignment is the known hard problem. Synthesis normalises text (`1.5` becomes "um ponto cinco"),
so word events cannot map one-to-one onto source offsets. The mitigation is to map at **sentence or
node granularity**, and to treat that as the designed precision rather than a defect to be
eliminated.

The pipeline MUST therefore be capable of sentence-level alignment to source nodes, and a test
MUST exercise it. Only the Edge endpoint is currently known to supply sentence boundaries; the
other engines supply none, or supply word events whose presence is unverified. An adapter MUST
declare which granularity it can deliver, and the pipeline MUST degrade to a coarser, declared
granularity when an adapter cannot deliver sentence alignment — observably, per Principle VI, never
by silently highlighting nothing.

No mature pt-BR number-to-words crate exists; this normaliser is written in this project and is
covered by tests.

## Development Workflow

### Spec-Driven Development
Every feature is a `specs/NNN-<short-name>/` directory containing `spec.md` (what), `plan.md`
(how), `tasks.md` (dependency-ordered), `checklists/` (requirements as tests), and `contracts/`
(JSON Schema for wire formats). This layout is inherited from the reference stack.

Voice selection, engine ranking within the TTS Engine Registry, Markdown-to-speech policy, and
the startup-time bound are feature-level decisions. The constitution fixes the closed set of
admissible engines, the escalation order, and the consent boundary; it does not fix the ranking,
which belongs to `/speckit.specify`.

### Verification Gates
`scripts/verify.sh` is the single entrypoint, mirroring the reference project's `ci.sh` ordering.
Exit status is zero if and only if every gate passes.

| # | Gate | Command | Checks |
|---|---|---|---|
| 1 | format | `cargo fmt --all -- --check` | rustfmt-clean |
| 2 | tests | `cargo nextest run --workspace` | unit, integration, E2E |
| 3 | documentation | `cargo doc --workspace --no-deps` | public API documented |
| 4 | lint-drift | root deny-tables intact; `[lints] workspace = true` in every member | rigour not quietly weakened |
| 5 | supply-chain | `cargo deny check advisories licenses bans sources` + `cargo audit` | Principle IV |
| 6 | deny-list | no denied crate in any manifest or in `Cargo.lock` | Principle IV |
| 7 | closed-set | no `_ =>` in `crates/*/src/` | Principle VII |
| 8 | pipeline | byte-budget test under 5,000 bytes/request; SSML capability declared per adapter; sentence-level alignment exercised | Principles VI, X |
| 9 | android-build | `cargo tauri android build --target aarch64` | the app actually compiles for the device |

Gate 9 exists because gates 1–8 are all satisfiable on a host that cannot produce an Android
artifact, which would let a workspace pass every deterministic check while having no working app.

Human review remains a requirement at phase boundaries, where a decision needs judgement rather
than a command.

### Exceptions Log
The reference stack (`~/development/MDMV/projetos/crsdd-fabro`) is the **default, not an
obligation**. Deviations are permitted and are recorded here so they are decisions rather than
drift. Each was authorized by the sole maintainer on 2026-10-01.

1. **Copyleft permitted** (deviates from the licence allow-list). Reason: `espeak-ng` is GPL-3.0
   and is linked by `sherpa-onnx`; the project is non-commercial and open source.
2. **`ort`/`onnxruntime` permitted** (deviates from the LLM framework deny-list). Reason: it is
   this project's inference runtime; no LLM is involved.
3. **Network clients permitted** (deviates from the network deny-list). Reason: an HTTPS client is
   the product; egress control is Principle XI.
4. **`ort` 2.0.0-rc.13 accepted** (deviates from stable-only). Reason: the Android arm64 prebuilt
   with the NNAPI provider is published only on the RC line. Re-evaluate when 2.0.0 stable ships.
5. **`cargo machete` not inherited.** Its unused-dependency check would flag legitimate
   dev-only and test-only crates. It applies from the first release onward.
6. **Android Auto deferred.** See Tauri Platform Contract.
7. **Edge read-aloud endpoint admitted as a fallback.** See Principle XI. It is a documented
   exception to the consent rule, not a ranked default, and the adapter speaks the protocol
   directly rather than through the OpenAI-compatible Python wrapper.

Any new deviation is added here in the same commit that introduces it, and is treated as a
minor amendment.

## Governance

### Amendment Procedure
An amendment is a proposed change to this document carrying: the intended outcome, the reason, and
the affected principles and constraints. It MUST be recorded as a Sync Impact Report at the top of
this file, stating the version change, the bump rationale, the modified principles, and any
deferred items.

This project has a single maintainer, who is also its sole user. An amendment is proposed by an
agent or by the maintainer, and is accepted when the maintainer explicitly accepts it. The
two-party approval rule that suits a multi-maintainer repository does not apply here and is not
imposed as ceremony. Acceptance is recorded in the commit that carries the amendment, so that it
is visible in history rather than asserted inside the document.

A proposal that changes a NON-NEGOTIABLE principle, the licence policy, or the
driving-safety requirement MUST be stated explicitly as such, so that accepting it is a
deliberate act rather than an incidental edit.

### Versioning Policy
MAJOR for a backward-incompatible governance change: removing or redefining a principle, or
weakening a MUST. MINOR for a new principle or section, or a material expansion of guidance.
PATCH for clarification, wording, typo fixes, and non-semantic refinements.

Rationale: the value of this document is that a reader can tell which changes matter, so a
version number that does not distinguish them is worse than none.

### Compliance Review
Every feature's Phase 0 MUST attest conformance to this constitution before research begins, and
name any exception rather than silently proceeding. Every amendment MUST be recorded in the
Exceptions Log when it is a deviation from the reference stack. Every new dependency MUST pass
gates 5 and 6 before it is used, and every model-weight licence MUST be recorded per voice before
the voice is offered.

Where this constitution and a convenience conflict, this constitution wins and the convenience is
either changed or proposed as an amendment. Where this constitution and the reference stack
conflict, the maintainer decides, and the decision is logged.

**Version**: 1.2.0 | **Ratified**: 2026-10-01 | **Last Amended**: 2026-10-01