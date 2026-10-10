# Resume-alignment research: boundary-aligned resume in TTS/audio navigation

**Feature**: `002-fr-024-synthesis-adapter` (C1/C2 evidence) · **Date**: 2026-10-06
**Question**: Is paragraph/heading resume defensible for v1 when Piper exposes zero
sentence-boundary events? What is the cost of wrong-boundary resume?

**Verdict**: Yes — defensible for v1, with three conditions: (1) never resume mid-word;
(2) declare `AlignmentGranularity::None` and let `player` branch on the declaration
(coarser granularity is explicit and observable, never silently inferred); (3) keep the
FR-015 bounded rewind (proportional re-listen acts as rehearsal context).
Note the premise correction: Monk et al. (2008) shows steep resumption-cost growth
**across 3→13 s** (conditions 3, 8, 13 s), roughly linear in that window and
asymptoting by ~1 min; a 15–45 s paragraph sits *past* the steep window, so
paragraph-vs-sentence **duration** buys little marginal relief — the dominant cost is
**where** playback resumes (boundary vs mid-event), which paragraph alignment already
addresses at the prose-event scale.

## 1. Event Segmentation Theory (boundary-aligned resume)

- Zacks, Speer, Swallow, Braver & Reynolds (2007). *Event perception: A mind–brain
  perspective.* Psychological Bulletin 133(2), 273–293. doi:10.1037/0033-2909.133.2.273.
  Perception segments continuous activity into discrete events; boundaries arise from
  transient prediction-error spikes and regulate attention and memory. Resuming **at** a
  boundary gives the listener a fresh event model; resuming **mid-event** forces
  reconstruction of the current event model from partial input (re-orienting cost).
- Gold, Zacks & Flores (2017). *Effects of cues to event segmentation on subsequent memory: The role of event boundaries.* Cognitive Research: Principles and Implications 2:1 (article 1; doi:10.1186/s41235-016-0043-2).
  memory.* Cognitive Research: Principles and Implications 2:7.
  doi:10.1186/s41235-016-0043-2. Cuing event boundaries improved memory in younger and
  older adults — *particularly for cued boundaries*; imposing segmental structure aids
  encoding even when segmentation is suboptimal. Boundaries are remembered better than
  middles.
- Supporting: Flores, Bailey, Eisenberg & Zacks (2017). JEP:LMC 43(9) — segmentation
  manipulation improves memory up to one month later. Swallow, Zacks & Abrams (2009).
  JEP:General 138(2) — boundaries trigger event-model updating; content without
  boundary context is encoded worse.
- Implication for granularity: in narrative prose, paragraph/heading ≈ the *event*
  scale; a sentence is a sub-event boundary. Paragraph/heading resume is therefore
  aligned to the scale at which the memory benefit is largest.

## 2. Resumption lag (interruption duration)

- Monk, Trafton & Boehm-Davis (2008). *The effect of interruption duration and demand on
  resuming suspended goals.* J. Experimental Psychology: Applied 14(4), 299–313.
  doi:10.1037/a0014402. Exp. 1 used interruptions of **3, 8, 13 s** (after Hodgetts &
  Jones 2006b: 6/18 s). Resumption lag rises with duration, ~linear across the 3→13 s
  window; a decay profile up to 1 min fit the data; cognitive demand interacts, and
  **goal rehearsal during the interruption mitigates decay** (Exps. 2–3).
- Model: Altmann & Trafton (2002). *Memory for goals: an activation-based model.*
  Cognitive Science 26(1) — the resumption-lag measure; goal activation decays with
  interruption duration.
- Altmann & Trafton (2007), *Timecourse of recovery from task interruption*: 30–45 s
  interruption range shows little additional resumption-lag variance (asymptote).
- Trafton, Altmann, Brock & Mintz (2003). IJHCS 58(5) — prospective goal encoding and
  retrospective rehearsal shorten resumption lag → the FR-015 re-listen (re-hearing
  context) is the mechanism the literature says helps, not just a cosmetic rewind.
- Read for this design: paragraph ≈ 15–45 s speech ≥ the 13 s steep-window end, but in
  the asymptotic region — the *marginal* duration cost of paragraph-vs-sentence
  interruption is small; absolute re-orienting cost is already near its ceiling, so the
  lever is boundary quality, not finer duration.

## 3. Mid-word / gating resume (the hard floor)

- Grosjean (1980). *Spoken word recognition processes and the gating paradigm.* Perception & Psychophysics 28(4), 267–283. doi:10.3758/BF03204386.
  Human Perception and Performance 6(2), 183–202. doi:10.1037/0096-1523.6.2.183.
  Words are identified from incrementally revealed fragments; the recognition point
  (unique identification) depends critically on the **word onset** — mid-word fragments
  (missing onset) severely degrade identification, worst for long/low-frequency words.
- Bashford & Warren (1987). Perception & Psychophysics 42:114–121 (and 42:431–438); Bashford, Meyers, Brubaker & Warren (1988), JASA 84(5):1635–1638 — phonemic restoration
  (filling in masked phonemes from following context, Warren 1970, Science 167) works
  only for **very brief (~50–100 ms) gaps** that obey speech-segmentation rules; it
  cannot bridge seconds-long gaps. → Resume mid-word is unrecoverable by the listener.
- Rule: resume point must always be ≥ the start of the word containing the anchor.
  Paragraph/heading boundary satisfies this; a raw cursor resume does not. If the
  anchor is missing, degrade to last heading / document start — never to raw cursor.

## 4. Platform capability-declaration patterns (how engines declare what they support)

- **Android TTS**: `Voice.getFeatures()` (API 21+) returns a `Set<String>` of
  framework-defined (e.g. `KEY_FEATURE_NETWORK_TIMEOUT_MS`, network/embedded synthesis)
  or engine-specific features; consumers query capabilities before use. Note Android TTS
  offers **only utterance-level** progress (`UtteranceProgressListener`:
  onStart/onDone/onError) — no sentence/word boundary events even in platform TTS.
  Paragraph-level alignment is the platform norm, not a defect.
- **Azure Speech SDK**: `SpeechSynthesizer.Boundary` event —
  `SpeechSynthesisBoundaryEventArgs` with `BoundaryType` (WordBoundary | SentenceBoundary),
  `AudioOffset` + `Text`/`TextOffset`/`TextLength` (dual audio↔text offset), raised only
  where the voice supports it. The gold standard: engines that *can* emit boundaries do,
  with offsets so the app maps text↔audio time.
- **CLAP** (audio plugin standard): `clap_plugin_descriptor.features` array — plugins
  declare capabilities statically; hosts branch on the declared set.
- Common contract: declare capabilities → consumer branches on declared granularity →
  degradation to coarser granularity is explicit and observable, never silent inference.
  (Workspace analogies: GStreamer clock single-authority, AES67 PTP grandmaster,
  Android `PlaybackState` service-writes/UI-reads; anti-pattern: Polys' silent
  `say-as` fallback.)

## 5. Engine-contract single-authority principle

One source of truth for position (`player`/Rust); the engine **declares** its
alignment capability (`AlignmentGranularity::None` for Piper/sherpa-onnx v1) and never
fabricates timing. Branch logic: `Sentence` declared → snap to sentence start;
`None` declared → snap to nearest completed paragraph/heading boundary (parser-side
`pulldown-cmark` + source-range mapping). Fabricating sentence timing from a
boundary-silent engine creates a second position authority → silent desync between
announced and actual position (the defect FR-003 exists to prevent).

## Recommendations

1. **v1 (confirm C1-A/C2-A)**: paragraph/heading resume; declare
   `(SsmlLevel::none, AlignmentGranularity::None)`; correct US2/US3/SC-027 to
   "nearest completed paragraph/heading boundary". Literature-convergent: paragraph ≈
   event scale (EST), satisfies the Grosjean onset floor, and sits at the resumption-lag
   asymptote where finer duration buys little.
2. **Hard invariant**: resume point ≥ word-start; on anchor miss degrade to last
   heading/document start, never raw cursor.
3. **Keep FR-015** bounded rewind (≤30 s, min(5% × paused-duration, 30 s)) —
   re-listening is the retrospective rehearsal Monk et al. (2008) and Trafton et al.
   (2003) show reduces resumption cost.
4. **v2 path**: when an engine declares `Sentence` (Azure-style dual-offset boundary
   events, or SSML `<mark>` if `SsmlLevel` declares support), snap to sentence; adding a
   variant = coordinated closed-set edit + semver bump + docs (constitution rule).
5. **Observability**: record resume granularity + boundary type + rewind magnitude in
   the JSONL diagnostic record (typed fields, no document text) so degradation stays
   observable, per the diag contract.
6. **UX honesty**: announcement copy must not over-claim ("resumed at section start",
   never "exact sentence") — matches declared capability.

## Evidence gaps (honest caveats)

- Direct "boundary vs midpoint resume" experiments exist in the event-segmentation /
  memory literature, not in TTS delivery; the case for paragraph alignment is
  convergent-indirect (EST + gating + resumption lag), which is the strongest available.
- Monk et al. (2008) measured *task* resumption (visually presented goals), not spoken
  prose; transfer assumes the memory-for-goals mechanism applies to the listener's
  narrative model.
- Grosjean (1980) full text not re-fetched (paywalled); citation verified across
  multiple secondary sources; gating conclusions are standard psycholinguistics.
