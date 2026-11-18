# Converge: 015 MD Semantic Prosody

Status: implemented (host), refined 2026-11-15. `prosody.rs` + wiring in
`synth.rs` (per-segment `GenerationConfig.speed`). 13 prosody unit tests + 4
invariant tests passing (host). Clippy clean. Fmt clean.

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

## Refinement (2026-11-15) — detectable structure/emphasis coverage

Host-side 80/20 refinement within 015 speed-only constraints (no new
`ProsodyPlan` variants, no `speed()` changes, no segment metadata/fingerprint
changes, deterministic classification on raw segment content):

- `crates/adapter-synthesis/src/prosody.rs`: ordered list items
  (`^\d+\.\s+`) classified as `List`; underscore strong (`__...__`) classified
  as `Strong` alongside `**...**`; conservative detectable highlight/emphasis
  hints (`==`, `<u`, `<mark`) mapped to existing `Emphasis` plan.
- Tests: `crates/adapter-synthesis/tests/prosody_unit.rs` adds
  `ordered_list_detected`, `strong_detected`, `underscore_strong_detected`,
  `highlight_detected_as_emphasis` (13 unit tests total).
- Verification (2026-11-15, host): targeted prosody tests passed (7 selected,
  including existing/affected cases); broader adapter-synthesis run
  `cargo nextest run -p adapter-synthesis -E 'not test(/arbitrary_documents_read_identically_from_stdin_and_input/)'`
  passed (170 tests run, 1 skipped, 4 slow). Full run had unrelated environment
  ENOSPC in `adapter-synthesis::cli_protocol arbitrary_documents_read_identically_from_stdin_and_input`
  (`/tmp` ~99% full), not attributed to this change.
  `cargo clippy -p adapter-synthesis -p player -p session --all-targets -- -D warnings`
  passed. `scripts/expected-tests.txt` updated for added prosody targets.
- Limitations: speed-only, segment-level prosody; no SSML/pitch contour/word-level
  emphasis. Subtitle/underline/grifado/chapter-name intonation remains
  heuristic/marker-dependent or engine-limited; device listening (T006/SC-PROS-004)
  `[NV]`.
