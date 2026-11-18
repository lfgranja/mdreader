# Data Model: 015 MD Semantic Prosody

## New types

- `ProsodyPlan`: enum { `Normal`, `Heading`, `Emphasis`, `Strong`, `Quote`,
  `Code`, `List`, `Question`, `Exclamation`, `ThematicBreak` }.
  Derive `Debug, Clone, Copy, PartialEq, Eq`.
- `ProsodyPlan::speed() -> f32`: conservative speed factor in `(0.0, 1.0]`.
- `plan_for_segment(&DocumentSegment) -> ProsodyPlan`: pure, deterministic.

## Unchanged

- `DocumentSegment`: content, source_range, segment_index, resume_point,
  byte_length — untouched.
- `SourceRange`, fingerprint (blake3) — untouched.
- `GenerationConfig`: only `speed` is set per segment.

## Invariants

- `plan_for_segment` is a pure function of the segment content.
- The plan never mutates the segment or its metadata.
- Every speed factor stays inside `(0.0, 1.0]`.
