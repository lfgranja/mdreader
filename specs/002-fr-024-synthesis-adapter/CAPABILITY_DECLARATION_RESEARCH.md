# Capability-declaration research: mandatory declaration in audio engine contracts

**Feature**: `002-fr-024-synthesis-adapter` (C2-A evidence) · **Date**: 2026-10-06
**Question**: Why is silent inference of engine capabilities an anti-pattern in
audio/TTS engine contracts (Android TTS, Azure Speech, CLAP, GStreamer, AES67
PTP, SSML 1.1), and why must the v1 adapter declare `(SsmlLevel::none,
AlignmentGranularity::None)` against the existing contract shapes rather than
adding a phantom trait?

**Verdict**: Every audited system makes capability declaration **mandatory,
explicit, and queryable**, and makes degradation **observable** — never
inferred and never silent. None of them encodes capability as a type-level
marker; all encode it as enumerated *data* the consumer branches on. The v1
adapter should declare `(none, none)` through the existing
`declared_capabilities() -> (SsmlLevel, AlignmentGranularity)` shape
(`engine-handoff.md:18`), which already exists in `engine-core`'s
`format.rs`/`closed.rs` shapes. A phantom marker trait would create a second
capability vocabulary outside the closed-set governance (Principle VII), and
would reduce to *inferring* `None` from "trait not implemented" — the exact
silent-inference anti-pattern all six systems are designed to prevent.
Confirms CLARIFICATION_BATCH.md C2-A from external primary sources.

## 1. Primary-source evidence, per system

### 1.1 Android TTS — `Voice.getFeatures()` / `TextToSpeech.getFeatures(Locale)`

- `Voice.getFeatures()` (API 21+): "Returns the set of features it supports
  for a given voice. Features can either be framework defined, e.g.
  `TextToSpeech.Engine.KEY_FEATURE_NETWORK_TIMEOUT_MS` or engine specific.
  **Engine specific keys must be prefixed by the name of the engine they are
  intended for.**" — `developer.android.com/reference/android/speech/tts/Voice`
- `TextToSpeech.Engine`: "A list of feature strings that engines might
  support, e.g `Engine.KEY_FEATURE_NETWORK_SYNTHESIS`... **The engine can be
  queried for the set of features it supports** through
  `TextToSpeech.getFeatures(java.util.Locale)`."
- Framework-defined keys form a closed set (`KEY_FEATURE_EMBEDDED_SYNTHESIS`,
  `KEY_FEATURE_NETWORK_SYNTHESIS`, `KEY_FEATURE_NETWORK_TIMEOUT_MS`,
  `KEY_FEATURE_NETWORK_RETRIES_COUNT`, `KEY_FEATURE_NOT_INSTALLED`);
  engine-specific keys are **namespaced by engine name** because only the
  engine knows its own feature set — the framework cannot infer it from the
  engine's identity or package.
- Consumers iterate the declared set **before** passing feature params to
  `speak()`/`synthesizeToFile()`. Android TTS also offers only
  utterance-level progress (`UtteranceProgressListener`: onStart/onDone/
  onError) — no sentence/word boundary events even in the platform engine;
  coarser granularity is the platform norm, not a defect.

### 1.2 Azure Speech SDK — `SpeechSynthesisWordBoundaryEventArgs`

- "Contains location and length details about words in synthesized speech."
  Properties: `AudioOffset` ("current word's offset in output audio, in ticks
  (100ns)"), `TextOffset` ("current word's text offset in input text, in
  characters"), `WordLength`, `Duration`, `Text`, `BoundaryType`, `ResultId`
  — `learn.microsoft.com/en-us/dotnet/api/microsoft.cognitiveservices.speech.speechsynthesiswordboundaryeventargs`
- The dual offset (`AudioOffset` ↔ `TextOffset`) is the gold standard: the
  engine that *can* emit boundaries emits them **with the mapping data** the
  consumer needs to align text↔audio. The capability arrives as structured
  event args, not as something the app reconstructs.
- The granularity itself is part of the event taxonomy
  (`BoundaryType`: WordBoundary | SentenceBoundary) — a closed, declared set.

### 1.3 CLAP — `clap_plugin_descriptor.features` + `get_extension`

- `clap_plugin_descriptor_t.features`: "Arbitrary list of keywords. They can
  be matched by the host indexer and used to classify the plugin. **The array
  of pointers must be null terminated.** For some standard features see
  plugin-features.h" — `github.com/free-audio/clap/blob/main/include/clap/plugin.h`
- `plugin-features.h`: "This file provides a set of standard plugin features
  meant to be used within `clap_plugin_descriptor.features`... **Non-standard
  features should be formatted as follow: `"$namespace:$feature"`**" — the
  same namespacing rule as Android TTS: standard closed vocabulary
  (`instrument`, `audio-effect`, `mono`, `stereo`, …) plus a namespaced
  extension channel, all declared as an **array of data**.
- Extension capability is queried explicitly: `get_extension`: "**Query an
  extension, returns null if not supported.**" — absence is a *returned
  value*, never an inference. Failure is equally explicit: `init` "If init
  returns false, the host must destroy the plugin instance."

### 1.4 GStreamer — pipeline clock selection (one authority, explicitly elected)

- "A `GstPipeline` object maintains a `GstClock` object and a base-time when
  it goes to the `PLAYING` state. **The pipeline gives a handle to the
  selected `GstClock` to each element in the pipeline** along with selected
  base-time... Because all objects in the pipeline have the same clock and
  base-time, they can thus all calculate the running-time according to the
  pipeline clock."
- "When the pipeline goes to the `PLAYING` state, it will **go over all
  elements in the pipeline from sink to source and ask each element if they
  can provide a clock**. The last element that can provide a clock will be
  used as the clock provider... This algorithm prefers a clock from an audio
  sink in a typical playback pipeline."
- Elements don't self-declare timing truth: "If an element with an internal
  clock needs to synchronize, it needs to ... **slave its clock to the
  pipeline clock**." Slaving is an explicit operation, not an assumption.
- Change is observable: "You can see what clock is selected in the pipeline by
  looking at the `NEW_CLOCK` message on the bus. When a clock provider is
  removed ... a `CLOCK_LOST` message is posted and the application should go
  to `PAUSED` and back to `PLAYING` to select a new clock."
- Default is *declared*, not inferred: "If no element provides a clock, a
  default `GstSystemClock` is used. The clock selection can be controlled
  with the `gst_pipeline_use_clock()` method, which will enforce a given
  clock on the pipeline." (`GstPipeline` reference)
- The plugin-dev manual fixes per-role obligations ("Obligations of each
  element": non-live sources must timestamp; sinks "should require a clock,
  and thus implement the method `set_clock`") — a normative per-role contract,
  not runtime guessing.

### 1.5 AES67 / IEEE 1588 PTP — grandmaster elected from advertised metrics

- IEEE 1588 PAR (primary): "Enables **heterogeneous systems that include
  clocks of various inherent precision, resolution, and stability to
  synchronize to a grandmaster clock**... Defines timing domains in which
  system timing is consistent. Establishes the timing topology... Defines
  formal mechanisms for message extensions and **the requirements for
  profiles** that allow customization for specific application domains." —
  `standards.ieee.org/ieee/1588/11795/`
- The charter's premise — clocks differ in precision/resolution/stability and
  that is a *property of the individual clock, not of the device class* — is
  exactly why PTP elects a grandmaster via the Best Master Clock Algorithm
  (BMCA) from **advertised** clock quality, instead of inferring one from
  device identity (Q-SYS documentation of AES67's PTPv2: "PTPv2
  (IEEE 1588-2008) utilizes a hierarchical selection algorithm known as the
  Best Master Clock Algorithm (BMCA) to determine the best PTPv2 clock within
  the network").
- The clock topology is **declared in the session description**, not
  discovered by trial: AES67 SDP carries `a=ts-refclk:ptp=IEEE1588-2008:
  <clock-id>:<domain>` and `a=mediaclk:...` attributes (FOSDEM 2017 AES67
  talk; real Dante AES67 SDP: `a=ts-refclk:ptp=IEEE1588-2008:00-00-00-FF-FE-
  00-00-00:0`, `a=mediaclk:direct=142410716`; EHX ST 2110 SDP:
  `a=clock-domain:PTPv2 0`). IANA's SDP Parameters registry has dedicated
  "Timestamp Reference Clock Source Parameters" and "Media Clock Source
  Parameters" sub-registries — a closed attribute vocabulary.
- AES67 FAQ (AIMS): "The key feature to look out for is **support for
  IEEE1588-2008**, which is the Precision Time Protocol (PTP) AES67 uses to
  keep all the devices synchronised"; AES67 "includes provisions for precise
  synchronization using the IEEE 1588 precision time protocol" (vs. EBU ACIP,
  which lacks it); discovery/control are optional but "mandating use of SIP
  for connection management and designating the SIP URI or **SDP description
  as the information that needs to be distributed**".

### 1.6 SSML 1.1 — profile model (declared conformance, enumerated responses)

- §2.2.5: "An SSML Profile is a collection of SSML elements and attributes.
  **There are only two profiles defined in this document**: Core profile ...
  Extended profile" — a closed two-member capability set.
- §2.2.4.1: "A Conforming Core SSML Processor **MUST** correctly understand
  and apply the semantics of the elements and attributes of the Core
  profile... When a Conforming Core SSML Processor encounters elements or
  attributes other than those included in the Core profile it **MAY**: ignore
  the non-standard elements and/or attributes; or, process ...; or, **reject
  the document** containing those elements and/or attributes." — conformance is
  declared per profile, and out-of-profile behavior is an **enumerated choice
  with exactly three legal arms** (no catch-all).
- Default is declared, not inferred: "When this attribute
  (`xsi:schemaLocation`) is not given, the Core profile **MUST be assumed**"
  (§2.1).
- Google Cloud TTS publishes exactly which subset it supports ("The Cloud TTS
  supports a subset of the available SSML tags, which are described in this
  topic") and documents bounded degradation (e.g. `<audio>` speed "supported
  range is [50%, 200%]. Values outside that range may (or may not) be
  adjusted to be within it"; `<mark>`: "Marks in rapid succession might not
  generate events. Additionally, if there is no audio generated between marks,
  then events won't be generated"). Vendors publish the declared set *because*
  support is partial and the difference is audible.

### 1.7 Rust-native corroboration — `cpal`

- "You can query all the supported configurations with the
  `supported_input_configs()` and `supported_output_configs()` methods. These
  produce a list of `SupportedStreamConfigRange` structs..." (`DeviceTrait`:
  "An iterator yielding output stream configurations that are supported by the
  device", erroring with `DeviceNotAvailable`/`UnsupportedOperation`) —
  `docs.rs/cpal/latest/cpal/traits/trait.DeviceTrait.html`
- The workspace's own audio stack follows the same rule: the only sanctioned
  way to know what a device accepts is to enumerate it; an unlisted
  configuration is an **error**, not a silent conversion.

## 2. Why silent inference is the anti-pattern (five failure modes)

1. **Capability is not derivable from identity.** IEEE 1588 exists precisely
   because precision/resolution/stability are per-clock properties; Android TTS
   namespaces engine-specific feature keys by engine name ("only the engine
   knows"); CLAP's `"$namespace:$feature"` convention says the same. Any
   "infer capability from engine ID / vendor / version" logic is either wrong
   or a duplicate of information only the adapter possesses.
2. **Two-authority desync.** If `player` infers capability instead of reading
   the declaration, there are two sources of truth — what the consumer
   *believes* and what the engine *does*. When they diverge, the consumer
   fabricates sentence timing from a boundary-silent engine: highlight
   position desyncs from actual audio position (the defect FR-003 / the
   single-authority rule exist to prevent). GStreamer's design is the inverse
   model: one clock authority distributed to all elements; per-element
   self-declared time is the failure mode it eliminates.
3. **Silent degradation is invisible — and here it is audible.** Constitution
   Principle VI (`constitution.md:304-308`): "degradation of prosody MUST be
   explicit and observable — never silent. A heading that was rendered with a
   `<break>` and pitch adjustment under one engine, and flattened to plain
   text under another, is a defect the user must be able to see, because the
   difference is audible". Google Cloud TTS publishes its SSML subset for the
   same reason; the documented hazard without capability filtering is that
   unsupported tags are "ignored silently ... spoken as literal text ... break
   TTS playback" (ssmd TTS docs).
4. **Absence must be a returned value, not a default.** CLAP's
   `get_extension` "returns null if not supported"; Android returns a `Set`;
   cpal returns an iterator or an `UnsupportedOperation` error. "Not
   implemented" must be *reported*, never treated as "therefore it probably
   does `None`" — the latter is inference by default, and it is how a phantom
   trait would behave.
5. **Degradation rides the same channel as the declaration.** Azure carries
   dual offsets in the boundary event itself; GStreamer posts `NEW_CLOCK` /
   `CLOCK_LOST` on the bus; AES67 declares `ts-refclk`/`mediaclk` in the SDP
   a receiver validates before joining. Observable degradation means the
   downgrade is carried in structured, queryable data — in this workspace, the
   JSONL diagnostic record's typed fields — not left for the user to notice.

## 3. Contract-pattern table

| System | Declaration site | Form | Consumer branch | Degradation |
|---|---|---|---|---|
| Android TTS | `Voice.getFeatures()` / `getFeatures(Locale)` | `Set<String>`, framework keys + engine-namespaced keys | query before `speak()` params | features absent ⇒ not passed; utterance-level progress only |
| Azure Speech | `SpeechSynthesisWordBoundaryEventArgs` | event args w/ `AudioOffset`+`TextOffset`, `BoundaryType` | branch on `BoundaryType` | no events ⇒ no fabricated offsets |
| CLAP | `clap_plugin_descriptor.features` + `get_extension` | null-terminated `const char*` array; `"$namespace:$feature"` | host indexer; extension query | `null` ⇒ extension unused; `init()==false` ⇒ destroy |
| GStreamer | pipeline queries elements for clock | `GstClock` handle + base-time distributed to all | elements slave to pipeline clock | no provider ⇒ declared `GstSystemClock`; `CLOCK_LOST` ⇒ re-select |
| AES67/PTP | SDP `a=ts-refclk`/`a=mediaclk`/`a=clock-domain` | IANA-registered attribute vocabulary | receiver validates before join | BMCA elects from *advertised* quality; no compatible grandmaster ⇒ no sync |
| SSML 1.1 | §2.2.5 profile (Core/Extended) | closed 2-member profile set | processor MUST apply its declared profile | out-of-profile ⇒ ignore / process / reject (enumerated) |

Common contract: **declare capabilities (enumerated data) → consumer branches
on the declared set → degradation is explicit, observable, and bounded.**

## 4. Recommendation for the v1 adapter

**R1 — Declare `(SsmlLevel::none, AlignmentGranularity::None)` through the
existing `declared_capabilities() -> (SsmlLevel, AlignmentGranularity)`
shape** (`engine-handoff.md:18-23`), against the existing
`DeclaredFormat`/`EngineError` shapes in `engine-core` (`format.rs`,
`closed.rs`). This is the Android/CLAP pattern exactly: the adapter *produces*
the declaration as data; `player` *consumes* it before branching. The
declaration is unconditional (constitution `L304-305`, `L582-585`): even
"this engine declares nothing" is a declared fact, not an inference.

**R2 — Do not add a phantom marker trait** (e.g. `trait HasSsmlLevel {}`,
`trait DeclaresAlignment {}`). Four reasons:

- **(a) It creates a second capability vocabulary outside closed-set
  governance.** Principle VII (`constitution.md:314-319`) makes capability
  levels closed sets whose growth requires coordinated edits + semver bump +
  docs; gate 7 audits `crates/*/src` for catch-alls. A parallel trait
  vocabulary is invisible to that audit and lets capability grow without the
  coordinated-edit ceremony the constitution mandates.
- **(b) It reduces to silent inference.** The registry holds adapters as data
  (rows in `offline_default` etc.). A trait object cannot be asked "what do
  you declare?" at runtime; a `None`-shaped marker would mean "the trait
  isn't implemented, therefore assume `None`" — absence-as-default, the
  precise anti-pattern §2.4 identifies. CLAP avoids it with an explicit
  `null` return; Android with an explicit `Set`; the contract avoids it with
  an explicit method returning the value pair.
- **(c) It answers the wrong question.** "Does this type implement X?" is a
  boolean; the contract asks "what do you declare for (SSML level, alignment
  granularity)?" — a per-axis *value pair*. Every audited system expresses
  this as enumerated data (feature Set, descriptor array, profile, SDP
  attributes), never as marker interfaces.
- **(d) It duplicates an existing declaration path.** `declared_capabilities()`
  is already the contract's own answer. Two declaration paths (trait + method)
  guarantee drift between them, and drift between declaration paths is silent
  inconsistency by construction.

**R3 — Observable-degradation wiring that `(none, none)` obliges**:

- `player` branches on the declared value: `AlignmentGranularity::None` ⇒
  resume at the nearest completed paragraph/heading boundary via parser-side
  `pulldown-cmark` segmentation + source-range mapping (FR-024a). Raw-cursor
  resume is not a legal fallback — it violates the word-onset floor
  (Grosjean 1980; see `RESUME_ALIGNMENT_RESEARCH.md` §3).
- Record the downgrade in the JSONL diagnostic record as typed fields
  (declared granularity, boundary type used, rewind magnitude; no document
  text) so the degradation is visible post-hoc.
- UX/announcement copy must not over-claim: "resumed at section start", never
  "exact sentence".
- Format deviation (anything ≠ mono f32 @ 22050 Hz) ⇒ named `EngineError`
  (`RateMismatch`/`ChannelMismatch`), playback does not start — never silent
  resample/downmix (`engine-handoff.md:13-14`, FR-024b, FR-035). This is the
  cpal rule: an unaccepted configuration is an error, not a conversion.

**R4 — v2 path.** When an engine can declare `Sentence` (Azure-style
dual-offset boundary events, or SSML `<mark>` once `SsmlLevel` declares
support), that is a coordinated closed-set edit + semver bump + docs — the
constitution's amendment procedure (Principle VII), not a new trait.

**Precedent already recorded internally**: CLARIFICATION_BATCH.md C2-A chose
exactly this ("Declare `(none, none)`; build adapter with no new trait changes
beyond declaration"), citing `constitution.md:304-308`, `580-585`, `615` and
`engine-handoff.md:18-23`. The external evidence above corroborates it from
six independent primary systems plus the workspace's own Rust audio stack.

## 5. Evidence gaps (honest caveats)

- The AES67 standard text itself is paywalled ($50, non-member); the FAQ
  (AIMS), FOSDEM 2017 slides, real Dante/ST 2110 SDP captures, and the IANA
  SDP registry corroborate the `ts-refclk`/`mediaclk` attributes; the IEEE 1588
  PAR and 1588-2019 abstract are public.
- BMCA election detail comes from vendor documentation (Q-SYS) of the IEEE
  1588 algorithm; the IEEE PAR text confirms grandmaster synchronization,
  timing domains, and profiles but not the algorithm's name.
- Azure also ships `SpeechSynthesisSentenceBoundaryEventArgs` with the same
  dual-offset structure; I cite the word-boundary class (the internal research
  already records both).
- Android's per-constant doc strings for the five `KEY_FEATURE_*` constants
  were not individually captured; the constants' names and the
  "engine can be queried" contract were.
- Direct "declared vs inferred capability" experiments exist in neither the
  TTS nor the audio-engineering literature; the case is convergent across
  standards bodies (W3C, IEEE, IANA/IANA-registered SDP attrs), platform
  APIs (Android, Azure), a plugin standard (CLAP), a media framework
  (GStreamer), and the Rust audio stack (cpal) — which is the strongest
  available evidence for an engineering convention.
