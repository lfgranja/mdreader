# Converge: 015 MD Semantic Prosody

Status: implemented (host). `prosody.rs` + wiring in `synth.rs`
(per-segment `GenerationConfig.speed`). 9 prosody unit tests + 4 invariant
tests passing. Clippy clean. Fmt clean.

## Scope (conservative 80/20)

The only per-segment prosodic lever exposed by sherpa-onnx 1.13.8 is
`GenerationConfig.speed`; `noise_scale`/`noise_scale_w`/`length_scale` are
engine-config-level, and word-level pitch/volume is not exposed. The plan is
therefore a conservative speed map:

| Plan | Speed |
|---|---|
| `Normal` / `Question` / `Exclamation` / `ThematicBreak` | 1.00 |
| `Emphasis` / `Quote` / `List` | 0.95 |
| `Heading` / `Strong` | 0.90 |
| `Code` | 0.85 |

No per-segment pitch/volume, no SSML, no theatricality. The classification is
derived from the raw segment content BEFORE normalization (structure first),
so the pipeline order is classify → plan → normalize → synthesize.

## Invariants (T004)

`crates/adapter-synthesis/tests/prosody_invariants.rs` asserts:

- the plan is deterministic for a given segment;
- planning never mutates `source_range` / `segment_index` / `resume_point` /
  `byte_length`;
- every speed factor stays inside the conservative `(0.0, 1.0]` band;
- plain text is `Normal` and its byte length is unchanged.

## Deferred

- T006 device listening test — optional, device-blocked. The host invariants
  and unit tests are green; the audible result needs a controlled device run.
- No user-facing prosody control (future opt-in), per the conservative default.

T232/T233 untouched. R4/R7 not claimed.
