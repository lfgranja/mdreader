//! T213 — Pre-roll cancellation on manual seek or skip (FR-014, tasks.md T213).
//!
//! The pre-synthesized buffer for chapter N+1 (10 s trigger) is cooperatively
//! cancelled and purged when the listener seeks or skips.

use player::skip::{NEXT_CHAPTER_PREROLL_TRIGGER_SECS, NextChapterPreRoll};

#[test]
fn trigger_fires_at_ten_seconds_remaining() {
    let pre = NextChapterPreRoll::new();
    assert!((NEXT_CHAPTER_PREROLL_TRIGGER_SECS - 10.0).abs() < f64::EPSILON);
    assert!(!pre.should_trigger(10.5));
    assert!(pre.should_trigger(10.0));
    assert!(pre.should_trigger(4.0));
}

#[test]
fn cancel_flags_worker_token_and_purges_buffer() {
    let mut pre = NextChapterPreRoll::new();
    let token = pre.begin(2);
    pre.buffer.extend([0.1_f32; 128]);
    assert_eq!(pre.pending_chapter(), Some(2));
    assert!(!token.is_cancelled());

    pre.cancel_and_purge();

    assert!(token.is_cancelled(), "worker must observe cancellation");
    assert!(pre.buffer.is_empty(), "pre-synthesized PCM must be purged");
    assert_eq!(pre.pending_chapter(), None);
}

#[test]
fn restart_after_cancel_uses_a_fresh_uncancelled_token() {
    let mut pre = NextChapterPreRoll::new();
    let old = pre.begin(1);
    pre.cancel_and_purge();
    let fresh = pre.begin(2);
    assert!(old.is_cancelled());
    assert!(!fresh.is_cancelled());
}
