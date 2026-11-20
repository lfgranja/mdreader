# Idea Research: TTS Symbol Normalization

- **Slug**: tts-symbol-normalization
- **Created**: 2026-11-20
- **Evidence confidence (overall)**: high

## Users & Demand

- Readers of Markdown audiobooks report fatigue from literal spoken markers ("asterisco"), a recurring listening complaint — [source: `specs/014-tts-symbol-normalization/spec.md`, user report 2026-11-07] (confidence: high)

## Prior Art

- `tts-symbol-fatigue` assessment (`.specify/assessments/tts-symbol-fatigue/`) already decided `go` on 2026-11-08 with deep research; spec 014 was generated from it — [source: local file] (confidence: high)
- `adapter-synthesis::synth::synthesize` is the single synthesis boundary where spoken text can be transformed before `generate_with_config` — [source: `crates/adapter-synthesis/src/synth.rs`] (confidence: high)
- `crates/adapter-synthesis/src/chunk.rs` projects segment metadata only; `session` computes blake3 fingerprints from raw bytes before synthesis — [source: local files] (confidence: high)

## Market & Context

- Cost of doing nothing: persistent listening fatigue in the app's core function (TTS reading), with no SSML escape hatch (`SsmlLevel::None`).

## Data & Constraints

- SSML unavailable (`SsmlLevel::None`); engine/model swap out of scope; normalization must be deterministic plain text in Rust. [source: spec 014 Non-Goals]
- Protected zones (fenced/inline code, URLs/paths, simple math) must remain byte-identical; `~~~` fences parity with ``` added in the 2026-11-15 hardening. [source: `converge.md`]
- `.md` scope only; `.txt` minimal normalization (FR-SYM-006). [source: spec 014]
- Host evidence (converge 2026-11-15): `normalize.rs` 17 unit tests + `normalize_invariants.rs` 8 tests pass; broader adapter-synthesis run 170 passed / 1 skipped (unrelated ENOSPC proptest). Device listening `[NV]`.

## Evidence Against the Idea

- Inline normalization risks mangling protected zones (math `2*3`, code, URLs). Mitigation: collision-resistant placeholders + invariant tests asserting byte-identical protected substrings and unchanged metadata. — [source: converge, measured] (confidence: high)

## Gaps & Open Questions

- Literal-mode UI toggle remains a follow-up; fixed Fluid default shipped (FR-SYM-005).

## Sources

- `specs/014-tts-symbol-normalization/{spec.md,converge.md,tasks.md}` (host: local repo, policy: allowlisted path)
- `.specify/assessments/tts-symbol-fatigue/` (host: local repo)
- `crates/adapter-synthesis/src/{synth.rs,chunk.rs,normalize.rs}`, `crates/adapter-synthesis/tests/normalize_invariants.rs` (host: local repo)
