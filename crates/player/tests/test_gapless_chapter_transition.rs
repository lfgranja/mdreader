//! T197 (TDD RED): automatic gapless chapter N -> N+1 transition.

use player::position::PositionClock;
use player::queue::{ChapterItem, ChapterQueue, ChapterStatus};

fn item(name: &str) -> ChapterItem {
    ChapterItem {
        filename: name.to_string(),
        file_uri: name.to_string(),
        relative_path: name.to_string(),
        byte_size: 1,
        content_fingerprint: [0u8; 32],
        playback_status: ChapterStatus::Pending,
    }
}

#[test]
fn transition_has_gap_under_one_second_and_resets_position() -> Result<(), String> {
    let mut queue = ChapterQueue::new(vec![item("01-a.md"), item("02-b.md")]);
    let mut clock = PositionClock::new(0.0);
    let _advanced = clock.on_audible_progress(42.0).map_err(|e| e.to_string())?;
    let t = queue.transition_gapless().map_err(|e| e.to_string())?;
    assert!(t.gap_secs < 1.0, "gap {} must be < 1.0s", t.gap_secs);
    assert!((t.new_position_secs - 0.0).abs() < f64::EPSILON);
    assert_eq!(queue.current_index(), 1);
    clock = PositionClock::new(t.new_position_secs);
    assert!(clock.position_secs().abs() < f64::EPSILON);
    Ok(())
}
