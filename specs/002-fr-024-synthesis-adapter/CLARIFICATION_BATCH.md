# Clarification Batch — FR-024 Synthesis Adapter (002-fr-024-synthesis-adapter)

**Agents spawned per candidate:** 2 (internal codebase deep-dive + scientific/engineering research). **Evidence sources:** `spec.md` (002, 001 contracts), `.specify/memory/constitution.md`, `crates/engine-core/src/closed.rs`, `crates/engine-core/src/format.rs`, `engine-handoff.md`, Piper/sherpa-onnx docs (verified via subagent), peer-reviewed literature (Zacks 2007 EST; Monk 2008 resumption lag; Cowan 2001 WM; Grosjean 1980 gating; Android TTS/Azure Speech/CLAP contracts; GDPR/CCPA consent; ExoPlayer/gRPC/errno taxonomy; SSRF/SSML 1.1 profiles).

---

## Candidate 1 — Resume boundary level (FR-024 / FR-036)

**Question:** Should adapter resume align to paragraph/heading boundaries (parser-side, since Piper exposes zero sentence-boundary events), and should the stale `[NEEDS CLARIFICATION]` marker at `spec.md:88` be removed with US/SC corrected to "nearest completed paragraph/heading"?

**Why does it matter:** The embedded marker is stale bookkeeping (previous clarification 2026-10-06 already answered "paragraph sufficient" at `spec.md:18`), but the spec's US2/US3/SC-027 still claim "sentence boundary" resume — which contradicts the registry record (`constitution.md:531`: Piper boundary events `none`) and the adapter's declared `AlignmentGranularity`. If uncorrected, `player` would either fabricate sentence timing (second authority, prohibited by `constitution.md:87`) or resume at raw cursor (violates FR-015 / FR-036).

**Findings:**
- Internal: `snap_to_boundary` (`crates/player/src/position.rs`; renamed from `snap_to_sentence_start`, accepted 2026-10-09, recorded in `contracts/adapter-capabilities.md`) is granularity-agnostic; `doc-structure` is a 1-line stub (`crates/doc-structure/src/lib.rs`); adapter chunk splitting is parser-owned (`FR-024a`). Piper has **no** timing events (`rhasspy/piper#361`, verified); sherpa-onnx segments internally but exposes only audio samples + rate (`k2-fsa/sherpa-onnx#3650`). Previous clarification correctly chose paragraph + source-range mapping.
- Research: Event Segmentation Theory (Zacks et al. 2007; Gold, Zacks, Flores 2017) shows boundary-aligned resume improves memory vs midpoint; Monk et al. 2008 resumption lag grows significantly past ~13 s — a paragraph (~15–45 s of speech) exceeds that window, favoring sentence for ideal conditions, but Piper's lack of events makes sentence-level resume *unimplementable* in v1 without a new segmentation layer. Mid-word resume (Grosjean 1980 gating) is clearly harmful. Recommendation: paragraph/heading resume is defensible for v1 given engine limits; never mid-word.
- The `AlignmentGranularity` closed set (`engine-handoff.md:20`) is `Sentence | Word | None` — **no paragraph arm exists**, and research (Azure `BoundaryType`, Android `rangeStart`, Web Speech) confirms paragraph is document/static domain, not engine dynamic capability. Adding `Paragraph` would be a constitution amendment (Principle VII / `closed.rs` pattern).

**Options:**
- **A (Recommended)** — Remove marker at `spec.md:88`; correct US2/US3/SC-027 to "nearest completed paragraph/heading boundary" (matching adapter's declared `None` + parser chunking); keep `AlignmentGranularity` at `{None, Word, Sentence}`; treat sentence-level as deferred enhancement once adapter can declare `Sentence` (e.g., via espeak-ng marks or enhanced sherpa boundary hooks).
- B — Implement sentence-level segmentation in adapter (requires new engine integration / new capability declaration); higher v1 cost, not required by Option A.
- C — Amend `AlignmentGranularity` to add `Paragraph` — not recommended; conflicts with engine-contract design (paragraph = parser domain).

**Reply with:** `C1-A` (or B/C / custom).

---

## Candidate 2 — Engine-core trait + capability declaration (FR-024c / Assumption 3)

**Question:** Should adapter declare `SsmlLevel::none` + `AlignmentGranularity::None` via `declared_capabilities()`, with `player` reading them before resume, and should `FR-024c` be rewritten as a normative declaration rather than an optional trait hypothesis?

**Why does it matter:** Constitution `L304-308`, `L580-585`, `L615` require explicit capability declaration; `engine-handoff.md:18-23` defines it; `T016` mandates the trait. The spec's Assumption 3 (`spec.md:123`) is factually false (zero `trait` declarations exist in `crates/`), and the adapter's `offline_default` registry row already fixes values (`SsmlLevel::none`, boundary events `none`). Without declaration, `FR-015`'s degradation branch is undecidable and silence cannot be observed — the exact failure mode `constitution.md:584-585` forbids.

**Findings:**
- Internal: `grep '\btrait\b' crates/**/*.rs` → 0 matches; `crates/engine-core/src/lib.rs` exposes only `closed`, `format`, `standin_a`, `standin_b`; `standin_a.rs` uses inherent `set_rate()` / `delivered_format()`. No `declared_capabilities()`. Previous clarifications (lines 17-21) do **not** address capability declaration at all.
- Research: Android TTS `Voice.getFeatures()`, Azure `SpeechSynthesisWordBoundaryEventArgs` (dual offset), CLAP `clap_plugin_descriptor.features`, SSML 1.1 profile model, GStreamer clock selection (one authority, slaves mirror), AES67 PTP (grandmaster elected, followers advertise), Android `PlaybackState` (service writes, UI reads). All support mandatory declaration + observable degradation, never silent inference. Polys' neural-voice silent fallback (`say-as` unsupported) is the anti-pattern to avoid.
- `FR-024c` (`spec.md:91`) says adapter "implements the `engine-core` trait, does not modify `player` transport/position logic" — but since trait = nonexistent, this is aspirational. Should become: adapter declares `(SsmlLevel::none, AlignmentGranularity::None)` and `player` branches on that (raw cursor for `None`; sentence snap only when `Sentence` declared).

**Options:**
- **A (Recommended)** — Declare `(none, none)`; build adapter with no new trait changes beyond declaration; fix `FR-024c` to reference declaration, not unspecified trait; correct `Assumption 3` (line 123) to state declaration is required and values are registry-fixed; update US/SC to match declared level (paragraph resume).
- B — Add `Paragraph` to `AlignmentGranularity` and build new cap — requires constitution amendment + `engine-handoff.md` edit + 6-file closed-set sync.
- C — Leave adapter silent — violates Principle VI; breaks gate-8-full exit condition.

**Reply with:** `C2-A` (or B/C / custom).

---

## Candidate 3 — SC-044 second engine / consent (spec.md line 107 / Assumption 1 / Principle XI)

**Question:** Should v1 (Option A) close SC-044 on the `test_double` branch (no consent required, verified contract) and explicitly defer Google Cloud TTS / consent-acquisition to post-v1?

**Why does it matter:** The consent mechanism (per-provider, revocable, app-private storage, read on every fallback attempt per `FR-023c`) is defined negatively but never constructed: no acquisition surface exists in either spec (`spec.md:133` explicitly excludes cloud fallback from Option A). A second real engine is required for SC-044, but `test_double` satisfies it by construction (`tasks.md:203`: "consent for test_double granted by construction"). Driving-context consent fatigue + GDPR/CCPA (voice = biometric data) argues strongly against per-session prompts.

**Findings:**
- Internal: `SC-044` says "second is either a second real engine (e.g., Google Cloud TTS with consent) or verified `test_double`" — two branches. `EngineTier::ORDER` puts `test_double` after cloud tiers; `spec.md:89` makes `test_double` inadmissible while any non-`test_double` adapter is registered — so with real adapter present, `test_double` is only fall-back if cloud tiers are unconsented/skipped. `spec.md:133`: "No SSML, no cloud fallback, no second voice... deferred".
- Research: Consent-fatigue literature (UXMag / Springer 2024-25) shows repeated prompts train click-through; NHTSA minimizes visual-manual interaction while driving; GDPR Art.7 (freely given, specific, unambiguous) + Art.9 (biometric) + Art.25 (fail-safe default) favor on-device default + pre-authorized opt-in; Android Auto DD-2 requires minimal interruption; explicit refusal must be named/non-blocking.
- `test_double` branch confirms fallback mechanism (`SC-051`) works with real audio (adapter + `test_double`) without introducing consent surface, cloud dependency, or DPA burden in v1.

**Options:**
- **A (Recommended)** — Confirm SC-044 closes on `test_double` for v1; state cloud + consent surface deferred to post-v1 / `/speckit.specify`; no consent FR needed in 002. This aligns with existing `spec.md:133` deferral and minimizes v1 risk.
- B — Include Google Cloud TTS in v1 with consent at settings — requires new consent-acquisition FR + DPA + cloud-infrastructure, out of Option A scope.
- C — Add consent surface to v1 — blocks gate-8 / increases risk; not justified by evidence.

**Reply with:** `C3-A` (or B/C / custom).

---

## Candidate 4 — Adapter failure taxonomy / clarification line 21 (spec.md lines 63-21 / FR-035 / SC-051)

**Question:** Should clarification line 21's failure names (`format_mismatch`, `engine_crash`, `refusal`, `deferred_grant`) be replaced with the existing `EngineError` closed-set members (`RateMismatch`, `ChannelMismatch`, `InitFailed`, `Unreachable`, `DeviceRefused`, `DeviceLost`, `StartRefused`), with an explicit condition→variant table and a clarification on SC-051 configuration (adapter-present vs standin-present)?

**Why does it matter:** **This is factually wrong against source.** The four named variants don't exist in `crates/engine-core/src/closed.rs:235-270`, `engine-handoff.md:25-27`, or any of the 6 synchronized sites. The real set is 7 variants; `format_mismatch` should split to `RateMismatch`/`ChannelMismatch`; `engine_crash` maps to `Unreachable`/`InitFailed`; `refusal` is a diagnostic `kind`, not `EngineError`; `deferred_grant` is a focus outcome, not engine failure. Additionally, `spec.md:21` says preserve `SC-051` with standins in `test_double` — but `spec.md:89` / `constitution.md:306-312` say `test_double` is inadmissible once a real adapter is registered; the spec's own US3 scenario and `SC-051` description assume adapter is active, not standin. The contradiction must be resolved.

**Findings:**
- Internal: `EngineError` closed set = `Unreachable | InitFailed | RateMismatch | ChannelMismatch | DeviceRefused | DeviceLost | StartRefused`. `FR-035` (`spec.md:90`, `spec.md:109`) correctly references `FR-035` / named failure, but line 21's names are wrong. Previous clarification (line 21) addresses preservation order only (not triggers). `spec.md:65` says failure simulated by "stopping sherpa-onnx process or supplying bad input" — but bad input is document/parse-level, not engine-level (Principle VII forbids catch-all). No detection bound defined (line 69 conflates 3 s offer window with detection). `SC-051` mechanism is cause-blind: branches on `EngineTier`, consent, and test_double admissibility — the failure variant only affects displayed reason / diagnostic record.
- Research: Closed-set error taxonomy best practice (ExoPlayer ~50 codes by family; gRPC 16 canonical; POSIX errno ~50) — new variant only justified if new recovery class (actionability). Lifecycle errors (process death, OOM, memory pressure) absorb into coarse members (`Unreachable`/`InitFailed`). Mapping is preferred over new variant unless caller behavior differs materially (`error-model treatise`; `Yuan et al. OSDI'14`: 92% catastrophic failures from incorrect handling of explicitly-signed errors — broader sets = more error-handling bugs). Postman's guidance: don't over-map to `INTERNAL`; map to best existing member.
- `SC-051` contradiction: if adapter is registered (`offline_default`), standins are not selectable (`spec.md:89`). The "preserve with standins" claim (line 20 / checklist line 16) describes pre-transition state; post-transition `SC-051` must run with adapter active, `test_double` as fallback option (if consented / available), and real engine failure simulated (not standin failure).

**Options:**
- **A (Recommended)** — Replace line 21 names with real wire names; add `FR-024d` / clarification entry: condition→variant table (rate/channel → `RateMismatch`/`ChannelMismatch`; init failure / bad voice data → `InitFailed`; process gone / OOM → `Unreachable`; device refusal → `DeviceRefused`; start refused → `StartRefused`; bad Markdown segment → document-level, not `EngineError`); clarify `SC-051` runs with adapter registered (`test_double` admissible only if unconsented / no other tier); no new `AdapterFailed` variant needed.
- B — Add `AdapterFailed` / `EngineCrashed` as new `EngineError` variants — requires constitutional amendment (`constitution.md:319-322`: coordinated edit + semver + docs), 6-file sync (`closed.rs`, `engine-handoff.md`, `shell-ipc.md`, `data-model.md`, `diagnostic-record.schema.json`, `prop_closed_sets.rs`), plus `tasks.md` updates (T010-T011, T071). Only justified if recovery action differs from `Unreachable`/`InitFailed` — evidence shows it doesn't.
- C — Leave line 21 as-is — produces spec that contradicts all source files.

**Reply with:** `C4-A` (or B/C / custom with specific mapping preferences).

---

### Integration readiness (pending your batch confirmations)

I can apply **C1-A / C2-A / C3-A** immediately (they are convergent with source + research + previous clarifications). **C4-A** requires your confirmation of the exact mapping table (especially: process-death → `Unreachable` vs `InitFailed`; bad input → document/parse vs engine error) and the `SC-051` state clarification.

**Quick reply format accepts:** `C1-A C2-A C3-A C4-A` (or specify alternatives / adjustments). If any option needs a custom answer, include the short phrase (≤5 words for open items, option letter for MC).

After your confirmations I will: (1) integrate all into `spec.md` + `checklists/requirements.md`; (2) save atomically; (3) dispatch `after_clarify` hooks (already verified: `.specify/extensions.yml` has `hooks: {}` — none to run, skip silently); (4) run validation (spec quality checklist re-check, terminology consistency, no stale markers); (5) emit Completion Report with before/after counts.

**Suggested next step after batch:** Run `/speckit.plan` once clarifications are integrated (C1/C2/C3 resolved; C4 confirmed). No further clarification needed if all four confirm A.
