//! Player crate — transport, buffering, rewind/snap math.
//!
//! Single authority for playback position and transport state
//! (FR-002, FR-003). Every surface routes through the same
//! Rust code path (Principle II).

#![expect(
    clippy::multiple_crate_versions,
    reason = "every duplicate is third-party-forced at newest-release pins: ndk 0.9.0 (directly and via cpal 0.18.2) carries thiserror 1.0.69 and jni-sys 0.3.1 — whose published 0.3.1 release itself depends on jni-sys 0.4.1 — while cpal's jni 0.22.4 carries thiserror 2.0.21 and jni-sys 0.4.1; cargo cannot unify across semver majors or a crate with its own re-export, crates.io shows no newer versions, and supply-chain governance for these transitive duplicates belongs to cargo-deny, not this lint"
)]

pub mod backend;
pub mod buffer;
pub mod cache;
pub mod cli;
pub mod dsp;
pub mod earcon;
pub mod focus;
pub mod interruption;
pub mod position;
pub mod queue;
pub mod rate;
pub mod recovery;
pub mod resume;
pub mod rewind;
pub mod routing;
pub mod skip;
pub mod surface;
pub mod transport;

/// Advances the chapter queue gaplessly when the current document ends.
///
/// Pure queue step: the audio stream is never torn down here. The caller
/// resets its `PositionClock` to `ChapterTransition::new_position_secs`.
///
/// # Errors
///
/// Never fails today; the `Result` keeps room for closed-set queue errors.
pub fn advance_on_document_end(
    queue: &mut queue::ChapterQueue,
) -> Result<Option<queue::ChapterTransition>, queue::ChapterQueueError> {
    if queue.current_index().saturating_add(1) >= queue.len() {
        return Ok(None);
    }
    queue.transition_gapless().map(Some)
}

#[cfg(test)]
mod advance_tests {
    use super::advance_on_document_end;
    use crate::queue::{ChapterItem, ChapterQueue, ChapterStatus};

    fn item(n: &str) -> ChapterItem {
        ChapterItem {
            filename: n.to_string(),
            file_uri: n.to_string(),
            relative_path: n.to_string(),
            byte_size: 1,
            content_fingerprint: [0u8; 32],
            playback_status: ChapterStatus::Pending,
        }
    }

    #[test]
    fn advances_then_none_on_last() -> Result<(), String> {
        let mut q = ChapterQueue::new(vec![item("a"), item("b")]);
        let t = advance_on_document_end(&mut q).map_err(|e| e.to_string())?;
        assert!(t.is_some());
        assert_eq!(q.current_index(), 1);
        let t = advance_on_document_end(&mut q).map_err(|e| e.to_string())?;
        assert!(t.is_none());
        assert_eq!(q.current_index(), 1);
        let mut empty = ChapterQueue::new(vec![]);
        assert!(
            advance_on_document_end(&mut empty)
                .map_err(|e| e.to_string())?
                .is_none()
        );
        Ok(())
    }
}
