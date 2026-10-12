# Fallback order: the declared engine descent

Grounded in `specs/001-android-media-shell/contracts/engine-handoff.md`
(§"Fallback order", the authority for this page),
`crates/engine-core/src/fallback.rs` (T075),
`crates/player/src/recovery.rs` (T076),
`crates/engine-core/src/closed.rs` (the closed tier registry), and
`.specify/memory/constitution.md` (v1.4.0, "TTS Engine Registry").
Every requirement token (FR-### / SC-###) refers to
`specs/001-android-media-shell/spec.md`.

## The registry is a closed set

The admissible engine set is closed: adding or removing an engine is a
constitution **amendment**, not a dependency-resolution decision, because
an engine arriving or leaving changes what the user hears and what
leaves the device. The constitution's TTS Engine Registry:

| Engine | Tier | Boundary events | SSML | Admissible |
|---|---|---|---|---|
| `sherpa-onnx` + Piper pt-BR (`faber`, `cadu`, `jeff`) | offline default | none | none | **Yes** — Principle IX |
| Google Cloud TTS WaveNet | cloud default | `timepoints` UNVERIFIED | full | **Yes** — Principle X |
| Google Cloud TTS Neural2 / Chirp 3 HD / Studio | cloud escalation | UNVERIFIED | full / partial | Yes, ranked by free quota |
| Amazon Polly Neural (speech marks) | cloud escalation | word-level | full | Yes, ranked by free quota |
| Azure Speech Neural | cloud escalation | word-level | full | Yes, ranked by free quota |
| Microsoft Edge read-aloud (via `msedge-tts`) | evaluated fallback | **`WordBoundary` + `SentenceBoundary`** | none | **Yes, under the XI exception** |
| Kokoro-82M pt-BR | offline alternative | none | none | Deferred — sherpa-onnx ships zh+en only |

Every tier except `test_double` refers to registry engines by name and
version. `test_double` is the sole exception (2026-10-05 clarification):
it exists only for the `standin_a`/`standin_b` stand-ins — fixtures for
the SC-051 failover drills, never selectable while any non-`test_double`
adapter is registered.

## The closed tier set and its single order

`engine_core::closed::EngineTier` declares the five tiers in ascending
fallback order — the one authoritative list (`EngineTier::ORDER`), which
the fallback selector consumes directly; a second order list would drift
(FR-044):

```
offline_default < cloud_default < cloud_escalation < test_double < evaluated_fallback
```

Wire form: the `snake_case` tier name (`engine_tier` field).
`EngineTier::is_selectable` reflects adapter registration, not a second
list: only `offline_default` is selectable at v1 (`reg.rs` declares
`REGISTERED_TIERS = [EngineTier::OfflineDefault]`); the cloud tiers are
unregistered (SC-044 deferred / consent deferred) and the evaluated
fallback is deferred — each with a distinct recorded reason
(`API_CONTRACT.md` §0.1 entry 2). Five explicit arms, never a wildcard
(Principle VII, gate 7).

## The declared fallback order (FR-023c)

1. **Remaining engines of the failing tier**, then
2. **higher tiers in registry order**, then
3. **the Edge evaluated fallback — last, never a default.**

Unconsented engines are never selected. Descent to a lower tier is
permitted only behind a fresh 3-second offer in the shape of FR-023a
naming the lower-tier engines. Offers name exactly the engines the
fallback may select. While any other engine is selectable, the
evaluated fallback is never first.

## The fallback selector (`engine-core`, T075)

`engine_core::fallback::FallbackSelector` holds the declared registry
(`EngineDescriptor { id, tier }`, registry order), the recorded
failures, the open 3-second offer, and the modeled app-private consent
store.

**`selectable()`** returns the admissible, consented, non-failed
engines, ascending `EngineTier::ORDER` with registry order preserved
within a tier (a stable sort):

- *Admissibility (both directions)*: a `test_double` is admissible
  only while no non-`test_double` adapter is registered — the
  selector never offers a stand-in while a real adapter is
  registered, and the stand-ins are reachable when only stand-ins are
  registered (the SC-051 drill configuration).
- *Consent*: read **per attempt** from app-private storage, never
  cached at construction. The read goes through `consent_from`, which
  takes the store as its parameter — the injection point the contract
  names, so a real app-private-storage read slots in without touching
  the selection logic. A `test_double` is consented by construction (it
  carries no provider/egress field at all, so there is no egress to
  gate). An id the registry does not declare reads as denied —
  fail-closed, because no consent record exists to read.
- *Failures*: every failed engine is recorded (FR-023) — never
  silently dropped, never re-offered.

Worked example (the property-suite fixture registry spanning every
declared tier): with `sherpa-onnx-piper-pt-br` (offline default),
`google-cloud-tts-wavenet` (cloud default), both stand-ins
(`test_double`), and `microsoft-edge-read-aloud-msedge-ts`
(evaluated fallback) registered, the stand-ins are inadmissible and the
selectable order is exactly:

```
sherpa-onnx-piper-pt-br → google-cloud-tts-wavenet → microsoft-edge-read-aloud-msedge-ts
```

**`on_failure(engine_id)`** records the failure and returns the fresh
3-second offer (FR-023a): the **first** remaining engine in the
declared order — the descent never skips a tier that still has a
remaining engine and never offers the failed engine again. A repeated
failure re-offers the same remaining set (the recording is idempotent,
so the set shrank exactly once per engine). The fresh offer supersedes
any open offer (FR-023's supersede rule). Returns `None` when nothing
remains selectable — the no-alternative case (FR-005).

**`expire_offer()`** consumes the open offer:

- window elapsed **with** an alternative →
  `FallbackExpiry::ContinueFromLastPosition { engine_id }` — the
  fallback auto-takes over from the last spoken position, never a
  restart; the continuation names the offered engine and carries **no
  position payload at all**, so the position stays the caller's;
- window elapsed **with no** open offer →
  `FallbackExpiry::PausedWithReason { reason }` — the stated reason
  `no alternative engine remains to continue from — the fallback offer
  expired (FR-005)`: never silence, never empty.

The 3-second window is modeled structurally: an offer is open from
`on_failure` until `expire_offer` consumes it (or the next failure
supersedes it). No wall-clock time crosses the core; the shell owns
the clock (SC-035).

## The recovery machine (`player`, T076)

`player::recovery::RecoveryMachine` is the player-side engine
authority (FR-023 series, SC-039, SC-040, SC-051). It composes the
engine-core selector for the declared order, the admissibility rules,
and the per-attempt consent read — a second order list or a second
consent model would drift (FR-044) — and owns the recovery-specific
state: the session's default engine and the open 3-second offer.

- The **session default** is the first selectable engine at
  construction — the engine selected at session start. It is never
  itself an offer: the offer names the remaining selectable engines, so
  every selectable engine except the default is offered exactly once
  over a full descent (the prop_fallback F7 oracle). `None` when
  nothing was selectable at construction.
- **`on_engine_failure(engine_id, position_secs)`** records the
  failure, latches `position_secs` as the last spoken position, and
  returns `RecoveryDecision::Offered { alternatives }` — the remaining
  selectable engines in declared ascending fallback order, the session's
  default and every failed engine excluded — or, with nothing
  remaining, `RecoveryDecision::PausedWithReason` (reason:
  `no remaining selectable engine to continue from — every registered
  engine has failed, is inadmissible, or is unconsented (FR-023 /
  FR-005)`).
- **`on_offer_expired()`** (the 3-second window elapsing) returns
  `RecoveryDecision::Takeover { engine_id, from_position_secs }` — the
  first offered alternative, the next engine in the declared order,
  continuing from exactly the reported position (no rounding, no
  tolerance) — or, with no open offer, `PausedWithReason` (reason:
  `no 3 s offer window is open to expire — … (FR-023a / FR-005)`).
- **`choose(engine_id)`** honours the reader's explicit choice inside
  the window: the chosen engine takes over from the last spoken
  position and the choice consumes the offer window. The choice is
  rejected with `PlayerError::EngineNotAvailable("engine_not_selectable")`
  when the engine is not selectable — failed, unregistered,
  inadmissible, unconsented, or locked out by the resolution — a
  named refusal, never a silent acceptance.
- The **takeover is locked** (FR-023, SC-039, SC-040): a failed
  engine is locked out permanently and no unlock exists — the closed
  `ShellVerb` set has no unlock member, so a locked resolution is
  mechanically unlock-free.
- The machine is **clock-free and deterministic** (like
  `session::offer::OfferMachine`): `on_offer_expired` represents the
  window elapsing, so every transition is deterministic and no caller
  waits on real time. It holds no transport, so the transport stays
  `Paused` through the offer window (FR-005). The failure, the
  offer, and the expiry bill nothing (SC-050 / SC-050b) — every
  decision is a state-machine decision, never a synthesis request.

Worked examples (from the machine's tests):

| Registry | Failure | Decision |
|---|---|---|
| `mixed_registry` (offline default + cloud default + Edge) | `google-cloud-tts-wavenet` at 10.0 s | `Offered { alternatives: ["microsoft-edge-read-aloud-msedge-ts"] }` — the healthy session default is excluded even though it is selectable; the failed cloud default is excluded; Edge is named, last, never a default |
| `mixed_registry` | `sherpa-onnx-piper-pt-br` (the default) at 10.0 s | `Offered { alternatives: ["google-cloud-tts-wavenet", "microsoft-edge-read-aloud-msedge-ts"] }` — every remaining selectable engine in declared ascending order |
| `mixed_registry` | repeated `google-cloud-tts-wavenet` failure at 20.0 s | the same offer as the first failure (idempotent recording) |
| `two_standin` (SC-051 configuration) | `standin_a` at 120.0 s, then expiry | `Takeover { engine_id: "standin_b", from_position_secs: 120.0 }` — exactly the reported position (the quarter-second grid, e.g. 3.25 s, is an exact binary fraction: stored and returned exactly) |
| `two_standin` | `standin_a` at 120.5 s, then `choose("standin_b")` | `Takeover { engine_id: "standin_b", from_position_secs: 120.5 }`; a later `on_offer_expired()` pauses with a stated reason — the choice consumed the window |
| `single_standin` | `standin_a` at 120.0 s | `PausedWithReason` with a non-empty reason — nothing remains to offer |

## The Edge endpoint

The evaluated fallback is consent-gated, fallback-last, and never a
default. An adapter MUST talk to the Edge endpoint directly and MUST
NOT proxy through the OpenAI-compatible Python wrapper
(`travisvn/openai-edge-tts`): that wrapper is a Python/Docker wrapper
around `edge-tts`, not a protocol implementation — its SSE stream
carries only `speech.audio.delta` and `speech.audio.done`, discarding
the boundary events, so proxying loses the only capability the engine
uniquely offers (`WordBoundary` + `SentenceBoundary`, the sole
sentence-granularity source in the registry). The maintained Rust path
is `msedge-tts` 0.4.0 (MIT OR Apache-2.0); `edge-tts-rs` 0.1.3 has
been dead since 2024-01-19 and is banned.

## Conformance (gate 8 + SC-051)

Adapters pass handoff-conformance tests: the shape invariant across all
7 rate rungs, granularity honestly declared, failure taxonomy exact.
The stand-in engines A (offline-like) and B (cloud-like) demonstrate
mid-document failover from the last spoken position and
forced-double-failure ⇒ paused-with-reason. Engine IDs live only in the
constitution registry — never a second list (FR-044).
