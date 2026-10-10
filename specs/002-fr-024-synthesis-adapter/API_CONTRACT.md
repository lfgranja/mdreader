# API Contract — `crates/adapter-synthesis` (frozen for T018–T021)

**Branch**: `002-fr-024-synthesis-adapter` | **Tasks**: T018, T019, T020 (tests) → T021 (implementation) → T022–T024 (verification)
**Status**: frozen interface. Tests are written against this contract *before* the implementation exists (rule 1: tests BLOCK core). The implementation agent MUST match this contract exactly — no additions, no renames, no removals.

Normative sources: `spec.md` (FR-024/024a/024b/024c), `data-model.md` (E1–E5, R1–R11), `contracts/adapter-handoff.md`, `contracts/adapter-capabilities.md`, `.specify/memory/constitution.md` (VI, VII, X), `AGENTS.md`.

---

## 0. Global rules that apply to every file in this crate

1. **No `_ =>` arm anywhere in `crates/adapter-synthesis/src/`** — constitution gate 7 (`constitution.md:614`) and Principle VII (`constitution.md:314-317`). Every `match` over a closed-set enum enumerates every arm explicitly. Adding a wildcard is a verification failure, not a style choice.
2. **`missing_docs`, `missing_debug_implementations`, `unreachable_pub`, `unused_results`, `unsafe_code` are `deny`**; `clippy::all` + `clippy::pedantic` are `deny`; `clippy::nursery` + `clippy::cargo` are `warn`. Every public item carries a `///` doc comment and a `#[derive(Debug, ...)]`.
3. **`unwrap` / `expect` / `panic!` / `todo!` / `unimplemented!` / `dbg!` / `println!` / `eprintln!` are `deny` — in tests too.** `#[allow]` is `deny` workspace-wide and is never permitted here. `#[expect(lint, reason = "…")]` is the **only** permitted suppression, per the workspace rule in `AGENTS.md`; it must carry a reason and must be justified in a comment. It is permitted **only where this frozen contract's own normative text mandates a shape that the `deny`-level `clippy::pedantic` set rejects** — there are exactly six such places, all enumerated in §0.1. Everywhere else, write `match`, `let Ok(x) = … else { … }`, or `prop_assert!`. **Amendment 2026-10-08:** this rule previously read "no `#[expect]`/`#[allow]` suppression … and must not gain one", which made this contract self-inconsistent — §3 and §4 mandate one explicit match arm per closed-set member (so `clippy::match_same_arms` fires) and §6 mandates `pcm_sample_count as f64` (so `clippy::cast_precision_loss` fires), while the workspace denies both lints and forbade the only sanctioned remedy. The workspace `AGENTS.md` rule (`#[expect(lint, reason = "…")]` as the single permitted suppression) governs.
4. **No new `EngineError` variant.** The set is re-exported from `engine_core::closed`; a second list would drift (`FR-044`). `crates/engine-core` is not modified.
5. **No phantom trait, no `trait_.rs`, no `format()` / `fail()` method, no marker trait** (`spec.md` Assumption 3; `contracts/adapter-capabilities.md` §5).
6. **No second authority over playback.** The adapter splits and declares; `player` remains the sole authority for position/buffering/transport (`constitution.md:486-490`). The adapter never converts audio to the device format.
7. **Bad Markdown is a document-layer concern, not an `EngineError`** (R8; `spec.md:85`).

### 0.1 The six sanctioned suppressions (§0 rule 3)

Exactly six `#[expect]` attributes exist in this crate, each mandated by this contract's own normative text and each carrying a `reason`:

| # | Location | Lint | Why the contract mandates the shape the lint rejects |
|---|---|---|---|
| 1 | `failure.rs` — `FailureCondition::variant()` | `clippy::match_same_arms` | §3 requires an exhaustive 10-arm match so every condition names its variant explicitly. Five of the ten share a body (`RateDeviation`/`WidthDeviation`/`PitchToleranceViolation` → `RateMismatch`; `ProcessDeath`/`TransportUnreachable` → `Unreachable`). Grouping them would satisfy the lint but weaken the explicit per-condition mapping the failure taxonomy depends on (`spec.md:86`). |
| 2 | `reg.rs` — `is_selectable()` | `clippy::match_same_arms` | §4 requires five distinct arms. Three of them (`CloudDefault`, `CloudEscalation`, `EvaluatedFallback`) legitimately return the same `false`, because all three are unregistered at v1 with distinct reasons (`SC-044` deferred, consent deferred, evaluated-fallback deferred). The reasons belong in doc comments, not in duplicated arm bodies. |
| 3 | `format_check.rs` — `check_delivery()` | `clippy::cast_precision_loss` | §6 mandates comparing `pcm_sample_count as f64` against `(duration_ms / 1000.0) * DECLARED_RATE_HZ`. `f64` has no lossless `From<usize>`, so the cast is unavoidable; narrowing to `u32` first would be lossy for a >4-billion-sample buffer and would disagree with the frozen test's identical computation at the half-sample boundary. |
| 4 | `audio_chunk.rs:239`, `synth.rs:246`, `tests/unit_failure.rs` — the data-model E2 duration derivation | `clippy::cast_precision_loss` | §6 (and data-model E2, which §6 freezes) mandates deriving `duration_ms` as `pcm_data.len() / rate_hz * 1000` — a `usize -> f64` cast with no lossless `f64::From<usize>`. The identical computation appears at three sites: `audio_chunk.rs:239` (registered 2026-10-09; the suppression landed with T043 unregistered and is registered here retroactively), `synth.rs:246` (T044 — the engine's pre-gate derivation), and `tests/unit_failure.rs` (T044 — the delivery-gate round-trip the updated §7 test performs). All three are measured against the same half-sample band entry 3 sanctions, all freeze the identical expression the delivery-unit tests pin (`tests/audio_chunk.rs:231-232, :516-518`), and a >2^53-sample buffer (over 2.8 years of audio at 22 050 Hz) is unreachable in memory. |
| 7 | `crates/engine-core/src/closed.rs` — `EngineTier::is_selectable()` (T060 wire) | `clippy::match_same_arms` | §4 / SC-051 requires five distinct arms (`OfflineDefault` true; `CloudDefault`, `CloudEscalation`, `EvaluatedFallback` false — unregistered at v1 with distinct deferral reasons `SC-044`/consent/evaluated; `TestDouble` false — fixtures never selectable while adapter registered per `001/spec.md:89`). The reasons live in doc comments, not duplicated bodies. Cross-crate mirror of adapter `reg.rs`: `is_selectable` (entry 2). Registered in `API_CONTRACT.md` §0.1 (T060). |

No eighth suppression is sanctioned. If a future change needs one, that is a coordinated edit to this section — not a local decision.

### 0.2 Sanctioned additions to the frozen surface (registered 2026-10-09)

§0.1 registers the sanctioned `#[expect]` suppressions; this section
registers sanctioned additions to the frozen surface — behaviours the
crate implements that the frozen prose does not state. Two were added
without registration. The behaviour is **kept** — nothing is removed or
changed to match the prose — and only the registration was missing.
Both are source-compatible, fail-closed strengthenings consistent with
Principle VII (`constitution.md:314-317`); with this registration they
are part of the frozen surface, and the header's "no additions, no
renames, no removals" binds future edits, not this record.

| # | Addition | Code site(s) | Contract text | Rationale |
|---|---|---|---|---|
| 1 | `const fn` signatures where the contract writes plain `pub fn` | `src/lib.rs:77` (`declared_capabilities`), `src/capabilities.rs:7` (`capabilities`), `src/format_check.rs:70` (`check_format`), `src/chunk.rs:150` (`byte_length`) | §1, §2, §6 and §8 each write `pub fn` | A `const fn` can always be called as a `fn`, so the strengthening is source-compatible: every caller and frozen test written against the `pub fn` text compiles and behaves identically. The four functions are pure — no state, no I/O — so nothing observable changes. |
| 2 | Non-finite `duration_ms` → `EngineError::RateMismatch` | `src/format_check.rs:119-121` | §6's `check_delivery` prose does not mention a non-finite duration | A NaN or ±inf `duration_ms` is not a derivable duration (`duration_ms` is computed from a buffer length), so the check fails closed: it names `RateMismatch` rather than silently passing or entering an incomparable half-sample band — consistent with Principle VII (a deviation is observable, never silent) and with §6's rule that any buffer whose length disagrees with its duration is a `RateMismatch`. |

---

## 1. `src/lib.rs` — module wiring + declaration (already present; keep)

```rust
use engine_core::closed::EngineTier;

pub mod capabilities;
pub mod chunk;
pub mod consume;
pub mod failure;
pub mod format_check;
pub mod reg;
pub mod synth;

pub const REGISTERED_TIER: EngineTier = EngineTier::OfflineDefault;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsmlLevel { None, Partial, Full }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentGranularity { None, Word, Sentence }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterCapabilities {
    pub ssml_level: SsmlLevel,
    pub alignment_granularity: AlignmentGranularity,
}

#[must_use]
pub fn declared_capabilities() -> AdapterCapabilities;
```

`declared_capabilities()` is unconditional and constant: `(SsmlLevel::None, AlignmentGranularity::None)`. It takes no argument, reads no state, and returns the same value on every call (`constitution.md:304-305`; `FR-024c`).

Convenience re-exports are **required** at the crate root so integration tests can reach the whole surface through one path:

```rust
pub use crate::chunk::{BoundaryKind, DocumentSegment, SourceRange, MAX_CHUNK_BYTES};
pub use crate::consume::{PlayerPlan, ResumeAnchor, SsmlDelivery};
pub use crate::failure::{EngineError, EngineFailure, FailureCondition, FailureReportError, MAX_CONDITION_DETAIL_CHARS, MAX_OFFER_MS};
```

---

## 2. `src/chunk.rs` — the byte-budget split (enforcement point for R3 / CHK016 / CHK008 in `segment-resume.md`)

```rust
use std::fmt;

use crate::failure::{EngineError, FailureCondition};

/// Hard upper bound, in UTF-8 **bytes**, for one delivered segment.
pub const MAX_CHUNK_BYTES: usize = 5_000;

/// A 1-based, inclusive line range plus 0-based byte columns within the
/// first/last line of that range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceRange {
    pub line_start: u32, // 1-based, inclusive
    pub line_end: u32,   // 1-based, inclusive; >= line_start
    pub col_start: u32,  // 0-based byte column inside `line_start`
    pub col_end: u32,    // 0-based byte column inside `line_end`
}

/// The closed boundary-kind set. There is **no** `Sentence` member
/// (`AlignmentGranularity::None`; R6). Adding one is a coordinated edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryKind { Paragraph, Heading }

/// One bounded, spoken chunk. Immutable after delivery (`spec.md:103`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSegment {
    pub content: String,
    pub source_range: SourceRange,
    pub boundary_kind: BoundaryKind,
    pub segment_index: u64,
    pub resume_point: Option<SourceRange>,
}

impl DocumentSegment {
    /// UTF-8 **byte** length of `content` — never a character count
    /// (`constitution.md:353`; R3). Derived from the content, not stored,
    /// so the byte invariant cannot drift from the text it measures.
    #[must_use] pub fn byte_length(&self) -> usize { self.content.len() }
}

/// Split `source` at paragraph/heading boundaries under `MAX_CHUNK_BYTES`.
#[must_use] pub fn split_document(source: &str, document_hash: &str) -> Vec<DocumentSegment>;

/// Same, with an explicit budget (property tests exercise small budgets).
#[must_use] pub fn split_document_within(source: &str, document_hash: &str, max_bytes: usize) -> Vec<DocumentSegment>;
```

### 2.1 Segmentation algorithm — normative

1. **Blocks.** Scan the source line by line into narration blocks.
   - A line whose trimmed form starts with 1–6 `#` followed by a space or end-of-line (at most 3 leading spaces) opens a **heading block**. The heading block extends over the heading line **and** the immediately following non-blank lines, so a heading and its body are narrated together and the split never lands between a heading and its text.
   - Any other run of consecutive non-blank lines is a **paragraph block**, ending at the next blank line or the next heading.
   - Blank lines separate blocks and carry no content.
   - `boundary_kind` is `Heading` for a block that opens with a heading line, `Paragraph` otherwise.
2. **Greedy packing.** Append blocks to the open segment while `open.byte_length() + separator + block.len() <= max_bytes`; otherwise close the segment and start a new one. Blocks inside one segment are joined with a single blank line (`"\n\n"`).
3. **Oversized single block.** A block that alone exceeds `max_bytes` and has no interior boundary **must** still be split — the byte budget is a hard bound, not a target (`segment-resume.md` CHK008; `constitution.md:353`), and the budget is the stronger constraint. Split it at the largest offset that keeps each piece `<= max_bytes`, snapped **down** to a UTF-8 character boundary so no piece ever splits a `char`. Those forced sub-pieces carry `boundary_kind: Paragraph`; there is no interior boundary to report.
4. **Trimming.** Each segment's `content` is trimmed of leading and trailing whitespace. A segment whose trimmed content is empty is not emitted.
5. **Budget.** `byte_length() <= max_bytes` holds for **every** emitted segment, unconditionally, including the oversized-block case.
6. **Losslessness.** Concatenating the emitted segments reproduces every non-blank source line, in order. No source line is dropped, duplicated, or reordered.
7. **Determinism.** The same `(source, document_hash, max_bytes)` always yields the same vector, including `source_range`, `boundary_kind` and `segment_index`.
8. **Empty input.** A source that is empty or contains only whitespace produces **zero** segments (`spec.md:102`: no segments → no `AudioChunk`, indeterminate progress).
9. **`segment_index`** is 0-based and strictly increasing by 1 across the emitted vector.
10. **`source_range`** refers to the *original* source: `line_start`/`line_end` are the 1-based inclusive line numbers the segment covers, `col_start` is `0`, `col_end` is the byte length of the segment's last source line. A forced sub-piece of an oversized block reports the same line range as the block it came from.
11. **`resume_point`** is `Some(source_range)` — the completed boundary at-or-before the yield point when the yield point falls inside this segment (`R6`; `CLARIFICATION_BATCH.md` C1-A: last boundary **at or before**, never after, never mid-sentence).
12. **`document_hash`** is *not* stored on the segment: the `blake3` document fingerprint is owned by `session` (`spec.md` Assumption 5; R5). It is carried on the split call so callers can bind segments to a document identity, and is deliberately not duplicated into adapter state. **Every call site passes it explicitly** — there is no default.

### 2.1a Recorded deviation from `spec.md`: line-scan segmentation (decision 2026-10-08)

`spec.md:94-95` (FR-024 / FR-024a) describes resume alignment as *parser-side
`pulldown-cmark` segmentation*. §2.1 rule 1 implements it as a **line scan**.
This is a recorded deviation, not an oversight. Decision: keep the line scan,
record the deviation, and drop the now-unconsumed parser dependency (T032).

**Why.** The line scan is what §2.1 normatively specifies and what the frozen
T019 tests assert; switching to `pulldown-cmark` would require revising both,
and this contract is frozen ("no additions, no renames, no removals"). Rule 1's
block model — a heading absorbs its following non-blank run, everything else is a
paragraph — is sufficient for v1's declared `AlignmentGranularity::None`, because
resume aligns to paragraph/heading boundaries and never to sentence boundaries
(`spec.md` clarification 2026-10-06, C1-A; R6).

**What it misclassifies.** Observed and pinned by
`tests/prop_segmentation_shapes.rs` (task T033):

| Construct | `boundary_kind` under rule 1 | Deviation |
|---|---|---|
| ATX heading (`# …` … `###### …`) | `Heading` | none — correct |
| Setext heading (`Title` + `====` / `----`) | `Paragraph` | under-classified: rule 1 recognises ATX only |
| `#` line inside a fenced code block | **`Heading`** | **over-classified**: the scan is not fence-aware, so a fenced comment is narrated as a heading block |
| List item (`- item`, `1. item`) | `Paragraph` | under-classified: a list is narrated as one paragraph block |
| Block quote (`> text`) | `Paragraph` | under-classified |

None of these affects the byte budget (rule 5), losslessness (rule 6),
determinism (rule 7) or resume correctness (rules 10-11) — resume snaps to a
completed block boundary either way. They affect only the `boundary_kind` label
and where narration blocks split. Sentence-level resume is deferred to v2
regardless, so the v1 user-visible promise (paragraph/heading resume) holds.

**Escalation path.** A future feature needing fence-aware, setext-aware or
list-aware segmentation — or sentence-level resume — must adopt `pulldown-cmark`
and revise §2.1 rule 1 and the frozen tests together. That is a coordinated edit
to this contract, not a local change.

### 2.1a Recorded deviation from `spec.md` and `data-model.md`: the segment carries no document fingerprint (decision 2026-10-09)

`spec.md:101` (Key Entities) describes `DocumentSegment` as "carrying a
`blake3` content hash of the document", and `data-model.md` E1 lists a
`blake3_hash` field on the segment (`data-model.md:23`) with rule R5
binding it to resume (`data-model.md:140`). §2.1 rule 12 governs
instead: the frozen §2 struct has exactly five fields and no hash, and
`document_hash` is accepted on the split call and discarded
(`src/chunk.rs:173-177`). This is a recorded deviation, not an
oversight. Decision: keep the frozen struct unchanged and record the
deviation here — the register above records only the line-scan
deviation.

**Why.** The fingerprint belongs to `session`, not the adapter
(`spec.md` Assumption 5: the "`redb` bookmark store and `blake3`
fingerprint remain the sole resume mechanism; the adapter does not
alter position storage"). The bookmark is keyed by the `blake3`
document fingerprint storing the playback offset, so storing the hash
on the segment would be an additive change to the frozen §2 surface
(the header freezes it: "no additions, no renames, no removals") and
a second declaration of session-owned state (`FR-044`).

**Consequence.** Nothing inside the adapter binds a segment to a
document identity: a caller passing a wrong or stale hash is currently
unobservable — the hash never reaches a segment, never influences the
scan, and cannot fail a split (rule 7's determinism is stated over
`(source, document_hash, max_bytes)` and holds trivially, the third
component being discarded). The binding is the caller's responsibility
and is owned by `session`, which keys its bookmark on the fingerprint
itself.

**Reconciliation condition.** If a future requirement needs
in-adapter validation of the fingerprint — the adapter must refuse a
split whose hash does not match the document it was passed — this
deviation is what must be revisited: either store the hash on the
segment (a coordinated additive amendment to the frozen §2 surface) or
validate it at the split boundary before scanning. Until one of those
happens, rule 12 governs, and `spec.md:101` and `data-model.md`
E1/R5 stand overridden by this entry.

### 2.1a Recorded deviation from §2.1 rule 2: heading blocks are isolated into their own segment (decision 2026-10-09)

**Behaviour.** When a heading block is scanned, the open segment is
closed first and the heading block is then emitted as its own segment,
so no other block ever shares a heading's segment
(`src/chunk.rs:197-205`).

**Departs from.** §2.1 rule 2, which is plain
greedy packing over all blocks — "Append blocks to the open segment
while `open.byte_length() + separator + block.len() <= max_bytes`" —
with no heading exemption; the contract states no "own segment" rule
anywhere.

**Affected cases.** Every heading whose block would otherwise join the
open segment (or pack with adjacent paragraphs) under rule 2: the code
closes the segment early instead of packing. Named case: the frozen
test `heading_absorbs_its_body_and_is_never_split_from_it`
(`tests/prop_chunk_refined.rs:341-368`) — for `"## Título\n\nPrimeira
frase do corpo.\n\nSegundo parágrafo."` the code yields **2** segments
(the heading block isolated per this entry, its body absorbed per the
next entry), where rules 1+2 alone pack all three blocks into **1**.

**Invariants.** Not affected: the byte-budget bound (rule 5 — closing
a segment early only makes segments smaller) and the 0-based,
consecutive `segment_index` (rule 9 — assigned in emission order at
close time either way). Also unaffected: losslessness (rule 6),
determinism (rule 7), empty input (rule 8) and the range/resume rules
(10-11). Affected: segmentation granularity only — how many segments a
source yields and which blocks group — which the frozen rules do not
pin and the code pins differently.

**Pinned test.** `tests/prop_chunk_refined.rs:341-368` asserts the
isolation jointly with the absorption below: 2 segments, the opening
one a `Heading` segment, the trailing paragraph a separate `Paragraph`
segment.

### 2.1a Recorded deviation from §2.1 rule 1: a heading block with no immediately-following body absorbs across blank lines (decision 2026-10-09)

**Behaviour.** When a heading has no immediately-following non-blank
line, its block extends across the blank lines to the next non-blank,
non-heading run, so a heading and a body separated by blank lines are
still one block (`src/chunk.rs:343-362`).

**Departs from.** §2.1 rule 1, which bounds
the heading block at "the heading line **and** the immediately
following non-blank lines" — a blank line ends it — and states "Blank
lines separate blocks and carry no content".

**Affected cases.** A heading followed by a blank line and then body
text: under rule 1 the heading is a one-line block and the body is a
separate paragraph block, both eligible for greedy packing; under the
code they are one block. Named case: `"## Título\n\nPrimeira frase do
corpo."` — rule 1 yields heading block `## Título` plus paragraph
block `Primeira frase do corpo.`; the code yields one heading block
carrying both.

**Invariants.** Not affected: the byte-budget bound (rule 5) and the
0-based, consecutive `segment_index` (rule 9) — absorption changes
which lines group, never a segment's byte count or its emission-order
index. Also unaffected: losslessness (rule 6), determinism (rule 7),
empty input (rule 8) and the range/resume rules (10-11) — the
absorbed block still reports its original-source range. Affected:
block grouping only, as above.

**Pinned test.** The same frozen test,
`tests/prop_chunk_refined.rs:341-368`: its heading segment must
contain `Primeira frase do corpo.`, which only the absorbing code
produces.

**Rule text and pinned test disagree.** The frozen rules 1-2 and the
frozen test contradict each other: rules 1-2 alone pack the three
blocks of `"## Título\n\nPrimeira frase do corpo.\n\nSegundo
parágrafo."` into one segment, while `tests/prop_chunk_refined.rs:341-368`
pins the two-segment split the code produces. (§9's test-surface table
already anticipates "a heading block absorbing
its body" — so the absorption was intended — but the normative rule 1
text never states it, and the isolation has no mention anywhere.) The
implementation matches the test, not the prose. Reconciling rule 1-2
prose to the code — or the code to the prose — is owed before the
surface is frozen again; until one of those happens, these two entries
are the authority of record for the behaviours they name.

### 2.1b Precondition on `max_bytes` (budget floor)

Rule 5 (the budget is a hard bound) and rule 6 (losslessness) are **jointly
unsatisfiable when `max_bytes` is below the maximum UTF-8 character width (4
bytes)**. For multi-byte content at such a budget: emitting the character whole
satisfies losslessness and violates the bound; emitting nothing violates
losslessness; splitting the character yields invalid UTF-8.

The bound is therefore a **precondition holding from `max_bytes >= 4` upward**.
Below the floor, rule 6 wins — the character is emitted whole, keeping the
content valid UTF-8 and lossless, exceeding the budget only where no char-safe
cut exists. The floor is pinned by
`budget_floor_at_max_utf8_char_width_keeps_bound_and_losslessness` in
`tests/prop_segmentation_shapes.rs:492`, and the two budget-drawing
property generators (`document_strategy` in `tests/prop_chunk_refined.rs`,
`mixed_document_and_budget_strategy` in `tests/prop_segmentation_shapes.rs`)
draw budgets from `4..=MAX_CHUNK_BYTES`.

`split_document` is unaffected: it always uses `MAX_CHUNK_BYTES` (5 000). The
floor concerns only `split_document_within`, which exists so property tests can
exercise small budgets.

### 2.2 `SourceRange` helpers

```rust
impl SourceRange {
    #[must_use] pub const fn new(line_start: u32, line_end: u32, col_start: u32, col_end: u32) -> Self;
    #[must_use] pub const fn whole_lines(line_start: u32, line_end: u32) -> Self; // col_start = 0, col_end = 0
    #[must_use] pub const fn line_count(&self) -> u32; // line_end - line_start + 1
    #[must_use] pub const fn is_empty(&self) -> bool;
    #[must_use] pub const fn contains_line(&self, line: u32) -> bool; // line_start <= line <= line_end
}

impl fmt::Display for SourceRange { /* "L{line_start}:{col_start}-L{line_end}:{col_end}" */ }
```

`fmt::Display` exists so the failure `observed` string can name a range without document text (R7 / diagnostic rule: identifier pattern, no document content).

---

## 3. `src/failure.rs` — condition → variant table (R7 / CHK005 / CHK003)

```rust
pub use engine_core::closed::EngineError;

/// Maximum lifetime of the failure offer, in milliseconds (FR-023a, FR-032c, FR-041).
pub const MAX_OFFER_MS: u32 = 3_000;

/// Maximum length of the identifier-pattern detail attached to a failure.
pub const MAX_CONDITION_DETAIL_CHARS: usize = 200;

/// The closed set of conditions the adapter can observe. Every member maps
/// to exactly one `EngineError` member — there is no "unmapped" arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureCondition {
    RateDeviation,             // delivered rate != 22050 Hz            -> RateMismatch
    WidthDeviation,            // delivered width != f32/32-bit          -> RateMismatch
    PitchToleranceViolation,   // pitch-tolerance +/-3 cents @ 2.0x     -> RateMismatch
    ChannelDeviation,          // delivered channels != 1                -> ChannelMismatch
    VoiceInitFailure,          // voice model / loader / init corruption -> InitFailed
    ProcessDeath,              // killed / OOM                          -> Unreachable
    TransportUnreachable,      // transport unreachable                 -> Unreachable
    DeviceRefusedAtOpen,       // AAudio rate/format refusal at open     -> DeviceRefused
    DeviceRemoved,             // BECOMING_NOISY OR onAudioDevicesRemoved -> DeviceLost
    StartRefusedAtStart,       // ForegroundServiceStartNotAllowed; consent missing at start -> StartRefused
}

impl FailureCondition {
    /// Every member, in declaration order.
    pub const ALL: [Self; 10];
    /// The single `EngineError` member this condition reports (exhaustive 10-arm match).
    #[must_use] pub const fn variant(self) -> EngineError;
    /// A stable snake_case identifier for this condition (`rate_deviation`, …).
    #[must_use] pub const fn detail(self) -> &'static str;
}

/// A named failure crossing the adapter → player boundary (data-model E3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineFailure {
    pub variant: EngineError,
    pub condition: FailureCondition,
    /// Identifier-pattern detail only (e.g. `delivered_rate=24000`). Never document text.
    pub observed: String,
    /// Echo of the segment's `resume_point` (falling back to its
    /// `source_range` when none is declared — `chunk.rs` always emits
    /// `Some(source_range)`) — derived from `DocumentSegment`, never
    /// adapter-computed; a failure before any delivery echoes the
    /// segment in synthesis (`player` MUST NOT treat the echo as
    /// playback authority; it derives resume position from its own
    /// delivered-chunk log — `constitution.md:82-86`). Closed per
    /// `checklists/segment-resume.md` CHK030.
    pub session_position: SourceRange,
    /// `<= MAX_OFFER_MS`.
    pub offer_expires_in_ms: u32,
    /// Always `EngineTier::OfflineDefault` at v1 (R11).
    pub tier_at_failure: EngineTier,
}

/// Why a failure report was refused at construction time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FailureReportError {
    #[error("condition detail is {len} chars; the diagnostic record allows at most {max} identifier-pattern chars")]
    DetailTooLong { len: usize, max: usize },
    #[error("failure offer is {offer_ms} ms; the offer must expire within {max_ms} ms")]
    OfferTooLong { offer_ms: u32, max_ms: u32 },
}

impl EngineFailure {
    /// Build a named failure, failing closed on an over-long detail or an
    /// over-long offer. `tier_at_failure` is always `REGISTERED_TIER`.
    pub fn report(
        condition: FailureCondition,
        observed: &str,
        session_position: SourceRange,
        offer_expires_in_ms: u32,
    ) -> Result<Self, FailureReportError>;
}
```

Bad Markdown is **not** a `FailureCondition`: it is rejected by the document layer before the adapter is invoked (R8).

---

## 4. `src/reg.rs` — registration + selectability (R11 / SC-051)

```rust
use engine_core::closed::EngineTier;

/// The only tier this adapter registers at v1.
pub const REGISTERED_TIERS: [EngineTier; 1] = [EngineTier::OfflineDefault];

/// The registered tiers (`&[EngineTier::OfflineDefault]`).
#[must_use] pub const fn registered_tiers() -> &'static [EngineTier];

/// Whether `tier` may be selected while this adapter is registered.
/// Exhaustive 5-arm match; `TestDouble` is `false` because the stand-ins
/// are fixtures and are never selectable while a non-`test_double` adapter
/// is registered (`001/spec.md:89`; `closed.rs`:305-312).
#[must_use] pub const fn is_selectable(tier: EngineTier) -> bool;
```

`src/reg.rs` **must not** re-declare tier names as `&str`: the registry in
`engine_core::closed` is the single list (`FR-044`). `is_selectable` must have
five distinct arms — never a grouped wildcard, never `_ =>`.

---

## 5. `src/consume.rs` — player declaration consumption (`FR-024c` / contract §4.1)

```rust
use crate::{AdapterCapabilities, AlignmentGranularity, SsmlLevel};

/// What the player must do with SSML-bearing source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsmlDelivery { PlainText, PartialSsml, FullSsml }

/// The boundary resume may snap to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeAnchor { ParagraphHeading, Word, Sentence }

/// The player's plan, derived only from the declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerPlan { pub ssml: SsmlDelivery, pub resume: ResumeAnchor }

#[must_use] pub const fn ssml_delivery(level: SsmlLevel) -> SsmlDelivery;      // exhaustive 3-arm
#[must_use] pub const fn resume_anchor(granularity: AlignmentGranularity) -> ResumeAnchor; // exhaustive 3-arm
#[must_use] pub const fn plan_for(capabilities: AdapterCapabilities) -> PlayerPlan;
```

`plan_for(declared_capabilities())` is `(PlainText, ParagraphHeading)`. The mapping
must be total over all 9 `(SsmlLevel, AlignmentGranularity)` pairs and must never
*silently* prefer a stronger capability than declared — a declared `None` must map
to `PlainText` / `ParagraphHeading`, never to `FullSsml` / `Sentence`
(Principle VI: degradation is observable, never silent).

The current `_ => "unsupported"` wildcard in `consume.rs` **is removed** by T021.

---

## 6. `src/format_check.rs` — the format gate (R1, R2, CHK025)

```rust
use crate::failure::EngineError;
use engine_core::format::DeclaredFormat;

/// Verify `(rate_hz, channels, width_bits)` against the declared handoff
/// shape. Delegates to `engine_core::format::check_handoff` — the single
/// declaration of the shape (FR-044). Never silently resamples or downmixes.
#[must_use] pub fn check_format(rate_hz: u32, channels: u16, width_bits: u16) -> Result<(), EngineError>;

/// Verify a delivered sample buffer: shape first, then that the buffer
/// length matches the declared rate. A `pcm_data` that is not a whole
/// number of 1000/… — in practice: any buffer whose length disagrees with
/// `duration_ms` — is a `RateMismatch`.
#[must_use] pub fn check_delivery(
    rate_hz: u32,
    channels: u16,
    width_bits: u16,
    pcm_sample_count: usize,
    duration_ms: f64,
) -> Result<(), EngineError>;

/// The declared shape, re-exported so callers need not depend on
/// `engine-core` directly.
pub const DECLARED_RATE_HZ: u32 = DeclaredFormat::SAMPLE_RATE;   // 22050
pub const DECLARED_CHANNELS: u16 = DeclaredFormat::CHANNELS;      // 1
pub const DECLARED_WIDTH_BITS: u16 = DeclaredFormat::SAMPLE_WIDTH_BITS; // 32
```

`check_delivery` compares `pcm_sample_count as f64` against
`(duration_ms / 1000.0) * DECLARED_RATE_HZ as f64` with a half-sample tolerance,
because `duration_ms` is a derived `f64` (data-model E2). A mismatch is
`EngineError::RateMismatch`; a shape deviation is whatever `check_handoff` names.

---

## 7. `src/synth.rs` — the real `offline_default` engine (bound 2026-10-09, T044)

```rust
use crate::chunk::DocumentSegment;
use crate::failure::EngineFailure;

/// Synthesize one bounded segment to mono `f32` at 22 050 Hz
/// through the embedded Piper pt-BR `faber` VITS voice.
pub fn synthesize(segment: &DocumentSegment) -> Result<Vec<f32>, EngineFailure>;
```

The earlier design-phase text — "the Piper `faber` voice model
is not bound in this design-phase crate … integration belongs to
`003-contract-edit`" — is **superseded**: T044 bound the real
engine in this feature (that deferral was recorded in the old
§7/§10 and in `plan.md` §Structure Decision, but `spec.md`
grants no such deferral — it defers only sentence-level resume
(`SC-044`), consent, and the active fallback switch). Amendment
recorded 2026-10-09:

- **Engine**: `sherpa-onnx` **1.13.8** (the constitution §TTS
  Engine Registry pin for `offline_default`), driven through
  `OfflineTts` + `OfflineTtsVitsModelConfig`; `num_threads: 1`
  — a single-threaded ONNX Runtime session spawns no
  uncontrolled per-session thread pool that would contend with
  the playback, focus and notification threads of the driving
  shell, and pre-roll (FR-015a) runs synthesis ahead of
  real-time.
- **Voice assets embedded in-repo** at
  `assets/voices/pt_BR-faber-medium/` (CC0 per the repository's
  `MODEL_CARD`; `PROVENANCE.md` records the source repository,
  the model's declared 22 050 Hz sample rate, the blake3 of the
  model file, and the download method). Paths are resolved at
  **compile time** from `CARGO_MANIFEST_DIR`; nothing is
  fetched from the network at synthesis time (Principle IX;
  `spec.md` Assumption 2).
- **One load per process**: the engine is a
  `OnceLock<Option<OfflineTts>>`; a failed load is cached as
  `None` and re-reported on every later call with the *current*
  segment's echoed position (the cause — missing or corrupt
  assets — is deterministic).
- **Success** (`Ok`): a non-empty mono-`f32`-at-22 050 Hz
  buffer, verified by `check_delivery` (§6) before it is
  returned. The delivered rate is read from the engine's own
  output and compared to the declared 22 050 Hz; a deviation
  is a named `RateMismatch`, never a silent resample (FR-024b /
  SC-035). An empty result for a non-empty segment is treated as
  a corrupt synthesis result → `VoiceInitFailure` → `InitFailed`
  (the silent-success defect mode this module exists to prevent).
- **Failure paths** — all through the fail-closed
  `EngineFailure::report`, with `session_position` echoing the
  segment's `resume_point` (falling back to its `source_range`),
  never adapter-computed (segment-resume.md CHK030):
  - voice model load failure → `VoiceInitFailure` → `InitFailed`;
  - interior NUL in `segment.content` (cannot cross the C FFI
    boundary as a `CString`) → `TransportUnreachable` →
    `Unreachable`;
  - `generate_with_config` returns `None` (engine process died
    — killed/OOM) → `ProcessDeath` → `Unreachable`;
  - delivered rate ≠ 22 050 Hz → `RateDeviation` → `RateMismatch`.

### 7.1 `ort` deviation record (exception 4, `constitution.md:412-413`)

`plan.md` (`:23`) and constitution exception 4 pin `ort`
2.0.0-rc.13 for ONNX/NNAPI inference. The bound engine does
**not** depend on the `ort` crate: `sherpa-onnx-sys` 1.13.8's
build script downloads the prebuilt static archive
`sherpa-onnx-v1.13.8-linux-x64-static-lib.tar.bz2`
(k2-fsa/sherpa-onnx GitHub releases; recorded in
`assets/voices/pt_BR-faber-medium/PROVENANCE.md`), which
**bundles ONNX Runtime, espeak-ng and piper_phonemize as static
libraries**. ONNX Runtime is therefore present in the dependency
tree exactly once — the bundled copy — so exception 4's intent
(offline ONNX inference with no second runtime) is preserved
with one runtime instead of two, and the `ort` RC's
pre-recorded stability risk is avoided entirely. Risks recorded:
(a) the native archive is fetched from GitHub at *build* time
(not crates.io); the voice *model* assets, unlike the native
archive, are embedded in-repo; (b) the prebuilt archive is
per-platform — this host exercises the x64-linux archive, and
the Android arm64 build (gate 9) exercises the corresponding
`android-arm64` archive of the same v1.13.8 release. No
non-crates.io *Rust* source is introduced; `sherpa-onnx`
1.13.8 is the constitution-pinned crates.io registry version
(latest stable of the pinned line).

---

## 8. `src/capabilities.rs` — declaration (already present; keep)

```rust
use crate::{AdapterCapabilities, AlignmentGranularity, SsmlLevel};

/// The unconditional `(None, None)` declaration.
#[must_use] pub fn capabilities() -> AdapterCapabilities;
```

It returns exactly what `crate::declared_capabilities()` returns; there is one
declaration, expressed once.

---

## 9. Test surface required by T018–T020 (integration tests, `tests/`)

Each file below is written **against the contract above**, before the
implementation exists. Tests must exercise the real crate — a test that
re-declares `EngineError`, `SsmlLevel`, `AdapterCapabilities` or its own
`split` implementation is a defect and must be rejected (F1/F2 root cause).

| File | Task | Must cover |
|---|---|---|
| `tests/prop_format.rs` | T018 | `check_format` / `check_delivery` against `engine_core::format::check_handoff` and the real `EngineError`; the 7-member closed set is enumerated exhaustively and serde round-trips; rate/width deviation ⇒ `RateMismatch`; channel deviation ⇒ `ChannelMismatch`; sample-count/duration disagreement ⇒ `RateMismatch`; proptest over `(rate, channels, width)` with the rule "any deviation names an error, `Ok` iff exactly `(22050, 1, 32)`" |
| `tests/prop_chunk.rs` | T019 | `split_document`: every `byte_length() <= MAX_CHUNK_BYTES`; `byte_length()` equals the byte count **and is not** the char count (use multi-byte input to distinguish); losslessness (every non-blank source line appears, in order); `segment_index` strictly 0,1,2,…; determinism; `source_range` 1-based and non-decreasing; proptest with a generator producing headings, paragraphs, blank runs, and multi-byte characters |
| `tests/prop_chunk_refined.rs` | T019 | Refined edge cases: `MAX_CHUNK_BYTES` exact-fit; an oversized single block with no interior boundary (hard split, char-safe, still `<= 5000`); empty and whitespace-only input ⇒ zero segments; a heading block absorbing its body; `document_hash` accepted without being stored on the segment; small custom budgets via `split_document_within` |
| `tests/prop_declare.rs` | T020/T2-T3 | `declared_capabilities()` is `(None, None)` on every call; immutability (repeat calls, interleaved with arbitrary work); `SsmlLevel` and `AlignmentGranularity` have exactly 3 members, exhaustively matched (a `const ALL` or an exhaustive `match` over both enums so a 4th member would not compile silently); no `Other`/`Unknown` member exists |
| `tests/unit_failure.rs` | T020/T5 | `FailureCondition::ALL` ⇒ the exact condition→variant table of §3; the 7 `EngineError` members are all reachable and none is invented; `EngineFailure::report` refuses `observed` over `MAX_CONDITION_DETAIL_CHARS` and an offer over `MAX_OFFER_MS`; `tier_at_failure` is always `OfflineDefault`; `session_position` equals the segment's `source_range`; SC-051 mechanism: a failure is named, offers `<= 3 s`, and no other tier is selectable |
| `tests/unit_consume.rs` | T020/T5 | `ssml_delivery` / `resume_anchor` are total over all 9 pairs; `plan_for(declared_capabilities()) == (PlainText, ParagraphHeading)`; a declared `None` never upgrades to `FullSsml`/`Sentence`; no wildcard arm exists |

---

## 10. Explicitly out of scope (do not implement)

- ~~`AudioChunk` / `DocumentSegment`-to-player delivery structs — delivery is the integration stage's contract (`003-contract-edit`).~~ **Superseded 2026-10-09 (T043):** `src/audio_chunk.rs` implements `AudioChunk` / `DeliveryError` / `deliver` / `MAX_PRE_ROLL_MS` and they are re-exported at the crate root (`src/lib.rs:84`). The delivery unit is in scope and implemented; what remains deferred is the `player`-side wiring of it (T045/T046 — blocked on this host, see the Phase 5 record).
- `T016` engine trait, `trait_.rs`, any marker trait.
- `snap_to_boundary` (in scope at v1 — the boundary-kind-agnostic snap; not deferred), sentence-level alignment, `BoundaryKind::Sentence`.
- `SC-044` active fallback switch, consent acquisition surface, cloud tiers, Edge fallback, a second voice.
- Any change to `crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`, `deny.toml`, or the constitution.
- **NOT suppressed by this clause (T060 amendment)**: `FR-024c`'s `consume::plan_for()` / `declared_capabilities()` / `ssml_delivery()` / `resume_anchor()` branch — the adapter consumes the declaration, not player transport/position/bookmark. `spec.md:144` out-of-scope applies to adapter-only description; `FR-024c` governs the declaration-consumption interface (`adapter-capabilities.md` §A). `API_CONTRACT.md` §10 deferred wire = `T045`/`T046` delivery-unit integration, not the branch itself.