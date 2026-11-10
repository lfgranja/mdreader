# Contract: Normalization Boundary

- Input: `&str` (segment.content), `NormalizationMode`.
- Output: `String` (spoken text).
- Applied in `synth.rs::synthesize` before `generate_with_config`; original segment.content unchanged.
- Fluid transforms (order): protect zones → strip emphasis markers → strip heading markers → strip list bullets → convert thematic breaks to pause (".") → restore protected zones.
- Literal: identity.
- Failure: never fail; always returns a String (may be empty → synth returns InitFailed for empty).