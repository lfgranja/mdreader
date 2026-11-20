# Release (R7): 017 Audio Cache & Queue Governance

- **Date**: 2026-11-20
- **Version**: v0.4.0 (target release recorded in spec.md)
- **Validation**: R4 PASS (prior-cycle attestation, `.specify/validations/audio-cache-queue-management/validate.md`)
- **Status**: CONVERGED (implemented and attested in prior cycles)

## Summary

Adds a bounded, in-memory `AudioChunkCache` (32 MiB, ~6 min of mono f32 @ 22 050 Hz) with cache-first synthesis, immutable Blake3-256 + rate composite keys, strict LRU eviction and rate-change invalidation — cutting rewind/repeat latency and bounding memory on 3–4 GB Android devices with zero residual flash I/O.

## Changelog (IDs)

- **Added**: `AudioChunkCache` with `ChunkCacheKey { content_fingerprint: [u8; 32], rate: PlaybackRate }`; cache-first query in the synthesis pipeline; proptest + unit invariant tests.
- **Changed**: synthesis pipeline consults cache before Sherpa-ONNX inference; eviction to the 32 MiB quota on insertion.
- **Removed**: redundant neural re-synthesis on cached rewind/repeat/section-loop paths.
- **Unchanged**: canonical mono f32 @ 22 050 Hz handoff; no Android API in Rust; no flash writes (FR-006).

## Rollback

Revert PR #195 / the cache commits and remove the cache-first query from the synthesis pipeline (restore always-synthesize path). No persisted data or remote configuration is written by the cache (in-memory only), so rollback is a code revert with no data migration.

## Residual risk / [NV]

- Gate/device evidence is attested from prior cycles and not re-run in this artifact pass; live CI/merge pending.
- `converge.md` is append-only and was not modified.
