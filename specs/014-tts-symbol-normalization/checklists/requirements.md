# Checklist: 014 TTS Symbol Normalization

- [x] FR-SYM-001..009 covered by tests — `normalize.rs` unit tests (15) +
  `normalize_invariants.rs` (7) cover the pure function, idempotency,
  protected zones, determinism, split independence and metadata invariance.
- [x] SC-SYM-001..005 evidenced — Fluid strips `#`/`**`/list bullets/thematic
  breaks; protected zones byte-identical; idempotency property test passes;
  range/resume/fingerprint invariants covered by
  `normalize_invariants.rs` + `crates/session/tests/prop_fingerprint.rs`;
  SC-SYM-005 (device listening) optional/deferred.
- [x] No regression in chunk/resume/fingerprint contracts — `split_document`
  unchanged; normalization is applied only to the spoken text inside
  `synth.rs::synthesize`; invariants assert metadata unchanged.
- [x] T232/T233 untouched — directory-reading tasks not modified by this
  cycle's normalization work.

## Hardening (Cycle 017, 2026-11-10)

Closed the research gaps from `findings/2026-11-10-next-cycle-research.md` §1.5:
links/images reduced to label/alt, blockquote markers, strikethrough, raw HTML
tags, intraword `a*b`, pipe-table rows, dead code removed, collision-resistant
placeholders.
