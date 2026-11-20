# Concept: 016-voz-lula

- **Slug**: voz-lula
- **Created**: 2026-11-20
- **Recommended option**: Option A

## Options

### Option A — Second selectable Piper VITS voice with fail-closed init
- **Sketch**: extend the synthesis adapter with a `Voice` enum (`Faber` | `Lula`), add `--voice`/`set_voice` selection, reuse the `pt_BR-faber-medium` asset layout and espeak-ng tree, keep mono f32 @ 22 050 Hz, and return `EngineError::InitFailed` when `pt_BR-lula-medium/*.onnx` is missing.
- **Appetite**: medium (code) / blocked (model acquisition)
- **Trade-offs**: Clean, offline-first, reversible selection seam; deliverable code-level support even before the model exists. Requires consent/licensing and a constitution/gate amendment before the asset can be bundled.
- **Rabbit holes**: placeholder asset tree breaking `exactly_one_bundled_voice_directory`; silent resample; non-fail-closed init.

### Option B — Bundled synthetic voice now, legal review later
- **Sketch**: ship a trained Lula voice without documented consent.
- **Appetite**: n/a
- **Trade-offs**: Unacceptable LGPD/STJ/TSE/store-policy exposure. Rejected.

### Option C — Clearly fictional CC0 "Brazilian" voice instead
- **Sketch**: offer a fictional pt-BR voice with clean licensing.
- **Appetite**: medium
- **Trade-offs**: Avoids personality-right risk and can satisfy the differentiator partially; loses the specific homage. Viable alternative if consent cannot be obtained.

## Recommendation

Option A at the design level (selectable, fail-closed, canonical handoff), gated on the model/consent/constitution prerequisites recorded in research.md and `specs/016-voz-lula/decision.md`. This assessment's `go` authorizes specification; implementation remains a separate deferred decision.

## Out of Scope (recommended option)

- Model training/donation, consent acquisition, constitution amendment, store disclosure — implementation prerequisites, not part of this artifact set.

## Assumptions to Validate

- `faber` remains the offline default (spec 016 assumptions).
- Lula is a second, explicit option; `.onnx` not included in repo.
- Offline-first: nothing fetched at synthesis time (Principle IX).
