//! Chapter queue and playlist data structures for directory reading.
//!
//! Provides the closed-set [`ChapterStatus`] enum and the [`ChapterItem`]
//! struct representing files discovered during directory scanning.

use serde::{Deserialize, Serialize};

/// Lifecycle status of an individual chapter in the playback queue.
///
/// Closed set: exactly four variants (`Pending`, `Playing`, `Completed`, `Skipped`).
/// Adding a variant requires coordinated edits across all consumers (Principle VII).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChapterStatus {
    /// Waiting in the queue for synthesis or playback.
    Pending,
    /// Currently being synthesized or played.
    Playing,
    /// Fully played to completion.
    Completed,
    /// Skipped manually or bypassed due to format refusal / errors.
    Skipped,
}

/// Represents an individual Markdown or text file within a directory session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChapterItem {
    /// File base name (e.g., `01-intro.md`).
    pub filename: String,
    /// Persistent or individual document URI (SAF `content://...` or file path).
    pub file_uri: String,
    /// Relative path from directory root (e.g., `capitulo1/01-intro.md`).
    pub relative_path: String,
    /// File size in bytes (checked against `MAX_DOC_BYTES`).
    pub byte_size: u64,
    /// Blake3-256 fingerprint of the raw file content.
    pub content_fingerprint: [u8; 32],
    /// Current lifecycle playback status of this chapter.
    pub playback_status: ChapterStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapter_status_wire_format() -> Result<(), serde_json::Error> {
        assert_eq!(
            serde_json::to_string(&ChapterStatus::Pending)?,
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&ChapterStatus::Playing)?,
            "\"playing\""
        );
        assert_eq!(
            serde_json::to_string(&ChapterStatus::Completed)?,
            "\"completed\""
        );
        assert_eq!(
            serde_json::to_string(&ChapterStatus::Skipped)?,
            "\"skipped\""
        );

        let parsed: ChapterStatus = serde_json::from_str("\"pending\"")?;
        assert_eq!(parsed, ChapterStatus::Pending);
        Ok(())
    }

    #[test]
    fn test_chapter_item_serde() -> Result<(), serde_json::Error> {
        let item = ChapterItem {
            filename: "01-intro.md".to_string(),
            file_uri: "content://com.android.providers.media/1".to_string(),
            relative_path: "01-intro.md".to_string(),
            byte_size: 1024,
            content_fingerprint: [7u8; 32],
            playback_status: ChapterStatus::Pending,
        };

        let serialized = serde_json::to_string(&item)?;
        let deserialized: ChapterItem = serde_json::from_str(&serialized)?;
        assert_eq!(item, deserialized);
        Ok(())
    }
}
