//! Chunk split — the byte-budget segmentation (T028;
//! `API_CONTRACT.md` §2 and §2.1, normative rules 1–12).
//!
//! The adapter owns the split (`FR-024a`): narration is cut
//! into bounded `DocumentSegment`s at paragraph/heading
//! boundaries, with the 5 000-byte budget (`MAX_CHUNK_BYTES`)
//! a hard upper bound measured in UTF-8 **bytes**, never
//! characters (`constitution.md:353`; R3), and each
//! segment's content trimmed of its edge whitespace (§2.1
//! rule 4).
//!
//! The scan is a deliberate line scan (§2.1 rule 1): it is
//! not fence-aware and does not recognise setext headings, so
//! a `#` line inside a fenced code block opens a heading
//! block and a setext underline is an ordinary paragraph
//! line — the recorded line-scan deviation (task T034), not
//! a defect of this module.
//!
//! `document_hash` rides the split call so callers can bind
//! segments to a document identity, but it is never stored on
//! a segment and never influences the split: the `blake3`
//! document fingerprint is owned by `session` (§2.1 rule 12;
//! `spec.md` Assumption 5; R5).

use std::fmt;

/// Hard upper bound, in UTF-8 **bytes**, for one delivered
/// segment.
pub const MAX_CHUNK_BYTES: usize = 5_000;

/// The separator that joins two blocks inside one segment
/// (§2.1 rule 2): a single blank line.
const SEPARATOR: &str = "\n\n";

/// A 1-based, inclusive line range plus 0-based byte columns
/// within the first/last line of that range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceRange {
    /// The first line of the range, 1-based inclusive.
    pub line_start: u32,
    /// The last line of the range, 1-based inclusive;
    /// `>= line_start`.
    pub line_end: u32,
    /// The 0-based byte column inside `line_start` where the
    /// range begins.
    pub col_start: u32,
    /// The 0-based byte column inside `line_end` where the
    /// range ends.
    pub col_end: u32,
}

impl SourceRange {
    /// The range `[line_start, line_end]` with byte columns
    /// `col_start`..`col_end`.
    #[must_use]
    pub const fn new(line_start: u32, line_end: u32, col_start: u32, col_end: u32) -> Self {
        Self {
            line_start,
            line_end,
            col_start,
            col_end,
        }
    }

    /// The whole-line range `[line_start, line_end]`: both
    /// byte columns are 0.
    #[must_use]
    pub const fn whole_lines(line_start: u32, line_end: u32) -> Self {
        Self::new(line_start, line_end, 0, 0)
    }

    /// The number of lines the range covers:
    /// `line_end - line_start + 1`. Every range constructed
    /// by this module upholds `line_end >= line_start`.
    #[must_use]
    pub const fn line_count(&self) -> u32 {
        self.line_end - self.line_start + 1
    }

    /// Whether the range covers no line at all
    /// (`line_end < line_start`).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.line_end < self.line_start
    }

    /// Whether `line` falls inside the range
    /// (`line_start <= line <= line_end`).
    #[must_use]
    pub const fn contains_line(&self, line: u32) -> bool {
        self.line_start <= line && line <= self.line_end
    }
}

impl fmt::Display for SourceRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            line_start,
            col_start,
            line_end,
            col_end,
        } = *self;
        write!(formatter, "L{line_start}:{col_start}-L{line_end}:{col_end}")
    }
}

/// The closed boundary-kind set. There is **no** `Sentence`
/// member (`AlignmentGranularity::None`; R6). Adding one is a
/// coordinated edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryKind {
    /// A block of consecutive non-blank, non-heading lines
    /// (§2.1 rule 1); also the kind of every forced
    /// sub-piece of an oversized block (§2.1 rule 3).
    Paragraph,
    /// A block that opens with a heading line (§2.1 rule 1).
    Heading,
}

/// One bounded, spoken chunk. Immutable after delivery
/// (`spec.md:103`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSegment {
    /// The segment's narration text: the covered non-blank
    /// source lines, joined with single newlines inside a
    /// block and with a single blank line between the blocks
    /// of one segment (§2.1 rules 1, 2 and 4).
    pub content: String,
    /// The original-source lines the segment covers (§2.1
    /// rule 10).
    pub source_range: SourceRange,
    /// The boundary the segment opens with (§2.1 rules 1
    /// and 3).
    pub boundary_kind: BoundaryKind,
    /// The 0-based position in the emitted vector, strictly
    /// increasing by 1 (§2.1 rule 9).
    pub segment_index: u64,
    /// The completed boundary at-or-before the yield point
    /// when the yield point falls inside this segment: always
    /// `Some(source_range)` (§2.1 rule 11; R6).
    pub resume_point: Option<SourceRange>,
}

impl DocumentSegment {
    /// UTF-8 **byte** length of `content` — never a character
    /// count (`constitution.md:353`; R3). Derived from the
    /// content, not stored, so the byte invariant cannot drift
    /// from the text it measures.
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        self.content.len()
    }
}

/// Split `source` at paragraph/heading boundaries under
/// `MAX_CHUNK_BYTES`.
///
/// `document_hash` is accepted so callers can bind the
/// segments to a document identity, but it is never stored on
/// a segment and never influences the split (§2.1 rule 12;
/// R5 — the `blake3` fingerprint is owned by `session`).
#[must_use]
pub fn split_document(source: &str, document_hash: &str) -> Vec<DocumentSegment> {
    split_document_within(source, document_hash, MAX_CHUNK_BYTES)
}

/// Same as [`split_document`], with an explicit byte budget
/// (the property tests exercise small budgets).
///
/// `document_hash` is accepted but never stored on a segment
/// and never influences the split (§2.1 rule 12; R5).
#[must_use]
pub fn split_document_within(
    source: &str,
    _document_hash: &str,
    max_bytes: usize,
) -> Vec<DocumentSegment> {
    let lines: Vec<&str> = source.lines().collect();
    let blocks = scan_blocks(&lines);
    let mut segments: Vec<DocumentSegment> = Vec::new();
    let mut open: Option<OpenSegment> = None;
    for block in blocks {
        let text = block_text(&lines, block);
        if text.len() > max_bytes {
            // §2.1 rule 2: an over-budget block can never join
            // the open segment — the open segment closes first
            // — and §2.1 rule 3 hard-splits the block into
            // character-safe pieces (the budget is a hard
            // bound, not a target).
            close_segment(&mut segments, &lines, open.take());
            let range = range_of(&lines, block.line_start, block.line_end);
            for piece in forced_pieces(&text, max_bytes) {
                push_piece(&mut segments, &piece, range);
            }
            continue;
        }
        if block.kind == BoundaryKind::Heading {
            // §2.1 rule 1: a heading block is narrated as its
            // own segment — the split never lands between a
            // heading and its text, and no other block shares
            // the heading's segment.
            close_segment(&mut segments, &lines, open.take());
            close_segment(&mut segments, &lines, Some(OpenSegment::new(text, block)));
            continue;
        }
        // §2.1 rule 2 — greedy packing: a paragraph block
        // joins the open segment while the joined content
        // stays within the budget; a block that no longer
        // fits closes the open segment and opens a new one.
        let joins_open = open.as_ref().is_some_and(|open_segment| {
            open_segment.content.len() + SEPARATOR.len() + text.len() <= max_bytes
        });
        let next_open = if joins_open {
            // The join condition was evaluated against `open`,
            // so the open segment exists; the fallback can only
            // fire if that invariant were ever broken, and it
            // re-opens with the block rather than dropping it.
            match open.take() {
                Some(mut open_segment) => {
                    open_segment.append(&text, block);
                    open_segment
                }
                None => OpenSegment::new(text, block),
            }
        } else {
            close_segment(&mut segments, &lines, open.take());
            OpenSegment::new(text, block)
        };
        open = Some(next_open);
    }
    close_segment(&mut segments, &lines, open.take());
    segments
}

/// A narration block (§2.1 rule 1): a heading block or a
/// paragraph block, covering the 1-based inclusive line range
/// `[line_start, line_end]` of the original source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Block {
    /// The block's first line, 1-based inclusive.
    line_start: u32,
    /// The block's last line, 1-based inclusive.
    line_end: u32,
    /// The block's boundary kind.
    kind: BoundaryKind,
}

/// A segment under construction: the blocks that greedy
/// packing (§2.1 rule 2) has joined so far.
#[derive(Debug)]
struct OpenSegment {
    /// The joined content: the packed blocks' texts, joined
    /// with a single blank line between blocks.
    content: String,
    /// The first line of the segment's opening block, 1-based
    /// inclusive.
    line_start: u32,
    /// The last line of the segment's last packed block,
    /// 1-based inclusive.
    line_end: u32,
    /// The boundary kind of the segment's opening block.
    kind: BoundaryKind,
}

impl OpenSegment {
    /// A new open segment holding exactly one block.
    #[must_use]
    const fn new(text: String, block: Block) -> Self {
        Self {
            content: text,
            line_start: block.line_start,
            line_end: block.line_end,
            kind: block.kind,
        }
    }

    /// Append a block's text to the open segment, joined with
    /// a single blank line, extending the covered line range.
    fn append(&mut self, text: &str, block: Block) {
        self.content.push_str(SEPARATOR);
        self.content.push_str(text);
        self.line_end = block.line_end;
    }
}

/// Scan the source lines into narration blocks (§2.1 rule 1).
///
/// A line whose trimmed form starts with 1–6 `#` followed by
/// a space or end-of-line (at most 3 leading spaces) opens a
/// heading block; the block extends over the heading line and
/// the following run of non-blank, non-heading lines — across
/// the blank lines between them when the heading has no
/// immediately-following body — so a heading and its body are
/// narrated together and the split never lands between them.
/// Any other run of consecutive non-blank lines is a paragraph
/// block. Blank lines separate blocks and carry no content.
///
/// The scan is deliberately not fence-aware and does not
/// recognise setext headings: a `#` line inside a fenced code
/// block opens a heading block, and a setext underline is an
/// ordinary paragraph line (the recorded line-scan deviation,
/// task T034).
///
/// `line` tracks the 1-based number of `lines[index]` in
/// lockstep with `index`; the counter saturates at `u32::MAX`
/// instead of overflowing, which is unreachable for any
/// document that fits in memory (over four billion lines would
/// exceed four gigabytes of text).
#[must_use]
fn scan_blocks(lines: &[&str]) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut index: usize = 0;
    let mut line: u32 = 1;
    while index < lines.len() {
        if lines[index].trim().is_empty() {
            index += 1;
            line = line.saturating_add(1);
            continue;
        }
        // The block's opening line is shared by both branch
        // shapes: consume it once here, before the branch, so
        // neither branch duplicates the prologue.
        let start_line = line;
        let is_heading = is_heading_line(lines[index]);
        index += 1;
        line = line.saturating_add(1);
        if is_heading {
            let mut end_line = start_line;
            // The heading block extends over the immediately
            // following non-blank, non-heading lines.
            while index < lines.len()
                && !lines[index].trim().is_empty()
                && !is_heading_line(lines[index])
            {
                end_line = line;
                index += 1;
                line = line.saturating_add(1);
            }
            // With no immediately-following body, the block
            // extends over the blank lines to the next run, so
            // a heading and its body are still narrated
            // together.
            if end_line == start_line {
                let mut cursor = index;
                let mut cursor_line = line;
                while cursor < lines.len() && lines[cursor].trim().is_empty() {
                    cursor += 1;
                    cursor_line = cursor_line.saturating_add(1);
                }
                while cursor < lines.len()
                    && !lines[cursor].trim().is_empty()
                    && !is_heading_line(lines[cursor])
                {
                    end_line = cursor_line;
                    cursor += 1;
                    cursor_line = cursor_line.saturating_add(1);
                }
                if end_line > start_line {
                    index = cursor;
                    line = cursor_line;
                }
            }
            blocks.push(Block {
                line_start: start_line,
                line_end: end_line,
                kind: BoundaryKind::Heading,
            });
        } else {
            while index < lines.len()
                && !lines[index].trim().is_empty()
                && !is_heading_line(lines[index])
            {
                index += 1;
                line = line.saturating_add(1);
            }
            blocks.push(Block {
                line_start: start_line,
                line_end: line - 1,
                kind: BoundaryKind::Paragraph,
            });
        }
    }
    blocks
}

/// Whether `line` opens a heading block under the §2.1 rule 1
/// line scan: its form, after at most 3 leading spaces,
/// starts with 1–6 `#` followed by a space or end-of-line.
#[must_use]
fn is_heading_line(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut leading_spaces = 0;
    while leading_spaces < bytes.len() && bytes[leading_spaces] == b' ' {
        leading_spaces += 1;
    }
    if leading_spaces > 3 {
        return false;
    }
    let rest = &bytes[leading_spaces..];
    let mut hashes = 0;
    while hashes < rest.len() && hashes < 6 && rest[hashes] == b'#' {
        hashes += 1;
    }
    hashes >= 1 && (hashes == rest.len() || rest[hashes] == b' ')
}

/// The content of a block: its covered source lines joined
/// with single newlines. A heading block's covered range
/// spans the blank lines it absorbed across, so those
/// separators are kept inside the block's content (§2.1
/// rules 1 and 4 — the content carries no leading or
/// trailing blank line).
#[must_use]
fn block_text(lines: &[&str], block: Block) -> String {
    let start = block.line_start as usize - 1;
    let end = block.line_end as usize;
    lines[start..end].join("\n")
}

/// Hard-split an over-budget block's text into pieces of at
/// most `max_bytes` bytes each (§2.1 rule 3): the budget is a
/// hard bound, so the block is cut at the largest offset that
/// keeps the piece within the budget, snapped **down** to a
/// UTF-8 character boundary so no piece ever splits a `char`.
///
/// The floor: when the budget is smaller than the next
/// character (a single `char` can exceed a 1-byte budget), no
/// character-safe cut is achievable, and the piece is the
/// character itself — whole and valid UTF-8 — so the split
/// stays deterministic and lossless and the bound holds
/// wherever a character-safe cut is achievable.
#[must_use]
fn forced_pieces(text: &str, max_bytes: usize) -> Vec<String> {
    let mut pieces: Vec<String> = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let mut end = rest.len().min(max_bytes);
        while end > 0 && !rest.is_char_boundary(end) {
            end -= 1;
        }
        if end == 0 {
            end = rest.chars().next().map_or(1, char::len_utf8);
        }
        pieces.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    pieces
}

/// The source range for the original-source lines
/// `[line_start, line_end]`: `col_start` at 0 and `col_end`
/// at the byte length of the last line (§2.1 rule 10).
#[must_use]
fn range_of(lines: &[&str], line_start: u32, line_end: u32) -> SourceRange {
    SourceRange::new(
        line_start,
        line_end,
        0,
        col_end(lines[line_end as usize - 1]),
    )
}

/// The 0-based byte column at the end of a source line: the
/// line's UTF-8 byte length (§2.1 rule 10).
///
/// A line longer than `u32::MAX` bytes (over four gigabytes)
/// cannot be represented in the contract's `u32` columns; the
/// length saturates at `u32::MAX` rather than panicking. No
/// real line approaches the bound.
#[must_use]
fn col_end(line: &str) -> u32 {
    u32::try_from(line.len()).unwrap_or(u32::MAX)
}

/// Close the open segment into the emitted vector (§2.1 rules
/// 4 and 9): the content is trimmed of leading and trailing
/// whitespace, a segment whose trimmed content is empty is not
/// emitted, and the segment is numbered 0-based and
/// consecutive. Trimming tightens — never loosens — the rule 5
/// byte bound, because `byte_length()` is derived from the
/// trimmed content.
fn close_segment(segments: &mut Vec<DocumentSegment>, lines: &[&str], open: Option<OpenSegment>) {
    let Some(open_segment) = open else {
        return;
    };
    // §2.1 rule 4: trim the joined content of its edge
    // whitespace (the first line's leading and the last
    // line's trailing whitespace). Interior whitespace —
    // including the edge whitespace of interior lines — is
    // content and survives.
    let content = open_segment.content.trim().to_string();
    if content.is_empty() {
        return;
    }
    let range = range_of(lines, open_segment.line_start, open_segment.line_end);
    let index = segments.len() as u64;
    segments.push(DocumentSegment {
        content,
        source_range: range,
        boundary_kind: open_segment.kind,
        segment_index: index,
        resume_point: Some(range),
    });
}

/// Emit a forced sub-piece of an oversized block: `Paragraph`
/// (§2.1 rule 3 — there is no interior boundary to report)
/// and reporting the source block's own line range (§2.1 rule
/// 10), numbered 0-based and consecutive (§2.1 rule 9). The
/// piece's content is trimmed of leading and trailing
/// whitespace, and a piece whose trimmed content is empty is
/// not emitted (§2.1 rule 4 — every emitted segment's content
/// is trimmed, and a forced sub-piece is a segment).
///
/// The trim cannot break the rule 3 hard split or the rule 5
/// bound: it only removes whitespace from a piece's edges, so
/// the cut points and the byte count are unchanged or
/// smaller. The byte-exact reassembly the rule-3 fixtures
/// pin (`reassembled_contents(&segments) == source`) holds
/// because those sources are whitespace-free, where the trim
/// is a no-op; wherever a cut does cross whitespace, rule 6
/// losslessness is measured whitespace-squashed, as the
/// property tests' `assert_lossless` helpers document.
fn push_piece(segments: &mut Vec<DocumentSegment>, piece: &str, range: SourceRange) {
    let content = piece.trim().to_string();
    if content.is_empty() {
        return;
    }
    let index = segments.len() as u64;
    segments.push(DocumentSegment {
        content,
        source_range: range,
        boundary_kind: BoundaryKind::Paragraph,
        segment_index: index,
        resume_point: Some(range),
    });
}
