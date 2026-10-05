# Contract: Engine Handoff (`engine-core`)

**Consumers**: this feature's `player` ↔ engine adapters (real engines land in another
feature; two stand-ins honour this contract for SC-051). FR-033/FR-034/FR-035/FR-043,
FR-023 series.

## Delivered audio shape (normative)

- `SAMPLE_RATE`: `22050` (`u32`), declared by the playback side per FR-033. The numeric
  value is fixed by the specification, not by implementation; planning owns stream
  negotiation against it, never substitution (FR-033, FR-035, SC-034).
- Every frame: **mono `f32` PCM resampled to exactly `SAMPLE_RATE`** by the engine.
- Any deviation (rate, channel count, sample width) ⇒ named `EngineError`
  (`RateMismatch | ChannelMismatch`), never silent resample/downmix.

## Engine trait (behavioural; signatures illustrative, not code)

- `declared_capabilities() -> (SsmlLevel, AlignmentGranularity)` — each adapter declares
  what it can deliver; pipeline degrades observably to the declared coarser level
  (Principle VI). `AlignmentGranularity`: `Sentence | Word | None`.
- `set_rate(rate: PlaybackRate)` — pitch-preserving time-stretch **in the engine**;
  MUST NOT alter the delivered sample rate (FR-043, SC-034).
- `stream(position_secs: f64, budget) -> PCM frames + boundary events` — pre-roll source;
  ≤60 s outstanding; position never advances on buffered-only audio.
- Failures surface as closed `EngineError`
  (`Unreachable | InitFailed | RateMismatch | ChannelMismatch | DeviceRefused |
  DeviceLost | StartRefused`) — never silence (FR-023).

## Fallback order (FR-023c, closed)

Remaining engines of the failing tier → higher tiers in registry order → Edge evaluated
fallback last, never default. Unconsented engines are never selected. Offers name exactly
the engines the fallback may select. Descent to a lower tier is permitted only behind a
fresh 3-second offer in the shape of FR-023a naming the lower-tier engines (FR-023c).

## Conformance (gate 8 + SC-051)

Adapters pass handoff-conformance tests: shape invariant across all 7 rate rungs,
granularity honestly declared, failure taxonomy exact. Stand-in engines A (offline-like)
and B (cloud-like) demonstrate mid-document failover from the last spoken position and
forced-double-failure ⇒ paused-with-reason.
