# Checklist: 014 TTS Symbol Normalization

- [x] FR-SYM-001..009 covered by tests — `normalize.rs` unit tests (17) +
  `normalize_invariants.rs` (8) cover the pure function, idempotency,
  protected zones (including table cells/tilde fences), determinism, split
  independence and metadata invariance.
- [x] SC-SYM-001..005 evidenced (host; device listening `[NV]`) — Fluid strips
  `#`/`**`/list bullets/thematic breaks and marker asterisks in covered table
  data cells outside protected zones; protected zones byte-identical;
  idempotency property test passes; range/resume/fingerprint invariants covered
  by `normalize_invariants.rs` + `crates/session/tests/prop_fingerprint.rs`;
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

## Hardening (2026-11-15)

- `~~~` fenced blocks treated as protected/verbatim (parity with ``` fences).
- Pipe-table data cells normalized via inline pipeline with protected zones
  preserved; delimiter rows remain pause (`"."`).
- Host evidence: targeted normalization tests passed; broader adapter-synthesis
  run `170 passed / 1 skipped` excluding unrelated env-blocked proptest temp-write
  ENOSPC case (`/tmp` full). Device listening `[NV]`.
