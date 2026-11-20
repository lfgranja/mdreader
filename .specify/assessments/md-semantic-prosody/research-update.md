# Research Update: md-semantic-prosody

## Research 1: Prosody control in Piper/eSpeak/sherpa (without SSML)

- Piper (VITS) synthesizes whole utterance at once; no per-word pitch/rate tags in text. Prosody is learned from training data; input text punctuation influences pauses.
- eSpeak-NG supports inline prosody markup: `[[...]]` espeak commands and SSML `<prosody rate/pitch/volume>`, `<break>`, `<emphasis>`. Source: https://github.com/espeak-ng/espeak-ng/blob/master/docs/markup.md
- sherpa-onnx `OfflineTts.generate_with_config(text, GenerationConfig, callback)`: `GenerationConfig` exposes `speed` (length scale), `sid` (speaker id), `reference_audio`/`reference_sample_rate` (voice cloning). No per-utterance pitch/volume in Piper path. Source: https://docs.rs/sherpa-onnx
- Therefore, per-segment prosody in mdreader can be achieved by: (a) text-level cues (punctuation, capitalization, pauses via segment boundaries/empty segments), (b) `GenerationConfig.speed` per segment for rate variation, (c) splitting into separate segments with pauses. Pitch/volume changes require post-processing PCM (gain) or separate synthesis — possible but adds complexity.
- No SSML in mdreader (`SsmlLevel::None`); eSpeak SSML tags would require passing SSML through to eSpeak, but Piper path may not honor them (needs empirical test).

## Research 2: Human reading prosody

See `research.md` sections (prosody by semantic element table, accessibility). Key: headings slower/louder with pauses; emphasis pitch accent; blockquotes lower pitch; lists item pauses; code monotone; questions rising terminal; exclamations louder; thematic break long pause.

## Research 3: Codebase map for prosody

- `chunk.rs`: `split_document` builds `DocumentSegment { content, source_range, segment_index, resume_point, byte_length }`. Segment content is trimmed text joined with `\n\n`. Insertion point: normalize/project spoken text per segment before synthesis (consistent with tts-symbol-fatigue, order: classify structure → derive prosodic plan → generate spoken text).
- `synth.rs::synthesize`: reads `segment.content`, passes to `generate_with_config`. Can apply per-segment `speed` via `GenerationConfig` if a prosodic plan provides it.
- `player`: rate control is global (`set_rate`, pitch-preserving); `PositionClock` advances on audible progress. Per-segment rate changes would require transport-level rate switching at segment boundaries — feasible but complex; recommend pauses first, rate changes later.
- `SourceRange`/`resume_point`/`segment_index`/fingerprint: must remain tied to original content, not spoken projection. Prosodic plan is metadata, not content.

## Research 4: Existing prosody approaches

- Speech Markdown: extends markdown with speech annotations (emphasis, breaks, IPA, pitch/rate/volume). Useful as target vocabulary for a prosodic plan DSL. (Inference/general knowledge.)
- md-tts (PyPI): TTS for technical Markdown with pauses on code blocks.
- SSML prosody: `<prosody rate/pitch/volume>`, `<emphasis>`, `<break>`. mdreader cannot use (SsmlLevel::None) but can emulate via text cues + speed params.
- Implication: implement a "prosodic plan" (metadata per segment: pause_before_ms, speed_factor, emphasis flag) computed from markdown semantics, applied at synthesis/playback boundary. Conservative defaults (conservadora) to avoid theatricality.