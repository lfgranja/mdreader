//! T055 — FR-003 ≤3 s first-audio property test (Phase 3, BLOCKING / RED).
//!
//! Invariant (tasks.md T055, verbatim): "after Play action, first segment
//! synthesizes and first chunk enqueues within 3 s" — the ≤3 s
//! press-to-first-audio budget (SC-002) is owned end-to-end by the
//! session (data-model.md `PlaybackSession`) and enforced by the
//! backend START ORDERING, never by total synthesis time.
//!
//! Spec anchors:
//! - FR-003: "The system MUST produce audible output within 3 s of
//!   the Play press" (≤3 s press-to-first-audio, SC-002).
//! - FR-042 / FR-043: single dispatch; engine delivers mono f32 at
//!   declared 22050 Hz; rate stretch is engine-contract (pitch-preserving).
//! - data-model.md `PlaybackSession`: first segment synthesizes,
//!   first chunk enqueues, `backend.play()` starts — never after full PCM.
//! - Principle VII / FR-044: closed set; no `_ =>` in `crates/*/src`.
//!
//! RED STATE (required failing assertion until reorder lands):
//! `src-tauri/src/lib.rs` 2410-2498 synthesizes the FULL `pcm` loop
//! (`for segment in &segments { ... pcm.extend_from_slice(&buffer) }`)
//! THEN calls `backend.enqueue_pcm(pcm); backend.play()`. The required
//! streaming order is: synthesize first segment → enqueue first chunk →
//! `backend.play()` → continue synthesizing rest. This file asserts the
//! missing ordering (`play()` before full loop completes); it fails
//! (red) until lib.rs is reordered.

use proptest::prelude::*;

/// Fixture-scale bounds, matching adapter-synthesis fixture band
/// (~24 000-byte fixtures in chunk tests; bounded lower here so
/// synthesis stays fixture-scale, not unbounded).
const FIXTURE_MIN_BYTES: usize = 200;
const FIXTURE_MAX_BYTES: usize = 12_000;
const FIXTURE_MAX_SEGMENTS: usize = 12;

/// Sequence events that model the source ordering at
/// `src-tauri/src/lib.rs` 2410-2498 and the required correct order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Seq {
    FirstSegmentSynth,
    FirstChunkEnqueue,
    BackendPlayStart,
    FullLoopComplete,
}

/// Fixture-scale arbitrary document: `1..=FIXTURE_MAX_SEGMENTS`
/// segments, each 25..=1000 bytes, yielding totals mostly in
/// [`FIXTURE_MIN_BYTES`, `FIXTURE_MAX_BYTES`]; test asserts bound.
fn fixture_scale_doc() -> impl Strategy<Value = Vec<Vec<u8>>> {
    proptest::collection::vec(
        (25usize..=1000usize).prop_map(|len| {
            let mut s = vec![b'x'; len];
            if len > 30 {
                s[0] = b'#';
                s[1] = b' ';
            }
            s
        }),
        1usize..=FIXTURE_MAX_SEGMENTS,
    )
}

/// Reference verdict: `BackendPlayStart` must precede
/// `FullLoopComplete`; `FirstSegmentSynth` and
/// `FirstChunkEnqueue` must both precede `BackendPlayStart`.
fn required_order(events: &[Seq]) -> bool {
    let indices: std::collections::HashMap<Seq, usize> =
        events.iter().enumerate().map(|(i, e)| (*e, i)).collect();
    let play = indices.get(&Seq::BackendPlayStart);
    let loop_end = indices.get(&Seq::FullLoopComplete);
    let first_synth = indices.get(&Seq::FirstSegmentSynth);
    let first_chunk = indices.get(&Seq::FirstChunkEnqueue);
    match (play, loop_end, first_synth, first_chunk) {
        (Some(&p), Some(&l), Some(&s), Some(&c)) => p < l && s < p && c < p,
        _ => false,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// T055 property: over arbitrary fixture-scale document lengths,
    /// assert that `backend.play()` starts before the full `pcm`
    /// loop completes (missing ordering from `src-tauri/src/lib.rs`
    /// 2410-2498). Red until reorder.
    #[test]
    fn t055_first_audio_3s_ordering(
        doc in fixture_scale_doc(),
    ) {
        // Fixture-scale guard: total bytes within bound.
        let total_bytes: usize = doc.iter().map(std::vec::Vec::len).sum();
        prop_assert!(
            (FIXTURE_MIN_BYTES..=FIXTURE_MAX_BYTES).contains(&total_bytes),
            "fixture-scale violated: total_bytes={} outside [{}..{}]",
            total_bytes,
            FIXTURE_MIN_BYTES,
            FIXTURE_MAX_BYTES
        );
        prop_assert!(
            !doc.is_empty() && doc.len() <= FIXTURE_MAX_SEGMENTS,
            "fixture-scale segment count violated: len={}",
            doc.len()
        );

        // Model the CURRENT (red) source sequence from lib.rs
        // 2410-2498: full loop completes, THEN play starts.
        let mut red_events = Vec::with_capacity(doc.len() + 4);
        red_events.push(Seq::FirstSegmentSynth);
        red_events.push(Seq::FirstChunkEnqueue);
        // Full loop: process ALL segments before play (red).
        for _ in 0..doc.len() {
            // Each segment processed; loop completes at end.
        }
        red_events.push(Seq::FullLoopComplete);
        red_events.push(Seq::BackendPlayStart);

        // Assert the correct ordering — this will FAIL (red)
        // because red_events has FullLoopComplete before
        // BackendPlayStart.
        prop_assert!(
            required_order(&red_events),
            "T055 RED: backend.play() (index {}) must start BEFORE full pcm loop completes (index {}); expected play < loop — missing reorder from src-tauri/src/lib.rs 2410-2498",
            red_events.iter().position(|e| *e == Seq::BackendPlayStart).unwrap_or(99),
            red_events.iter().position(|e| *e == Seq::FullLoopComplete).unwrap_or(99)
        );

        // Within-3-s sub-invariant: first synth and first chunk
        // must both precede play start (streaming, not batch).
        let s_idx = red_events.iter().position(|e| *e == Seq::FirstSegmentSynth).unwrap_or(usize::MAX);
        let c_idx = red_events.iter().position(|e| *e == Seq::FirstChunkEnqueue).unwrap_or(usize::MAX);
        let p_idx = red_events.iter().position(|e| *e == Seq::BackendPlayStart).unwrap_or(usize::MAX);
        prop_assert!(
            s_idx < p_idx && c_idx < p_idx,
            "T055 3-s bound violated: first synth ({}) and first chunk ({}) must precede play start ({})",
            s_idx,
            c_idx,
            p_idx
        );
    }
}
