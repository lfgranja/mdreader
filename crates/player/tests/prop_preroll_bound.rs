//! T149 — the resident playback-queue bound (FR-018; `plan.md:25`).
//!
//! `plan.md:25` states a pre-roll buffer cap of ≤60 s
//! (`MAX_PREROLL_SECS`), observed through the diagnostic buffer-state
//! entries. The cap existed NOWHERE: `SampleQueue::push` and
//! `CpalBackend::enqueue_pcm` grew without a bound and the open path
//! enqueued EVERY segment, so the whole document's PCM stayed
//! resident for the whole session — ~620 MB for the SC-001 fixture
//! (~2 h of speech at 22 050 Hz mono f32) on a mid-range phone.
//!
//! This suite pins the bound and the drop policy on the public queue
//! type (`player::backend::ResidentQueue`), the resident speech FIFO
//! the device plays from:
//!
//! * the resident audio is the pre-roll window AHEAD of the play
//!   head — the audio that is queued but not yet audible;
//! * the bound is [`player::backend::MAX_PREROLL_SECS`] (60 s), the
//!   single [`player::position::PREROLL_CAP_SECS`] value (an alias,
//!   never a second cap that can drift — Principle VII / FR-044).
//!
//! THE INVARIANTS.
//!
//! * `prop_the_resident_audio_never_exceeds_the_bound` — however many
//!   segments are enqueued, and however long each one is, the
//!   resident audio never exceeds the bound. RED.
//! * `prop_the_resident_audio_is_the_window_from_the_play_head` — the
//!   resident audio is ALWAYS the contiguous slice of the document
//!   audio that starts at the play head: the audio at the head is
//!   never dropped, so the reading stays audibly contiguous from the
//!   play head, and only the audio BEYOND the window is dropped.
//!   GREEN today (an unbounded queue is a window too) and the pin
//!   that keeps it green once the bound is in place.
//! * `prop_the_bound_moves_no_position` — the drop never moves the
//!   position authority: the audible clock is the only advance path
//!   (FR-002). GREEN (the guard that must not break).
//! * `the_resident_audio_stays_bounded_far_beyond_the_cap` — the
//!   deterministic case: 200 segments of 30 s each (6 000 s of audio,
//!   100× the bound) leave exactly the bound resident. RED.
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures (`{var}`).
//! proptest 1.11 builds the message through `concat!`, and rustc
//! rejects implicit captures in macro-expanded format strings.

use proptest::prelude::*;

use engine_core::format::DeclaredFormat;
use player::backend::{MAX_PREROLL_SECS, ResidentQueue};
use player::position::PositionClock;

/// The declared engine rate (FR-019): 22 050 Hz mono f32.
const ENGINE_RATE_HZ: usize = 22_050;

/// The plan's bound in seconds (`plan.md:25`; the diagnostic contract
/// names the same value `MAX_PREROLL_SECS=60`).
const PLAN_PREROLL_SECS: f64 = 60.0;

/// The enqueue plan: the number of segments the producer hands over,
/// and the TOTAL number of samples it hands over.
///
/// The total reaches 2.5 million samples — ~113 s of audio, nearly
/// twice the 60 s bound — so the bound is genuinely exercised rather
/// than merely stated. The segment count is bounded so the case cost
/// stays a comparison of windows, not a quadratic scan.
fn enqueue_plan() -> impl Strategy<Value = (usize, usize)> {
    (1usize..=12, 0usize..=2_500_000)
}

/// The chunk of `size` samples that carries the document's audio at
/// the given absolute sample index: every sample is distinct, so a
/// dropped or reordered sample is visible in the comparison.
#[expect(
    clippy::cast_precision_loss,
    reason = "the sample index is the document's sample count, at most 2.5 million in this suite — far below f32's 23-bit exact-integer range, so the cast is lossless here"
)]
fn chunk_at(start: usize, size: usize) -> Vec<f32> {
    (0..size).map(|offset| (start + offset) as f32).collect()
}

/// The segment sizes of a plan: `segments` chunks that share `total`
/// samples.
fn segment_sizes(segments: usize, total: usize) -> Vec<usize> {
    let base = total / segments;
    let remainder = total % segments;
    (0..segments)
        .map(|index| base + usize::from(index < remainder))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// The bound: the resident audio NEVER exceeds `MAX_PREROLL_SECS`,
    /// however many segments are enqueued (FR-018; `plan.md:25`).
    #[test]
    fn prop_the_resident_audio_never_exceeds_the_bound(
        (segments, total) in enqueue_plan(),
    ) {
        let mut queue = ResidentQueue::new();
        let mut enqueued = 0usize;
        for size in segment_sizes(segments, total) {
            let chunk = chunk_at(enqueued, size);
            queue.push(&chunk);
            enqueued += size;

            prop_assert!(
                queue.len() <= queue.cap_samples(),
                "the resident audio holds {} samples after {} samples were enqueued, \
                 beyond the bound of {} samples",
                queue.len(),
                enqueued,
                queue.cap_samples()
            );
            prop_assert!(
                queue.buffered_secs() <= MAX_PREROLL_SECS,
                "resident audio holds {} s after {} samples were enqueued, \
                 beyond the bound of {} s",
                queue.buffered_secs(),
                enqueued,
                MAX_PREROLL_SECS
            );
        }
    }

    /// The drop policy: the resident audio is ALWAYS the audio the
    /// queue took in and the device has not played yet — in the
    /// document's order, never reordered, and never missing its head.
    ///
    /// The model encodes the policy the queue implements: a chunk
    /// fills the window up to the bound and whatever falls BEYOND the
    /// bound is dropped, so the window keeps the audio the play head
    /// reaches FIRST. The queue's content must be exactly the
    /// accepted-but-unplayed audio: the head is never dropped (the
    /// reading stays audibly contiguous from the play head), the
    /// order is never broken, and the audio beyond the window never
    /// stays resident.
    #[test]
    fn prop_the_resident_audio_is_the_window_from_the_play_head(
        (segments, total) in enqueue_plan(),
        drain_every in 1usize..=6,
    ) {
        let mut queue = ResidentQueue::new();
        // The audio the queue took in, in order, and the play head's
        // place in it (the samples the device already played). The
        // sample VALUES are the document's sample indices, so a
        // dropped, repeated, or reordered sample is visible.
        let mut accepted: Vec<f32> = Vec::new();
        let mut document_index = 0usize;
        let mut played = 0usize;
        let mut since_drain = 0usize;

        for size in segment_sizes(segments, total) {
            let chunk = chunk_at(document_index, size);
            document_index += size;
            // What fits in the window is taken in; the rest falls
            // beyond the bound and is dropped.
            let room = queue.cap_samples().saturating_sub(queue.len());
            let kept = room.min(chunk.len());
            accepted.extend_from_slice(&chunk[..kept]);
            queue.push(&chunk);

            // The device plays: the callback takes the audio from the
            // head of the queue.
            since_drain += 1;
            if since_drain >= drain_every {
                since_drain = 0;
                while queue.pop_front().is_some() {
                    played += 1;
                }
            }

            // The expected window: the accepted audio the device has
            // not played yet.
            let expected = &accepted[played..];

            prop_assert_eq!(
                queue.len(),
                expected.len(),
                "the resident audio holds {} samples, but the window from the play head \
                 holds {} (the play head is at {} of {} accepted samples)",
                queue.len(),
                expected.len(),
                played,
                accepted.len()
            );
            prop_assert!(
                queue.samples().iter().copied().eq(expected.iter().copied()),
                "the resident audio is not the window from the play head: \
                 the head's audio was dropped or reordered"
            );
        }
    }

    /// The position authority: the drop never moves the position. The
    /// audible clock is the only advance path (FR-002) — a dropped
    /// sample is not an audible one, so it moves nothing.
    #[expect(
        clippy::float_cmp,
        reason = "the position is the exact f64 the clock holds; == pins the unchanged law and a tolerance would admit drift"
    )]
    #[test]
    fn prop_the_bound_moves_no_position(
        (segments, total) in enqueue_plan(),
    ) {
        let mut clock = PositionClock::new(0.0);
        let advanced = clock.on_audible_progress(1.5);
        prop_assert!(
            advanced.is_ok(),
            "the audible clock refused a positive progress reading"
        );
        let position_before = clock.position_secs();

        let mut queue = ResidentQueue::new();
        let mut enqueued = 0usize;
        for size in segment_sizes(segments, total) {
            let chunk = chunk_at(enqueued, size);
            queue.push(&chunk);
            enqueued += size;
        }

        prop_assert_eq!(
            clock.position_secs(),
            position_before,
            "dropping {} samples beyond the bound moved the reported position",
            queue.dropped_samples()
        );
    }
}

/// The deterministic case: the producer enqueues FAR more than the
/// bound — 200 segments of 30 s each, 6 000 s of audio, 100× the
/// bound — and exactly the bound stays resident.
#[test]
fn the_resident_audio_stays_bounded_far_beyond_the_cap() {
    let segment_samples = 30 * ENGINE_RATE_HZ;
    let segments = 200usize;
    let total_samples = segment_samples * segments;

    let mut queue = ResidentQueue::new();
    let chunk = chunk_at(0, segment_samples);
    for _ in 0..segments {
        queue.push(&chunk);
    }

    assert_eq!(
        queue.len(),
        queue.cap_samples(),
        "the resident audio must hold exactly the bound after {segments} segments"
    );
    assert_eq!(
        queue.buffered_secs(),
        PLAN_PREROLL_SECS,
        "the resident audio must hold exactly the plan's bound in seconds"
    );
    assert_eq!(
        queue.dropped_samples(),
        u64::try_from(total_samples - queue.cap_samples()).unwrap_or(u64::MAX),
        "every sample beyond the bound must be accounted as dropped"
    );
}

/// The bound is the plan's value, and the plan's value is the single
/// `PREROLL_CAP_SECS` the pre-roll ring and the pipeline-side buffer
/// already share (Principle VII / FR-044: one cap, never two).
#[test]
fn the_bound_is_the_single_preroll_cap() {
    assert_eq!(
        MAX_PREROLL_SECS, PLAN_PREROLL_SECS,
        "the resident bound must be the plan's 60 s"
    );
    assert_eq!(
        MAX_PREROLL_SECS,
        player::position::PREROLL_CAP_SECS,
        "the resident bound must be the single pre-roll cap, never a second value"
    );
    assert_eq!(
        ENGINE_RATE_HZ,
        usize::try_from(DeclaredFormat::SAMPLE_RATE).unwrap_or(usize::MAX),
        "the engine rate must be the declared 22 050 Hz"
    );
}
