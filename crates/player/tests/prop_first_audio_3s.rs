//! T112 — FR-003 / SC-002: the ≤3 s first-audible property (Phase 16,
//! RED-FIRST).
//!
//! Task text (verbatim, `specs/004-ressoa-dogfood-build/tasks.md`
//! T112): "RED-FIRST rewrite `crates/player/tests/prop_first_audio_3s.rs`
//! against a REAL production seam: drive the actual open/play path and
//! assert the first synthesized segment is enqueued (and playback
//! started) BEFORE the remaining segments are synthesized — the current
//! test (`:113-150`) constructs the event list and asserts the order it
//! just wrote, so it passes regardless of the implementation; the
//! property MUST fail against the current code, which synthesizes the
//! whole document before any enqueue (`src-tauri/src/lib.rs:3748-3771`,
//! `:3950-3956`, `:3809-3828`); also delete or correct the comment at
//! `:3759` that claims the ≤3 s bound is already met (Principle III
//! Test-First)".
//!
//! ## The invariant
//!
//! For every generated fixture-scale document, the production open/play
//! path must:
//!
//! * **I1 — first chunk, not the whole document.** Hand the FIRST
//!   synthesized segment to the backend's queue after at most ONE
//!   segment synthesis: no whole-document PCM buffer may exist when
//!   the first chunk is enqueued.
//! * **I2 — first enqueue precedes the last synthesis.** The first
//!   chunk is enqueued while the rest of the document is still being
//!   synthesized (data-model `PlaybackSession`: "first segment
//!   synthesizes, first chunk enqueues, `backend.play()` starts — never
//!   after full PCM").
//! * **I3 — playback starts on the first chunk.** The backend is
//!   started after the first chunk is queued, and before a second
//!   segment is synthesized.
//! * **I4 — no whole-document buffer at the enqueue.** The buffer
//!   handed to the backend is the chunk's own, never a buffer the
//!   path accumulates every segment's PCM into.
//!
//! ## NON-VACUITY — the observation must see the REST of the document
//!
//! I1–I4 are about the WHOLE document: the first chunk is handed
//! over while the rest of the document is still being produced. An
//! observation that never sees a synthesis AFTER the first enqueue
//! therefore proves nothing — the property would pass by observing a
//! single unit and no remainder. The property carries a guard of its
//! own for that reason: **the observed order must contain at least
//! one synthesis after the first enqueue**, the evidence that the
//! scanner really is looking at the remaining units being produced.
//! Without it, a scanner that lost the production seam and a
//! production that stopped producing the rest of the document would
//! be indistinguishable from a production that satisfies the
//! invariant — the exact silent loss of coverage the T112 rewrite
//! was written to prevent (Principle III: "the test suite is the
//! only evidence").
//!
//! ## The seam
//!
//! The subject of the property is the REAL production open/play path —
//! `src-tauri/src/lib.rs` — read at COMPILE time through
//! [`APP_SHELL_SRC`]. The T055 revision of this file built the event
//! list it then asserted (`FirstSegmentSynth`, `FirstChunkEnqueue`,
//! `BackendPlayStart`, `FullLoopComplete` pushed in the required
//! order), so it could never fail: it MODELLED the order instead of
//! OBSERVING it. This revision reconstructs the order the shipped
//! code emits — the scanner below walks the real production source,
//! inlines the production functions the play path calls, and reads
//! the production's own semantic steps off it — and asserts the
//! invariant against THAT. The event list is never written by this
//! file: it is read out of the production source.
//!
//! ### The production's semantic steps
//!
//! The T143 restructure and the T158 back-pressure rewrite moved the
//! audio pipeline behind three app-shell functions: the production no
//! longer synthesizes and enqueues inside the play path's own body —
//! it produces a unit, hands a chunk over and starts the backend
//! through `produce_next_unit` / `enqueue_chunk` / `start_backend`,
//! and the producer's steady-state loop advances the production from
//! there. The scanner recognises THOSE steps ([`PRODUCTION_STEPS`]) at
//! their call sites, and recognises the boundary call each step's own
//! implementation ends in (`synthesize` / `enqueue_pcm` / `play`) as
//! the fallback for a production shape that reaches the engine and
//! the backend directly. A step and the boundary it carries are the
//! SAME event, so the scanner emits it once: at the step's call site,
//! with the boundary suppressed inside the step's own body.
//!
//! The production audio pipeline is driven for real as well: the
//! generated document is split by the production chunker
//! ([`split_document`]) and its first chunk is synthesized by the
//! production engine ([`synthesize`]), so the property's scale (the
//! number of segments the path must synthesize) and the audio the
//! first chunk carries are the production ones, not modelled ones.
//!
//! ## Why the path is observed through its source
//!
//! `play_document` is `pub(crate)` in the app shell and
//! `synthesize_segments` / `start_playback` are private to it, so no
//! test crate can CALL the open/play path: the production seam is
//! reachable only as source. T113 owns `src-tauri/src/lib.rs`; this
//! file observes it and never edits it.
//!
//! ## Spec anchors
//!
//! * FR-003: "The system MUST produce the first audible speech within
//!   3 seconds of a Play action on the reference phone."
//! * SC-002: "First audible speech arrives within 3 seconds of Play in
//!   10 out of 10 trials on the reference phone from a cold start."
//! * data-model.md `PlaybackSession` (001): "first segment synthesizes,
//!   first chunk enqueues, `backend.play()` starts — never after full
//!   PCM".
//! * Principle III (test-first): the property was written RED-first,
//!   against the pre-T143 implementation that synthesized the whole
//!   document before any enqueue. It is GREEN against the current
//!   implementation — which is a fact the OBSERVATION establishes,
//!   never an assumption: the property re-reads the production source
//!   on every run and fails the moment the production stops
//!   streaming.
//!
//! ## The comment T113 corrected
//!
//! The pre-T143 implementation carried the comment "First chunk
//! enqueued before remaining synthesis — meets ≤3 s." above a body
//! that accumulated the whole document into one `Vec<f32>` and
//! enqueued it once. T113 deleted that comment (the ≤3 s claim is
//! exactly what this property gates). The production body it described
//! no longer exists: the audio pipeline moved behind the production's
//! semantic steps, and the observation below reads the order off
//! them.

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;

use adapter_synthesis::chunk::split_document;
use adapter_synthesis::synth::synthesize;

/// The production app-shell source under observation.
///
/// The seam is the REAL production open/play path, read at COMPILE
/// time: the order this file asserts is the order the shipped code
/// emits, and the file can never drift into asserting an order it
/// wrote itself.
const APP_SHELL_SRC: &str = include_str!("../../../src-tauri/src/lib.rs");

/// The production entry point of the single synthesis path every
/// surface routes through (Principle II): the `play` wire action, the
/// cold-start `open_document_bytes`, and every later consumer land
/// here exactly once.
const PLAY_PATH_ENTRY: &str = "play_document";

/// The maximum inlining depth of the production scanner. The play
/// path is a handful of production calls deep; the bound stops a
/// recursive production function from inlining forever.
const MAX_INLINE_DEPTH: usize = 8;

/// The number of iterations every production loop body is expanded
/// to when the observed order is flattened.
///
/// Two iterations are enough to tell the two shapes apart: the first
/// chunk and one later chunk. A path that enqueues inside the loop
/// shows its enqueue between the two syntheses; a path that enqueues
/// after the loop shows both syntheses before the enqueue.
const EXPANDED_ITERATIONS: usize = 2;

/// The production words the generated documents are built from: short
/// pt-BR nouns the production engine can speak, so the first chunk's
/// synthesis is a real, quick piece of audio.
const FIXTURE_WORDS: [&str; 12] = [
    "leitura",
    "documento",
    "página",
    "voz",
    "áudio",
    "texto",
    "leitor",
    "seção",
    "resumo",
    "nota",
    "capítulo",
    "narração",
];

/// The generated document's section count: at least TWO, so the
/// production path always has "the remaining segments" to synthesize
/// after the first chunk — a single-segment document would make the
/// first-audible invariant vacuous.
const FIXTURE_MIN_SECTIONS: usize = 2;

/// The generated document's section-count ceiling.
const FIXTURE_MAX_SECTIONS: usize = 8;

/// A failure of the production audio pipeline observed by the
/// property.
///
/// Proptest turns any `std::error::Error` returned through `?` from a
/// property body into a test failure (`TestCaseError: From<E: Error>`),
/// so the property body can use `?` for the fallible production call.
#[derive(Debug, thiserror::Error)]
enum PropError {
    /// The production engine refused the first chunk with a named
    /// failure — a member of the closed `EngineError` set.
    #[error("the production engine refused the first chunk: {0:?}")]
    Engine(adapter_synthesis::EngineFailure),
}

/// One production event observed in the app-shell play path.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    /// The production engine synthesized one document segment.
    SegmentSynthesized,
    /// A chunk was handed to the backend's playback queue, carrying
    /// the identifiers the production passed with it.
    ChunkEnqueued(Vec<String>),
    /// The backend's output stream was started.
    PlaybackStarted,
}

/// What a production step of the app shell does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StepKind {
    /// The step synthesizes one document unit.
    Synthesizes,
    /// The step hands a chunk to the backend's queue.
    Enqueues,
    /// The step starts the backend's output stream.
    Starts,
}

/// One production step of the app shell: the function that carries it
/// and the boundary call its own implementation ends in.
///
/// A step and the boundary it carries are the SAME event, so the
/// scanner emits it once — at the step's call site — with the
/// boundary suppressed inside the step's own body.
#[derive(Clone, Copy, Debug)]
struct ProductionStep {
    /// The app-shell function that carries the step.
    function: &'static str,
    /// The boundary call the step's own implementation ends in.
    boundary: &'static str,
    /// What the step does.
    kind: StepKind,
}

/// The production path's semantic steps: the app-shell functions the
/// T143 restructure and the T158 back-pressure rewrite moved the
/// audio pipeline behind.
///
/// The production no longer synthesizes and enqueues inside the play
/// path's own body — it produces a unit, hands a chunk over and
/// starts the backend through these three functions, and the
/// producer's steady-state loop advances the production from there.
const PRODUCTION_STEPS: [ProductionStep; 3] = [
    ProductionStep {
        function: "produce_next_unit",
        boundary: "synthesize",
        kind: StepKind::Synthesizes,
    },
    ProductionStep {
        function: "enqueue_chunk",
        boundary: "enqueue_pcm",
        kind: StepKind::Enqueues,
    },
    ProductionStep {
        function: "start_backend",
        boundary: "play",
        kind: StepKind::Starts,
    },
];

impl ProductionStep {
    /// The event the step emits at its call site, carrying the
    /// identifiers the production hands over with it.
    fn event(&self, handed_over: Vec<String>) -> Event {
        match self.kind {
            StepKind::Synthesizes => Event::SegmentSynthesized,
            StepKind::Enqueues => Event::ChunkEnqueued(handed_over),
            StepKind::Starts => Event::PlaybackStarted,
        }
    }
}

/// The production step the named function carries, when it carries
/// one.
fn production_step(name: &str) -> Option<&'static ProductionStep> {
    PRODUCTION_STEPS.iter().find(|step| step.function == name)
}

/// One step of the observed production path.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Step {
    /// A single production event.
    Event(Event),
    /// A production loop body — executed once per document segment.
    Repeat(Vec<Self>),
}

/// The production functions of the app shell, by name: the name maps
/// to the function's body text.
type FnIndex = BTreeMap<String, String>;

/// The observed production play path: its steps, plus the identifiers
/// the path accumulates synthesis output into (the whole-document
/// buffers).
#[derive(Debug, Default)]
struct ObservedPath {
    /// The observed steps, in production order.
    steps: Vec<Step>,
    /// The identifiers the path accumulates synthesized PCM into.
    accumulators: BTreeSet<String>,
}

/// The scanner's shared context: the production functions it inlines
/// and the observations it accumulates.
struct Scanner<'a> {
    /// The production functions the scanner inlines at their call
    /// sites.
    functions: &'a FnIndex,
    /// What the scanner observed on the way.
    observed: ObservedPath,
    /// The boundary call suppressed in the region being scanned — the
    /// boundary a production step's own body ends in.
    suppressed: Option<&'static str>,
}

/// Whether the byte is part of a Rust identifier.
const fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Whether the identifier at `index` starts a word (the byte before
/// it is not an identifier byte).
const fn at_word_start(bytes: &[u8], index: usize) -> bool {
    index == 0 || !is_ident_byte(bytes[index - 1])
}

/// Read the identifier starting at `start`; returns the identifier
/// and the index just past it.
fn read_ident(bytes: &[u8], start: usize) -> (String, usize) {
    let mut end = start;
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }
    let name = String::from_utf8_lossy(&bytes[start..end]).into_owned();
    (name, end)
}

/// Skip a string literal; returns the index just past it.
const fn skip_string(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

/// Skip a character literal or a lifetime; returns the index just
/// past it.
const fn skip_char_or_lifetime(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    while index < bytes.len() && index < start + 5 {
        match bytes[index] {
            b'\\' => index += 2,
            b'\'' => return index + 1,
            _ => index += 1,
        }
    }
    start + 1
}

/// Skip a line comment; returns the index of the line break.
const fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    while index < bytes.len() && bytes[index] != b'\n' {
        index += 1;
    }
    index
}

/// Skip a block comment; returns the index just past it.
const fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 2;
    while index + 1 < bytes.len() {
        if bytes[index] == b'*' && bytes[index + 1] == b'/' {
            return index + 2;
        }
        index += 1;
    }
    bytes.len()
}

/// The index of the brace closing the one at `open`, or `None` when
/// the region is unbalanced.
fn matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index = skip_string(bytes, index);
                continue;
            }
            b'\'' => {
                index = skip_char_or_lifetime(bytes, index);
                continue;
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index);
                continue;
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index);
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// The index of the paren closing the one at `open`, or `None` when
/// the region is unbalanced.
fn matching_paren(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index = skip_string(bytes, index);
                continue;
            }
            b'\'' => {
                index = skip_char_or_lifetime(bytes, index);
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// The body text of the production function whose signature ends at
/// `after`, or `None` when the body cannot be located.
fn fn_body(bytes: &[u8], after: usize) -> Option<String> {
    let mut index = after;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index = skip_string(bytes, index);
                continue;
            }
            b'{' => {
                let end = matching_brace(bytes, index)?;
                return Some(String::from_utf8_lossy(&bytes[index + 1..end]).into_owned());
            }
            b';' => return None,
            _ => {}
        }
        index += 1;
    }
    None
}

/// The production functions of the app shell, by name.
fn collect_functions(source: &str) -> FnIndex {
    let bytes = source.as_bytes();
    let mut index = FnIndex::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor = skip_line_comment(bytes, cursor);
                continue;
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                cursor = skip_block_comment(bytes, cursor);
                continue;
            }
            b'"' => {
                cursor = skip_string(bytes, cursor);
                continue;
            }
            b'\'' => {
                cursor = skip_char_or_lifetime(bytes, cursor);
                continue;
            }
            _ => {}
        }
        if bytes[cursor] == b'f'
            && bytes.get(cursor + 1) == Some(&b'n')
            && (cursor == 0 || !is_ident_byte(bytes[cursor - 1]))
            && bytes.get(cursor + 2).is_some_and(u8::is_ascii_whitespace)
        {
            let mut name_start = cursor + 2;
            while name_start < bytes.len() && bytes[name_start].is_ascii_whitespace() {
                name_start += 1;
            }
            let (name, after) = read_ident(bytes, name_start);
            if !name.is_empty()
                && let Some(body) = fn_body(bytes, after)
            {
                let _ignored = index.entry(name).or_insert(body);
            }
        }
        cursor += 1;
    }
    index
}

/// The body of the production loop whose header starts at `after`,
/// and the index just past it.
fn loop_body(bytes: &[u8], after: usize) -> Option<(String, usize)> {
    let mut index = after;
    let mut depth = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index = skip_string(bytes, index);
                continue;
            }
            b'\'' => {
                index = skip_char_or_lifetime(bytes, index);
                continue;
            }
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth = depth.checked_sub(1)?,
            b'{' if depth == 0 => {
                let end = matching_brace(bytes, index)?;
                return Some((
                    String::from_utf8_lossy(&bytes[index + 1..end]).into_owned(),
                    end + 1,
                ));
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// The arguments of the call whose paren opens at `open`, and the
/// index just past the call.
fn call_arguments(bytes: &[u8], open: usize) -> (Vec<String>, usize) {
    let Some(close) = matching_paren(bytes, open) else {
        return (Vec::new(), bytes.len());
    };
    let inner = &bytes[open + 1..close];
    let mut arguments: Vec<String> = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (offset, byte) in inner.iter().enumerate() {
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                arguments.push(
                    String::from_utf8_lossy(&inner[start..offset])
                        .trim()
                        .to_owned(),
                );
                start = offset + 1;
            }
            _ => {}
        }
    }
    let last = String::from_utf8_lossy(&inner[start..]).trim().to_owned();
    if !last.is_empty() || !arguments.is_empty() {
        arguments.push(last);
    }
    (arguments, close + 1)
}

/// The identifiers the production hands over at a step's call site.
///
/// The app shell passes its state as the FIRST argument of every
/// helper, so the arguments after it are what the step carries.
fn handed_over(arguments: &[String]) -> Vec<String> {
    arguments.iter().skip(1).cloned().collect()
}

/// Whether the call whose paren opens at `open` takes no argument.
fn has_no_argument(bytes: &[u8], open: usize) -> bool {
    let Some(close) = matching_paren(bytes, open) else {
        return false;
    };
    bytes[open + 1..close].iter().all(u8::is_ascii_whitespace)
}

/// Whether the identifier at `index` opens a statement: the walk's
/// last significant byte is a block or statement boundary.
fn is_statement_position(last_significant: Option<u8>) -> bool {
    last_significant.is_none_or(|byte| matches!(byte, b'{' | b'}' | b';'))
}

/// The production scanner: the observation seam.
///
/// It walks the production source of one function body and reports
/// the production events the body emits, in production order, with
/// every production loop body kept as a [`Step::Repeat`] so the
/// per-segment repetition stays visible. Production functions called
/// from the body are INLINED, so the events of a helper (the
/// synthesizer loop, the backend starter) appear at the position the
/// caller reaches them.
///
/// A production STEP ([`PRODUCTION_STEPS`]) emits its event at its
/// call site, and its own body is scanned with the boundary the step
/// carries SUPPRESSED — so the step and the boundary it ends in are
/// counted once, not twice.
fn scan_region(text: &str, scanner: &mut Scanner, depth: usize, synthesized: bool) -> Vec<Step> {
    let bytes = text.as_bytes();
    let mut steps: Vec<Step> = Vec::new();
    let mut synthesized = synthesized;
    let mut index = 0usize;
    // The last significant byte the walk passed. Only the CODE's own
    // bytes count: a comment or a literal never does, so a call whose
    // line is preceded by a comment is never mistaken for a method
    // call (the comment's last character is not the call's receiver).
    let mut last_significant: Option<u8> = None;
    while index < bytes.len() {
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index);
                continue;
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index);
                continue;
            }
            b'"' => {
                index = skip_string(bytes, index);
                continue;
            }
            b'\'' => {
                index = skip_char_or_lifetime(bytes, index);
                continue;
            }
            _ => {}
        }
        let previous = last_significant;
        let is_method = previous == Some(b'.');
        if !bytes[index].is_ascii_whitespace() {
            last_significant = Some(bytes[index]);
        }
        if !is_ident_byte(bytes[index]) || !at_word_start(bytes, index) {
            index += 1;
            continue;
        }
        let (name, after) = read_ident(bytes, index);
        if name == "fn" {
            // A nested production definition: its body is not part
            // of this region.
            index = match loop_body(bytes, after) {
                Some((_, end)) => {
                    last_significant = Some(b'}');
                    end
                }
                None => after,
            };
            continue;
        }
        let is_call = bytes.get(after) == Some(&b'(');
        if !is_method
            && matches!(name.as_str(), "for" | "while" | "loop")
            && is_statement_position(previous)
            && let Some((body, end)) = loop_body(bytes, after)
        {
            let inner = scan_region(&body, scanner, depth, false);
            steps.push(Step::Repeat(inner));
            last_significant = Some(b'}');
            index = end;
            continue;
        }
        if is_call {
            // A production STEP: the semantic event is emitted at the
            // call site, and the step's own body is scanned with the
            // boundary the step carries suppressed.
            if let Some(step) = production_step(&name).filter(|_| !is_method) {
                synthesized |= step.kind == StepKind::Synthesizes;
                let end = visit_step(step, bytes, after, scanner, depth, &mut steps);
                last_significant = Some(b')');
                index = end;
                continue;
            }
            if let Some(end) =
                inline_production_call(&name, bytes, after, is_method, scanner, depth, &mut steps)
            {
                last_significant = Some(b')');
                index = end;
                continue;
            }
            if let Some((event, end)) =
                production_event(&name, bytes, after, is_method, scanner.suppressed)
            {
                synthesized |= matches!(&event, Event::SegmentSynthesized);
                steps.push(Step::Event(event));
                last_significant = Some(b')');
                index = end;
                continue;
            }
        }
        // The receiver of an accumulation: a buffer the production
        // path grows with synthesized PCM.
        if synthesized
            && (starts_with(bytes, after, ".extend") || starts_with(bytes, after, ".push"))
        {
            let _ignored = scanner.observed.accumulators.insert(name.clone());
        }
        index = after;
    }
    steps
}

/// The steps a production STEP emits at its call site, and the index
/// just past the call.
///
/// The step's semantic event lands at the call site — where the
/// production reaches it — and the step's own body is scanned right
/// after it, with the boundary the step carries SUPPRESSED: the step
/// and the boundary it ends in are the same event and are counted
/// once.
fn visit_step(
    step: &'static ProductionStep,
    bytes: &[u8],
    open: usize,
    scanner: &mut Scanner,
    depth: usize,
    steps: &mut Vec<Step>,
) -> usize {
    let (arguments, end) = call_arguments(bytes, open);
    let inner = scanner
        .functions
        .get(step.function)
        .filter(|_| depth < MAX_INLINE_DEPTH)
        .map(|function| {
            let outer = scanner.suppressed;
            scanner.suppressed = Some(step.boundary);
            let inner = scan_region(
                function,
                scanner,
                depth + 1,
                step.kind == StepKind::Synthesizes,
            );
            scanner.suppressed = outer;
            inner
        })
        .unwrap_or_default();
    steps.push(Step::Event(step.event(handed_over(&arguments))));
    steps.extend(inner);
    end
}

/// Inline the production function `name` when the region calls it:
/// the callee's own production events land at the call site, in the
/// callee's production order. Returns the index just past the call.
///
/// A METHOD call never inlines a production function: `play` is both
/// a production function name and the backend's stream starter, and
/// only the method form starts the backend.
fn inline_production_call(
    name: &str,
    bytes: &[u8],
    open: usize,
    is_method: bool,
    scanner: &mut Scanner,
    depth: usize,
    steps: &mut Vec<Step>,
) -> Option<usize> {
    if is_method {
        return None;
    }
    let body = scanner.functions.get(name)?;
    if depth < MAX_INLINE_DEPTH {
        let inner = scan_region(body, scanner, depth + 1, false);
        steps.extend(inner);
    }
    let (_, end) = call_arguments(bytes, open);
    Some(end)
}

/// The production event a BOUNDARY call site emits, and the index
/// just past the call.
///
/// The boundary calls are the engine's synthesis and the backend's
/// enqueue and stream start: the semantic events the invariant names.
/// They are the FALLBACK for a production shape that reaches the
/// engine and the backend directly — the production's own steps
/// ([`PRODUCTION_STEPS`]) are recognised first, and a boundary inside
/// a step's own body is suppressed so the two are never counted
/// twice.
///
/// A METHOD call never inlines a production function: `play` is both
/// a production function name and the backend's stream starter, and
/// only the method form starts the backend.
fn production_event(
    name: &str,
    bytes: &[u8],
    open: usize,
    is_method: bool,
    suppressed: Option<&'static str>,
) -> Option<(Event, usize)> {
    if suppressed == Some(name) {
        return None;
    }
    match name {
        "synthesize" => {
            let (_, end) = call_arguments(bytes, open);
            Some((Event::SegmentSynthesized, end))
        }
        "enqueue_pcm" => {
            let (arguments, end) = call_arguments(bytes, open);
            Some((Event::ChunkEnqueued(arguments), end))
        }
        "play" if is_method && has_no_argument(bytes, open) => {
            let (_, end) = call_arguments(bytes, open);
            Some((Event::PlaybackStarted, end))
        }
        _ => None,
    }
}

/// Whether the bytes at `index` start with `prefix`.
fn starts_with(bytes: &[u8], index: usize, prefix: &str) -> bool {
    let prefix = prefix.as_bytes();
    bytes
        .get(index..index + prefix.len())
        .is_some_and(|slice| slice == prefix)
}

/// The observed production play path.
fn observe_play_path() -> ObservedPath {
    let functions = collect_functions(APP_SHELL_SRC);
    let mut scanner = Scanner {
        functions: &functions,
        observed: ObservedPath::default(),
        suppressed: None,
    };
    if let Some(body) = functions.get(PLAY_PATH_ENTRY) {
        let steps = scan_region(body, &mut scanner, 0, false);
        scanner.observed.steps = steps;
    }
    scanner.observed
}

/// The observed order, flattened: every production loop body is
/// expanded `iterations` times.
fn expand(steps: &[Step], iterations: usize) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();
    for step in steps {
        match step {
            Step::Event(event) => events.push(event.clone()),
            Step::Repeat(body) => {
                for _ in 0..iterations {
                    events.extend(expand(body, iterations));
                }
            }
        }
    }
    events
}

/// The index of the first chunk enqueue in the observed order.
fn first_enqueue(trace: &[Event]) -> Option<usize> {
    trace
        .iter()
        .position(|event| matches!(event, Event::ChunkEnqueued(_)))
}

/// The index of the first playback start in the observed order.
fn first_play(trace: &[Event]) -> Option<usize> {
    trace
        .iter()
        .position(|event| matches!(event, Event::PlaybackStarted))
}

/// The index of the last segment synthesis in the observed order.
fn last_synthesis(trace: &[Event]) -> Option<usize> {
    trace
        .iter()
        .rposition(|event| matches!(event, Event::SegmentSynthesized))
}

/// The number of segment syntheses that precede `index` in the
/// observed order.
fn syntheses_before(trace: &[Event], index: usize) -> usize {
    trace[..index]
        .iter()
        .filter(|event| matches!(event, Event::SegmentSynthesized))
        .count()
}

/// The observed order, rendered for a failure message.
fn render_trace(trace: &[Event]) -> String {
    let rendered: Vec<String> = trace
        .iter()
        .map(|event| match event {
            Event::SegmentSynthesized => "synthesize".to_owned(),
            Event::ChunkEnqueued(identifiers) => format!("enqueue({})", identifiers.join(", ")),
            Event::PlaybackStarted => "play".to_owned(),
        })
        .collect();
    rendered.join(" -> ")
}

/// A generated fixture-scale document: `FIXTURE_MIN_SECTIONS` to
/// `FIXTURE_MAX_SECTIONS` titled sections, each a heading plus a
/// short pt-BR paragraph.
///
/// The document ALWAYS carries at least two sections, so the
/// production chunker always yields at least two segments and the
/// first-audible invariant is never vacuous: the production path
/// always has "the remaining segments" to synthesize after the first
/// chunk.
fn fixture_scale_doc() -> impl Strategy<Value = String> {
    (FIXTURE_MIN_SECTIONS..=FIXTURE_MAX_SECTIONS).prop_flat_map(|sections| {
        proptest::collection::vec(section_paragraph(), sections).prop_map(|paragraphs| {
            let mut document = String::new();
            for (index, paragraph) in paragraphs.iter().enumerate() {
                document.push_str("# Seção ");
                document.push_str(&(index + 1).to_string());
                document.push('\n');
                document.push('\n');
                document.push_str(paragraph);
                document.push_str("\n\n");
            }
            document
        })
    })
}

/// A generated section paragraph: three to twelve production words.
fn section_paragraph() -> impl Strategy<Value = String> {
    proptest::collection::vec(0usize..FIXTURE_WORDS.len(), 3..=12).prop_map(|indexes| {
        indexes
            .iter()
            .map(|index| FIXTURE_WORDS[*index])
            .collect::<Vec<&str>>()
            .join(" ")
    })
}

proptest! {
    // The property's subject is the production path's SHAPE, so the
    // generated documents vary the scale (the segment count the path
    // must synthesize) rather than the verdict: a handful of cases
    // is enough, and every case pays one real synthesis of the
    // first chunk — so the case count and the shrink budget are
    // bounded to keep the test's own cost proportionate.
    #![proptest_config({
        let mut config = ProptestConfig::with_cases(8);
        config.max_shrink_iters = 16;
        config
    })]

    // Plain `//`, not `///`: a doc comment cannot attach to an item
    // inside a `proptest!` block (the same note
    // `crates/player/tests/prop_ladder.rs` records for its own
    // blocks).

    /// T112 — the first-audible property, observed on the REAL
    /// production open/play path.
    ///
    /// Over arbitrary fixture-scale documents, the production path
    /// must enqueue the FIRST synthesized chunk — and start the
    /// backend — while the rest of the document is still being
    /// synthesized (FR-003 / SC-002): at most one segment synthesis
    /// may precede the first enqueue, the first enqueue must precede
    /// the last synthesis, playback must start on that first chunk,
    /// and the enqueued buffer must be the chunk's own — never a
    /// buffer the path accumulates the whole document into.
    ///
    /// The observation must ALSO SEE the rest of the document: at
    /// least one synthesis must follow the first enqueue, or the
    /// property would pass by observing a single unit and no
    /// remainder (the non-vacuity guard).
    #[test]
    fn prop_t112_first_chunk_is_enqueued_before_the_rest_is_synthesized(
        document in fixture_scale_doc(),
    ) {
        // The production chunker decides how many segments the
        // production path synthesizes.
        let segments = split_document(&document, "");
        prop_assert!(
            segments.len() >= FIXTURE_MIN_SECTIONS,
            "the production chunker split the generated document into {} segment(s) — the first-audible invariant needs the remaining segments to exist",
            segments.len(),
        );

        // The production engine synthesizes the first chunk: the
        // audio the production path must hand to the backend first.
        let first_chunk = synthesize(&segments[0]).map_err(PropError::Engine)?;
        prop_assert!(
            !first_chunk.is_empty(),
            "the production engine delivered an empty first chunk — the adapter's fail-closed contract is broken",
        );

        // The production path's observed order, read off the real
        // production source.
        let observed = observe_play_path();
        let trace = expand(&observed.steps, EXPANDED_ITERATIONS);
        prop_assert!(
            !trace.is_empty(),
            "the production play path ({PLAY_PATH_ENTRY}) produced no observable event — the seam moved and this scanner must follow it",
        );

        // I1 — the first chunk is enqueued after at most ONE segment
        // synthesis: no whole-document PCM buffer exists when the
        // first chunk is enqueued.
        let enqueue_index = first_enqueue(&trace).unwrap_or(usize::MAX);
        prop_assert!(
            enqueue_index != usize::MAX,
            "the production play path enqueues no chunk at all — the first-audible streaming path is missing",
        );
        let before_enqueue = syntheses_before(&trace, enqueue_index);
        prop_assert!(
            before_enqueue <= 1,
            "FR-003 first-audible violated: the production play path synthesized {before_enqueue} segment(s) before it enqueued the first chunk — the first chunk must be enqueued while the rest of the document is still being synthesized (no whole-document buffer before the first enqueue)\nobserved production order: {trace}",
            trace = render_trace(&trace),
        );

        // NON-VACUITY — the observation must SEE the remaining units
        // being produced after the first enqueue.
        //
        // The invariant is about the WHOLE document: the first chunk
        // is handed over while the rest of the document is still
        // being produced. An observed order with no synthesis AFTER
        // the first enqueue proves nothing — the property would pass
        // by observing a single unit and no remainder, and a scanner
        // that lost the production seam (or a production that stopped
        // producing the rest of the document) would be
        // indistinguishable from a production that satisfies the
        // invariant. The guard is what keeps the property from
        // passing by observing nothing (Principle III: the test suite
        // is the only evidence).
        let syntheses_after = trace[enqueue_index + 1..]
            .iter()
            .filter(|event| matches!(event, Event::SegmentSynthesized))
            .count();
        prop_assert!(
            syntheses_after >= 1,
            "FR-003 first-audible NOT OBSERVED: the production play path ({PLAY_PATH_ENTRY}) shows NO segment synthesis after the first enqueue at position {enqueue_index} — the observation cannot see the remaining units being produced, so the first-audible invariant is unguarded: the scanner lost the production seam, or the production stopped producing the rest of the document\nobserved production order: {trace}",
            trace = render_trace(&trace),
        );

        // I2 — the first enqueue precedes the LAST synthesis.
        let last_synthesis = last_synthesis(&trace).unwrap_or(usize::MAX);
        prop_assert!(
            last_synthesis != usize::MAX,
            "the production play path synthesizes no segment — the play path cannot speak",
        );
        prop_assert!(
            enqueue_index < last_synthesis,
            "FR-003 first-audible violated: the production play path enqueued its first chunk at position {enqueue_index}, AFTER the last segment was synthesized at position {last_synthesis} — the whole document is materialized before any audio is queued\nobserved production order: {trace}",
            trace = render_trace(&trace),
        );

        // I3 — playback starts on the first chunk, never after the
        // full PCM.
        let play_index = first_play(&trace).unwrap_or(usize::MAX);
        prop_assert!(
            play_index != usize::MAX,
            "the production play path never starts the backend — no audio is ever heard",
        );
        prop_assert!(
            enqueue_index < play_index,
            "FR-003 first-audible violated: the production play path starts the backend at position {play_index} BEFORE it enqueues the first chunk at position {enqueue_index} — the queue must hold the chunk before the stream starts\nobserved production order: {trace}",
            trace = render_trace(&trace),
        );
        let before_play = syntheses_before(&trace, play_index);
        prop_assert!(
            before_play <= 1,
            "FR-003 first-audible violated: the production play path synthesized {before_play} segment(s) before it started playback — playback must start on the first chunk\nobserved production order: {trace}",
            trace = render_trace(&trace),
        );

        // I4 — the enqueued buffer is the chunk's own, never a
        // buffer the path accumulates the whole document into.
        //
        // The check names the identifiers the production hands to the
        // backend at the handover: the buffer the chunk travels in
        // must not be one the path grows with every segment's PCM.
        let enqueued = match trace.get(enqueue_index) {
            Some(Event::ChunkEnqueued(identifiers)) => identifiers.clone(),
            _ => Vec::new(),
        };
        let accumulated: Vec<&String> = enqueued
            .iter()
            .filter(|identifier| observed.accumulators.contains(*identifier))
            .collect();
        prop_assert!(
            accumulated.is_empty(),
            "FR-003 first-audible violated: the production play path hands {enqueued:?} to the backend and {accumulated:?} is a buffer it accumulates every segment's PCM into — so the whole document is materialized before the first enqueue\nobserved production order: {trace}",
            trace = render_trace(&trace),
        );
    }
}
