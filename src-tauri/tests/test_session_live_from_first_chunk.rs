//! T142 — RED-FIRST property test: the session liveness window
//! (Feature 004-ressoa-dogfood-build, task T142).
//!
//! THE FINDING (F2, HIGH). The open path hands the FIRST synthesized
//! chunk to the audio backend and starts the stream on it, and only
//! AFTER the whole document has been synthesized does it install the
//! surface resolver, land the playing state, and publish the surface.
//! The audio therefore plays while the session still sits in its
//! initial state, and every mechanism that is gated on the playing
//! state is inert for that whole window:
//!
//! * the durable write (FR-007's 20 s heartbeat) is gated on the
//!   playing state, so a kill inside the window loses everything
//!   since the open;
//! * the in-RAM tracking (FR-002's ~1 s clock feed) is gated on the
//!   playing state, so the reading's offset never advances;
//! * the per-section diagnostic entries (FR-017) need the resolver,
//!   which is not installed yet, so no section index and no section
//!   entry exist;
//! * the lock-screen surface (FR-011, FR-014) is not published, so no
//!   surface exists while the audio plays.
//!
//! For the SC-001 fixture (about two hours of speech) that window is
//! minutes to tens of minutes of audible audio.
//!
//! MECHANISM. The ORDER of these operations is a PRODUCTION-SOURCE
//! property: the open path is not host-runnable (it needs the audio
//! backend and the Tauri app), so the suite reads the production
//! source and asserts the order of the real call sites — the same
//! discipline `src-tauri/tests/test_lockscreen_resume_control.rs` and
//! `crates/player/tests/test_earcon_mirror.rs` already use. The
//! production prefix (everything before the bridge's first inline
//! test module) is the input, so a test module's own text can never
//! satisfy an assertion.
//!
//! The suite therefore walks the production call graph: the open path
//! is the function that calls the streaming synthesis, and the first
//! audible chunk is the enqueue inside that synthesis. An obligation
//! is LIVE when its call site runs BEFORE that enqueue — directly in
//! the open path, inside the synthesis ahead of the enqueue, or in a
//! helper the open path reaches before it descends into the
//! synthesis.
//!
//! THE INVARIANTS.
//!
//! * From the FIRST enqueued chunk the session is live: the playing
//!   landing, the resolver's installation and the surface publish
//!   all run before the enqueue (FR-002, FR-007, FR-011, FR-013,
//!   FR-014, FR-017).
//! * The parsed-content unit total and the section count stay fixed
//!   at the session's start: the resolver is built ONCE, from the
//!   parse's own counts, before the first chunk (FR-013), and its
//!   projection never moves backward and never exceeds the fixed
//!   total.
//! * The stall rule still holds: a playing landing is always
//!   followed by a strictly greater offset — the audible credit is
//!   what advances it, and the playing state is what gates the
//!   credit (the diagnostic record's own scanner).
//!
//! SHELL-VOCABULARY NOTE. This file is outside the shell-purity
//! gate's vocabulary allowlist, so it names no shell-vocabulary word
//! anywhere — not even in prose, not even in a comment. The session's
//! own words are spoken of as "the state", "the reading offset" and
//! "the surface" instead. Keep it that way: the gate greps the WHOLE
//! file, and a word introduced here fails the gate for every other
//! task.

use std::collections::BTreeSet;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use adapter_synthesis::chunk::BoundaryKind;
use player::cli::{PlaybackSession, ShellVerb};
use player::surface::SurfaceResolver;
use proptest::prelude::*;

/// The Tauri bridge — the file that owns the open path, the streaming
/// synthesis, the session structure and the surface publish.
const RUST_BRIDGE: &str = "src-tauri/src/lib.rs";

/// The player crate's session source — the file that owns the closed
/// transition table and the audible-credit gate.
const PLAYER_SESSION: &str = include_str!("../../crates/player/src/cli.rs");

/// The name of the streaming synthesis — the whole-document loop the
/// open path descends into.
const SYNTHESIS: &str = "stream_synthesis";

/// The name of the per-chunk enqueue — the call that hands a chunk to
/// the backend's playback queue.
const ENQUEUE: &str = "enqueue_chunk";

/// The name of the durable write the heartbeat performs.
const DURABLE_WRITE: &str = "persist_bookmark";

/// The name of the tick that feeds the audible clock into the
/// session.
const CLOCK_FEED: &str = "feed_audible_progress";

/// The name of the heartbeat tick.
const HEARTBEAT: &str = "heartbeat_once";

/// The name of the watch tick.
const WATCH: &str = "playback_watch_tick";

/// The playing state, as the production source spells it.
const PLAYING: &str = "TransportState::Playing";

/// The verb that lands the playing state.
const PLAY_VERB: &str = "ShellVerb::Play";

// ===========================================================================
// The production source, read and analysed once per test binary.
// ===========================================================================

/// The repository root: the WORKSPACE manifest's parent — the
/// ancestor whose manifest declares `[workspace]`, not merely any
/// manifest (this crate's own manifest would otherwise match first
/// and resolve the root to `src-tauri/`).
fn repo_root() -> Result<PathBuf, Box<dyn Error>> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && std::fs::read_to_string(candidate.join("Cargo.toml"))
                    .is_ok_and(|text| text.contains("[workspace]"))
        })
        .map(Path::to_path_buf)
        .ok_or("the workspace root carries the manifest declaring [workspace]")?)
}

/// One repository source file, read whole.
fn read_source(relative: &str) -> Result<String, Box<dyn Error>> {
    Ok(std::fs::read_to_string(repo_root()?.join(relative))?)
}

/// The source with every comment removed, the line structure kept.
///
/// The contract assertions read CODE, never prose: a doc comment that
/// names a verb proves nothing about the route that carries it, and a
/// comment that names a state proves nothing about the gate that
/// reads it. A comment marker inside a string literal is not a
/// comment and is kept, so the scanner tracks the string state.
fn strip_comments(text: &str) -> String {
    let source: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while index < source.len() {
        let ch = source[index];
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            index += 1;
            continue;
        }
        if ch == '/' && index + 1 < source.len() {
            let next = source[index + 1];
            if next == '/' {
                while index < source.len() && source[index] != '\n' {
                    index += 1;
                }
                continue;
            }
            if next == '*' {
                index += 2;
                let mut depth = 1usize;
                while index < source.len() && depth > 0 {
                    if source[index] == '/' && index + 1 < source.len() && source[index + 1] == '*'
                    {
                        depth += 1;
                        index += 2;
                        continue;
                    }
                    if source[index] == '*' && index + 1 < source.len() && source[index + 1] == '/'
                    {
                        depth -= 1;
                        index += 2;
                        continue;
                    }
                    if source[index] == '\n' {
                        out.push('\n');
                    }
                    index += 1;
                }
                continue;
            }
        }
        out.push(ch);
        index += 1;
    }
    out
}

/// The production part of a Rust source: everything before its
/// first inline test module.
///
/// The contract reads PRODUCTION code — a test module may name any
/// call it likes, and a test module's mention of an obligation is not
/// a route the production takes.
fn production_prefix(text: &str) -> &str {
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("mod ") && trimmed.contains("_tests") && trimmed.ends_with('{') {
            return &text[..offset];
        }
        offset += line.len();
    }
    text
}

/// Whether a byte is part of an identifier.
const fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The offset of the bracket that closes the one at `open`, or
/// `None` when the source is unbalanced. A bracket inside a string
/// literal never closes anything.
const fn closing_bracket(source: &str, open: usize, opener: u8, closer: u8) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut index = open;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
        } else if byte == opener {
            depth += 1;
        } else if byte == closer {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

/// The offset of the brace that opens a declaration's body, or
/// `None` when the source carries no such brace.
const fn body_start(source: &str, after_name: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = after_name;
    let mut parens = 0usize;
    let mut brackets = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else {
            match byte {
                b'"' => in_string = true,
                b'(' => parens += 1,
                b')' => parens = parens.saturating_sub(1),
                b'[' => brackets += 1,
                b']' => brackets = brackets.saturating_sub(1),
                b'{' if parens == 0 && brackets == 0 => return Some(index),
                _ => {}
            }
        }
        index += 1;
    }
    None
}

/// A function declared in the production source, with the span of its
/// body.
#[derive(Debug, Clone)]
struct FnSpan {
    /// The declared name.
    name: String,
    /// The offset of the body's opening brace.
    body_start: usize,
    /// The offset just past the body's closing brace.
    body_end: usize,
}

/// Every function declared in the production source, with the span of
/// its body.
fn function_spans(source: &str) -> Vec<FnSpan> {
    let mut spans = Vec::new();
    let mut search_from = 0usize;
    while let Some(found) = source[search_from..].find("fn ") {
        let at = search_from + found;
        let name_start = at + 3;
        let rest = &source[name_start..];
        let name_len = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        search_from = name_start;
        if name_len == 0 {
            continue;
        }
        let name_end = name_start + name_len;
        let Some(open) = body_start(source, name_end) else {
            continue;
        };
        let Some(close) = closing_bracket(source, open, b'{', b'}') else {
            continue;
        };
        spans.push(FnSpan {
            name: source[name_start..name_end].to_string(),
            body_start: open,
            body_end: close + 1,
        });
        search_from = name_end;
    }
    spans
}

/// The innermost declared function that holds an offset.
fn enclosing(spans: &[FnSpan], offset: usize) -> Option<&FnSpan> {
    spans
        .iter()
        .filter(|span| span.body_start <= offset && offset < span.body_end)
        .max_by_key(|span| span.body_start)
}

/// A call site in the production source.
#[derive(Debug, Clone, Copy)]
struct CallSite {
    /// The offset of the callee identifier.
    name_at: usize,
    /// The offset of the argument list's opening parenthesis.
    open_paren: usize,
}

/// Every call in the production source: the callee's name and the
/// offset of the identifier that names it.
fn all_calls(source: &str) -> Vec<(String, usize)> {
    let bytes = source.as_bytes();
    let mut calls = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        let starts_a_word =
            is_word_byte(bytes[index]) && (index == 0 || !is_word_byte(bytes[index - 1]));
        if !starts_a_word {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && is_word_byte(bytes[index]) {
            index += 1;
        }
        let mut cursor = index;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor < bytes.len() && bytes[cursor] == b'(' {
            calls.push((source[start..index].to_string(), start));
        }
    }
    calls
}

/// Whether a call site is a declaration rather than a call: the
/// identifier is the name a `fn` declares.
fn is_declaration(source: &str, site: &CallSite) -> bool {
    source[..site.name_at].trim_end().ends_with("fn")
}

/// The argument region of a call site: the text between the
/// parentheses of its argument list.
fn argument_region<'a>(source: &'a str, site: &CallSite) -> &'a str {
    let Some(close) = closing_bracket(source, site.open_paren, b'(', b')') else {
        return "";
    };
    &source[site.open_paren + 1..close]
}

/// The call sites of `name` in the production source.
fn call_sites(source: &str, name: &str) -> Vec<CallSite> {
    let bytes = source.as_bytes();
    let needle = name.as_bytes();
    let mut sites = Vec::new();
    let mut index = 0usize;
    while index + needle.len() <= bytes.len() {
        if &bytes[index..index + needle.len()] == needle {
            let starts_a_word = index > 0 && is_word_byte(bytes[index - 1]);
            let mut cursor = index + needle.len();
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if !starts_a_word && cursor < bytes.len() && bytes[cursor] == b'(' {
                sites.push(CallSite {
                    name_at: index,
                    open_paren: cursor,
                });
                index = cursor + 1;
                continue;
            }
        }
        index += 1;
    }
    sites
}

// ===========================================================================
// The liveness obligations and the production analysis.
// ===========================================================================

/// The liveness obligations of the open path: the three operations
/// that must have run before the first chunk reaches the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Obligation {
    /// The landing on the playing state — what makes the durable
    /// write, the in-RAM tracking and the section entries live.
    PlayingLanding,
    /// The installation of the surface resolver — what the progress
    /// projection and the section entries read.
    ResolverInstall,
    /// The publish of the surface — the lock-screen surface.
    SurfacePublish,
}

impl Obligation {
    /// Every obligation, in a stable order.
    const ALL: [Self; 3] = [
        Self::PlayingLanding,
        Self::ResolverInstall,
        Self::SurfacePublish,
    ];

    /// The obligation's name, for the failure messages.
    const fn label(self) -> &'static str {
        match self {
            Self::PlayingLanding => "the landing on the playing state",
            Self::ResolverInstall => "the surface resolver's installation",
            Self::SurfacePublish => "the surface publish",
        }
    }
}

/// The production analysis: the open path, the first-chunk anchor,
/// and the call sites each obligation owns.
#[derive(Debug)]
struct Analysis {
    /// Every declared function, with its body span.
    spans: Vec<FnSpan>,
    /// The open path's declared name.
    open_name: String,
    /// The offset of the synthesis call inside the open path.
    synthesis_at: usize,
    /// The offset of the first chunk's enqueue inside the synthesis.
    enqueue_at: usize,
    /// The functions whose bodies run before the first chunk reaches
    /// the backend.
    live: BTreeSet<String>,
    /// The call sites of each obligation, by obligation.
    sites: Vec<(Obligation, Vec<CallSite>)>,
    /// The offsets of the resolver's construction sites.
    constructions: Vec<usize>,
}

impl Analysis {
    /// The call sites an obligation owns.
    fn sites_of(&self, obligation: Obligation) -> &[CallSite] {
        self.sites
            .iter()
            .find(|(owned, _)| *owned == obligation)
            .map_or(&[], |(_, sites)| sites.as_slice())
    }

    /// Whether a call site runs before the first chunk reaches the
    /// backend.
    fn runs_before_the_first_chunk(&self, site: &CallSite) -> bool {
        let Some(span) = enclosing(&self.spans, site.name_at) else {
            return false;
        };
        if span.name == self.open_name {
            return site.name_at < self.synthesis_at;
        }
        if span.name == SYNTHESIS {
            return site.name_at < self.enqueue_at;
        }
        self.live.contains(&span.name)
    }

    /// Where a call site sits relative to the first chunk, in words.
    fn describe(&self, site: &CallSite) -> String {
        let Some(span) = enclosing(&self.spans, site.name_at) else {
            return format!("at offset {} (in no declared function)", site.name_at);
        };
        if span.name == self.open_name {
            let relation = if site.name_at < self.synthesis_at {
                "BEFORE the synthesis call"
            } else {
                "AFTER the synthesis call"
            };
            return format!(
                "in the open path `{}` at offset {}, {} (the synthesis call is at offset {})",
                self.open_name, site.name_at, relation, self.synthesis_at
            );
        }
        if span.name == SYNTHESIS {
            let relation = if site.name_at < self.enqueue_at {
                "BEFORE the first chunk's enqueue"
            } else {
                "AFTER the first chunk's enqueue"
            };
            return format!(
                "in `{}` at offset {}, {} (the enqueue is at offset {})",
                span.name, site.name_at, relation, self.enqueue_at
            );
        }
        if self.live.contains(&span.name) {
            return format!(
                "in `{}` at offset {}, a function the open path reaches before it descends into \
                 the synthesis",
                span.name, site.name_at
            );
        }
        format!(
            "in `{}` at offset {}, a function the open path reaches only after the synthesis",
            span.name, site.name_at
        )
    }
}

/// The functions the named function calls before the given offset of
/// its own body — the callees that are declared in the production
/// source.
fn callees(
    source: &str,
    spans: &[FnSpan],
    declared: &BTreeSet<&str>,
    name: &str,
    until: usize,
) -> Vec<String> {
    let Some(span) = spans.iter().find(|span| span.name == name) else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for (callee, offset) in all_calls(source) {
        if offset < span.body_start || offset >= span.body_end || offset >= until {
            continue;
        }
        if !declared.contains(callee.as_str()) || callee == name {
            continue;
        }
        if !out.contains(&callee) {
            out.push(callee);
        }
    }
    out
}

/// The functions whose bodies run before the first chunk reaches the
/// backend: the open path's own calls made ahead of the synthesis,
/// the synthesis's own calls made ahead of the enqueue, and every
/// function those reach in turn.
fn live_before_the_first_chunk(
    source: &str,
    spans: &[FnSpan],
    open_name: &str,
    synthesis_at: usize,
    enqueue_at: usize,
) -> BTreeSet<String> {
    let declared: BTreeSet<&str> = spans.iter().map(|span| span.name.as_str()).collect();
    let mut live: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();
    queue.extend(callees(source, spans, &declared, open_name, synthesis_at));
    queue.extend(callees(source, spans, &declared, SYNTHESIS, enqueue_at));
    while let Some(name) = queue.pop() {
        if live.contains(&name) {
            continue;
        }
        let _inserted = live.insert(name.clone());
        queue.extend(callees(source, spans, &declared, &name, usize::MAX));
    }
    live
}

/// The call sites of an obligation in the production source.
fn obligation_sites(source: &str, spans: &[FnSpan], obligation: Obligation) -> Vec<CallSite> {
    match obligation {
        Obligation::PlayingLanding => call_sites(source, "execute")
            .into_iter()
            .filter(|site| !is_declaration(source, site))
            .filter(|site| argument_region(source, site).contains(PLAY_VERB))
            .collect(),
        Obligation::ResolverInstall => {
            let mut sites = Vec::new();
            for span in spans {
                if !(span.name.contains("install") || span.name.contains("structure")) {
                    continue;
                }
                for site in call_sites(source, &span.name) {
                    if is_declaration(source, &site) {
                        continue;
                    }
                    if argument_region(source, &site).contains("resolver") {
                        sites.push(site);
                    }
                }
            }
            sites.extend(resolver_writes(source));
            sites
        }
        Obligation::SurfacePublish => call_sites(source, "publish_current_state")
            .into_iter()
            .filter(|site| !is_declaration(source, site))
            .collect(),
    }
}

/// The offsets of the resolver's construction sites.
fn resolver_constructions(source: &str) -> Vec<usize> {
    let mut sites = Vec::new();
    let mut search_from = 0usize;
    while let Some(found) = source[search_from..].find("SurfaceResolver::new") {
        let at = search_from + found;
        sites.push(at);
        search_from = at + 1;
    }
    sites
}

/// The offsets of the writes that hand the resolver to the shared
/// state — the installation itself, whether it runs through a helper
/// or is written inline.
fn resolver_writes(source: &str) -> Vec<CallSite> {
    let mut sites = Vec::new();
    let mut search_from = 0usize;
    while let Some(found) = source[search_from..].find("= Some(resolver)") {
        let at = search_from + found;
        sites.push(CallSite {
            name_at: at,
            open_paren: at,
        });
        search_from = at + 1;
    }
    sites
}

/// The production analysis, computed once per test binary.
fn analysis() -> Result<&'static Analysis, String> {
    static ANALYSIS: OnceLock<Result<Analysis, String>> = OnceLock::new();
    let slot = ANALYSIS.get_or_init(|| {
        let production = production_source()?;
        analyse(&production)
    });
    match slot {
        Ok(analysis) => Ok(analysis),
        Err(reason) => Err(reason.clone()),
    }
}

/// The bridge's production prefix — the comment-free source up to the
/// first inline test module — read once per test binary.
fn production_source() -> Result<String, String> {
    static SOURCE: OnceLock<Result<String, String>> = OnceLock::new();
    let slot = SOURCE.get_or_init(|| {
        let raw = read_source(RUST_BRIDGE).map_err(|error| error.to_string())?;
        Ok(strip_comments(&raw))
    });
    match slot {
        Ok(source) => Ok(production_prefix(source).to_string()),
        Err(reason) => Err(reason.clone()),
    }
}

/// Analyse a production prefix: the open path, the first-chunk
/// anchor, and the call sites each obligation owns.
fn analyse(production: &str) -> Result<Analysis, String> {
    let production = production.to_string();
    let spans = function_spans(&production);
    if spans.is_empty() {
        return Err("the production prefix declares no function".to_string());
    }

    let synthesis_sites: Vec<CallSite> = call_sites(&production, SYNTHESIS)
        .into_iter()
        .filter(|site| !is_declaration(&production, site))
        .collect();
    if synthesis_sites.len() != 1 {
        return Err(format!(
            "the production prefix calls the streaming synthesis {synthesis_sites:?} times — exactly one open path is expected"
        ));
    }
    let synthesis_site = synthesis_sites[0];
    let Some(open_span) = enclosing(&spans, synthesis_site.name_at) else {
        return Err("the synthesis call belongs to no declared function".to_string());
    };

    let enqueue_sites: Vec<CallSite> = call_sites(&production, ENQUEUE)
        .into_iter()
        .filter(|site| !is_declaration(&production, site))
        .filter(|site| enclosing(&spans, site.name_at).is_some_and(|span| span.name == SYNTHESIS))
        .collect();
    if enqueue_sites.len() != 1 {
        return Err(format!(
            "the streaming synthesis enqueues a chunk {enqueue_sites:?} times — exactly one first-chunk enqueue is expected"
        ));
    }
    let enqueue_at = enqueue_sites[0].name_at;

    let open_name = open_span.name.clone();
    let synthesis_at = synthesis_site.name_at;
    let live =
        live_before_the_first_chunk(&production, &spans, &open_name, synthesis_at, enqueue_at);
    let sites = Obligation::ALL
        .iter()
        .map(|obligation| {
            (
                *obligation,
                obligation_sites(&production, &spans, *obligation),
            )
        })
        .collect();
    let constructions = resolver_constructions(&production);

    Ok(Analysis {
        spans,
        open_name,
        synthesis_at,
        enqueue_at,
        live,
        sites,
        constructions,
    })
}

// ===========================================================================
// The properties.
// ===========================================================================

/// A markdown document: a sequence of lines drawn from a small
/// vocabulary of headings and sentences.
fn document_strategy() -> impl Strategy<Value = String> {
    proptest::collection::vec(0u8..5, 1..24).prop_map(|codes| {
        let mut document = String::new();
        for code in codes {
            match code {
                0 => document.push_str("# Capítulo Um\n"),
                1 => document.push_str("## Seção Interna\n"),
                2 => document.push_str("A primeira frase do parágrafo.\n"),
                3 => document.push_str("A segunda frase do parágrafo.\n"),
                _ => document.push('\n'),
            }
        }
        document
    })
}

/// The production analysis, or a failed test case.
fn analysis_or_fail() -> Result<&'static Analysis, TestCaseError> {
    analysis().map_err(TestCaseError::fail)
}

// T142 RED: from the FIRST enqueued chunk the session MUST be live
// (FR-002, FR-007, FR-011, FR-013, FR-014, FR-017).
//
// The three obligations — the landing on the playing state, the
// resolver's installation and the surface publish — must all run
// BEFORE the first chunk reaches the backend. A declaration proves
// nothing: the obligation's CALL SITE must run first, so the
// assertion walks the production call graph rather than matching a
// name.
//
// Today the open path runs the whole-document synthesis first and
// only then lands the playing state, installs the resolver and
// publishes the surface — so the audio plays with no live session
// behind it, and the assertion fails.
proptest! {
    #[test]
    fn prop_the_session_is_live_before_the_first_chunk_is_enqueued(
        picks in proptest::collection::vec(0usize..3, 1..=3),
    ) {
        let analysis = analysis_or_fail()?;
        let picked: BTreeSet<usize> = picks.into_iter().collect();
        prop_assert!(!picked.is_empty(), "the property picks at least one obligation");
        for index in picked {
            let obligation = Obligation::ALL[index];
            let sites = analysis.sites_of(obligation);
            prop_assert!(
                !sites.is_empty(),
                "the production source names no call site for {} — the open path must perform it",
                obligation.label()
            );
            let live = sites
                .iter()
                .any(|site| analysis.runs_before_the_first_chunk(site));
            prop_assert!(
                live,
                "{} MUST run before the first chunk reaches the backend (FR-002, FR-007, \
                 FR-011, FR-013, FR-014, FR-017): the production open path `{}` reaches the \
                 streaming synthesis at offset {}, and the streaming synthesis enqueues the \
                 first chunk at offset {}. The obligation runs {:?} — after the audio already \
                 plays. A session that is not live yet writes nothing durably, advances no \
                 offset, emits no section entry and publishes no surface.",
                obligation.label(),
                analysis.open_name,
                analysis.synthesis_at,
                analysis.enqueue_at,
                sites
                    .iter()
                    .map(|site| analysis.describe(site))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
    }
}

// T142 RED: the parsed-content unit total and the section count stay
// fixed at the session's start (FR-013).
//
// The resolver is built ONCE per session, from the parse's own
// counts, and the build runs before the first chunk — the totals are
// content facts, never re-estimated mid-session. The projection the
// resolver owns never moves backward and never exceeds the fixed
// total.
proptest! {
    #[test]
    fn prop_the_parsed_content_totals_are_fixed_at_session_start(
        document in document_strategy(),
    ) {
        let analysis = analysis_or_fail()?;

        // The resolver is fed once: a second construction would be a
        // second total, and a total that changes mid-session is the
        // backward jump FR-013 forbids.
        prop_assert_eq!(
            analysis.constructions.len(),
            1,
            "the surface resolver MUST be built exactly once per session (FR-013) — the \
             production source builds it at offsets {:?}",
            analysis.constructions
        );
        let construction = analysis.constructions[0];
        let construction_site = CallSite {
            name_at: construction,
            open_paren: construction,
        };
        prop_assert!(
            analysis.runs_before_the_first_chunk(&construction_site),
            "the surface resolver MUST be built before the first chunk reaches the backend \
             (FR-013, FR-017): the production open path `{}` reaches the streaming synthesis at \
             offset {} and the streaming synthesis enqueues the first chunk at offset {}, while \
             the resolver is built {}. A resolver built after the audio already plays leaves the \
             progress projection and the section index absent for the whole window.",
            analysis.open_name,
            analysis.synthesis_at,
            analysis.enqueue_at,
            analysis.describe(&construction_site)
        );

        // The totals are the parse's own counts: the unit total is
        // the segment count and the section count is the count of
        // the heading segments.
        let segments = adapter_synthesis::chunk::split_document(&document, "");
        let total_units = u64::try_from(segments.len()).unwrap_or(u64::MAX);
        let section_count = segments
            .iter()
            .filter(|segment| segment.boundary_kind == BoundaryKind::Heading)
            .count();

        // The projection the resolver owns: monotone, and never past
        // the fixed total.
        let units = u32::try_from(total_units).unwrap_or(u32::MAX);
        let duration_secs = f64::from(units.min(4_096)).mul_add(1.5, 12.0);
        let resolver = SurfaceResolver::new(
            Some(String::from("Documento")),
            String::from("documento.md"),
            total_units,
            duration_secs,
            section_count > 0,
        );
        let mut previous = 0u64;
        for step in 0..=units.clamp(1, 64) {
            let at = duration_secs * (f64::from(step) / f64::from(units.clamp(1, 64)));
            let consumed = resolver.consumed_units_at(at);
            prop_assert!(
                consumed >= previous,
                "the content-offset projection MUST never move backward (FR-013): at {at} seconds \
                 it reported {consumed} units after {previous}"
            );
            prop_assert!(
                consumed <= total_units,
                "the content-offset projection MUST never pass the session-start unit total \
                 (FR-013): at {at} seconds it reported {consumed} of {total_units} units"
            );
            previous = consumed;
        }
        prop_assert_eq!(
            resolver.consumed_units_at(duration_secs * 4.0),
            total_units,
            "the content-offset projection MUST clamp at the session-start unit total (FR-013)"
        );
    }
}

// T142: the stall rule still holds — a playing landing is always
// followed by a strictly greater offset.
//
// The audible credit is the only advance path, and the playing state
// is what gates it. So the credit is inert outside the playing
// state, advances the offset inside it, and the closed transition
// table's playing cell is what a landing reaches.
proptest! {
    #[test]
    fn prop_a_playing_landing_is_followed_by_a_strictly_greater_offset(
        credit in 0.001f64..60.0,
    ) {
        // Outside the playing state the credit changes nothing: the
        // advance is gated on the playing state.
        let mut silent = PlaybackSession::new();
        let before_silent = silent.snapshot();
        silent.on_audible_progress(credit);
        let after_silent = silent.snapshot();
        prop_assert!(
            before_silent == after_silent,
            "an audible credit outside the playing state MUST NOT advance the reading offset \
             (FR-002): the session reported {:?} before the credit and {:?} after it",
            &before_silent,
            &after_silent
        );

        // The landing reaches the playing state: the closed table
        // accepts the play verb from the session's initial state and
        // refuses it from the playing cell.
        let mut session = PlaybackSession::new();
        prop_assert!(
            session.execute(ShellVerb::Play).is_ok(),
            "the play verb MUST be accepted from the session's initial state (FR-020)"
        );
        prop_assert!(
            session.execute(ShellVerb::Play).is_err(),
            "the play verb MUST be refused from the playing cell — the landing reached the playing \
             state (FR-020)"
        );

        // Inside the playing state the credit advances the offset,
        // and the credit only ever adds.
        let before = session.snapshot();
        session.on_audible_progress(credit);
        let after = session.snapshot();
        prop_assert!(
            before != after,
            "a playing landing MUST be followed by a strictly greater reading offset (the \
             diagnostic record's stall rule): the session reported {:?} before the credit and \
             {:?} after it",
            before,
            after
        );

        // The gate the credit path itself carries: the session
        // advances the clock only while the playing state holds.
        let gate = PLAYER_SESSION
            .find("pub fn on_audible_progress")
            .unwrap_or(usize::MAX);
        prop_assert!(
            gate < PLAYER_SESSION.len(),
            "the player session MUST own the audible-credit path"
        );
        let body = &PLAYER_SESSION[gate..];
        let playing_gate = body.find(PLAYING).unwrap_or(usize::MAX);
        prop_assert!(
            playing_gate < body.len(),
            "the session's audible-credit path MUST be gated on the playing state (FR-002): the \
             production player session names no {PLAYING} in its credit path"
        );
    }
}

// T142: the durable write and the in-RAM tracking are gated on the
// playing state.
//
// This is the mechanism the finding turns on: until the open path
// lands the playing state, the heartbeat writes nothing durably and
// the watch tick advances nothing.
proptest! {
    #[test]
    fn prop_the_durable_write_and_the_in_ram_tracking_are_gated_on_the_playing_state(
        picks in proptest::collection::vec(0usize..2, 1..=2),
    ) {
        let production = production_source().map_err(TestCaseError::fail)?;

        for index in picks {
            let (tick, work) = match index {
                0 => (HEARTBEAT, DURABLE_WRITE),
                _ => (WATCH, CLOCK_FEED),
            };
            let body = production
                .find(&format!("fn {tick}("))
                .map(|at| &production[at..])
                .ok_or_else(|| {
                    TestCaseError::fail(format!("the production source names no {tick}"))
                })?;
            let gate = body.find(PLAYING).unwrap_or(usize::MAX);
            let work_site = body.find(work).unwrap_or(usize::MAX);
            prop_assert!(
                gate < body.len(),
                "{tick} MUST be gated on the playing state (FR-002, FR-007): the production \
                 source names no {PLAYING} in it"
            );
            prop_assert!(
                work_site < body.len(),
                "{tick} MUST perform {work}: the production source names no {work} in it"
            );
            prop_assert!(
                gate < work_site,
                "{tick} MUST read the playing state BEFORE it performs {work} (FR-002, FR-007): \
                 the gate sits at offset {gate} and the work at offset {work_site}"
            );
        }
    }
}
