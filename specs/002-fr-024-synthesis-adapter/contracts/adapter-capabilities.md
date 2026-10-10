# Contract: Adapter Capability Declaration (`FR-024c`)

**Branch**: `002-fr-024-synthesis-adapter`  
**Feature**: FR-024c (derived from FR-024; requirement: adapter MUST declare capabilities)  
**Status**: Design-phase contract (Phase 1); implementation deferred until `T016` trait design completes (or adapter defines enums locally).  
**Constitution references**: §Engine-Neutral Core (VI, 299–312, especially `T016` mandate 299); §Audio Pipeline Contracts (568, alignment granularity paragraph); §Closed Sets (VII); §TTS Engine Registry (524 — `offline_default` boundary events = none, SSML = none)

---

## 1. Purpose

This contract defines how the synthesis adapter declares what it delivers, specifically the interface at `declared_capabilities()` (`FR-024c` / `spec.md:97` / `Assumption 3` corrected). Because the `engine-core` trait (`T016`) is mandated by `constitution.md:299-312` but **not yet built** (`crates/engine-core/src/lib.rs` exposes `closed`, `format`, `standin_a`, `standin_b` — no `trait_.rs`), the adapter must declare against the existing `DeclaredFormat` + `EngineError` contract shapes (`format.rs`, `closed.rs`) without inventing phantom methods (`format()`, `fail()` — neither exists in workspace; `grep '\btrait\b'` = 0; `grep 'declared_capabilities'` = 0).

**This is not optional.** `constitution.md:304-305` / `582-585` states declaration is unconditional. Omitting it = violation of `FR-024c` and `constitution.md`. The adapter does not wait for `T016` to declare; declaration is part of adapter contract, not trait inheritance. Adapter declares against existing `DeclaredFormat` + `EngineError` shapes (`format.rs`, `closed.rs`) only; it does not inherit from `T016` (not built — `trait_.rs` absent) and assumes no phantom `format()` / `fail()` methods.

---

## 2. Declaration Interface

### 2.1 Conceptual interface (design target)

Based on `engine-handoff.md:18-23` (present as `001/contracts/engine-handoff.md`; references the `declared_capabilities()` tuple + `set_rate()` / `stream()` obligations — adapter declares against existing `format.rs` / `closed.rs` shapes only, not against phantom `trait_`). Retire this exclusion when `T016` lands; re-check adapter surface against `T016`'s actual methods (`format()` / `fail()` must exist in `crates/engine-core/src/trait_.rs` before adapter claims them).

```rust
/// Declared capabilities of this adapter — unconditional (constitution.md:304-305).
/// Implemented against existing contract shapes (format.rs, closed.rs),
/// NOT against a phantom trait (Assumption 3 corrected; T016 not yet built).
pub fn declared_capabilities() -> AdapterCapabilities {
    AdapterCapabilities {
        ssml_level: SsmlLevel::None,
        alignment_granularity: AlignmentGranularity::None,
    }
}

pub struct AdapterCapabilities {
    /// SSML rendering capability. `None` = adapter delivers plain text only; no <break>, <prosody>, <emphasis>, embedded markup, or other SSML tag interpretation — including but not limited to any Core/Extended profile element (W3C SSML 1.1 §2.2.4–2.2.5; not a Conforming Core Processor). Any embedded markup is document-layer error (not EngineError).
    /// With `offline_default` (sherpa-onnx + Piper): none supported.
    pub ssml_level: SsmlLevel,

    /// Source-alignment precision. `None` = resume at paragraph/heading boundary only.
    /// `Sentence` = resume at sentence start (requires engine boundary events — deferred to v2).
    pub alignment_granularity: AlignmentGranularity,
}

/// Closed set — no Other / Unknown / catch-all (Principle VII).
/// These enums do not currently exist in workspace; adapter may define locally
/// or propose for engine-core ahead of T016.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsmlLevel {
    None,
    Partial,  // reserved for future cloud engine with partial SSML
    Full,     // reserved for future engine with full SSML
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentGranularity {
    None,     // paragraph/heading boundary (v1 — this adapter)
    Word,     // word boundary (reserved)
    Sentence, // sentence boundary (deferred — requires Piper/Edge events)
}
```

### 2.2 Implementation choices (with trade-off record)

| Choice | Decision | Trade-off / alternative rejected |
|---|---|---|
| Define enums locally in adapter | **Chosen** (`AdapterCapabilities` + `SsmlLevel` + `AlignmentGranularity` in adapter crate or `engine-core`) | Rejected: invent `trait_::EngineAdapter` file prematurely (would touch `engine-core` before `T016` design complete; `Assumption 3` corrected explicitly warns against phantom `format()`/`fail()`) |
| Declare `(None, None)` unconditionally | **Chosen** (`FR-024c`; `constitution.md:304-305`) | Rejected: omit declaration (violates contract); claim `Partial`/`Sentence` falsely (Piper exposes no boundary events; false declaration = silent inference risk) |
| Use string/constant representation | **Rejected** as primary — acceptable temporarily if enums not yet added to workspace | Rejected as permanent: closes set at compile time; enables `player` to branch on enum variant (not string comparison); satisfies Principle VII (no catch-all arm possible with enum) |

---

## 3. Declared Values (v1 — Option A) — Evidence

### 3.1 `SsmlLevel::None`

**Evidence** (from `constitution.md` §TTS Engine Registry, line 524):

| Engine | Tier | SSML |
|---|---|---|
| `sherpa-onnx` + Piper pt-BR | `offline_default` | **none** |
| Google Cloud TTS WaveNet | `cloud_default` | **full** |
| Microsoft Edge (`msedge-tts`) | `evaluated_fallback` | none |

The adapter uses only `offline_default`; therefore `SsmlLevel::None` is correct and required.

**Player branch behavior when `None`**:
- No SSML rendering; plain-text-to-speech directly.
- No `<break>` voltage / pitch / prosody adjustments between segments (would require SSML capability; adapter does not provide).
- `player` must not silently infer SSML presence from adapter output (Principle VI: degradation observable, never silent).

### 3.2 `AlignmentGranularity::None`

**Evidence** (from `spec.md` clarifications; `CLARIFICATION_BATCH.md` C1-A; `constitution.md` Audio Pipeline Contracts, 568):

| Evidence | Finding |
|---|---|
| `Piper` boundary events | **None exposed** (`spec.md:18`; clarification 2026-10-01 / 2026-10-05) |
| `Edge` (`msedge-tts`) | `SentenceBoundary` + `WordBoundary` (verified in source `src/edge_tts/submaker.py`; `constitution.md` 542–547) — but deferred (no consent / not registered) |
| Google Cloud TTS `timepoints` | **UNVERIFIED** (`constitution.md` 542; `spec.md` 125; no REST confirmation) — cannot rely on sentence-level |
| `pulldown-cmark` | Produces headings + paragraphs (sufficient for `None`; sentence-level deferred to v2 per `spec.md:23`; `Assumption 4`) |

**Player branch behavior when `None`**:
- Resume aligns to nearest completed paragraph/heading boundary (`FR-024`; `spec.md:59`; `FR-015`).
- Sentence-level resume deferred to v2 (`AlignmentGranularity::Sentence` is unused in v1).
- **Rename decision (2026-10-09, accepted; supersedes the open request at `CLARIFICATION_BATCH_DECISION_2026-10-08.md:44`)**: the `player` function formerly named `snap_to_sentence_start` (`crates/player/src/position.rs`) is renamed to the boundary-kind-agnostic `snap_to_boundary(boundaries: &[f64])`. Rationale: `BoundaryKind` has no `Sentence` member in v1 (`API_CONTRACT.md` §2.1), so a function named after one deferred member hard-codes a closed-set member as if it were the only one; no sentence semantics remain in its signature or body. **No constitution amendment and no Exceptions Log entry**: the rename implements the C1-A deferral already approved in `CLARIFICATION_BATCH_RESOLVED.md:13`, so no principle changes and no deviation is retained.
- Source-range mapping at node granularity (paragraph / heading), not word / sentence.
- Degradation from `Sentence` (desired) to `None` (actual) is **observable** — `player` knows from declaration; user sees resume at coarser boundary; never silent (Principle VI).

---

## 4. Contract with Player / Core (integration points)

### 4.1 Adapter → Player (declaration consumption)

`player` reads `AdapterCapabilities` (via adapter registration or direct call to `declared_capabilities()`). Based on values:

| Declared value | Player action |
|---|---|
| `SsmlLevel::None` | No SSML rendering path; direct text-to-audio; no `<break>` / `<prosody>` / `<emphasis>` / embedded markup — adapter passes Markdown-derived raw text only; any embedded markup is document-layer error (not `EngineError`). Bad Markdown / unparseable segment: document-layer only (`pulldown-cmark` unreachable in v1; `grep` workspace `Cargo.toml` → 0); reserve `document_unparseable` on existing `failure`/`refusal` for `FR-044` coordinated edit — never new `diag` `kind`. |
| `AlignmentGranularity::None` | Resume snap to `Paragraph` / `Heading`; `snap_to_boundary` used (the v1 path); bookmark keyed by `blake3` document fingerprint, value = playback offset (`redb`); `segment_index` is not a bookmark key or field |

**Constraint**: `player` must branch explicitly on declared value; must not silently fall back to `Sentence` or `Full` if adapter declares `None`. Silent inference = violation of Principle VI (.degradation must be observable) and `FR-024c`.

### 4.2 Adapter → Core (registration + contract shapes)

- Adapter registers `OfflineDefault` (not `CloudDefault` / `CloudEscalation` / `EvaluatedFallback` / `TestDouble`).
- Adapter delivers `AudioChunk` that matches `DeclaredFormat` (`format.rs` 23–33). If mismatch: adapter reports `EngineError`; does not deliver mismatched chunk.
- Adapter declares `(None, None)` unconditionally. If declaration missing: `player` must treat as error (fail-closed; `spec.md:97`; `constitution.md:304`).

---

## 5. Verification (before adapter file written)

- [ ] Confirm `SsmlLevel` and `AlignmentGranularity` enums can be added to adapter crate (or proposed for `engine-core`) without breaking existing `standin_a` / `standin_b` (they use `DeclaredFormat` only; no capability fields — safe to extend).
- [ ] Confirm `declared_capabilities()` can be called at registration time (before first chunk delivered) — unconditional declaration requires no synthesis;
- [ ] Confirm `player` has branch logic for `None` (even if currently only `standin_a`/`standin_b` exist — mechanism preserved; `SC-051` passes with adapter registered).
- [ ] Confirm no phantom `trait_` file created; `T016` design deferred to `003-contract-edit` / separate feature.
- [ ] Confirm declaration matches `constitution.md` §TTS Engine Registry: `offline_default` has `none` SSML, `none` boundary events.
- [ ] Confirm `CLARIFICATION_BATCH.md` C2-A preserved: declaration is against existing `DeclaredFormat` + `EngineError` shapes; `FR-024c` normative; `Assumption 3` corrected (trait mandated but not built — no phantom methods).

---

## 6. References

- Feature spec: `/home/luis/development/mdreader/specs/002-fr-024-synthesis-adapter/spec.md` (lines 94, 97, 125, 137; Assumption 3 line 129; clarifications 13–25)
- Design plan: `/home/luis/development/mdreader/specs/002-fr-024-synthesis-adapter/plan.md`
- Research (C2-A resolution, trait evidence): `/home/luis/development/mdreader/specs/002-fr-024-synthesis-adapter/research.md` (§3.3, §2)
- Constitution: `/home/luis/development/mdreader/.specify/memory/constitution.md` (§VI 299–312; §Audio Pipeline Contracts 568; §TTS Engine Registry 524; §Development Workflow 643 Exception 8; §Single Playback Authority 82–86)
- Source verification (no trait, no declared_capabilities): `/home/luis/development/mdreader/crates/engine-core/src/lib.rs` (no `trait_.rs`); `grep '\btrait\b'` = 0; `grep 'declared_capabilities'` = 0; `grep 'SsmlLevel\|AlignmentGranularity'` = 0
- Stand-in fixtures (reference for engine-handoff contract): `/home/luis/development/mdreader/crates/engine-core/src/standin_a.rs`, `standin_b.rs`
