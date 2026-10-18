//! CHK030 — US2 resume-boundary property suite: the FR-024a
//! paragraph/heading snap precedence and the single-authority
//! `session_position` echo rule.
//!
//! Spec anchors:
//! - CHK030 open item (`specs/002-fr-024-synthesis-adapter/checklists/
//!   segment-resume.md`): the single-authority half is pending —
//!   `EngineFailure.session_position` (`specs/002-fr-024-synthesis-adapter/
//!   spec.md:106`, `contracts/adapter-handoff.md:165`) must be restated
//!   as an echo of the last delivered `DocumentSegment.source_range_ref`,
//!   never adapter-computed; the derivation rule must be stated; and the
//!   `snap_to_sentence_start` decision (`crates/player/src/position.rs:152`)
//!   must be recorded. This suite records the decision as the rename to
//!   `snap_to_boundary` and encodes the echo rule as executable assertions.
//! - FR-024a (`specs/002-fr-024-synthesis-adapter/spec.md:95`): the
//!   adapter splits Markdown into bounded chunks at paragraph/heading
//!   boundaries; chunk boundaries become resume-alignment points; if a
//!   sentence boundary falls within a paragraph/heading, the
//!   paragraph/heading boundary takes precedence (sentence-level resume
//!   deferred to v2, `AlignmentGranularity::None`).
//! - FR-015 (`specs/001-android-media-shell/spec.md:59`): resume at the
//!   nearest completed paragraph/heading boundary containing the 5%-back
//!   point; the rewind is 5% of the paused duration capped at 30 s; total
//!   backward movement from the yield point MUST NOT exceed 30 seconds
//!   under any pause length.
//! - `contracts/adapter-handoff.md:30`: `AudioChunk` is immutable after
//!   delivery — the adapter never modifies chunk fields after delivery.
//! - `.specify/memory/constitution.md:82-86`: single playback authority —
//!   Rust owns position; a position computed in the adapter would be a
//!   second source of truth.
//! - `specs/002-fr-024-synthesis-adapter/data-model.md` E1/E2/E3: the
//!   `DocumentSegment` / `AudioChunk` / `EngineFailure` shapes mirrored by
//!   the fixtures below.
//!
//! Expected API contract (blocking — the CHK029/CHK030 code decision):
//! 1. `player::position::PositionClock::snap_to_boundary(&self,
//!    boundaries: &[f64]) -> f64` — the renamed, boundary-kind-agnostic
//!    snap (CHK029: rename from `snap_to_sentence_start`). Boundary kinds
//!    are enforced upstream by the FR-024a filter ([`eligible_boundaries`]),
//!    never inside the snap: only paragraph/heading boundaries are
//!    resume-alignment points. The snap returns the greatest boundary at
//!    or before the cursor, never earlier than the cap point — the cap
//!    takes precedence over the zero-mid-word rule (FR-015).
//! 2. The cap point is measured from the clock's stored basis — the
//!    position at the last audible-progress tick, i.e. the yield point of
//!    FR-015 — not from the rewound cursor:
//!    `(self.stored_basis() - REWIND_CAP_SECS).max(0.0)`. Measured from
//!    the rewound cursor instead, the rewind and the snap compound to
//!    twice the cap (probe: yield 100 s, 600 s pause, boundary 45 s →
//!    resume 45 s, 55 s backward — an FR-015 violation the probe in the
//!    verification scratch crate pinned). The existing red property
//!    `the_sentence_snap_lands_on_a_boundary_never_mid_word` in
//!    `prop_transport_math.rs` already asserts this origin; this suite and
//!    that suite go green together with the one-line correction.
//! 3. The CHK030 echo rule: the player's `session_position` is the
//!    `source_range_ref` of the last delivered `DocumentSegment` — a copy
//!    held by the player, derived from delivery history alone, never from
//!    adapter state and never from an adapter computation such as a
//!    "nearest completed boundary".
//!
//! FAILS before the rename, the cap-origin correction, and the echo rule
//! land: `snap_to_boundary` is unresolved (the crate still carries
//! `snap_to_sentence_start`), so this test target does not build. That
//! build failure is the required pre-implementation red state — the same
//! pattern as `prop_transport_math.rs` before T025. The two existing unit
//! tests in `crates/player/src/position.rs` that call the old name
//! (`snap_to_sentence_start_lands_on_boundary`,
//! `snap_to_sentence_start_respects_cap`) are renamed by the same change.
//!
//! Exactness note: every position and boundary is generated on the
//! quarter-second grid (`k × 0.25`), an exact binary-fraction grid, and
//! the oracle asserts with the same expressions the contract states, so
//! all comparisons are bit-exact. A tolerance would wrongly admit an
//! off-boundary snap or a cap violation (the 30-second cap is a hard
//! ceiling, FR-015).
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures
//! (`{var}`). proptest 1.11 builds the message through
//! `concat!`, and rustc (1.98) rejects implicit captures in
//! macro-expanded format strings ("`format_args!` cannot capture
//! variables when the format string is expanded from a macro").

use proptest::prelude::*;

use engine_core::closed::EngineError;
use engine_core::format::{DeclaredFormat, check_handoff};
use player::position::{PositionClock, REWIND_CAP_SECS, REWIND_FRACTION};

/// A source range in the document (data-model.md E1 `source_range`: a
/// `pulldown-cmark` event range — byte offsets into the Markdown source).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SourceRange {
    /// The byte offset at which the range starts.
    start: u32,
    /// The byte offset at which the range ends (exclusive).
    end: u32,
}

/// The boundary kinds of the document's structure (data-model.md E1
/// `boundary_kind`). `Sentence` is reserved for the deferred v2
/// granularity (`AlignmentGranularity::None`, FR-024a): it exists in a
/// document's boundary list but never wins a resume decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BoundaryKind {
    /// A paragraph split — a resume-alignment point (FR-024a).
    Paragraph,
    /// A heading split — a resume-alignment point (FR-024a).
    Heading,
    /// A sentence split — deferred to v2; never a resume-alignment point.
    Sentence,
}

/// One boundary of the document's structure: a position in playback time
/// carrying the kind of split it names.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResumeBoundary {
    /// The boundary's position, in seconds from the document start.
    at_secs: f64,
    /// The kind of split the boundary names.
    kind: BoundaryKind,
}

/// A bounded chunk of spoken content produced adapter-side before
/// synthesis (data-model.md E1, trimmed to the fields the CHK030 echo
/// rule reads).
#[derive(Clone, Debug, PartialEq, Eq)]
struct DocumentSegment {
    /// The `blake3` content fingerprint of the parent document (FR-036).
    blake3_hash: String,
    /// The source range this segment covers — the echo basis of CHK030.
    source_range: SourceRange,
    /// The split kind: `Paragraph` or `Heading` only (FR-024a).
    boundary_kind: BoundaryKind,
    /// The segment's byte length — the hard `≤ 5000` bound (rule R3).
    byte_length: usize,
    /// The segment's index, monotonic within the session (E1).
    segment_index: u64,
}

/// The delivery unit handed to the player (data-model.md E2), immutable
/// after delivery (adapter-handoff.md:30).
#[derive(Clone, Debug, PartialEq)]
struct AudioChunk {
    /// Mono `f32` PCM at the declared 22050 Hz (FR-024 / FR-033).
    pcm_data: Vec<f32>,
    /// The declared sample rate: `22050` (never substituted, FR-035).
    rate_hz: u32,
    /// The declared channel count: `1` (mono; never downmixed, FR-033).
    channels: u16,
    /// The chunk's duration, computed from `pcm_data` (E2).
    duration_ms: f64,
    /// The remaining pre-roll, bounded at `60_000` ms (FR-015a).
    pre_roll_remaining_ms: Option<u32>,
    /// The link back to the delivered `DocumentSegment` (E2
    /// `source_range_ref`).
    source_range_ref: SourceRange,
}

/// The failure view the player receives (data-model.md E3). CHK030: its
/// `session_position` is the echo of the last delivered segment's
/// `source_range_ref` — never an adapter computation.
#[derive(Clone, Debug, PartialEq, Eq)]
struct EngineFailureView {
    /// The named failure variant (the closed 7-member set, FR-023/FR-035).
    variant: EngineError,
    /// The identifier-pattern condition (≤ 200 chars, FR-045).
    condition: String,
    /// The session position at failure: the echo (CHK030).
    session_position: SourceRange,
}

/// The mock adapter's forbidden computation: a "nearest completed
/// boundary" derived from adapter-internal state. CHK030 /
/// constitution.md:82-86: this is NOT a source of truth for the session
/// position — the echo of the last delivered `DocumentSegment`
/// `source_range_ref` is (REVIEW_BATCH_FINAL_2026-10-08: "state
/// `EngineFailure.session_position` is echoed from last delivered
/// `DocumentSegment`, derived, never adapter-computed").
#[derive(Clone, Copy, Debug)]
struct MockAdapter {
    /// The adapter-computed resume position — never a source of truth.
    computed_nearest_boundary: SourceRange,
}

/// The contract-violating mutation a mock adapter may attempt after
/// delivery: rewrite every field of its retained chunk copy. The player's
/// snapshot must be unaffected (adapter-handoff.md:30 — "Immutable after
/// delivery: adapter never modifies chunk fields after delivery").
fn violate_immutability(chunk: &mut AudioChunk) {
    chunk.pcm_data.push(0.0);
    chunk.rate_hz = u32::MAX;
    chunk.channels = u16::MAX;
    chunk.duration_ms = f64::MAX;
    chunk.pre_roll_remaining_ms = Some(u32::MAX);
    chunk.source_range_ref = SourceRange {
        start: u32::MAX,
        end: u32::MAX,
    };
}

/// The player's delivery record: the single authority's view of what the
/// adapter delivered (constitution.md:82-86). The CHK030 echo rule lives
/// here — `session_position` is derived from delivery history alone.
#[derive(Clone, Debug, Default)]
struct DeliveryLog {
    /// Every delivered segment, in delivery order.
    delivered: Vec<DocumentSegment>,
    /// The player's snapshot of the last delivered chunk — a copy, not a
    /// live view into adapter state (the immutability property).
    last_chunk: Option<AudioChunk>,
}

impl DeliveryLog {
    /// Record one delivery: a segment and the chunk that carried it. The
    /// chunk's `source_range_ref` must reference the segment (the E1/E2
    /// linkage the echo rule depends on).
    fn deliver(&mut self, segment: DocumentSegment, chunk: AudioChunk) {
        assert_eq!(
            chunk.source_range_ref, segment.source_range,
            "a delivered chunk's source_range_ref must reference its DocumentSegment (data-model.md E2)"
        );
        self.delivered.push(segment);
        self.last_chunk = Some(chunk);
    }

    /// CHK030 single authority: the session position is the ECHO of the
    /// last delivered `DocumentSegment`'s `source_range` — a copy of a
    /// delivered fact, never a computation over adapter state.
    fn session_position(&self) -> Option<SourceRange> {
        self.delivered.last().map(|segment| segment.source_range)
    }

    /// The player's snapshot of the last delivered chunk.
    const fn last_chunk(&self) -> Option<&AudioChunk> {
        self.last_chunk.as_ref()
    }
}

/// Positions on the exact quarter-second grid `k × 0.25` ∈ [0, 3600].
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds the value at 14 400 quarter-seconds (2^12), far below f64's exact-integer range (2^53); the grid position converts exactly"
)]
fn position_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// Paused durations `d` ∈ [0, 7200] s as exact centisecond multiples.
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds the value at 720 000 centiseconds (< 2^20), far below f64's exact-integer range (2^53); the pause duration converts exactly"
)]
fn paused_duration_strategy() -> impl Strategy<Value = f64> {
    (0u64..=720_000).prop_map(|centis| centis as f64 / 100.0)
}

/// The boundary kinds a document's structure carries — including the
/// deferred `Sentence`, so the precedence property can generate sentence
/// boundaries falling inside paragraphs.
fn boundary_kind_strategy() -> impl Strategy<Value = BoundaryKind> {
    prop_oneof![
        Just(BoundaryKind::Paragraph),
        Just(BoundaryKind::Heading),
        Just(BoundaryKind::Sentence),
    ]
}

/// The split kinds a delivered segment may carry: `Paragraph` or
/// `Heading` only — a `Sentence` is never a chunk split (FR-024a).
fn split_kind_strategy() -> impl Strategy<Value = BoundaryKind> {
    prop_oneof![Just(BoundaryKind::Paragraph), Just(BoundaryKind::Heading),]
}

/// Mixed boundary lists on the quarter-second grid: every position of the
/// document's structure — paragraph, heading, and the deferred sentence
/// boundaries — sorted by position and deduplicated with the FR-024a
/// precedence already applied at coincident positions (a paragraph/heading
/// boundary survives where a sentence boundary shares its position).
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every position at 14 400 quarter-seconds (2^12), far below f64's exact-integer range (2^53); the boundary positions convert exactly"
)]
fn mixed_boundaries_strategy() -> impl Strategy<Value = Vec<ResumeBoundary>> {
    proptest::collection::vec((0u64..=14_400, boundary_kind_strategy()), 0..=32).prop_map(
        |mut entries| {
            // The document start is always a paragraph boundary.
            entries.push((0, BoundaryKind::Paragraph));
            entries.sort_by_key(|(position, kind)| (*position, kind_rank(*kind)));
            entries.dedup_by_key(|(position, _)| *position);
            entries
                .into_iter()
                .map(|(position, kind)| ResumeBoundary {
                    at_secs: position as f64 * 0.25,
                    kind,
                })
                .collect()
        },
    )
}

/// The rank of a boundary kind in the FR-024a precedence order: a
/// paragraph/heading boundary outranks a sentence boundary sharing its
/// position, so the dedup in [`mixed_boundaries_strategy`] keeps the
/// paragraph/heading boundary.
const fn kind_rank(kind: BoundaryKind) -> u8 {
    match kind {
        BoundaryKind::Paragraph | BoundaryKind::Heading => 0,
        BoundaryKind::Sentence => 1,
    }
}

/// The FR-024a filter: only paragraph/heading boundaries are
/// resume-alignment points. Sentence boundaries are deferred to v2
/// (`AlignmentGranularity::None`) and never enter the list handed to
/// `snap_to_boundary` — which is boundary-kind-agnostic (CHK029).
fn eligible_boundaries(boundaries: &[ResumeBoundary]) -> Vec<f64> {
    boundaries
        .iter()
        .filter(|boundary| {
            matches!(
                boundary.kind,
                BoundaryKind::Paragraph | BoundaryKind::Heading
            )
        })
        .map(|boundary| boundary.at_secs)
        .collect()
}

/// A source range anywhere in a document of up to ~22 kB.
fn source_range_strategy() -> impl Strategy<Value = SourceRange> {
    (0u32..=20_000, 1u32..=2_000).prop_map(|(start, length)| SourceRange {
        start,
        end: start + length,
    })
}

/// A delivery sequence: segments in document order with increasing,
/// non-overlapping source ranges and monotonic segment indices (E1).
fn segment_sequence_strategy() -> impl Strategy<Value = Vec<DocumentSegment>> {
    proptest::collection::vec(
        (
            0u32..=500,
            1u32..=1_000,
            split_kind_strategy(),
            1usize..=5_000,
        ),
        0..=16,
    )
    .prop_map(|entries| {
        let mut cursor = 0u32;
        entries
            .into_iter()
            .enumerate()
            .map(|(index, (gap, length, kind, byte_length))| {
                cursor += gap;
                let source_range = SourceRange {
                    start: cursor,
                    end: cursor + length,
                };
                cursor += length;
                DocumentSegment {
                    blake3_hash: format!("{index:032x}"),
                    source_range,
                    boundary_kind: kind,
                    byte_length,
                    segment_index: index as u64,
                }
            })
            .collect()
    })
}

/// One delivery: a segment and the chunk that carried it, with the E1/E2
/// linkage already satisfied — the chunk's `source_range_ref` references
/// the segment's `source_range`, the chunk is mono `f32` at the declared
/// 22050 Hz, and the pre-roll is bounded at 60 s (FR-015a).
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds the buffer at 2 205 samples and the start byte below 15 000, far below f64's exact-integer range (2^53); the derived duration converts exactly"
)]
fn delivery_strategy() -> impl Strategy<Value = (DocumentSegment, AudioChunk)> {
    (
        0u32..=8_000,
        1u32..=1_000,
        split_kind_strategy(),
        1usize..=5_000,
        proptest::collection::vec(any::<f32>(), 0..=2_205),
        prop_oneof![Just(None), (0u32..=60_000).prop_map(Some)],
    )
        .prop_map(
            |(start, length, kind, byte_length, pcm_data, pre_roll_remaining_ms)| {
                let source_range = SourceRange {
                    start,
                    end: start + length,
                };
                let rate_hz = DeclaredFormat::SAMPLE_RATE;
                (
                    DocumentSegment {
                        blake3_hash: format!("{start:032x}"),
                        source_range,
                        boundary_kind: kind,
                        byte_length,
                        segment_index: 0,
                    },
                    AudioChunk {
                        duration_ms: pcm_data.len() as f64 / rate_hz as f64 * 1000.0,
                        pcm_data,
                        rate_hz,
                        channels: DeclaredFormat::CHANNELS,
                        pre_roll_remaining_ms,
                        source_range_ref: source_range,
                    },
                )
            },
        )
}

/// The rewound cursor: the yield position moved back by exactly
/// `min(REWIND_FRACTION × d, REWIND_CAP_SECS)`, clamped at the floor
/// (FR-015) — the position the snap operates on.
fn rewound_cursor(yield_position: f64, paused_duration: f64) -> f64 {
    let rewind_amount = (REWIND_FRACTION * paused_duration).min(REWIND_CAP_SECS);
    (yield_position - rewind_amount).max(0.0)
}

/// The cap point: the earliest position the resume may land on — the
/// yield position less the hard 30-second cap, clamped at the floor
/// (FR-015: "Total backward movement from the yield point MUST NOT exceed
/// 30 seconds under any pause length"). Measured from the yield, not from
/// the rewound cursor, so the rewind and the snap cannot compound past
/// the cap.
fn cap_point(yield_position: f64) -> f64 {
    (yield_position - REWIND_CAP_SECS).max(0.0)
}

/// The greatest boundary at or before the cursor, or `None` when no
/// boundary precedes the cursor.
fn greatest_boundary_at_or_before(cursor: f64, boundaries: &[f64]) -> Option<f64> {
    let mut best: Option<f64> = None;
    for &boundary in boundaries {
        if boundary <= cursor && best.is_none_or(|previous| boundary > previous) {
            best = Some(boundary);
        }
    }
    best
}

/// Whether a sentence boundary sits strictly inside a paragraph/heading:
/// the greatest eligible boundary at or before it is strictly earlier, so
/// the sentence boundary falls between two paragraph/heading splits.
fn is_strictly_inside_paragraph(at_secs: f64, eligible: &[f64]) -> bool {
    greatest_boundary_at_or_before(at_secs, eligible).is_some_and(|boundary| boundary < at_secs)
}

/// The expected snap result, derived from the specification rather than
/// from the implementation: the greatest boundary at or before the cursor
/// when that boundary is at or after the cap point, and the cap point
/// otherwise — the cap takes precedence over the zero-mid-word rule
/// (FR-015). The cap point is measured from `basis` — the yield position
/// in the composed resume flow, the cursor itself for a direct snap.
fn expected_snap(cursor: f64, basis: f64, boundaries: &[f64]) -> f64 {
    let floor = (basis - REWIND_CAP_SECS).max(0.0);
    match greatest_boundary_at_or_before(cursor, boundaries) {
        Some(boundary) if boundary >= floor => boundary,
        Some(_) | None => floor,
    }
}

/// The resume point under test: rewind 5% of the paused duration capped at
/// 30 s (FR-015), then snap to the greatest eligible boundary at or before
/// the rewound cursor through the renamed, boundary-kind-agnostic
/// `snap_to_boundary` (CHK029/CHK030 code decision). The clock is created
/// at the yield position, so its stored basis — the snap's cap origin —
/// is the yield.
fn resume_point(yield_position: f64, paused_duration: f64, boundaries: &[ResumeBoundary]) -> f64 {
    let mut clock = PositionClock::new(yield_position);
    let _cursor = clock.rewind_for_silence(paused_duration);
    let eligible = eligible_boundaries(boundaries);
    clock.snap_to_boundary(&eligible)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[expect(
        clippy::float_cmp,
        reason = "quarter-second grid positions, yields, pauses and cap points are exact binary fractions; == is the correct assertion and a tolerance would admit an off-boundary snap"
    )]
    #[test]
    fn given_any_cursor_and_paragraph_heading_splits_when_snapping_then_the_greatest_boundary_at_or_before_the_cursor_wins(
        (cursor, boundaries) in (position_strategy(), mixed_boundaries_strategy())
    ) {
        // FR-024a / FR-015: the snap returns the greatest boundary at or
        // before the cursor — never a mid-word position — and never lands
        // earlier than the cap point. A clock created at the cursor has
        // its stored basis at the cursor, so the cap is measured from the
        // cursor for a direct snap.
        let clock = PositionClock::new(cursor);
        let eligible = eligible_boundaries(&boundaries);
        let snapped = clock.snap_to_boundary(&eligible);
        let expected = expected_snap(cursor, cursor, &eligible);
        prop_assert_eq!(
            snapped,
            expected,
            "snap from cursor {} did not land on the greatest boundary at or before it",
            cursor
        );
        prop_assert!(
            snapped <= cursor,
            "the snap {} landed after the cursor {}",
            snapped, cursor
        );
        prop_assert!(
            snapped >= (cursor - REWIND_CAP_SECS).max(0.0),
            "the snap {} landed before the cap point for cursor {}",
            snapped, cursor
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "quarter-second grid positions, yields, pauses and cap points are exact binary fractions; == is the correct assertion and a tolerance would admit an off-boundary snap"
    )]
    #[test]
    fn given_a_sentence_boundary_falling_inside_a_paragraph_when_computing_the_resume_point_then_the_paragraph_boundary_takes_precedence(
        (yield_position, paused_duration, boundaries) in
            (position_strategy(), paused_duration_strategy(), mixed_boundaries_strategy())
    ) {
        // FR-024a: "if a sentence boundary falls within a paragraph/heading,
        // the paragraph/heading boundary takes precedence". The resume point
        // computed over the full boundary list (sentences included) must
        // equal the resume point over the paragraph/heading list alone — a
        // sentence boundary inside a paragraph never wins.
        let with_sentences = resume_point(yield_position, paused_duration, &boundaries);
        let eligible_only: Vec<ResumeBoundary> = boundaries
            .iter()
            .copied()
            .filter(|boundary| {
                matches!(
                    boundary.kind,
                    BoundaryKind::Paragraph | BoundaryKind::Heading
                )
            })
            .collect();
        let without_sentences =
            resume_point(yield_position, paused_duration, &eligible_only);
        prop_assert_eq!(
            with_sentences,
            without_sentences,
            "a sentence boundary inside a paragraph changed the resume point — the paragraph/heading boundary must take precedence (FR-024a)"
        );
        // Corollary: no sentence boundary strictly inside a paragraph/heading
        // is ever the resume point — the resume may coincide with such a
        // boundary's position only through the cap point, where the cap,
        // not the sentence, is what won.
        let eligible = eligible_boundaries(&boundaries);
        let cursor = rewound_cursor(yield_position, paused_duration);
        for boundary in &boundaries {
            if boundary.kind == BoundaryKind::Sentence
                && is_strictly_inside_paragraph(boundary.at_secs, &eligible)
                && boundary.at_secs <= cursor
            {
                prop_assert!(
                    with_sentences != boundary.at_secs
                        || with_sentences == cap_point(yield_position),
                    "the resume landed on sentence boundary {} inside a paragraph without the cap accounting for it — the enclosing paragraph/heading boundary must win (FR-024a precedence)",
                    boundary.at_secs
                );
            }
        }
        // And the resume point is always an eligible boundary or the cap point.
        let on_an_eligible_boundary = eligible.contains(&with_sentences);
        let at_the_cap_point = with_sentences == cap_point(yield_position);
        prop_assert!(
            on_an_eligible_boundary || at_the_cap_point,
            "the resume {} is neither a paragraph/heading boundary nor the cap point",
            with_sentences
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "quarter-second grid positions, yields, pauses and cap points are exact binary fractions; == is the correct assertion and a tolerance would admit an off-boundary snap"
    )]
    #[test]
    fn given_an_empty_boundary_list_when_snapping_then_the_resume_lands_at_the_cap_point(
        (yield_position, paused_duration) in (position_strategy(), paused_duration_strategy())
    ) {
        // Boundary case: an empty boundary list has no boundary at or before
        // the cursor, so the snap lands at the cap point — the floor 0.0
        // for any yield within the first 30 s of the document.
        let resumed = resume_point(yield_position, paused_duration, &[]);
        prop_assert_eq!(
            resumed,
            cap_point(yield_position),
            "an empty boundary list must snap to the cap point measured from the yield, not to a mid-word position"
        );
        if yield_position <= REWIND_CAP_SECS {
            prop_assert_eq!(
                resumed,
                0.0,
                "an empty boundary list must snap to 0 within the first 30 s of the document"
            );
        }
        prop_assert!(
            yield_position - resumed <= REWIND_CAP_SECS,
            "an empty boundary list snapped further back than the 30 s cap"
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "quarter-second grid positions, yields, pauses and cap points are exact binary fractions; == is the correct assertion and a tolerance would admit an off-boundary snap"
    )]
    #[test]
    fn given_a_single_boundary_exactly_at_the_cursor_when_snapping_then_the_boundary_is_returned(
        (yield_position, paused_duration) in
            (position_strategy(), paused_duration_strategy())
    ) {
        // Boundary case: a boundary exactly at the cursor is the greatest
        // boundary at or before the cursor (`boundary <= cursor` is
        // inclusive), and it is never earlier than the cap point because
        // the cursor itself is never earlier than the cap point.
        let mut clock = PositionClock::new(yield_position);
        let cursor = clock.rewind_for_silence(paused_duration);
        let boundaries = [ResumeBoundary {
            at_secs: cursor,
            kind: BoundaryKind::Heading,
        }];
        let snapped = clock.snap_to_boundary(&eligible_boundaries(&boundaries));
        prop_assert_eq!(
            snapped,
            cursor,
            "a boundary exactly at the cursor {} must be the snap result",
            cursor
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "quarter-second grid positions, yields, pauses and cap points are exact binary fractions; == is the correct assertion and a tolerance would admit an off-boundary snap"
    )]
    #[test]
    fn given_a_paragraph_longer_than_the_rewind_cap_when_snapping_then_the_resume_lands_at_the_cap_point(
        (yield_position, paused_duration) in (position_strategy(), paused_duration_strategy())
    ) {
        // Boundary case: the only eligible boundary is the document start —
        // a paragraph longer than the 30 s cap. The snap must land at the
        // cap point, never at the paragraph start: the cap takes precedence
        // and the paragraph/heading-boundary snap must not push the resume
        // further back than the cap (FR-015, edge cases spec.md:283-290).
        let boundaries = [ResumeBoundary {
            at_secs: 0.0,
            kind: BoundaryKind::Paragraph,
        }];
        let resumed = resume_point(yield_position, paused_duration, &boundaries);
        prop_assert_eq!(
            resumed,
            cap_point(yield_position),
            "the snap landed on a paragraph start older than the cap point — the cap must take precedence"
        );
        prop_assert!(
            yield_position - resumed <= REWIND_CAP_SECS,
            "total backward movement exceeded the 30 s cap"
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "quarter-second grid positions, yields, pauses and cap points are exact binary fractions; == is the correct assertion and a tolerance would admit an off-boundary snap"
    )]
    #[test]
    fn given_a_paused_duration_when_rewinding_then_the_cursor_is_five_percent_capped_at_thirty_seconds_and_the_yield_is_preserved(
        (yield_position, paused_duration) in
            (position_strategy(), paused_duration_strategy())
    ) {
        // The 5% rewind the snap operates on: exactly
        // `min(REWIND_FRACTION × d, REWIND_CAP_SECS)`, clamped at the
        // floor (FR-015) — and the yield stays as the clock's stored
        // basis, which is the snap's cap origin (the FR-004 kill-replay
        // basis and the FR-015 rewind origin are the same last-audible
        // position).
        let mut clock = PositionClock::new(yield_position);
        let cursor = clock.rewind_for_silence(paused_duration);
        prop_assert_eq!(
            cursor,
            rewound_cursor(yield_position, paused_duration),
            "the rewind of a {}s pause from {} did not land at the 5%-of-pause cursor capped at 30 s",
            paused_duration, yield_position
        );
        prop_assert_eq!(
            clock.stored_basis(),
            yield_position,
            "the rewind moved the stored basis — the yield must stay the cap origin for the snap (FR-015)"
        );
        prop_assert!(
            yield_position - cursor <= REWIND_CAP_SECS,
            "the rewind alone exceeded the 30 s cap"
        );
    }

    #[test]
    fn given_delivered_segments_and_an_adapter_computed_boundary_when_deriving_session_position_then_only_the_last_delivered_source_range_is_the_source_of_truth(
        (segments, adapter_computed) in (segment_sequence_strategy(), source_range_strategy())
    ) {
        // CHK030 single authority (constitution.md:82-86): the session
        // position is the echo of the last delivered DocumentSegment's
        // source_range_ref. The adapter-computed "nearest completed
        // boundary" is NOT a source of truth — only the delivered
        // segment's source_range_ref is.
        let mut log = DeliveryLog::default();
        for segment in &segments {
            let chunk = AudioChunk {
                pcm_data: Vec::new(),
                rate_hz: DeclaredFormat::SAMPLE_RATE,
                channels: DeclaredFormat::CHANNELS,
                duration_ms: 0.0,
                pre_roll_remaining_ms: None,
                source_range_ref: segment.source_range,
            };
            log.deliver(segment.clone(), chunk);
        }
        let echoed = log.session_position();
        let last_delivered = segments.last().map(|segment| segment.source_range);
        prop_assert_eq!(
            echoed,
            last_delivered,
            "session_position must be the echo of the last delivered DocumentSegment.source_range_ref (CHK030)"
        );
        // The adapter computation is not a source of truth: whenever it
        // diverges from the echo, the echo is what stands.
        let adapter = MockAdapter {
            computed_nearest_boundary: adapter_computed,
        };
        if Some(adapter.computed_nearest_boundary) != last_delivered {
            prop_assert_ne!(
                echoed,
                Some(adapter.computed_nearest_boundary),
                "session_position carried the adapter-computed nearest completed boundary — an adapter computation is never a source of truth (CHK030)"
            );
        }
        // The failure view carries the echo across the adapter→player
        // boundary, not the adapter computation.
        if let Some(echoed_position) = echoed {
            let failure = EngineFailureView {
                variant: EngineError::Unreachable,
                condition: "engine_unreachable: stand-in failure".to_string(),
                session_position: echoed_position,
            };
            prop_assert_eq!(
                failure.session_position,
                echoed_position,
                "the failure's session_position must be the echoed source_range_ref, not an adapter computation"
            );
            if adapter.computed_nearest_boundary != echoed_position {
                prop_assert_ne!(
                    failure.session_position,
                    adapter.computed_nearest_boundary,
                    "the failure's session_position must not be the adapter-computed boundary (CHK030)"
                );
            }
        }
    }

    #[test]
    fn given_a_delivered_chunk_when_the_adapter_mutates_its_copy_after_delivery_then_the_delivered_fields_do_not_change(
        ((segment, chunk), adapter_computed) in
            (delivery_strategy(), source_range_strategy())
    ) {
        // adapter-handoff.md:30: "Immutable after delivery: adapter never
        // modifies chunk fields after delivery." The mock adapter keeps a
        // mutable copy and violates the contract post-delivery; the player's
        // snapshot — the echo basis — must be unaffected.
        let delivered = chunk.clone();
        let mut log = DeliveryLog::default();
        log.deliver(segment.clone(), chunk.clone());
        let mut adapter_copy = chunk;
        violate_immutability(&mut adapter_copy);
        let adapter = MockAdapter {
            computed_nearest_boundary: adapter_computed,
        };
        prop_assert_eq!(
            log.last_chunk(),
            Some(&delivered),
            "the delivered AudioChunk fields changed after delivery — a delivered chunk is immutable (adapter-handoff.md:30)"
        );
        prop_assert_eq!(
            log.session_position(),
            Some(segment.source_range),
            "the session position echo changed when the adapter mutated its retained copy"
        );
        prop_assert_ne!(
            adapter_copy,
            delivered,
            "the mock adapter's post-delivery mutation did not change its retained copy — the immutability violation did not fire"
        );
        prop_assert_ne!(
            adapter.computed_nearest_boundary,
            segment.source_range,
            "the adapter-computed boundary coincided with the delivered segment's source_range — generate a diverging value to exercise the divergence path"
        );
    }

    #[test]
    fn given_a_delivered_chunk_when_checked_against_the_declared_handoff_shape_then_it_conforms(
        (_segment, chunk) in delivery_strategy()
    ) {
        // FR-024 / FR-033 / FR-035: the delivered chunk is mono f32 at the
        // declared 22050 Hz — any deviation is a named failure, never a
        // silent resample or downmix. The fixture carries the declared
        // shape, and the real handoff check accepts it.
        prop_assert_eq!(
            chunk.rate_hz,
            DeclaredFormat::SAMPLE_RATE,
            "a delivered chunk must carry the declared 22050 Hz rate"
        );
        prop_assert_eq!(
            chunk.channels,
            DeclaredFormat::CHANNELS,
            "a delivered chunk must carry the declared mono channel count"
        );
        prop_assert!(
            check_handoff(
                chunk.rate_hz,
                chunk.channels,
                DeclaredFormat::SAMPLE_WIDTH_BITS
            )
            .is_ok(),
            "a delivered chunk failed the declared handoff check — the player must refuse it with a named error, never play it"
        );
    }
}

/// Boundary case: an empty boundary list with the cursor inside the first
/// 30 seconds of the document snaps to exactly 0.0 — there is no boundary
/// to land on, and the cap point is the floor.
#[expect(
    clippy::float_cmp,
    reason = "the boundary values 12.0 and 0.0 are exact binary fractions on the quarter-second grid; == is the correct assertion and a tolerance would admit an off-boundary snap"
)]
#[test]
fn given_no_boundaries_and_a_cursor_within_the_first_thirty_seconds_when_snapping_then_the_resume_is_zero()
 {
    let clock = PositionClock::new(12.0);
    let snapped = clock.snap_to_boundary(&[]);
    assert_eq!(
        snapped, 0.0,
        "an empty boundary list must snap to the cap point, which is 0.0 within the first 30 s"
    );
}

/// Boundary case: a single boundary exactly at the cursor is returned
/// as-is — the greatest-boundary-at-or-before lookup is inclusive.
#[expect(
    clippy::float_cmp,
    reason = "the boundary values 25.0 and 0.0 are exact binary fractions on the quarter-second grid; == is the correct assertion and a tolerance would admit an off-boundary snap"
)]
#[test]
fn given_a_single_boundary_at_the_cursor_when_snapping_then_the_boundary_is_returned() {
    let clock = PositionClock::new(25.0);
    let boundaries = [25.0];
    let snapped = clock.snap_to_boundary(&boundaries);
    assert_eq!(
        snapped, 25.0,
        "a boundary exactly at the cursor is the greatest boundary at or before it"
    );
}

/// Boundary case: the 5% rewind saturates at the 30-second cap, and the
/// snap of a paragraph longer than the cap lands at the cap point — total
/// backward movement is exactly 30 s, never more (FR-015).
#[expect(
    clippy::float_cmp,
    reason = "the boundary values 3600.0, 3570.0, 30.0 and 0.0 are exact binary fractions and 0.05 × 7200.0 caps at exactly 30.0, so == is the correct assertion; a tolerance would admit an off-boundary rewind or a cap violation"
)]
#[test]
fn given_a_rewind_saturating_at_the_cap_when_snapping_then_total_backward_is_exactly_thirty_seconds()
 {
    let yield_position = 3600.0;
    let mut clock = PositionClock::new(yield_position);
    // 5% of 7200 s = 360 s > 30 s → the rewind saturates at the cap.
    let cursor = clock.rewind_for_silence(7200.0);
    assert_eq!(
        cursor,
        yield_position - 30.0,
        "the rewind of a 7200 s pause saturates at the 30 s cap"
    );
    // A single paragraph spanning the whole document: its start (0.0) is
    // older than the cap point, so the snap lands at the cap point.
    let boundaries = [0.0];
    let resumed = clock.snap_to_boundary(&boundaries);
    assert_eq!(
        resumed,
        yield_position - 30.0,
        "the snap lands at the cap point when the containing paragraph is longer than the cap"
    );
    assert!(
        yield_position - resumed <= REWIND_CAP_SECS,
        "total backward movement exceeded the 30 s cap"
    );
}

/// Boundary case: no delivered segments means no session position — the
/// echo of an empty delivery history is empty (CHK030).
#[test]
fn given_no_delivered_segments_when_deriving_session_position_then_no_session_position_exists() {
    let log = DeliveryLog::default();
    assert_eq!(
        log.session_position(),
        None,
        "an empty delivery history has no session position to echo"
    );
}

/// Boundary case: a single delivered segment's `source_range_ref` is
/// echoed verbatim as the session position, and the failure view carries
/// that echo (CHK030).
#[test]
fn given_one_delivered_segment_when_a_failure_occurs_then_the_failure_echoes_its_source_range_ref()
{
    let segment = DocumentSegment {
        blake3_hash: format!("{:032x}", 1),
        source_range: SourceRange {
            start: 42,
            end: 142,
        },
        boundary_kind: BoundaryKind::Paragraph,
        byte_length: 100,
        segment_index: 0,
    };
    let chunk = AudioChunk {
        pcm_data: Vec::new(),
        rate_hz: DeclaredFormat::SAMPLE_RATE,
        channels: DeclaredFormat::CHANNELS,
        duration_ms: 0.0,
        pre_roll_remaining_ms: None,
        source_range_ref: segment.source_range,
    };
    let mut log = DeliveryLog::default();
    log.deliver(segment.clone(), chunk);
    assert_eq!(
        log.session_position(),
        Some(segment.source_range),
        "the session position is the echo of the single delivered segment's source_range_ref"
    );
    if let Some(echoed_position) = log.session_position() {
        let failure = EngineFailureView {
            variant: EngineError::DeviceLost,
            condition: "device_lost: output device removed".to_string(),
            session_position: echoed_position,
        };
        assert_eq!(
            failure.session_position, segment.source_range,
            "the failure's session_position must be the echoed source_range_ref, never an adapter computation (CHK030)"
        );
    }
}

/// Boundary case: an adapter-computed "nearest completed boundary" that
/// diverges from the echo must never become the session position — the
/// echo of the last delivered `source_range_ref` is the only source of
/// truth (CHK030, constitution.md:82-86).
#[test]
fn given_an_adapter_computed_boundary_diverging_from_the_echo_when_a_failure_occurs_then_the_failure_carries_the_echo()
 {
    let segment = DocumentSegment {
        blake3_hash: format!("{:032x}", 7),
        source_range: SourceRange {
            start: 100,
            end: 200,
        },
        boundary_kind: BoundaryKind::Paragraph,
        byte_length: 100,
        segment_index: 0,
    };
    // The adapter's own computation: an earlier boundary it considers
    // "nearest completed" — not a source of truth (CHK030).
    let adapter_computed = SourceRange { start: 0, end: 100 };
    let chunk = AudioChunk {
        pcm_data: Vec::new(),
        rate_hz: DeclaredFormat::SAMPLE_RATE,
        channels: DeclaredFormat::CHANNELS,
        duration_ms: 0.0,
        pre_roll_remaining_ms: None,
        source_range_ref: segment.source_range,
    };
    let mut log = DeliveryLog::default();
    log.deliver(segment.clone(), chunk);
    let echoed = log.session_position();
    assert_eq!(
        echoed,
        Some(segment.source_range),
        "the session position is the echo of the last delivered source_range_ref"
    );
    assert_ne!(
        echoed,
        Some(adapter_computed),
        "the session position must not be the adapter-computed nearest completed boundary (CHK030)"
    );
    if let Some(echoed_position) = echoed {
        let failure = EngineFailureView {
            variant: EngineError::Unreachable,
            condition: "engine_unreachable: stand-in failure".to_string(),
            session_position: echoed_position,
        };
        assert_ne!(
            failure.session_position, adapter_computed,
            "the failure's session_position must not be the adapter-computed boundary (CHK030)"
        );
    }
}
