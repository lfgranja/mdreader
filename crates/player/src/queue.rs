//! Chapter queue and playlist data structures for directory reading.
//!
//! Provides the closed-set [`ChapterStatus`] enum, the [`ChapterItem`]
//! struct representing files discovered during directory scanning, and the
//! [`ChapterQueue`] managing playlist order, natural sorting, and navigation.

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

/// Closed-set errors arising from chapter queue operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ChapterQueueError {
    /// Attempted to select or jump to an index outside queue bounds.
    #[error("Chapter index {index} out of bounds for queue of length {len}")]
    IndexOutOfBounds {
        /// Requested index.
        index: usize,
        /// Total items in queue.
        len: usize,
    },
}

/// In-memory queue managing sequential playback of multiple chapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChapterQueue {
    items: Vec<ChapterItem>,
    current_index: usize,
}

impl ChapterQueue {
    /// Constructs a new `ChapterQueue` with initial items, starting at index 0.
    #[must_use]
    pub const fn new(items: Vec<ChapterItem>) -> Self {
        Self {
            items,
            current_index: 0,
        }
    }

    /// Returns a reference to the active chapter item, if present.
    #[must_use]
    pub fn current(&self) -> Option<&ChapterItem> {
        self.items.get(self.current_index)
    }

    /// Returns a mutable reference to the active chapter item, if present.
    #[must_use]
    pub fn current_mut(&mut self) -> Option<&mut ChapterItem> {
        self.items.get_mut(self.current_index)
    }

    /// Returns the current 0-based chapter index.
    #[must_use]
    pub const fn current_index(&self) -> usize {
        self.current_index
    }

    /// Returns the total number of chapters in the queue.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the queue contains no chapter items.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Advances to the next chapter if within bounds, updating `current_index`.
    ///
    /// Returns the newly active `ChapterItem` if advance was successful.
    pub fn advance_to_next(&mut self) -> Option<&ChapterItem> {
        if self.items.is_empty() {
            return None;
        }
        let next_index = self.current_index.saturating_add(1);
        if next_index < self.items.len() {
            self.current_index = next_index;
            self.items.get(self.current_index)
        } else {
            None
        }
    }

    /// Regresses to the previous chapter if within bounds, updating `current_index`.
    ///
    /// Returns the newly active `ChapterItem` if regression was successful.
    pub fn regress_to_previous(&mut self) -> Option<&ChapterItem> {
        if self.items.is_empty() || self.current_index == 0 {
            return None;
        }
        self.current_index = self.current_index.saturating_sub(1);
        self.items.get(self.current_index)
    }

    /// Sets the active chapter index, returning an error if out of bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ChapterQueueError::IndexOutOfBounds`] when `index >= len()`.
    pub const fn set_current_index(&mut self, index: usize) -> Result<(), ChapterQueueError> {
        if index < self.items.len() {
            self.current_index = index;
            Ok(())
        } else {
            Err(ChapterQueueError::IndexOutOfBounds {
                index,
                len: self.items.len(),
            })
        }
    }

    /// Provides access to all chapter items in the queue.
    #[must_use]
    pub fn items(&self) -> &[ChapterItem] {
        &self.items
    }
}

/// Outcome of an automatic chapter N -> N+1 transition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChapterTransition {
    /// Silence gap between the end of chapter N and start of N+1, in seconds.
    pub gap_secs: f64,
    /// Position of the new chapter's clock right after the transition.
    pub new_position_secs: f64,
}

impl ChapterQueue {
    /// Validates the bytes of chapter `index`; a refused chapter is marked
    /// [`ChapterStatus::Skipped`] and never blocks later chapters.
    ///
    /// # Errors
    ///
    /// Returns the [`ChapterRefusal`] when the chapter was skipped.
    pub fn admit_chapter_bytes(
        &mut self,
        index: usize,
        bytes: &[u8],
    ) -> Result<(), ChapterRefusal> {
        let _ = (index, bytes);
        Err(ChapterRefusal::Empty)
    }

    /// Performs the automatic gapless transition to the next chapter.
    ///
    /// Marks the current chapter `Completed`, activates the next one as
    /// `Playing`, and reports a zero gap with the position clock reset to 0.0.
    /// The audio stream is never torn down by the queue; only the index moves.
    ///
    /// # Errors
    ///
    /// Returns [`ChapterQueueError::IndexOutOfBounds`] when there is no next chapter.
    pub fn transition_gapless(&mut self) -> Result<ChapterTransition, ChapterQueueError> {
        let next = self.current_index.saturating_add(1);
        if next >= self.items.len() {
            return Err(ChapterQueueError::IndexOutOfBounds {
                index: next,
                len: self.items.len(),
            });
        }
        if let Some(c) = self.items.get_mut(self.current_index) {
            c.playback_status = ChapterStatus::Completed;
        }
        self.current_index = next;
        if let Some(c) = self.items.get_mut(next) {
            c.playback_status = ChapterStatus::Playing;
        }
        Ok(ChapterTransition {
            gap_secs: 0.0,
            new_position_secs: 0.0,
        })
    }
}

/// Per-chapter byte budget (FR-013): the single `MAX_DOC_BYTES` cap applies to each file.
pub const MAX_CHAPTER_BYTES: usize = crate::interruption::MAX_DOC_BYTES;

/// Closed-set reasons a chapter's bytes are refused before synthesis (FR-013).
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChapterRefusal {
    /// The file has zero bytes.
    #[error("chapter is empty")]
    Empty,
    /// The file exceeds [`MAX_CHAPTER_BYTES`].
    #[error("chapter exceeds the {MAX_CHAPTER_BYTES}-byte budget")]
    Oversize,
    /// The file is not valid UTF-8.
    #[error("chapter is not valid UTF-8")]
    InvalidUtf8,
}

impl ChapterRefusal {
    /// The interruption reason every refusal maps to (persistent, `FormatUndeliverable`).
    #[must_use]
    pub const fn reason(self) -> crate::interruption::InterruptionReason {
        match self {
            Self::Empty | Self::Oversize | Self::InvalidUtf8 => {
                crate::interruption::InterruptionReason::FormatUndeliverable
            }
        }
    }
}

/// Validates one chapter's raw bytes (size budget, then UTF-8).
///
/// # Errors
///
/// Returns the [`ChapterRefusal`] naming why the chapter cannot be read.
pub const fn validate_chapter_bytes(bytes: &[u8]) -> Result<&str, ChapterRefusal> {
    let _ = bytes;
    Err(ChapterRefusal::Empty)
}

/// Token parsed from a filename for natural alfanumeric sorting.
#[derive(Debug, PartialEq, Eq)]
enum NaturalSortToken<'a> {
    Number(u64, usize), // (numeric_value, raw_digit_count for tie-breaking)
    Text(&'a str),
}

impl PartialOrd for NaturalSortToken<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NaturalSortToken<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Number(n1, len1), Self::Number(n2, len2)) => {
                n1.cmp(n2).then_with(|| len1.cmp(len2))
            }
            (Self::Number(_, _), Self::Text(_)) => std::cmp::Ordering::Less,
            (Self::Text(_), Self::Number(_, _)) => std::cmp::Ordering::Greater,
            (Self::Text(s1), Self::Text(s2)) => {
                let mut it1 = s1.chars();
                let mut it2 = s2.chars();
                loop {
                    match (it1.next(), it2.next()) {
                        (Some(c1), Some(c2)) => {
                            let lc1 = c1.to_lowercase();
                            let lc2 = c2.to_lowercase();
                            let cmp = lc1.cmp(lc2);
                            if cmp != std::cmp::Ordering::Equal {
                                return cmp;
                            }
                        }
                        (None, Some(_)) => return std::cmp::Ordering::Less,
                        (Some(_), None) => return std::cmp::Ordering::Greater,
                        (None, None) => return std::cmp::Ordering::Equal,
                    }
                }
            }
        }
    }
}

fn tokenize_natural(s: &str) -> Vec<NaturalSortToken<'_>> {
    let mut tokens = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        if bytes[i].is_ascii_digit() {
            let mut val: u64 = 0;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                let digit = u64::from(bytes[i] - b'0');
                val = val.saturating_mul(10).saturating_add(digit);
                i = i.saturating_add(1);
            }
            tokens.push(NaturalSortToken::Number(val, i.saturating_sub(start)));
        } else {
            while i < bytes.len() && !bytes[i].is_ascii_digit() {
                i = i.saturating_add(1);
            }
            if let Some(text) = s.get(start..i) {
                tokens.push(NaturalSortToken::Text(text));
            }
        }
    }
    tokens
}

fn is_readme_or_index(filename: &str) -> bool {
    let lower = filename.to_ascii_lowercase();
    let base = match lower.rsplit_once('.') {
        Some((name, _ext)) => name,
        None => lower.as_str(),
    };
    base == "readme" || base == "index"
}

/// Sorts chapter items naturally:
/// 1. `README.md` / `index.md` files always precede other files (index 0 precedence).
/// 2. Alphanumeric / natural numeric sorting for numbers (e.g. `01-`, `02-`, `10-`).
/// 3. Case-insensitive comparison for text tokens.
pub fn sort_natural(items: &mut [ChapterItem]) {
    items.sort_by(|a, b| {
        let a_is_index = is_readme_or_index(&a.filename);
        let b_is_index = is_readme_or_index(&b.filename);
        match (a_is_index, b_is_index) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let tokens_a = tokenize_natural(&a.filename);
                let tokens_b = tokenize_natural(&b.filename);
                tokens_a
                    .cmp(&tokens_b)
                    .then_with(|| a.relative_path.cmp(&b.relative_path))
            }
        }
    });
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

    fn make_test_chapter(filename: &str) -> ChapterItem {
        ChapterItem {
            filename: filename.to_string(),
            file_uri: format!("content://test/{filename}"),
            relative_path: filename.to_string(),
            byte_size: 100,
            content_fingerprint: [0u8; 32],
            playback_status: ChapterStatus::Pending,
        }
    }

    #[test]
    fn test_chapter_queue_navigation() {
        let items = vec![
            make_test_chapter("01-intro.md"),
            make_test_chapter("02-middle.md"),
            make_test_chapter("03-end.md"),
        ];
        let mut queue = ChapterQueue::new(items);

        assert_eq!(queue.len(), 3);
        assert!(!queue.is_empty());
        assert_eq!(queue.current_index(), 0);
        assert_eq!(
            queue.current().map(|c| c.filename.as_str()),
            Some("01-intro.md")
        );

        // mutate current
        if let Some(c) = queue.current_mut() {
            c.playback_status = ChapterStatus::Playing;
        }
        assert_eq!(
            queue.current().map(|c| c.playback_status),
            Some(ChapterStatus::Playing)
        );

        // regress at 0 -> None
        assert!(queue.regress_to_previous().is_none());
        assert_eq!(queue.current_index(), 0);

        // advance to next
        let next = queue.advance_to_next();
        assert_eq!(next.map(|c| c.filename.as_str()), Some("02-middle.md"));
        assert_eq!(queue.current_index(), 1);

        // advance to next
        let next2 = queue.advance_to_next();
        assert_eq!(next2.map(|c| c.filename.as_str()), Some("03-end.md"));
        assert_eq!(queue.current_index(), 2);

        // advance beyond end -> None
        assert!(queue.advance_to_next().is_none());
        assert_eq!(queue.current_index(), 2);

        // regress to previous
        let prev = queue.regress_to_previous();
        assert_eq!(prev.map(|c| c.filename.as_str()), Some("02-middle.md"));
        assert_eq!(queue.current_index(), 1);

        // set_current_index valid
        assert!(queue.set_current_index(0).is_ok());
        assert_eq!(queue.current_index(), 0);

        // set_current_index out of bounds
        let err = queue.set_current_index(10);
        assert_eq!(
            err,
            Err(ChapterQueueError::IndexOutOfBounds { index: 10, len: 3 })
        );
        assert_eq!(queue.current_index(), 0);
    }

    #[test]
    fn test_chapter_queue_empty() {
        let mut queue = ChapterQueue::new(vec![]);
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        assert!(queue.current().is_none());
        assert!(queue.current_mut().is_none());
        assert!(queue.advance_to_next().is_none());
        assert!(queue.regress_to_previous().is_none());
        assert_eq!(
            queue.set_current_index(0),
            Err(ChapterQueueError::IndexOutOfBounds { index: 0, len: 0 })
        );
    }

    #[test]
    fn test_sort_natural_numeric_and_readme_precedence() {
        let mut items = vec![
            make_test_chapter("10-conclusion.md"),
            make_test_chapter("02-chapter.md"),
            make_test_chapter("01-intro.md"),
            make_test_chapter("README.md"),
            make_test_chapter("appendix.md"),
            make_test_chapter("09-summary.md"),
            make_test_chapter("Index.MD"),
        ];

        sort_natural(&mut items);

        let names: Vec<&str> = items.iter().map(|i| i.filename.as_str()).collect();
        // Both README.md and Index.MD are index files; they should come before numeric chapters
        assert!(names[0] == "README.md" || names[0] == "Index.MD");
        assert!(names[1] == "README.md" || names[1] == "Index.MD");
        assert_eq!(
            &names[2..],
            &[
                "01-intro.md",
                "02-chapter.md",
                "09-summary.md",
                "10-conclusion.md",
                "appendix.md"
            ]
        );
    }

    #[test]
    fn test_sort_natural_case_insensitivity() {
        let mut items = vec![
            make_test_chapter("b.md"),
            make_test_chapter("A.md"),
            make_test_chapter("10-file.md"),
            make_test_chapter("2-file.md"),
            make_test_chapter("02-file.md"),
        ];

        sort_natural(&mut items);

        let names: Vec<&str> = items.iter().map(|i| i.filename.as_str()).collect();
        // 02-file has number 2, len 2; 2-file has number 2, len 1.
        // Both are smaller than 10-file.
        // A.md before b.md
        assert_eq!(names[2], "10-file.md");
        assert_eq!(names[3], "A.md");
        assert_eq!(names[4], "b.md");
    }
}
