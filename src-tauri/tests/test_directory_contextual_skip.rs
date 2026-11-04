//! T211 — Contract test (TDD RED): contextual skip logic for directories.
//!
//! Assertions (spec.md FR-012, tasks.md T211):
//! 1. `ACTION_SKIP_TO_NEXT` on the last section jumps to chapter N+1 at 0.0s.
//! 2. `ACTION_SKIP_TO_PREVIOUS` within the first 3.0 seconds jumps to chapter N-1 at 0.0s.
//! 3. `ACTION_SKIP_TO_PREVIOUS` after the first 3.0 seconds restarts current chapter at 0.0s.

use player::queue::{ChapterItem, ChapterQueue, ChapterStatus};

fn sample_queue() -> ChapterQueue {
    let items = vec![
        ChapterItem {
            filename: String::from("01-intro.md"),
            file_uri: String::from("content://vault/01-intro.md"),
            relative_path: String::from("01-intro.md"),
            byte_size: 512,
            content_fingerprint: [0x11; 32],
            playback_status: ChapterStatus::Completed,
        },
        ChapterItem {
            filename: String::from("02-arquitetura.md"),
            file_uri: String::from("content://vault/02-arquitetura.md"),
            relative_path: String::from("02-arquitetura.md"),
            byte_size: 1024,
            content_fingerprint: [0x22; 32],
            playback_status: ChapterStatus::Playing,
        },
        ChapterItem {
            filename: String::from("03-estado.md"),
            file_uri: String::from("content://vault/03-estado.md"),
            relative_path: String::from("03-estado.md"),
            byte_size: 2048,
            content_fingerprint: [0x33; 32],
            playback_status: ChapterStatus::Pending,
        },
    ];
    let mut q = ChapterQueue::new(items);
    let _ = q.set_current_index(1);
    q
}

#[test]
fn test_skip_to_next_on_last_section_advances_chapter() -> Result<(), Box<dyn std::error::Error>> {
    let mut queue = sample_queue();
    assert_eq!(queue.current_index(), 1);

    // Contextual skip: at end of chapter (or no further sections), skip next advances to chapter 2
    let advanced = player::skip::contextual_skip_forward(&mut queue, true)?;
    assert!(advanced);
    assert_eq!(queue.current_index(), 2);
    assert_eq!(
        queue.current().map(|c| c.filename.as_str()),
        Some("03-estado.md")
    );
    Ok(())
}

#[test]
fn test_skip_to_previous_within_3s_jumps_to_previous_chapter()
-> Result<(), Box<dyn std::error::Error>> {
    let mut queue = sample_queue();
    assert_eq!(queue.current_index(), 1);

    // Within first 3 seconds (position = 1.5s), skip back jumps to chapter N-1 (chapter 0)
    let jumped_prev = player::skip::contextual_skip_backward(&mut queue, 1.5)?;
    assert!(jumped_prev);
    assert_eq!(queue.current_index(), 0);
    assert_eq!(
        queue.current().map(|c| c.filename.as_str()),
        Some("01-intro.md")
    );
    Ok(())
}

#[test]
fn test_skip_to_previous_after_3s_restarts_current_chapter()
-> Result<(), Box<dyn std::error::Error>> {
    let mut queue = sample_queue();
    assert_eq!(queue.current_index(), 1);

    // After first 3 seconds (position = 12.0s), skip back restarts current chapter (chapter index unchanged)
    let jumped_prev = player::skip::contextual_skip_backward(&mut queue, 12.0)?;
    assert!(!jumped_prev);
    assert_eq!(queue.current_index(), 1);
    assert_eq!(
        queue.current().map(|c| c.filename.as_str()),
        Some("02-arquitetura.md")
    );
    Ok(())
}
