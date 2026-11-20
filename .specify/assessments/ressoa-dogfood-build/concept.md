# Concept: Ressoa Dogfood Build

- **Slug**: ressoa-dogfood-build
- **Created**: 2026-11-20 (backfilled)
- **Recommended option**: Option A

## Options

### Option A — Maintainer-only single-voice offline sideload (recommended)
- **Sketch**: Consume 001 shell + 002 adapter + 003 reference device; one bundled pt-BR voice; SAF pick → BLAKE3 snapshot → synthesis; lock-screen + single in-app screen with closed-surface discipline; closed-set interruption states with earcon/haptic confirmation; content-hash resume with last-spoken atomic cursor; private diagnostics retrieved by `adb pull`. No publication.
- **Appetite**: large but bounded (dogfood slice only).
- **Trade-offs**: Produces the missing measurements and is reversible (sideload only). Cost: careful pt-BR string discipline and interruption mapping; risk contained by closed sets and structural tests.

### Option B — Full multi-voice store release now
- **Sketch**: Ship catalog + consent + cloud fallback + store publication in this feature.
- **Appetite**: very large.
- **Trade-offs**: Exceeds the accepted `go` scope, needs consent/UI/storage surfaces and gate-9 publication authorization; rejected.

### Option C — Stay host-only (no installable build)
- **Sketch**: Keep adding host tests without a device artifact.
- **Appetite**: none.
- **Trade-offs**: Cannot produce the required on-device measurements; rejected.

## Recommendation

Option A. It matches the accepted assessment `go` for Option A only and is the only path that yields real device evidence without publication risk.

## Out of Scope (recommended option)

- Publication/store, multi-voice catalog, network voices, Android Auto, consent surface (SC-044 in 002 stays DEFERRED).

## Assumptions to Validate

- Reference phone bound ≤10 s cold-start (003) is the accepted measurement basis (verified: 002 amendment).
- Single bundled voice suffices for a 30-min fixture session (SC-001) — device trial [NV].
- `ACTION_OPEN_DOCUMENT` persistable grant is best-effort with re-prompt (FR-018) — device trial [NV].
