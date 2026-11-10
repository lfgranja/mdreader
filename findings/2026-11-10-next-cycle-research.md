# Next-Cycle Research — mdreader / Ressoa

**Date:** 2026-11-10
**Author:** research subagent (multilingual web research, EN/PT/ES)
**Scope:** Eight topics requested for the next 80/20 cycle of `mdreader` (product **Ressoa**), an Android offline Markdown TTS audiobook reader. Rust owns playback/synthesis; Kotlin is a mirror; Tauri bridge; Piper pt-BR via sherpa-onnx; mono `f32` @ 22 050 Hz; zero `INTERNET` permission; design system `DESIGN-SONA`.
**Method:** Primary sources prioritized (W3C, CommonMark, ISO/ECMA-style specs, official vendor docs, GitHub issues/source, government gazettes, court opinions, platform policy pages). Secondary sources used only where marked. All URLs accessed **2026-11-10** unless stated.

**Confidence legend**
- **HIGH** — verified against a primary source fetched in this session.
- **MED** — verified against a reputable secondary source that itself cites primary sources, or against two independent sources.
- **[UNVERIFIED]** — could not be confirmed from a primary source; treat as a lead, not a fact.

---

## 0. Executive summary

| # | Topic | Headline finding | Confidence |
|---|---|---|---|
| 1 | Markdown→speech normalization | There is a mature, convergent industry pattern: strip markers, keep visible text, protect code/URL/math verbatim, reduce links to their label. `***` is usually a *structural* break, not three asterisks. The project already implements a first cut (`normalize.rs`) but it is regex-line-based and has gaps (setext, links/images, tables, blockquotes, nested emphasis, `a*b`, global orphan-`*` removal). | HIGH |
| 2 | Semantic prosody | Without SSML, the only reliable per-segment lever in sherpa-onnx/Piper VITS is **speed** (`GenerationConfig.speed`). Pitch/energy/duration variation is set at **engine config** level (`noise_scale`, `noise_scale_w`, `length_scale`), not per request. Research supports quote/dialogue pitch differentiation, but VITS control is coarse. | HIGH |
| 3 | sherpa-onnx 1.13.x + Piper VITS | **Critical:** sherpa-onnx **2.0.0 (released 2026-10-23) removed espeak-ng and piper-phonemize** (GPL-3.0) and deprecated/ignored `data_dir`. The project pins `=1.13.8`, whose prebuilt static lib **bundles GPL-3.0 espeak-ng** — a real licensing exposure for an MIT/Apache-2.0 app. Word-level timestamps are **not implemented** (issue #3705 open); `GeneratedAudio` exposes only `samples()`, `sample_rate()`, `save()`. | HIGH |
| 4 | Voice cloning ethics & licensing | A synthetic "Lula" voice in a commercial app is **high-risk**: LGPD treats voice as sensitive biometric data (art. 5º, II; art. 11 requires specific consent), STJ case law protects voice as a personality right, PL 1530/2026 would require express documented consent, TSE Res. 23.755/2026 restricts synthetic voice of candidates/public figures, and Google Play / Apple policies cover apps that create voice recordings of real people. Piper `faber` is CC0 *dataset* but fine-tuned from `lessac` (Blizzard 2013 research-only license). | HIGH |
| 5 | Android accessibility | Android 16 **deprecates `announceForAccessibility` / `TYPE_ANNOUNCEMENT`**; use `setAccessibilityPaneTitle`, `setAccessibilityLiveRegion`, `CONTENT_CHANGE_TYPE_ERROR`. Word-level highlighting needs engine word timings, which sherpa-onnx does not provide — segment/sentence highlighting is the realistic target. MediaSession is required for lock-screen/Assistant control. | HIGH |
| 6 | Market & competitors | The UVP holds: no major competitor is 100% offline + air-gapped + Markdown-native. Closest open-source analog is **Lector** (F-Droid, GPLv3): Markdown + folder + zero permissions + system TTS. Pricing gap is real (Speechify $139/yr, NaturalReader $79–199/yr, Voice Dream $59.99–79.99/yr vs. planned one-time $9.99–14.99). | HIGH (pricing MED; some figures [UNVERIFIED]) |
| 7 | Rust audio on Android | cpal 0.18.x is the right backend (AAudio, `realtime` feature, `StreamTrait::now()`, unified errors). Known Android pitfalls: huge reported min buffer (#890, fixed Nov 2025), fixed buffer size not honored (#902), low-latency mode not default, callback must never block. Time-stretch is a separate crate problem: `timestretch` (MIT) is the best pure-Rust option; avoid `rubberband-rs` (GPL-2.0). | HIGH |
| 8 | Spec-driven development / dark factory | The 2026 consensus: spec is the unit of work; acceptance criteria written first become gates; two-gate review (spec compliance + code quality); human writes the failing test before the agent writes code; deterministic verification; human gates on irreversible/high-risk changes. "Dark software factory" is a 2025–26 coinage, not a standard. | HIGH (practice), MED (terminology) |

---

## 1. Markdown-to-speech normalization best practices (2025–2026)

### 1.1 The normative baseline: CommonMark 0.31.2

- The current CommonMark specification is **0.31.2 (2024-01-28)**, CC-BY-SA 4.0. It defines ATX headings (1–6 `#` followed by space/EOL), setext headings (`=` / `-` underlines), fenced code (3+ backticks or tildes), indented code (4 spaces), thematic breaks (`***`, `---`, `___` with 3+ markers), emphasis/strong (`*`/`_`), links/images, blockquotes (`>`), lists, HTML blocks, and backslash escapes. Backslash escapes **do not work** inside code spans, code blocks, autolinks, or raw HTML.
- **Why it matters for TTS:** CommonMark gives an unambiguous parse of what is *visible text* vs. *markup*. The correct normalization is "render to visible text, then speak that" — not "delete punctuation characters with regex." CommonMark also documents that `***` is a **thematic break** (`<hr/>`), i.e. a structural section boundary, in the overwhelming majority of real documents.

### 1.2 The convergent industry pattern

Four independent implementations converge on the same rule set:

| Source | Rule set |
|---|---|
| **Microsoft Azure Speech `mstts:markdown`** (primary vendor doc) | Wrap markdown in `<mstts:markdown>`; "TTS only reads the text content in the markdown and ignores the markdown syntax." Supports CommonMark syntax. Example: `# This is headline` + `And this is a **bold** text` reads as two sentences, syntax not read. |
| **Pipecat `MarkdownTextFilter` / `StripMarkdown`** (open source, BSD-2) | Strips bold/italic, backtick code spans, fenced code blocks, ATX headers, list bullets, blockquote markers, horizontal rules. Struck-through text is removed with its markers ("text the reply takes back"). Links/images reduced to label/alt text; **URL not spoken** by default. An asterisk that is *content* stays (`2*3`, `\*`, `Press * then #`). Alphanumeric-preserving outside fenced code/struck-through/URLs. |
| **NousResearch `hermes-agent` `tts_text_normalize.py`** (MIT) | Deterministic pipeline: non-spoken blocks → markdown → symbols/units → identifier-dense tokens → line formatting → single line. Strips code blocks, images (keeps alt), links (keeps label), URLs, media paths, inline code, bold/italic. Normalizes `%`→"percent", `/`→"per" in numeric contexts, dates, money. Prunes machine tokens that would be spelled out character-by-character. |
| **White Noise Android issue #1714** (product requirement, 2026-07) | The clearest statement of desired behavior: headings speak text without `#`; emphasis/strong/strikethrough speak child text; lists/quotes preserve item/block boundaries without marker syntax; fenced/inline code retain visible content without fences/backticks; a labeled link speaks only a non-URL visible label, destination **never** spoken; normalize whitespace around omitted URLs; preserve literal characters that are actual visible content (escaped markdown, code); preserve sentence/block boundaries. |

**Consensus rules for a TTS normalizer:**
1. **Strip, don't verbalize.** Markers are deleted, not read as "asterisk". Reading them is the failure mode.
2. **Protect verbatim zones:** fenced code, inline code, URLs/paths, math (`$...$`, `2*3`), and any escaped literal.
3. **Thematic break → pause/boundary**, not "dash dash dash". In the project's chunker this maps to a section boundary or a `.` placeholder (already implemented).
4. **Links/images → visible label/alt text.** Never speak the destination. If a segment is URL-only, either omit or say "link".
5. **Keep whitespace that affects prosody** (paragraph/block boundaries), but collapse formatting-induced runs.
6. **Idempotency** is a hard requirement (normalizing twice must be a no-op) — the project already tests this.
7. **`.md` scope only** for structure-aware rules; `.txt` should get at most URL handling.

### 1.3 Screen-reader evidence (NVDA / VoiceOver / TalkBack)

- Screen readers do **not** read most punctuation; commas/periods become pauses, question marks raise pitch, exclamation marks are often not conveyed. Bold/italic are silent by default; `<em>`/`<strong>` make no difference to the listener. (Accessibility.tools, citing Bohman 2014.)
- Symbol support is uneven and *user-configurable*. A 2014 test of ~91 symbols found only 17 read by all three of JAWS/NVDA/VoiceOver; a 2023 retest found many more read by all three, but **not in the same way** (`$21` → "dollar twenty one" in JAWS vs. "twenty one dollars" in NVDA). (Deque 2014; Eleven Ways 2023.)
- VoiceOver exposes punctuation groups (All / Some / None / Custom) and per-symbol actions (Ignore / Replace / Remove) — e.g. read `*` as "star" instead of "asterisk". (Apple Support.)
- NVDA exposes punctuation/symbol levels and `symbols.dic` per locale; pt-BR maps `*` → "asterisco". (NV Access docs; NVDA pt-BR `symbols.dic`.)
- **Implication:** verbosity is a *user setting* in screen readers, so an app cannot assume markers are inaudible. A TTS reader must make its own policy. The project's `NormalizationMode::{Fluid, Literal}` (Fluid default, Literal opt-in) mirrors the screen-reader "some/none" tradeoff and is the right product shape.

### 1.4 "Symbol fatigue" — terminology note

"Symbol fatigue" is the project's own term (`.specify/assessments/tts-symbol-fatigue/`), **not** a standard industry term. The underlying phenomena are documented:
- **Listening-related fatigue** from non-essential auditory load (Esser, Picou & Hornsby, *American Journal of Audiology*, 2025, cited in the project's assessment).
- **Verbosity-induced cognitive load** in screen-reader use (Deque; Eleven Ways; W3C WAI/COGA guidance on formatting and punctuation).
Treat "symbol fatigue" as a project-level UX label, and cite the listening-fatigue literature when writing the spec.

### 1.5 Current project implementation vs. the consensus

`crates/adapter-synthesis/src/normalize.rs` (read 2026-11-10) already implements:
- Fenced-code protection (line scan), inline-code protection via PUA placeholders, URL protection, `$...$` math protection, `\d+*\d+` multiplication protection.
- Thematic break → `.`, ATX heading strip, list bullet strip, bold/italic strip, orphan `*` removal, placeholder restore.
- `NormalizationMode::{Fluid, Literal}`, idempotency test.

**Gaps relative to the consensus (actionable):**
1. **Setext headings** are not handled (chunker is line-scan, not fence-aware; module docs already record this deviation).
2. **Links/images** `[label](url)` and `![alt](url)` are not reduced to label/alt — only bare URLs are protected. A markdown link will be spoken with its destination.
3. **Tables** (pipe tables) are not handled at all.
4. **Blockquotes** (`>`) are not stripped.
5. **Strikethrough** (`~~x~~`) is not removed.
6. **Nested/intraword emphasis** and `a*b` (non-numeric multiplication) are not protected; the global `s.replace('*', "")` at the end can mangle literal asterisks that survived placeholder protection.
7. **HTML tags** are not stripped.
8. The placeholder mechanism uses PUA codepoints `U+E000`/`U+E001` and a regex `\u{E000}(\d+)\u{E001}`; a document containing those codepoints would collide. (Low probability, but a closed-set/purity concern for a fail-closed project.)
9. Dead code: `let out = String::with_capacity(...)` and `let fence_buf = String::new()` are unused and discarded with `let _ = (out, fence_buf);`.

**Sources — Topic 1**
- W3C, *Speech Synthesis Markup Language (SSML) Version 1.1*, W3C Recommendation 2010-09-07 — https://www.w3.org/TR/speech-synthesis11/ (accessed 2026-11-10).
- CommonMark, *Spec 0.31.2* (2024-01-28) — https://spec.commonmark.org/0.31.2/ (accessed 2026-11-10).
- CommonMark, changelog — https://spec.commonmark.org/changelog.txt (accessed 2026-11-10).
- Microsoft Learn, *Pronunciation with SSML* (section "Markdown Reading") — https://learn.microsoft.com/en-us/azure/ai-services/speech-service/speech-synthesis-markup-pronunciation (updated 2026-02-25; accessed 2026-11-10).
- Pipecat, *MarkdownTextFilter* — https://docs.pipecat.ai/server/utilities/text/markdown-text-filter (accessed 2026-11-10).
- Pipecat, `strip_markdown` source — https://reference-server.pipecat.ai/en/stable/_modules/pipecat/utils/text/transforms/strip_markdown.html (accessed 2026-11-10).
- NousResearch, `hermes-agent/tools/tts_text_normalize.py` — https://github.com/NousResearch/hermes-agent/blob/main/tools/tts_text_normalize.py (accessed 2026-11-10).
- White Noise Android issue #1714, *Read rendered markdown aloud without formatting syntax or URLs* — https://github.com/marmot-protocol/whitenoise-android/issues/1714 (2026-07-28; accessed 2026-11-10).
- openclaw issue #10485, *Strip markdown formatting before TTS conversion* — https://github.com/openclaw/openclaw/issues/10485 (closed 2026-04-26; accessed 2026-11-10).
- Accessibility.tools, *Do screen readers recognize different typographic elements?* — https://accessibility.tools/docs/accessibility/sr-typography (accessed 2026-11-10).
- Deque, *Don't screen readers read what's on the screen? Part 1: Punctuation and typographic symbols* (2014) — https://www.deque.com/blog/dont-screen-readers-read-whats-screen-part-1-punctuation-typographic-symbols (accessed 2026-11-10).
- Eleven Ways, *How screen readers read special characters: an update* (2023-03-17) — https://www.elevenways.be/en/articles/screenreaders-special-characters (accessed 2026-11-10).
- Apple Support, *Customize punctuation in VoiceOver on Mac* — https://support.apple.com/guide/voiceover/customize-punctuation-vod1f1248b5e/mac (accessed 2026-11-10).
- NV Access, NVDA 2025.2 User Guide — https://download.nvaccess.org/releases/2025.2/documentation/userGuide.html (accessed 2026-11-10); NVDA pt-BR `symbols.dic` — https://raw.githubusercontent.com/nvaccess/nvda/master/source/locale/pt_BR/symbols.dic (accessed 2026-11-10).
- W3C WAI/COGA, *Use Clear, Unambiguous Formatting and Punctuation* — https://www.w3.org/WAI/WCAG2/supplemental/patterns/o3p06-format-punctuation/ (accessed 2026-11-10).
- Esser, Picou & Hornsby, *Listening-Related Fatigue*, American Journal of Audiology (2025) — https://pubs.asha.org/doi/10.1044/2025_AJA-25-00018 (cited in project assessment; accessed 2026-11-10).
- Local: `crates/adapter-synthesis/src/normalize.rs`, `specs/014-tts-symbol-normalization/spec.md`, `.specify/assessments/tts-symbol-fatigue/research.md` (read 2026-11-10).

---

## 2. Semantic prosody for structured documents

### 2.1 What SSML would give (and why the project doesn't use it)

SSML 1.1 (W3C Rec 2010-09-07) is the declarative standard. Relevant elements:
- `<p>` / `<s>` — paragraph/sentence structure; processors infer pauses.
- `<prosody pitch rate volume contour>` — the main prosodic control.
- `<emphasis>` — a shorthand for prosodic emphasis.
- `<break strength time>` — explicit pauses (`weak`/`medium`/`strong`/`x-strong`, or `time="300ms"`).
- `<say-as>`, `<phoneme>`, `<sub>`, `<lang>` — normalization/pronunciation.
- `<mark>` — a marker for later reference (e.g. trimming).

**SSML 1.1 is the latest version; there is no SSML 2.0.** Vendor support varies widely (Azure, Google, AWS, IBM, Polly all implement subsets; ElevenLabs v3/v4 do not support `<break>`). The project declares `SsmlLevel::None`, so all prosody must be achieved through **plain text + engine parameters + segmentation**.

### 2.2 What Piper VITS / sherpa-onnx actually exposes

Two distinct layers:

**(a) Engine/model config — set once at engine creation (`OfflineTtsVitsModelConfig`, sherpa-onnx 2.0.1 docs):**
- `noise_scale: f32` — "noise added to the generator" (default 0.667). Controls voice/pitch variation.
- `noise_scale_w: f32` — variation in phoneme durations (default 0.8). Only used when the model has a stochastic duration predictor (`use_sdp`).
- `length_scale: f32` — speaking speed (default 1.0; >1 slower, <1 faster).
- `model`, `lexicon`, `tokens`, `dict_dir`; `data_dir` is **deprecated and ignored since v2.0.0**.
- These are **global to the engine instance**. Changing them per segment would require rebuilding/reconfiguring the engine (or multiple engine instances).

**(b) Per-request generation (`GenerationConfig`, sherpa-onnx 2.0.1 docs):**
- `speed: f32` — per-request speed.
- `silence_scale: f32` — trailing silence scaling.
- `sid: i32` — speaker id (multi-speaker models).
- `reference_audio`, `reference_sample_rate`, `reference_text`, `num_steps`, `extra`, `phoneme_codepoints`, `tokens`.
- **No per-request pitch, volume, energy, or duration-variation control.**

**Answer to the research question:** sherpa-onnx does **not** expose per-segment pitch/energy/duration controls. The only per-segment prosodic lever is `GenerationConfig.speed`. VITS's pitch/energy variation is governed by `noise_scale`/`noise_scale_w` at model-config level, and by the model's stochastic latent sampling. `length_scale` is the model-level equivalent of speed.

**Streaming vs. batch:** `OfflineTts::generate_with_config` is synchronous and returns a complete `GeneratedAudio` (samples + sample rate). There is an async API with a progress callback, but the callback reports post-synthesis progress, not intra-segment streaming. VITS is a non-autoregressive model, so it does not naturally stream; the project's spec 013 correctly treats "intra-model streaming" as out of scope.

### 2.3 What the research says about prosody in audiobook reading

- *Prosody Analysis of Audiobooks* (arXiv 2310.06930) trained models to predict pitch, volume, and rate from narrative text and found predicted prosody correlated better with human audiobook readings than a state-of-the-art commercial TTS system (higher pitch correlation in 22/24 books; volume in 23/24). Readers use **higher pitch for dialogue/quotes and lower pitch for descriptive text**. This is the strongest empirical support for differentiating quote/dialogue prosody.
- SSML `<prosody>` is the standard mechanism when supported; without it, text structure and punctuation are the only signals.

### 2.4 Practical recommendations for mdreader

1. **Use `GenerationConfig.speed` per segment** — this is the one lever that exists. The project's `prosody.rs` already maps `Heading|Strong → 0.9`, `Emphasis|Quote|List → 0.95`, `Code → 0.85`. That is appropriately conservative.
2. **Do not attempt per-segment pitch/volume** — it is not exposed. Any "posture" change beyond speed will be inconsistent.
3. **Order the pipeline: classify → plan → normalize → synthesize.** Normalizing first destroys the markers needed for classification (a known conflict recorded in the project's own assessment). The project's spec 015 FR-PROS-006 already calls for this coordination.
4. **Protected zones:** code should be read literally, slowly, and *without* prosodic exaggeration; math and URLs verbatim.
5. **Paragraph/heading boundaries** are better signalled by segment boundaries and short pauses than by pitch changes, given the engine's limits.
6. **Conservative defaults** (spec 015 FR-PROS-005) are correct: over-prosody is fatiguing and can reduce comprehension.

**Sources — Topic 2**
- W3C SSML 1.1 — https://www.w3.org/TR/speech-synthesis11/ (accessed 2026-11-10).
- sherpa-onnx Rust docs, `OfflineTtsVitsModelConfig` (2.0.1) — https://docs.rs/sherpa-onnx/latest/sherpa_onnx/struct.OfflineTtsVitsModelConfig.html (accessed 2026-11-10).
- sherpa-onnx Rust docs, `GenerationConfig` (2.0.1) — https://docs.rs/sherpa-onnx/2.0.1/sherpa_onnx/struct.GenerationConfig.html (accessed 2026-11-10).
- sherpa-onnx Rust docs, `GeneratedAudio` (2.0.1) — https://docs.rs/sherpa-onnx/2.0.1/sherpa_onnx/struct.GeneratedAudio.html (accessed 2026-11-10).
- Piper documentation, *Training Guide* (inference parameters: `noise_scale`, `length_scale`, `noise_w`) — https://tderflinger.github.io/piper-docs/guides/training (accessed 2026-11-10).
- Piper documentation, `config.py` (defaults 0.667 / 1.0 / 0.8) — https://tderflinger.github.io/piper-docs/codereading/python_run/piper/config (accessed 2026-11-10).
- Piper source, `piper_train/vits/models.py` (`w = torch.exp(logw) * x_mask * length_scale`) — https://github.com/OHF-Voice/piper1-gpl/blob/main/src/piper/train/vits/models.py (accessed 2026-11-10).
- StreamKit, `plugin::native::piper` parameter reference — https://streamkit.dev/reference/plugins/plugin-native-piper (accessed 2026-11-10).
- arXiv, *Prosody Analysis of Audiobooks* (2310.06930) — https://arxiv.org/html/2310.06930v2 (accessed 2026-11-10).
- Local: `crates/adapter-synthesis/src/prosody.rs`, `crates/adapter-synthesis/src/synth.rs`, `specs/015-md-semantic-prosody/spec.md`, `.specify/assessments/md-semantic-prosody/research.md` (read 2026-11-10).

---

## 3. sherpa-onnx 1.13.x + Piper VITS capabilities (2026)

### 3.1 Version status — the single most important finding

| Version | Status | Notes |
|---|---|---|
| **1.13.8** | **Pinned by mdreader** (`crates/adapter-synthesis/Cargo.toml`: `sherpa-onnx = "=1.13.8"`; `Cargo.lock` confirms) | Prebuilt static lib bundles ONNX Runtime, **espeak-ng (GPL-3.0)** and **piper-phonemize**. `data_dir` is required and used. |
| **1.13.3–1.13.8** | GitHub releases Jun–Aug 2026 | Last 1.x line. |
| **2.0.0** | **Released 2026-10-23** | **Removes espeak-ng and piper-phonemize** (PR #4055, merged 2026-10-20, closing issue #3731). Breaking change. |
| **2.0.1** | Rust crate published 2026-10-27 | `data_dir` deprecated/ignored; `GenerationConfig::{phoneme_codepoints, tokens}` added for external phonemization. |

**Why 2.0.0 exists:** espeak-ng is GPL-3.0; linking it into sherpa-onnx (Apache-2.0) is a one-way-incompatible combination. Upstream removed it to keep sherpa-onnx Apache-2.0-clean. The maintainer confirmed model `.onnx` files are unchanged; a `lexicon.txt` is added to model directories instead.

**Migration paths for 2.x:**
1. Provide a **`lexicon.txt`** (`word || p1 p2 p3 ...` or `word1 word2 ... || p1 p2 ...`).
2. Use an **external phonemizer** (e.g. `piper-phonemize`, or the permissively licensed `floravox-capi`, Apache-2.0/MIT) and pass `GenerationConfig::phoneme_codepoints` / `tokens`.
3. Community lexicons: `AACTools/voicegarden-lexicons` (MIT/BSD, no GPL, 14 languages incl. `pt`, derived from gruut, whose phoneme symbols match Piper's non-English voices).

### 3.2 Capabilities and limits (verified)

- **Word-level timestamps: NOT available.** GitHub issue #3705 (*[FEATURE] Word-Level Timestamps in TTS Output*, opened 2026-06-26) is **open**, unassigned, no milestone. The proposed API (`return_timestamps`, `WordTimestamp`) is a feature request, not a shipped feature. `GeneratedAudio` exposes only `samples()`, `sample_rate()`, `save()`. Phoneme timing is implemented in the **piper-plus** fork (extracts VITS `durations` tensor → JSON/TSV/SRT), but that is not sherpa-onnx.
- **Forced alignment:** issue #3536 (opened 2026-04-21) is a feature request; maintainer prefers HMM/GMM (Montreal Forced Aligner) over CTC. Not shipped.
- **Streaming:** offline TTS is batch. Async generation with a progress callback exists, but the callback is post-synthesis. No intra-model VITS streaming.
- **Languages/voices:** Piper's catalog lists ~175 voices across 56 language/locale combinations (secondary source). For **pt_BR** the catalog has **four voices, all male**: `cadu` (medium, CC0), `edresson` (low, CC-BY-4.0), `faber` (medium, CC0), `jeff` (medium). **There is no female pt-BR Piper voice** (feature request rhasspy/piper#766, open since 2025-04). sherpa-onnx additionally hosts `vits-piper-pt_BR-dii-high`, `miro-high`, and `supertonic-3-pt`. pt_PT has `tugão` (male).
- **Known quirks:**
  - `pt_BR-edresson-low` drops every nasal tilde (U+0303 missing from its phoneme map) — OHF-Voice/piper1-gpl issue #350 (2026-11-03).
  - Punctuation sensitivity in Piper/eSpeak — rhasspy/piper#122 (open since 2023).
  - OOV words are **silently skipped** by lexicon frontends (reported for Kokoro; relevant to any 2.x lexicon migration).
  - `data_dir` is ignored in 2.x; passing it has no effect.

### 3.3 Licensing exposure for mdreader (must-fix)

- The project's `PROVENANCE.md` states the 1.13.8 static lib "bundles ONNX Runtime, espeak-ng and piper_phonemize as static libraries."
- `adapter-synthesis` is **MIT**; the workspace uses `cargo-deny`. **GPL-3.0 code statically linked into a distributed APK is a license conflict** for an MIT/Apache-2.0 app, independent of `cargo-deny` (which inspects crate metadata, not bundled binary assets).
- Two independent third-party projects hit exactly this: `loyality7/speech-to-speech-mobile` issue #27 (project license changed to GPL-3.0 pending sherpa-onnx 2.0.0) and an RPG-to-Rust porting write-up ("Trap 1: the released sherpa-onnx (1.13.8) builds espeak-ng and piper-phonemize, both GPL, into every build that includes speech").
- **Recommendation:** treat the espeak-ng bundling as a release-blocking compliance item. Either (a) migrate to sherpa-onnx 2.x with a permissively licensed lexicon (e.g. voicegarden `pt` bundle) and document the removal of `espeak-ng-data/`, or (b) obtain a proper GPL compliance path (GPL-3.0 the app, or replace the phonemizer). Option (a) is aligned with upstream and with the project's MIT/Apache posture.

**Sources — Topic 3**
- sherpa-onnx, *Breaking Change: Remove espeak-ng and piper-phonemize dependency* (#3731, closed 2026-10-20 via #4055) — https://github.com/k2-fsa/sherpa-onnx/issues/3731 (accessed 2026-11-10).
- sherpa-onnx PR #4055, *Remove espeak-ng and piper-phonemize from sherpa-onnx* (merged 2026-10-20) — https://github.com/k2-fsa/sherpa-onnx/pull/4055 (accessed 2026-11-10).
- sherpa-onnx, *Release v2.0.0* (2026-10-23) — https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.0 (accessed 2026-11-10).
- sherpa-onnx Rust docs 2.0.1 (`OfflineTtsVitsModelConfig`, `GenerationConfig`, `GeneratedAudio`) — https://docs.rs/sherpa-onnx/2.0.1/sherpa_onnx/ (accessed 2026-11-10).
- sherpa-onnx issue #3705, *Word-Level Timestamps in TTS Output* (open) — https://github.com/k2-fsa/sherpa-onnx/issues/3705 (accessed 2026-11-10).
- sherpa-onnx issue #3536, *Support forced alignment* (open) — https://github.com/k2-fsa/sherpa-onnx/issues/3536 (accessed 2026-11-10).
- sherpa-onnx issue #4002, *eSpeak-NG provenance and licensing evidence* (closed; maintainer: "espeak-ng has been removed since sherpa-onnx v2.0.0") — https://github.com/k2-fsa/sherpa-onnx/issues/4002 (accessed 2026-11-10).
- piper-plus, *Phoneme Timing Output* — https://github.com/ayutaz/piper-plus/blob/dev/docs/features/phoneme-timing.md (accessed 2026-11-10).
- rhasspy/piper-voices, `pt_BR/faber/medium/MODEL_CARD` — https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/pt/pt_BR/faber/medium/MODEL_CARD (accessed 2026-11-10).
- rhasspy/piper-voices, `pt_BR/cadu/medium/MODEL_CARD` — https://huggingface.co/rhasspy/piper-voices/resolve/67265bba2397cfb86ff687cfc7ffe3a0e3c3aa55/pt/pt_BR/cadu/medium/MODEL_CARD (accessed 2026-11-10).
- rhasspy/piper-voices discussion #105, *Licensing basis for lessac-derived voices* — https://huggingface.co/rhasspy/piper-voices/discussions/105 (accessed 2026-11-10).
- rhasspy/piper-voices discussion #111, *Confirm commercial-use license for pt_BR Cadu and Faber* — https://huggingface.co/rhasspy/piper-voices/discussions/111 (accessed 2026-11-10).
- rhasspy/piper issue #766, *Add Piper Voice Female in Portuguese Br and Pt* (open) — https://github.com/rhasspy/piper/issues/766 (accessed 2026-11-10).
- rhasspy/piper issue #122, *Incorrect Reading of Punctuation Marks* — https://github.com/rhasspy/piper/issues/122 (accessed 2026-11-10).
- OHF-Voice/piper1-gpl issue #350, *pt_BR-edresson-low drops every nasal tilde* — https://github.com/OHF-Voice/piper1-gpl/issues/350 (accessed 2026-11-10).
- AACTools/voicegarden-lexicons (MIT/BSD lexicons, 14 languages) — https://github.com/AACTools/voicegarden-lexicons (accessed 2026-11-10); `floravox-capi` — https://crates.io/crates/floravox-capi (accessed 2026-11-10).
- `loyality7/speech-to-speech-mobile` issue #27, *espeak-ng is GPL-3.0 and is compiled into the binary we redistribute* — https://github.com/loyality7/speech-to-speech-mobile/issues/27 (accessed 2026-11-10).
- Cekura, *Piper TTS: Definition, Metrics and Why It Matters* (2026-08-28; secondary) — https://www.cekura.ai/discover/piper-tts (accessed 2026-11-10).
- Local: `crates/adapter-synthesis/Cargo.toml`, `Cargo.lock`, `crates/adapter-synthesis/src/synth.rs`, `crates/adapter-synthesis/assets/voices/pt_BR-faber-medium/PROVENANCE.md`, `specs/013-ttfa-synthesis-pipeline/spec.md` (read 2026-11-10).

---

## 4. Voice cloning / real-person voice synthesis — ethics & licensing (2026)

### 4.1 The project's exposure

`specs/016-voz-lula/spec.md` (Draft, created 2026-11-08) proposes a synthetic voice honouring Brazilian President Luiz Inácio Lula da Silva. The spec is a **bare draft**: the `assessments/`, `checklists/`, `contracts/`, `converge/`, `proposals/`, `research/`, and `tasks/` directories are all empty. `assets/voices/pt_BR-lula-medium/PROVENANCE.md` states the model is "pendente" (pending), the license is "pendente", and the model is expected from "treinamento a partir de dados de domínio público do Presidente ... ou doação de modelo pré-treinado."

Separately, `.specify/assessments/custom-voices/intake.md` records a request for voices "tipo o Waze", explicitly naming a dubbing voice (Patrícia Coutinho as Una Chin-Riley) and flagging the legal question of cloning dubbing actors' voices.

### 4.2 United States

- **No federal statute prohibits cloning a real person's voice** as of 2026. Protection is a patchwork of state right-of-publicity laws, trademark/unfair-competition claims, and fraud statutes. The federal **NO FAKES Act** remains a bill.
- **Tennessee ELVIS Act (2024)** — the first state law to add "voice" to right-of-publicity protections. Primary text (TN HB 2091 / SB 2096, effective 2024-07-01):
  - Defines "Voice" as "a sound in a medium that is readily identifiable and attributable to a particular individual, **regardless of whether the sound contains the actual voice or a simulation of the voice** of the individual."
  - Creates civil liability for publishing/distributing/making available an individual's voice or likeness **with knowledge** that use was not authorized; and for distributing a tool "the primary purpose or function of which is the production of a particular, identifiable individual's ... voice."
  - Amends Tenn. Code Ann. § 47-25-1105; unauthorized use is also a **Class A misdemeanor**.
  - Expresses are protected for "comment, criticism, scholarship, satire, or parody."
- **California** AB 2602 / AB 1836 (2025) regulate performer contracts and digital replicas, including deceased performers.
- **Washington** SSB 5886 (effective 2026-06-11) adds "forged digital likeness" — explicitly including audio — to the personality-rights law; requires the likeness to be "indistinguishable from a genuine" one and "likely to deceive a reasonable person."
- **Texas** has no general voice right; its deepfake statute (Penal Code 21.165) targets sexually explicit deepfakes. **New York** has several rules (Civil Rights Law 50/51; GOL 5-302; GBL 396-b disclosure).
- **FCC:** AI-voice robocalls require prior express consent (TCPA); a $6,000,000 forfeiture arose from the AI Biden robocall.

### 4.3 European Union

- **EU AI Act Article 50** (Regulation (EU) 2024/1689) **applies from 2026-08-02**. Key duties:
  - **Art. 50(1)** — providers must ensure AI systems interacting directly with people make it known that they are AI.
  - **Art. 50(2)** — providers must mark AI-generated/manipulated image, audio, video, and text in a **machine-readable** format and make it detectable as artificially generated.
  - **Art. 50(3)** — deployers of emotion recognition / biometric categorisation must inform exposed persons.
  - **Art. 50(4)** — deployers must disclose deepfakes and AI-generated text on matters of public interest.
- **Article 3(60)** defines "deepfake" as AI-generated or manipulated image, audio, or video that resembles existing persons/objects/places/entities and would falsely appear authentic. Three cumulative criteria: resemblance, existing subject, false appearance of authenticity.
- **Crucially: Article 50 does not require consent to clone a voice.** It is a transparency/marking obligation. The only consent-based prohibition added by the Digital Omnibus covers non-consensual intimate images (applies 2026-12-02) and does not extend to voice generally.
- **Digital Omnibus (Regulation (EU) 2026/1744**, adopted 2026-07-08, in force 2026-07-27): kept Art. 50 in force; a transitional deadline of **2026-12-02** applies only to Art. 50(2) marking for generative AI systems placed on the market before 2026-08-02. High-risk rules were deferred to 2027/2028.
- Penalties: up to €15,000,000 or 3% of worldwide turnover for Article 50 violations (lower for SMEs per Art. 99(6)).

### 4.4 Brazil — the jurisdiction that matters most for "voz Lula"

- **No specific voice-cloning law is in force.** The Constitution (art. 5º, X and XXVIII), the Civil Code (arts. 20–21), the LGPD, and STJ jurisprudence protect image and voice.
- **LGPD (Lei 13.709/2018)** — primary text verified:
  - **Art. 5º, II** — "dado pessoal sensível" includes "dado genético ou **biométrico**, quando vinculado a uma pessoa natural." Voice is biometric data.
  - **Art. 11, I** — sensitive personal data may only be processed with the data subject's **specific and highlighted consent for specific purposes** (or another Art. 11 basis).
  - **Art. 20** — the data subject may request human review of decisions taken solely by automated processing that affect their interests, including decisions defining their personality aspects.
  - **Art. 7º, §3º** — processing of publicly accessible personal data must consider the purpose, good faith, and public interest that justified its availability (a limit on "it was public, so I could use it").
- **STJ jurisprudence** (primary opinion excerpt, REJ 201403080659): "A voz humana encontra proteção nos direitos da personalidade, seja como direito autônomo ou como parte integrante do direito à imagem ou do direito à identidade pessoal." Consent for image use must in general be **express**; tacit consent is admitted only restrictively and exceptionally. **Súmula 403**: unauthorized publication of a person's image for economic/commercial purposes is compensable **without proof of loss**.
- **Pending legislation:**
  - **PL 2338/2023** (approved by the Federal Senate in Dec 2024; pending in the Chamber; art. 66 would require AI use of image/audio/voice identifying natural persons to respect personality rights under the Civil Code). The government also sent PL 6.237/2025 on AI governance. As of 2026-11, no final vote.
  - **PL 1530/2026** — would require **express, specific, and documented consent** from the image-right holder for synthetic content reproducing voice, image, biometric traits, or other identifiable attributes, specifying the piece, purpose, term, and means of dissemination; for deceased persons, consent from heirs/the estate.
  - **PL 1460/2026** — would create a specific regime protecting image, voice, and artistic expression against unauthorized digital replicas.
- **Electoral regulation (TSE):**
  - **Resolução TSE nº 23.755/2026** (2026-03-02) — regulates AI in the 2026 elections: mandatory labelling of synthetic content; a **72-hour pre-election / 24-hour post-election blackout** on new synthetic content using the image, voice, or manifestation of candidates or public figures, even if labelled; provider liability for non-removal.
  - **TSE thesis (2026-09-01)** — "deepfake" requires synthetic content produced/manipulated by AI with a degree of realism, capable of creating/reproducing/altering the image, voice, or manifestation of a living, deceased, or fictitious person; the prohibition applies when the content is characterised as **electoral propaganda**.
  - **TSE–ElevenLabs agreement (2026-08-06)** — creates a list of "prohibited voices" of candidates and electoral authorities, restricting synthetic generation of those voices; valid to 2026-12-31. This is direct evidence that a sitting president's voice is treated as a protected electoral asset in Brazil.
- **Enforcement context:** the "falso advogado" scheme (São Paulo Civil Police, May 2026) used AI to reproduce real lawyers' voices; China's Supreme People's Court (Sept 2026) ruled that using or disclosing an AI-replicated identifiable face or voice without consent may infringe personality rights; Japan's Tokyo District Court (2026-09-30) ruled voice is protected by image/publicity rights (first such ruling).

### 4.5 Platform policies

- **Google Play AI-Generated Content policy** — covers "apps that create voice and/or video recordings of real-life individuals using AI." Examples of violative content include "voice or video recordings of real-life individuals that facilitate scams." Developers must comply with Deceptive Behavior and Restricted Content policies. Play Console requires **self-declaration** of AI-generated content/assets, which are then AI-labelled on the store.
- **Apple App Review Guidelines** — as updated **2026-10-16**: Guideline 1.2 now explicitly names **nonconsensual intimate imagery, including AI-generated**, and requires apps with UGC/AI content creation to filter objectionable material, provide reporting, allow blocking, and publish contact information. Guideline 2.5.19 prohibits bait-and-switch (changing behaviour after review).
- Neither policy is a voice-cloning-specific rule, but both would apply to an app that ships a synthetic voice of a real, identifiable person.

### 4.6 Piper voice licensing

- **`faber` (mdreader's default):** MODEL_CARD says dataset `https://github.com/NabuCasa/voice-datasets`, **License: CC0**; training: "Finetuned from U.S. English lessac voice (medium quality)."
- **The lessac problem:** the Lessac corpus (Blizzard 2013) is licensed for research/exploration only and excludes "using the Materials for any commercial purpose, including the development, marketing, commercialisation, sale or licencing of voice synthesis or speech recognition products or services" (license text quoted in HF discussion #105). The piper-voices maintainer's position: the model-card license covers only the dataset directly used; whether weights are derivative works is left to the user's discretion. `faber` is therefore **CC0-labelled but lessac-derived** — a residual, unquantified risk for commercial distribution.
- **`cadu`:** CC0 dataset, also lessac-derived.
- **`edresson`:** CC-BY-4.0 (attribution required).
- **`jeff`:** model card exists; license per repository (CC0 dataset, lessac-derived) — [UNVERIFIED] at the individual card level in this session.
- **Engine licensing:** the original `rhasspy/piper` was archived (2025-10-06) under MIT; active development moved to `OHF-Voice/piper1-gpl` under **GPL-3.0** (secondary source, "confirmed directly in its repository metadata"). Combined with the espeak-ng bundling in sherpa-onnx 1.13.8, this makes the **voice stack's licensing the project's most urgent legal issue**, independent of the Lula question.

### 4.7 Recommendation for spec 016 (and custom-voices)

**Do not ship a synthetic Lula voice without all of the following:**
1. **Written, specific consent** from the rights holder (or a documented licence) naming the voice, the permitted uses, the term, and the withdrawal terms — matching PL 1530/2026's proposed standard and LGPD art. 11.
2. **A licence file and provenance record** in the voice asset directory (the `PROVENANCE.md` template already exists; it currently says "pendente").
3. **Legal review** in every distribution jurisdiction (BR, US, EU), because the app would be a "deployer" of synthetic audio of an identifiable person.
4. **A synthetic-content disclosure** in the app (EU Art. 50(4); Google Play self-declaration; TSE labelling norms).
5. **A fallback design** that does not depend on a real person's identity — e.g. a clearly fictional "Brazilian storyteller" voice trained on CC0 data, which is defensible and avoids the personality-rights, LGPD, electoral, and platform-policy exposure entirely.
6. **Governance:** spec 016 must pass the project's own assessment gate (`intake → research → problem → concept → decision`) before implementation. It currently has no assessment artifacts.

**Sources — Topic 4**
- Tennessee General Assembly, HB 2091 / SB 2096 (ELVIS Act), enrolled act — https://publications.tnsosfiles.com/acts/113/pub/pc0588.pdf (accessed 2026-11-10); amendment text — https://www.capitol.tn.gov/Bills/113/Amend/SA0574.pdf (accessed 2026-11-10).
- European Commission, *Transparency obligations under Article 50 of the AI Act* (FAQ, updated 2026-07-24) — https://digital-strategy.ec.europa.eu/en/faqs/transparency-obligations-under-article-50-ai-act (accessed 2026-11-10).
- AI Act Service Desk, *Article 50* and *Implementation timeline* — https://ai-act-service-desk.ec.europa.eu/en/ai-act/article-50 ; https://ai-act-service-desk.ec.europa.eu/en/ai-act/eu-ai-act-implementation-timeline (accessed 2026-11-10).
- Regulation (EU) 2026/1744 (Digital Omnibus on AI) commentary — https://www.euai-act.com/articles/eu-ai-act-digital-omnibus-2026 (accessed 2026-11-10).
- Brasil, Lei nº 13.709/2018 (LGPD) — https://www.planalto.gov.br/ccivil_03/_ato2015-2018/2018/lei/l13709.htm (accessed 2026-11-10).
- STJ, REJ 201403080659 (voice as personality right; Súmula 403) — https://www.stj.jus.br/websecstj/cgi/revista/REJ.exe/ITA?seq=1595136 (accessed 2026-11-10).
- Câmara dos Deputados, PL 2338/2023 (text) — https://www.camara.leg.br/proposicoesWeb/prop_mostrarintegra?codteor=3150675 (accessed 2026-11-10); Gov.br deepfake working-group report quoting art. 66 — https://www.gov.br/secom/pt-br/arquivos/2606_gt_relatorio-deepfake-v4-web.pdf (accessed 2026-11-10).
- Câmara dos Deputados, PL 1530/2026 — https://www.camara.leg.br/proposicoesWeb/prop_mostrarintegra?codteor=3141456 (accessed 2026-11-10).
- Câmara dos Deputados, PL 1460/2026 — https://www.camara.leg.br/proposicoesWeb/prop_mostrarintegra?codteor=3137282 (accessed 2026-11-10).
- TSE, Resolução nº 23.755/2026 — https://www.tse.jus.br/legislacao/compilada/res/2026/resolucao-no-23-755-de-2-de-marco-de-2026 (accessed 2026-11-10).
- TSE, *TSE fixa tese sobre deepfake e delimita regra para as Eleições 2026* (2026-09-02) — https://www.tse.jus.br/comunicacao/noticias/2026/Setembro/tse-fixa-tese-sobre-deepfake-e-delimita-regra-para-as-eleicoes-2026 (accessed 2026-11-10).
- TSE / Agência Brasil, *Eleições 2026: TSE faz parceria para impedir clonagem de voz por IA* (2026-08-07) — https://agenciabrasil.ebc.com.br/radioagencia-nacional/justica/audio/2026-08/eleicoes-2026-tse-faz-parceria-para-impedir-clonagem-de-voz-por-ia (accessed 2026-11-10).
- Google Play Console Help, *Understanding Google Play's AI-Generated Content policy* — https://support.google.com/googleplay/android-developer/answer/14094294 (accessed 2026-11-10); *Declaring AI-generated content in Play Console* — https://support.google.com/googleplay/android-developer/answer/17262077 (accessed 2026-11-10).
- Apple Developer, *App Review Guidelines* (updated 2026-10-16) — https://developer.apple.com/app-store/review/guidelines/ ; Apple news, *Strengthening App Review Guidelines against nonconsensual intimate imagery and bait-and-switch apps* — https://developer.apple.com/news/?id=vvzx6cul (accessed 2026-11-10).
- rhasspy/piper-voices MODEL_CARDs and discussions #105 / #111 — https://huggingface.co/rhasspy/piper-voices (accessed 2026-11-10).
- Cekura, *Piper TTS* (2026-08-28; secondary, engine licensing) — https://www.cekura.ai/discover/piper-tts (accessed 2026-11-10).
- Recording Law, *AI Voice Cloning Laws* (2026-08-21; secondary with primary citations) — https://www.recordinglaw.com/us-laws/deepfake-laws/ai-voice-cloning-laws/ (accessed 2026-11-10).
- Cooley, *Washington State Expands Personality Rights Law to Cover AI-Generated Deepfakes* (2026-04-06) — https://www.cooley.com/news/insight/2026/2026-04-06-washington-state-expands-personality-rights-law-to-cover-ai-generated-deepfakes (accessed 2026-11-10).
- Digital Today, *China clarifies liability for AI deepfakes and face, voice cloning* (2026-09-08) — http://www.digitaltoday.co.kr/en/view/101132/ (accessed 2026-11-10).
- AdTestBench, *Tribunal de Tóquio decide: voz é protegida por direito de imagem* (2026-10-01; secondary citing AP) — https://adtestbench.com/pt-br/news/tokyo-court-rules-a-voice-is-protected-by-publicity-rights/ (accessed 2026-11-10).
- Local: `specs/016-voz-lula/spec.md`, `crates/adapter-synthesis/assets/voices/pt_BR-lula-medium/PROVENANCE.md`, `.specify/assessments/custom-voices/intake.md` (read 2026-11-10).

---

## 5. Android accessibility for audiobook / TTS apps (2025–2026)

### 5.1 TalkBack and announcements

- **Android 16 deprecates accessibility announcements** characterised by `announceForAccessibility` or `TYPE_ANNOUNCEMENT` events. Official alternatives:
  - Significant UI/window changes → `Activity.setTitle(...)` / `setAccessibilityPaneTitle(...)`; in Compose, `Modifier.semantics { paneTitle = "..." }`.
  - Critical UI changes → `setAccessibilityLiveRegion(int)`; in Compose, `Modifier.semantics { liveRegion = LiveRegionMode.Polite | Assertive }`. **Use sparingly** — these can fire on every view update.
  - Errors → `AccessibilityEvent` of type `CONTENT_CHANGE_TYPE_ERROR` plus `AccessibilityNodeInfo.setError(...)`, or `TextView.setError(...)`.
- **Compose semantics** are the modern surface: `heading()`, `liveRegion`, `paneTitle`, `error`, `progressBarRangeInfo`, `collectionInfo`/`collectionItemInfo`, `stateDescription`, `customActions`. `mergeDescendants = true` makes a composite control a single focusable node (e.g. a list row with open/bookmark actions).
- **TalkBack 17.1 (2026)** adds in-app TTS voice selection, improved text selection with "Speak selected text", and smarter speech interruptions.
- **Implication for mdreader:** the project claims "TalkBack 10/10" (spec 005/007). If any code path uses `announceForAccessibility` (a common pattern for "playback started" / "chapter changed"), it is now deprecated and should be migrated to `liveRegion`/`paneTitle`. This is a concrete, low-effort hardening item.

### 5.2 MediaSession, lock screen, and Assistant

- Media3 `MediaSessionService` / `MediaLibraryService` is the required home for background playback. The `MediaSession` is the key to: hardware media keys, lock-screen/System UI media controls, Wear OS, TV, and Google Assistant.
- Since **Android 13 (API 33)**, media-control action buttons are derived from the **Player state**; custom actions must be declared via `SessionCommands`. `MediaStyle` notifications carry transport controls.
- **Google Assistant audiobook intents:** "Read my audiobook on (app name)" / "Read (audiobook) on (app name)"; `onPlay`, `onPause`, `onStop`, `onSkipToNext`, `onSkipToPrevious`, `onSeekTo`, `onPlayFromSearch`, `onPlayFromUri`. Unsupported actions should return `ERROR_CODE_NOT_SUPPORTED`.
- **Implication:** mdreader's Kotlin mirror already owns MediaSession/notification (constitution). The research confirms the design; the next-cycle item is verifying Assistant intents and lock-screen behaviour on a physical device (spec 003/004 device validation).

### 5.3 Word-level highlighting — what is actually achievable

- Word-level highlighting requires **engine word timings**. Evidence:
  - **Readest docs:** word-by-word read-along needs an engine that reports word timings — **Edge TTS and Web Speech do; on-device Native voices highlight by sentence; CJK highlights by sentence.**
  - **Speech Central (2025):** on Android, Google's built-in voices support `onRangeStart` (API 26+) for word ranges; Microsoft Azure generally supports synchronized highlighting; **Google Cloud TTS requires heavy SSML** (more than doubling character consumption); **OpenAI TTS does not support word highlighting** (token-based synthesis).
  - **sherpa-onnx:** no word timestamps (issue #3705 open; `GeneratedAudio` has no timing API).
- **Implication:** with Piper/sherpa-onnx, mdreader **cannot** do word-level highlighting. The realistic target is **segment/sentence-level highlighting driven by `PositionClock`** (which advances only on audible progress). This is a genuine product differentiator versus cloud readers, and it should be stated honestly in ASO/store copy rather than overclaimed.
- The project's `AutoHighlightTTS`-style competitors (Audeus, @Voice, Legere, NaturalReader) do word or sentence highlighting because they use system TTS engines (Google/Samsung) or cloud engines with timing metadata — not because they solved offline neural highlighting.

**Sources — Topic 5**
- Android Developers, *Behavior changes: all apps* (Android 16; deprecating disruptive accessibility announcements) — https://developer.android.com/about/versions/16/behavior-changes-all (accessed 2026-11-10).
- Android Developers, *Semantics in Compose* (headings, liveRegion, paneTitle, error, custom actions) — https://developer.android.com/develop/ui/compose/accessibility/semantics (accessed 2026-11-10).
- Android Developers, *Principles for improving app accessibility* — https://developer.android.com/guide/topics/ui/accessibility/principles (updated 2026-10-23; accessed 2026-11-10).
- Android Developers, *Control and advertise playback using a MediaSession* — https://developer.android.com/media/media3/session/control-playback (updated 2026-10-28; accessed 2026-11-10).
- Android Developers, *Google Assistant and media apps* (audiobook intents) — https://developer.android.com/media/implement/assistant (accessed 2026-11-10).
- Android Developers, *Media controls* (Android 13+ state-derived actions) — https://developer.android.com/media/implement/surfaces/mobile (accessed 2026-11-10).
- Android Accessibility Help, *What's new with TalkBack 17.1* — https://support.google.com/accessibility/android/answer/18145136 (accessed 2026-11-10).
- Readest Docs, *Listen (TTS) — Sentence highlighting* — https://www.readest.com/docs/listen (accessed 2026-11-10).
- Speech Central, *Text-to-Speech with Word Highlighting* (2025-03-29) — https://speechcentral.net/2025/03/29/text-to-speech-with-word-highlighting-enhance-your-reading-experience-with-speech-central/ (accessed 2026-11-10).
- Audeus, *Text to Speech Android App* (word-level highlighting claim; vendor marketing) — https://www.audeus.com/text-to-speech-android (accessed 2026-11-10).
- sherpa-onnx issue #3705 (word timestamps; open) — https://github.com/k2-fsa/sherpa-onnx/issues/3705 (accessed 2026-11-10).
- Local: `specs/005-adopt-design-sona/`, `specs/007-sona-complete-design/`, `crates/player/src/position.rs` (referenced), `ROADMAP.md` (read 2026-11-10).

---

## 6. Offline TTS audiobook reader market & competitors (2026)

### 6.1 Competitor matrix (prices in USD, checked 2026-11-10 unless noted)

| App | Price (2026) | Offline | Markdown | Word highlighting | Notes |
|---|---|---|---|---|---|
| **Speechify** | Free (10 voices, 1.5×); Premium **$29/mo or $139/yr**; Google Play **$99.99/yr**; API $10–499/mo | Pre-downloaded content only | Help-center file list: PDF, .doc, .docx, .txt, .ePub, .rtf, HTML — **no .md** | Yes (supported voices) | 1,000+ voices, 60+ languages, 5× speed; "60M+ users" is vendor marketing. A third-party review claims the web Reader accepts Markdown — **[UNVERIFIED]** against the primary help-center list. |
| **NaturalReader** | Free (unlimited device voices; 20k chars/day Lite, 4k Plus); Lite **$13.90/mo or $79/yr**; Starter **$29/mo or $198/yr**; Team $33/user/mo | Mobile offline mode; premium AI narration falls back to system voices offline | No (PDF/EPUB/Word) | Yes (word-by-word) | MP3 export, OCR, dyslexia font. |
| **Voice Dream Reader** | Subscription required; **$4.99/mo or $59.99/yr** (US App Store, per vendor-tracked data); other trackers cite **$79.99/yr** — **[UNVERIFIED]**, conflicting | All voices offline, background playback | No (PDF, TXT, Word, PPT, RTF, Google Docs, EPUB, DAISY) | Yes (synchronized word tracking in reflowable view) | iOS/macOS only. Writer app supports a Markdown subset. |
| **TTSReader** | Free (unlimited device voices; 5,000 premium chars); Premium **$10.99/mo or $99/yr**; credit packs $19–$99 | Yes (offline playing) | No (PDF, DOCX, TXT, CSV, EPUB, web) | Yes (reader highlights current text) | iOS app launched Aug 2026; Android "expected to follow". |
| **@Voice Aloud Reader** | Free + one-time Premium | Yes (system TTS) | No (PDF, EPUB, AZW3, Word, HTML, TXT) | Yes (word/sentence) | TalkBack support; records to MP3/OGG/WAV; 10M+ downloads (store listing). |
| **Legere Reader** (Voice Dream Android) | **$7.49 one-time** (Google Play) | Yes | No (PDF, DRM-free EPUB, DAISY) | Yes (synchronized word/line) | Folders, 50–700 wpm, Acapela voices IAP. |
| **Lector** (F-Droid) | Free, **GPLv3** | Yes | **Yes** (TXT, Markdown, EPUB, PDF) | Sentence-level | **Closest open-source analog to mdreader's UVP:** single file or whole folder, zero permissions, no network, system TTS. No Piper, no semantic prosody. |
| **Moon+ Reader Pro** | $6.99 one-time | Yes (system TTS) | No (EPUB/MOBI/PDF/TXT) | No | Ebook reader with Read Aloud. |
| **Ressoa (mdreader)** | Planned Freemium + one-time **R$29.90–49.90 / $9.99–14.99** | **100% air-gapped** | **Native** (headings, sections) | Segment-level (Piper has no word timings) | Zero `INTERNET` permission; Piper pt-BR local. |

### 6.2 Market reading

- **The UVP is defensible.** No major competitor combines 100% offline, air-gapped operation, and Markdown-native structure. Speechify and NaturalReader are cloud-first; Voice Dream is iOS-only; @Voice/TTSReader/Legere are document-format readers, not Markdown readers.
- **The closest threat is Lector (F-Droid).** It already ships Markdown + folder reading + zero permissions + offline, using system TTS. mdreader's differentiation must therefore be: (a) **Piper neural pt-BR voice quality** (vs. system TTS), (b) **semantic prosody / structure-aware reading**, (c) **durable bookmarks/fingerprint/resume**, (d) **SONA design**. If the Piper advantage is not audible, Lector wins on simplicity.
- **Pricing is well-positioned.** A one-time $9.99–14.99 is below Legere ($7.49 is the floor), far below Speechify/NaturalReader subscriptions, and comparable to Voice Dream's annual subscription. The "Pay Once, Own Forever" framing aligns with documented anti-subscription sentiment in the productivity/developer segment.
- **Caution on competitor claims:** the project's own `ROADMAP.md` competitor table says Speechify does not support Markdown; the primary help-center list supports that, while a third-party review claims otherwise. Store copy should avoid asserting competitor capabilities that cannot be verified.

**Sources — Topic 6**
- Speechify Help Center, *How can I import my files to Speechify on a computer?* (supported types) — https://help.speechify.com/en/articles/7958983 (updated 2026-09-30; accessed 2026-11-10); Speechify pricing — https://speechify.com/blog/how-much-does-speechify-cost/ (updated 2026-10-01; accessed 2026-11-10); Speechify API pricing — https://speechify.com/pricing-api (accessed 2026-11-10).
- ToolColumn, *Speechify Pricing Guide* (checked 2026-11-09; secondary) — https://www.toolcolumn.com/tools/speechify (accessed 2026-11-10).
- NaturalReader Help Center, *Plans & Pricing (Personal / Commercial)* — https://help.naturalreaders.com/en/articles/8854700 ; https://help.naturalreaders.com/en/articles/9363601 (accessed 2026-11-10).
- Voice Dream — http://www.voicedream.com/ ; App Store listing — https://apps.apple.com/us/app/voice-dream-reader/id496177674 (accessed 2026-11-10); Freedom251 pricing tracker — https://freedom251.com/apps/voice-dream-reader (accessed 2026-11-10).
- TTSReader pricing — https://ttsreader.com/ (accessed 2026-11-10); Capterra pricing — https://www.capterra.com/p/10267957/TTSReader/pricing (accessed 2026-11-10); TTSReader blog (iOS app, 2026-08-12) — https://ttsreader.com/blog/ttsreader-for-iphone-a-first-look-at-our-new-mobile-app/ (accessed 2026-11-10).
- @Voice Aloud Reader, Google Play listing — https://play.google.com/store/apps/details?id=com.hyperionics.avar (updated 2026-11-02; accessed 2026-11-10).
- Legere Reader — https://legere.io/reader ; Google Play listing — https://play.google.com/store/apps/details?id=voicedream.reader (accessed 2026-11-10).
- Lector, F-Droid — https://f-droid.org/en/packages/io.github.quantember.lector/ (v0.6.3, 2026-07-28; accessed 2026-11-10).
- Mira Reader, *Best Text-to-Speech Chrome Extensions* (2026-04-19; secondary, claims Speechify web Reader accepts Markdown) — https://www.mirareader.com/blog/best-text-to-speech-chrome-extensions-2026 (accessed 2026-11-10).
- Local: `ROADMAP.md` §4, `OBJETIVO.md` (read 2026-11-10).

---

## 7. Rust audio playback on Android (2026)

### 7.1 cpal status (the project pins 0.18.2)

- **cpal 0.18.0** was released **2026-06-06**. Highlights:
  - Two new first-class Linux/BSD backends: **PipeWire** and **PulseAudio** (runtime selection PipeWire > PulseAudio > ALSA).
  - **Unified error API** with new actionable kinds: `DeviceBusy` (retryable), `PermissionDenied`, `RealtimeDenied`, `ResourceExhausted`, `StreamInvalidated`, `UnsupportedConfig`, `UnsupportedOperation`, `BackendError`.
  - **Accurate timestamps / A/V sync:** AAudio pipeline buffer depth accounted for; new `StreamTrait::now()` to read the stream clock outside the callback.
  - **48 kHz is the new default** (then 44.1 kHz) on all backends.
  - **BufferSize refactor** with range support; default buffer sizes now match host configuration.
  - **Stream lifecycle normalization:** `start` / `pause` / `stop(timeout)` with draining stop.
  - **`realtime` feature** raises the audio callback thread priority (Android/Windows; Linux with D-Bus).
- **Android backend:** AAudio, **minimum Android 8.0 (API 26)**, MSRV 1.85. The project's `minSdk 29` is comfortably above the floor.

### 7.2 Known Android pitfalls (with issue numbers)

1. **Huge reported minimum buffer size.** cpal issue #890: `Device::default_output_config` reported a ~1 s min buffer on a Pixel 7 Pro because it used `AudioTrack.getMinBufferSize`. Fixed by PR #1039 (closed 2025-11-14). If the project clamps to the reported range, it can inherit a ~1 s latency.
2. **Fixed buffer size not honored.** cpal issue #902: Android does not adhere to a requested fixed buffer size; the callback's `data.len()` varies. The robust pattern is to **read `data.len()` every callback** rather than assuming a static size. A commenter found that explicitly requesting **low-latency mode** stabilised the buffer.
3. **Low-latency mode is not default.** AAudio/Oboe require `AAUDIO_PERFORMANCE_MODE_LOW_LATENCY` (or Oboe `PerformanceMode::LowLatency`) and, for the lowest latency, `AAUDIO_SHARING_MODE_EXCLUSIVE` with MMAP. cpal does not expose performance mode directly; the `realtime` feature only raises thread priority. If the project needs sub-20 ms latency, it may need to request it via the NDK/Oboe or accept cpal's default.
4. **Callback must never block.** No allocation/free, file/network I/O, mutex waits, sleep, or heavy one-time computation in the callback. Buffer size should be tuned to **2× framesPerBurst**.
5. **Warmup latency.** The first enqueued audio incurs a cold-start cost; enqueue silence before the first audible sample to avoid a glitch.
6. **AAudio MMAP** (Android 8.1+) gives the lowest latency in exclusive mode; shared mode goes through the mixer.

### 7.3 Time-stretch / pitch-preserving rate

- **cpal does not time-stretch.** It delivers PCM at the device rate. Pitch-preserving rate change must be implemented separately.
- Crate options (2026):
  - **`timestretch`** (pure Rust, MIT) — hybrid WSOLA (transients) + phase vocoder (tonal); streaming pull-based engine with dynamic ratio; `pitch_shift()` with formant correction; batch `stretch()`. Best fit for a real-time audiobook rate control.
  - **`pitch_shift`** (MIT) — phase vocoder, fixed 1024-sample latency, small.
  - **`soundtouch-rs`** (pure Rust SoundTouch port) — tempo/pitch/rate, SIMD, optional spectral stretch.
  - **`rubberband-rs`** — high quality but **GPL-2.0** and only `LiveShifter` is bound; avoid in an MIT/Apache project.
  - **`rubato`** — resampling only (not time-stretch).
- **Practical note:** `md-tts` (a Markdown TTS CLI) lists "real-time rate change (requires a streaming pitch-preserving resampler — non-trivial)" as an open problem, which corroborates the difficulty. The project's rate ladder `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]` is ambitious; a streaming time-stretch at 4.0× will be the hardest case.

### 7.4 Time-to-first-audio (TTFA) — the project's own evidence

- `specs/013-ttfa-synthesis-pipeline/spec.md` records device diagnostics (2026-11-07): 24 `play_to_audio_ms` samples, **1 659–17 309 ms common**, with extreme cases of **110 466 ms and 120 422 ms** correlated with high `position_secs`. Root cause: CPU-bound sherpa-onnx/Piper synthesis runs synchronously on the Tauri/Tokio tick path, and resume paths synthesise-and-discard segments before first useful audio.
- The spec's remedy (FR-TTFA-001) is a **dedicated blocking worker/producer** so synthesis never runs on the UI/JNI/Tokio-async tick. This is the correct architecture and is supported by the Android guidance that audio callbacks must not block and that heavy work belongs off the caller thread.
- **Note:** cpal/Oboe cannot fix synthesis latency. The fix is scheduling, not the audio backend.

**Sources — Topic 7**
- RustAudio/cpal, *Release v0.18.0* (2026-06-06) — https://github.com/RustAudio/cpal/releases/tag/v0.18.0 (accessed 2026-11-10).
- RustAudio/cpal, repository/README (backend table, `realtime` feature, Android build) — https://github.com/RustAudio/cpal (accessed 2026-11-10).
- cpal docs 0.18.2 — https://docs.rs/cpal/0.18.2/cpal/ (accessed 2026-11-10).
- cpal `UPGRADING.md` (0.17→0.18: start/pause/stop, CallbackInfo::xrun, BufferSize, realtime-dbus) — https://github.com/RustAudio/cpal/blob/master/UPGRADING.md (accessed 2026-11-10).
- cpal issue #890, *Min buffer size range detected on Android is huge (~1sec)* (closed via PR #1039) — https://github.com/RustAudio/cpal/issues/890 (accessed 2026-11-10).
- cpal issue #902, *Android won't adhere to fixed buffer size* — https://github.com/RustAudio/cpal/issues/902 (accessed 2026-11-10).
- Android NDK, *AAudio* — https://developer.android.com/ndk/guides/audio/aaudio/aaudio (updated 2026-03-06; accessed 2026-11-10).
- Android, *Low latency audio* (Oboe/AAudio recommendations, latency table) — https://developer.android.com/games/sdk/oboe/low-latency-audio (updated 2026-02-26; accessed 2026-11-10).
- Android NDK, *Audio latency* (warmup latency) — https://developer.android.com/ndk/guides/audio/audio-latency (accessed 2026-11-10).
- AOSP, *AAudio and MMAP* — https://source.android.com/docs/core/audio/aaudio (updated 2026-10-09; accessed 2026-11-10).
- `timestretch` crate — https://docs.rs/timestretch/latest/timestretch/ ; https://github.com/robmorgan/timestretch-rs (accessed 2026-11-10).
- `pitch_shift` crate — https://lib.rs/crates/pitch_shift (accessed 2026-11-10).
- `soundtouch-rs` — https://github.com/apoint123/soundtouch-rs (accessed 2026-11-10).
- `rubberband-rs` (GPL-2.0) — https://github.com/hswangTW/rubberband-rs (accessed 2026-11-10).
- `md-tts` (Markdown TTS CLI; real-time rate change note) — https://github.com/jmponcebe/md-tts (accessed 2026-11-10).
- Local: `specs/013-ttfa-synthesis-pipeline/spec.md`, `Cargo.toml` (cpal 0.18.2), `crates/player/src/backend.rs` (referenced) (read 2026-11-10).

---

## 8. Spec-Driven Development and the "dark software factory" (2026)

### 8.1 The 2026 consensus on spec-driven development (SDD)

Drawing on Microsoft's enterprise SDD guidance, the commercetools comparison of Spec Kit / OpenSpec / BMAD, the Certance Standard (v2026.5.1), FISTA's agent-SDD methodology, and the `stel1os/ai-sdd-sop` SOP:

- **The specification is the unit of work.** Requirements → design → tasks → implement, each a file the agent reads. Acceptance criteria are written **before** implementation and become the verification gates.
- **Two-gate review per task:** (1) spec compliance, (2) code quality. They can run in parallel for mechanical tasks.
- **Test-first discipline:** the human (or a separate agent) writes the failing test before the implementation agent writes code. "An agent that writes both the code and its tests will validate its own mistakes."
- **Scope and prohibited changes** are explicit controls (what must not be modified: auth paths, migrations, public APIs, tests, gate configuration).
- **Deterministic verification gates** in CI: format, tests, clippy, docs, supply chain, closed-set, shell purity. Fail-closed.
- **Human gates** at high-risk/irreversible changes (security, billing, migrations, production data).
- **Sandbox agents** with kernel-enforced boundaries; never approve permission prompts on autopilot.
- **"Generative first for exploration, deterministic thereafter for execution."** Recurring agent work gets pulled into typed tools.
- **Commercetools finding:** the framework does not necessarily make the code smarter; it changes **how reliably best practices reach the model**. OpenSpec enforced practices automatically and cost less to run; Spec Kit only asked the agent to consult them (weakest); BMAD carried them furthest but relied on a single handoff.
- **Certance:** AI diffs get *more* review minutes per line than human diffs; a December 2025 study of 470 PRs found AI-written tests derived from AI-written code are unreliable; the human-owned specification is the control that closes that gap.

### 8.2 "Dark software factory"

- **Definition (2025–26 coinage, not a standard):** an agentic software factory that ships code with little or no human involvement — a "lights-out" factory. The term borrows from manufacturing (FANUC has run lights-out plants since 2001).
- **The loop:** market/operational signals → Feature Council (decide what to build) → Creation phase (build, PRs) → Operation phase (run, feed back) → production → signals back. People "tend the harness" and "steer"; they do not work the line.
- **Readiness evidence (darkfactory.dev):** confidence is **high** for verification/quality truth; **medium** for security/supply chain and for integration/review/change control; **low** for data/database/irreversible state and for release/deployment/rollback. The site explicitly warns that the manufacturing metaphor is "useful rhetoric but a poor literal design goal" and that the evidence does not support removing humans from accountability, product judgment, or exception handling.
- **Public examples:** StrongDM (Feb 2026) — "code must not be written by humans, and code must not be reviewed by humans"; validation against scenarios and a "Digital Twin Universe". Stripe's Minions and Harvey are "lights-on" variants where humans approve merges.
- **Practical read for mdreader:** the project's 8-host-gate + device-gate-9 setup is already a "factory" with deterministic stations. The evidence supports automating the **low-risk, high-verification** lanes (format/lint/test/closed-set/docs) and keeping humans at the **irreversible** stations (voice-asset licensing, Android device validation, store release). The 80/20 framing in `ROADMAP.md` §5 ("Ação 80/20: maximum impact vs. effort") is the right selection rule for which lane to automate next.

**Sources — Topic 8**
- Joran Bergfeld, *Dark Software Factory* (blueprint) — https://joranbergfeld.github.io/dark-software-factory (accessed 2026-11-10).
- Dark Factory Dev, *State of the dark factory* — https://darkfactory.dev/factory (accessed 2026-11-10).
- Microsoft Tech Community, *Spec-Driven Development for AI-Enabled Enterprise Systems* (2026-05-18) — https://techcommunity.microsoft.com/blog/educatordeveloperblog/spec-driven-development-for-ai-enabled-enterprise-systems/4520807 (accessed 2026-11-10).
- commercetools, *Taming vibe coding: our takeaways from spec-driven development* (2026-09-07) — http://commercetools.com/blog/spec-driven-development-what-we-learned-at-commercetools (accessed 2026-11-10).
- Certance, *The Certance Standard* v2026.5.1 (October 2026) — https://cdn.prod.website-files.com/69f9f1481037f25b6feb6e40/6ac9245d39c4f3e7b325c430/The_Certance_Standard.pdf (accessed 2026-11-10).
- FISTA Solutions, *Spec-Driven Development With Coding Agents* (2026-09-04) — https://www.fistasolutions.com/blog/spec-driven-development-with-coding-agents (accessed 2026-11-10).
- stel1os/ai-sdd-sop, *Spec-Driven Development (SDD) — a Standard Operating Procedure* v1.4.0 (2026-06-11) — https://github.com/stel1os/ai-sdd-sop (accessed 2026-11-10).
- Upsun, *What is a software factory?* — https://upsun.com/software-factory (accessed 2026-11-10).
- Agentastic, *What Is a Dark Software Factory?* (2026-10-09) — https://www.agentastic.dev/blog/dark-software-factory (accessed 2026-11-10).
- Microcelium, *The Dark Software Factory* — https://www.microcelium.com/dark-software-factory (accessed 2026-11-10).
- Verdent, *Spec Driven Development for AI Coding Teams* (2026-08-20) — https://www.verdent.ai/guides/coding/spec-driven-development (accessed 2026-11-10).
- intent-driven.dev, *Best Practices for Spec-Driven Development* (updated 2026-11-02) — http://intent-driven.dev/knowledge/best-practices (accessed 2026-11-10).
- JavaCodeGeeks, *Applying the 80:20 Rule in Software Development* (2013; secondary, classic 80/20 sources) — https://www.javacodegeeks.com/2013/11/applying-the-8020-rule-in-software-development.html (accessed 2026-11-10).
- Local: `ROADMAP.md` §5, `.specify/memory/constitution.md` (referenced), `scripts/verify.sh` (referenced) (read 2026-11-10).

---

## 9. Implications for mdreader's next 80/20 cycle

The project's own `ROADMAP.md` §5 defines the 80/20 rule as "maximum impact vs. effort." Applying that rule to the findings above, and weighting by the project's constitutional constraints (zero `INTERNET`, closed sets, fail-closed, Rust authority, MIT/Apache posture), the next cycle should be sequenced as follows.

### 9.1 Release-blocking (do first; high impact, bounded effort)

1. **Resolve the espeak-ng GPL-3.0 exposure.** This is the highest-severity finding. sherpa-onnx 2.0.0 removed espeak-ng; the project pins 1.13.8, whose static lib bundles GPL-3.0 espeak-ng. Options: (a) migrate to sherpa-onnx 2.x with a permissively licensed `pt` lexicon (voicegarden-lexicons) and delete `espeak-ng-data/`; or (b) GPL-3.0 the app. Option (a) is upstream-aligned and preserves the MIT/Apache posture. **Effort: medium. Impact: avoids a distribution-blocking license conflict.**
2. **Close out spec 016 (voz Lula) at the assessment gate.** The spec is a bare draft with no assessment artifacts and a "pendente" license/provenance. A synthetic sitting-president voice in a commercial app is high-risk under LGPD (sensitive biometric data), STJ personality-rights jurisprudence, PL 1530/2026's consent standard, TSE Res. 23.755/2026, and platform policies. **Recommendation: do not implement; if pursued, require written consent + licence + legal review + synthetic-content disclosure, and prefer a clearly fictional CC0 voice.** **Effort: low (a decision). Impact: avoids legal, platform-removal, and reputational risk.**
3. **Device-gate the TTFA fix (spec 013).** The diagnostics (1.6–17.3 s common; up to 120 s extreme) are the project's most visible quality gap. The worker/producer architecture is specified and correct. **Effort: medium. Impact: directly improves the core listening experience and unblocks `gate-8-full`.**

### 9.2 High-value product work (the 20% that delivers 80% of the UVP)

4. **Finish spec 014 (symbol normalization) and harden `normalize.rs`.** Add setext headings, link/image label reduction, tables, blockquotes, strikethrough, HTML stripping; protect `a*b`; fix the global orphan-`*` removal; remove dead code; replace the PUA placeholder scheme with a safer mechanism. **Effort: medium. Impact: removes the "asterisco asterisco" fatigue that motivated the spec.**
5. **Land spec 015 (semantic prosody) with `GenerationConfig.speed`.** Keep the conservative speed map. Do **not** attempt per-segment pitch/volume — it is not exposed. Classify → plan → normalize → synthesize. **Effort: low-medium. Impact: audible structure without SSML.**
6. **Be honest about highlighting.** Piper/sherpa-onnx cannot provide word timings (issue #3705 open). Target **segment/sentence highlighting driven by `PositionClock`**, and state this accurately in store copy. Do not promise word-level read-along. **Effort: low (a copy/positioning decision). Impact: avoids a false marketing claim.**

### 9.3 Hardening and governance (lower effort, defensive value)

7. **Migrate deprecated accessibility announcements.** Replace any `announceForAccessibility`/`TYPE_ANNOUNCEMENT` usage with `setAccessibilityLiveRegion`/`setAccessibilityPaneTitle`/`setError`. **Effort: low. Impact: Android 16 correctness; protects the "TalkBack 10/10" claim.**
8. **Verify Assistant/lock-screen behaviour on device** (spec 003/004), including the audiobook intents ("Read my audiobook on Ressoa"). **Effort: low-medium. Impact: confirms the MediaSession mirror is complete.**
9. **Audit the cpal Android configuration** against issues #890/#902: read `data.len()` per callback, confirm buffer sizing, confirm `realtime` feature, and measure TTFA before/after the spec 013 worker. **Effort: low. Impact: prevents intermittent glitches and unbounded latency.**
10. **Keep the competitive positioning honest.** Lector (F-Droid) already does Markdown + folder + offline + zero permissions with system TTS. mdreader's defensible advantages are the Piper neural voice, semantic prosody, durable resume, and SONA design. Store copy should not claim competitor capabilities that cannot be verified (e.g. Speechify Markdown support). **Effort: low. Impact: protects credibility in a niche market.**

### 9.4 Suggested 80/20 cycle shape

| Phase | Work | Gate |
|---|---|---|
| **Cycle N (now)** | espeak-ng licensing decision + migration plan; spec 016 gate decision; spec 013 worker implementation; spec 014 normalizer hardening | Gates 1–8 host; device gate 9 for TTFA |
| **Cycle N+1** | spec 015 prosody; accessibility announcement migration; Assistant/lock-screen device validation; competitive copy audit | Gates 1–9 |
| **Cycle N+2** | sherpa-onnx 2.x migration (lexicon/external phonemizer); `timestretch` evaluation for the rate ladder; optional fictional CC0 voice | Gates 1–9 + legal review |

**Methodological note:** the 2026 SDD evidence supports automating the deterministic lanes (format/lint/test/closed-set/docs) and keeping humans at the irreversible stations. For mdreader, the irreversible stations are: **voice-asset licensing, Android device validation, and store release.** Everything else is a candidate for the next automated lane.

---

## 10. Verification notes and unverified claims

**Explicitly [UNVERIFIED] in this report:**
- Speechify web-Reader Markdown support (third-party claim; contradicted by the primary help-center file list).
- Voice Dream Reader's exact current annual price ($59.99 vs. $79.99 across trackers).
- `jeff` voice's individual MODEL_CARD license (repository-level CC0 dataset inferred; card not fetched).
- Piper engine's GPL-3.0 relicensing (secondary source claiming repository-metadata confirmation; the repo metadata itself was not fetched).
- The "175 voices across 56 languages" Piper catalog figure (secondary source).
- "60M+ users" / "1M+ 5-star reviews" for Speechify (vendor marketing, not independently audited).
- Any statistic not attributed to a named primary source above.

**Claims verified against primary sources fetched in this session:** CommonMark 0.31.2; SSML 1.1; Azure `mstts:markdown`; Pipecat/NousResearch normalizers; NVDA/VoiceOver punctuation controls; sherpa-onnx 2.0.0 release and PR #4055; `data_dir` deprecation; `GenerationConfig`/`GeneratedAudio` fields; issue #3705 (open); Piper MODEL_CARDs for `faber`/`cadu`; cpal 0.18.0 release notes and 0.18.2 docs; cpal issues #890/#902; Android 16 accessibility deprecation; Android MediaSession/Assistant docs; LGPD arts. 5º II, 11, 20; STJ voice/image opinion and Súmula 403; TN ELVIS Act text; EU AI Act Art. 50 and the Digital Omnibus; Google Play AI-Generated Content policy; Apple App Review Guidelines (2026-10-16); TSE Res. 23.755/2026 and the 2026-09-01 deepfake thesis; PL 2338/2023, PL 1530/2026, PL 1460/2026 texts.

**Known discrepancies to resolve before use:**
- The project's `ROADMAP.md` competitor table says Voice Dream is $19.99 one-time; current sources describe a subscription ($4.99/mo or $59.99–79.99/yr). The roadmap figure appears stale.
- The project's `ROADMAP.md` says Speechify does not support Markdown; a third-party review says the web Reader does. The primary help-center list supports the roadmap.
- The project's `PROVENANCE.md` for `faber` records the model as CC0; the model card also discloses lessac fine-tuning, whose upstream dataset license is research-only. Both facts are true simultaneously; the risk is in the derivation chain.

---

*End of report. All sources accessed 2026-11-10 unless a different date is shown. This document is research, not legal advice; voice-licensing and regulatory questions require qualified counsel in each distribution jurisdiction.*
