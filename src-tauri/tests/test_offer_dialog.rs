//! T116 — Phase 16 RED-FIRST contract test: the in-app resume
//! offer surface (feature `004-ressoa-dogfood-build`).
//!
//! The finding (tasks.md F2): `src-tauri/src/status_view.html`
//! has ZERO hits for `offer` — the single in-app screen has no
//! resume-offer dialog, no frontend code reads the
//! `offer_route` / `offer_window_ms` the shell publishes
//! (`src-tauri/src/lib.rs:2646-2668`, `:4238`), and no code
//! path anywhere sends the `resume_at_stored` / `start_over`
//! verbs (only the verb-name constants exist, in
//! `MediaMirror.kt:366,370`). The consequence: "start over" is
//! unreachable and the deliberate-pause offer resolves only
//! through the automatic 3-second silence rule (FR-009).
//!
//! The contract this file pins (FR-009, FR-011, FR-019 and
//! `specs/004-ressoa-dogfood-build/contracts/surface.md` —
//! "Resume: deliberate-pause reopen → 3 s visual dialog
//! (continue vs start-over; silence = continue from stored)"):
//!
//! 1. **The surface consumes the published offer shape.** The
//!    shell publishes the route key `offer_route` (the
//!    cold-start open payload and the `offer_route` command's
//!    reply) and the FR-032c window; the surface must read the
//!    route, name the in-app route token `in_app_dialog` — the
//!    only route that opens a dialog (the `lockscreen_resume`
//!    route opens NO in-app dialog, per the same contract), read
//!    the published window, and carry a semantic dialog
//!    element.
//! 2. **Exactly two choices.** The offer-choice verbs the
//!    surface sends are exactly `resume_at_stored` (continue
//!    from the stored position) and `start_over` (begin at the
//!    first spoken position) — and nothing beyond the
//!    transport/rate controls FR-019 and FR-012 name.
//! 3. **Purity.** The offer surface paints no document prose,
//!    no numeric rate value, no voice identity, and no
//!    elapsed/remaining time (FR-011 / FR-019).
//! 4. **The 3-second bound, owned by the shell.** The window
//!    the surface consumes is the shell's own 3-second offer
//!    window, and only the deliberate pause routes to the in-app
//!    dialog — the unexpected end waits paused for the
//!    lock-screen resume control (FR-009, FR-031).
//!
//! The mechanism (the surface is plain HTML with no JavaScript
//! test harness): a static contract assertion over the surface
//! source — the same mechanism `test_noscope.rs` uses for its
//! HTML guard — plus the Rust-side assertions over the
//! production dispatcher (`commands::parse_action`), the offer
//! machine (`session::offer`), and the shell's published offer
//! shape. Every assertion is an INVARIANT, never a literal HTML
//! snapshot: the surface is free to name its elements as it
//! likes, as long as it handles the offer shape, offers exactly
//! the two closed choices, and paints nothing prohibited.
//!
//! T151 — the literal scanner this file reads the surface with is
//! COMMENT-AWARE and REGION-AWARE. The surface is one HTML file
//! with three regions (markup, a `<style>` block, a `<script>`
//! block), and each carries its own comment syntax and its own
//! quoting rules. A scanner that treats every quote in the file as
//! a delimiter pairs an apostrophe inside a comment (or inside HTML
//! prose) with the NEXT quote and swallows the verb literals behind
//! it — the verbs then vanish from the "exactly two choices"
//! assertion, which weakens the contract SILENTLY. The negative
//! controls at the end of this file prove the property: an
//! apostrophe in a comment opens no literal, and the offer verbs
//! behind the comment are still found.

use std::collections::BTreeSet;

use engine_core::closed::SessionEnd;
use player::cli::ShellVerb;
use session::offer::{ColdStart, ExpiryOutcome, OfferMachine, OpenOffer, cold_start};
use session::store::StoredPosition;
use src_tauri::commands::{BridgeAction, parse_action};

/// The in-app surface — the single screen the reader reads on
/// (FR-019).
const STATUS_VIEW: &str = include_str!("../src/status_view.html");

/// The shell source — the publisher of the offer shape the
/// surface must consume.
const SHELL_SOURCE: &str = include_str!("../src/lib.rs");

/// The transport and rate control verbs the in-app screen owns:
/// FR-019's transport controls (play, pause, skip-forward/back,
/// and the dedicated Stop) plus FR-012's value-blind speed
/// stepper. They are the surface's own vocabulary, never the
/// offer's.
const CONTROL_VERBS: [&str; 8] = [
    "play",
    "pause",
    "stop",
    "skip_forward5s",
    "skip_back5s",
    "rate_up",
    "rate_down",
    "repeat_section",
];
/// The two closed offer-choice verbs (FR-009): continue from
/// the stored position, and start over from the first spoken
/// position. The set is closed — a third choice does not exist.
const OFFER_CHOICE_VERBS: [&str; 2] = ["resume_at_stored", "start_over"];

/// The numeric rate values the surface may never paint (FR-012
/// value-blind; the same convention `test_purity.rs` uses).
const RATE_VALUE_STRINGS: [&str; 7] = ["0.75x", "1.0x", "1.25x", "1.5x", "1.75x", "2.0x", "4.0x"];

/// The identifiers no painted line may carry (FR-011 /
/// FR-019): the numeric rate value, a reading-time field, a
/// voice identity, and a document-prose field. The list is
/// matched EXACTLY, so a ladder position (`rateIndex`) and a
/// control element (`btnRateUp`) are not rate values.
const PROHIBITED_PAINT_IDENTIFIERS: [&str; 16] = [
    "rate",
    "positionSecs",
    "position_secs",
    "elapsed",
    "remaining",
    "durationSecs",
    "duration_secs",
    "voice",
    "voiceId",
    "engine",
    "engineId",
    "prose",
    "paragraph",
    "bodyText",
    "documentText",
    "docText",
];

/// The time words no painted line may carry (pt-BR and
/// English): an elapsed or remaining reading time.
const PROHIBITED_PAINT_WORDS: [&str; 5] = [
    "elapsed",
    "remaining",
    "restante",
    "decorrido",
    "transcorrido",
];

/// Every string literal the source holds — single-quoted,
/// double-quoted, and template literals (an embedded `${…}`
/// expression is scanned in turn, so a verb inside one is still
/// found).
///
/// The scan is COMMENT-AWARE and REGION-AWARE (T151): the surface
/// is one HTML file with three regions, and each region carries its
/// own comment syntax and its own quoting rules. A scanner that
/// treats every quote in the file as a delimiter pairs an
/// apostrophe inside a comment (or inside HTML prose) with the NEXT
/// quote and swallows the verb literals behind it — a silently
/// weakened assertion, the dangerous direction.
fn string_literals(source: &str) -> Vec<String> {
    Scanner::new(source, Region::Markup).scan()
}

/// The region of the surface a character belongs to.
///
/// The three regions carry DIFFERENT comment syntaxes and
/// DIFFERENT quoting rules, so a scan that applies one rule to the
/// whole file mis-pairs the delimiters (T151 / F9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Region {
    /// HTML markup: `<!-- … -->` comments; a quote delimits an
    /// ATTRIBUTE VALUE inside a tag — in text content a quote is
    /// prose, never a delimiter.
    Markup,
    /// A `<style>` block: CSS. `/* … */` comments (never `//`);
    /// `'` and `"` delimit strings.
    Style,
    /// A `<script>` block: `JavaScript`. `//` and `/* … */`
    /// comments; `'`, `"` and `` ` `` delimit strings.
    Script,
}

/// The comment the scanner is skipping, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Comment {
    /// An HTML comment: `<!-- … -->`, across lines.
    Html,
    /// A block comment: `/* … */`. The depth counts the nesting —
    /// `JavaScript` nests block comments, CSS does not.
    Block { depth: usize },
    /// A line comment: `// …` up to the end of the line.
    Line,
}

/// The comment-aware literal scanner over the surface.
#[derive(Debug)]
struct Scanner {
    /// The source characters — the scanner walks characters, so a
    /// multi-byte character never splits a delimiter.
    chars: Vec<char>,
    /// The index of the next character to read.
    index: usize,
    /// The region the scanner reads.
    region: Region,
    /// The comment being skipped, if any.
    comment: Option<Comment>,
    /// Whether the scanner reads inside an HTML tag (the markup
    /// region): inside a tag a quote delimits an attribute value; in
    /// text content a quote is prose.
    in_tag: bool,
    /// The name of the tag being read (the markup region) — the tag
    /// decides which region follows it.
    tag_name: String,
    /// Whether the scanner is still reading the tag NAME: the name
    /// ends at the first character that cannot be part of it.
    reading_name: bool,
    /// The literals found so far.
    literals: Vec<String>,
}

impl Scanner {
    /// A scanner over `source`, reading the given region.
    fn new(source: &str, region: Region) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            region,
            comment: None,
            in_tag: false,
            tag_name: String::new(),
            reading_name: false,
            literals: Vec::new(),
        }
    }

    /// Read the whole source and return the literals it holds.
    fn scan(mut self) -> Vec<String> {
        while self.index < self.chars.len() {
            if self.comment.is_some() {
                self.skip_comment();
            } else {
                self.step();
            }
        }
        self.literals
    }

    /// Read one position outside any comment.
    fn step(&mut self) {
        let ch = self.chars[self.index];
        match self.region {
            Region::Markup => self.step_markup(ch),
            Region::Style => self.step_style(ch),
            Region::Script => self.step_script(ch),
        }
    }

    /// Read one position of the HTML markup: a comment, a tag, or
    /// text content — where a quote is prose, never a delimiter.
    fn step_markup(&mut self, ch: char) {
        if self.in_tag {
            self.step_tag(ch);
            return;
        }
        if ch != '<' {
            self.index += 1;
            return;
        }
        if self.starts_with("<!--") {
            self.comment = Some(Comment::Html);
            self.index += 4;
            return;
        }
        if self.starts_with("<!") || self.starts_with("<?") {
            self.skip_to_tag_end();
            return;
        }
        if self.starts_with("</") {
            self.skip_to_tag_end();
            return;
        }
        if self.starts_with_alpha() {
            self.in_tag = true;
            self.tag_name.clear();
            self.reading_name = true;
            self.index += 1;
            return;
        }
        self.index += 1;
    }

    /// Read one position inside an HTML tag: the attribute values
    /// (each a literal) and the tag end.
    fn step_tag(&mut self, ch: char) {
        match ch {
            '"' | '\'' => {
                let literal = self.take_quoted(ch);
                self.literals.push(literal);
            }
            '>' => {
                self.in_tag = false;
                self.index += 1;
                self.enter_tagged_region();
            }
            _ => {
                if self.reading_name {
                    if ch.is_ascii_alphanumeric() || ch == '-' {
                        self.tag_name.push(ch);
                    } else {
                        self.reading_name = false;
                    }
                }
                self.index += 1;
            }
        }
    }

    /// Switch the region when the tag that just closed opens a
    /// `<style>` or a `<script>` block.
    fn enter_tagged_region(&mut self) {
        if self.tag_name.eq_ignore_ascii_case("style") {
            self.region = Region::Style;
        } else if self.tag_name.eq_ignore_ascii_case("script") {
            self.region = Region::Script;
        }
        self.tag_name.clear();
        self.reading_name = false;
    }

    /// Read one position of a `<style>` block (CSS): the end of the
    /// block, a comment, or a string.
    fn step_style(&mut self, ch: char) {
        if ch == '<' && self.starts_with_ignore_case("</style") {
            self.region = Region::Markup;
            self.index += 1;
            return;
        }
        if self.starts_with("/*") {
            self.comment = Some(Comment::Block { depth: 1 });
            self.index += 2;
            return;
        }
        match ch {
            '"' | '\'' => {
                let literal = self.take_quoted(ch);
                self.literals.push(literal);
            }
            _ => self.index += 1,
        }
    }

    /// Read one position of a `<script>` block (`JavaScript`): the
    /// end of the block, a comment, a string, or a template literal.
    fn step_script(&mut self, ch: char) {
        if ch == '<' && self.starts_with_ignore_case("</script") {
            self.region = Region::Markup;
            self.index += 1;
            return;
        }
        if self.starts_with("//") {
            self.comment = Some(Comment::Line);
            self.index += 2;
            return;
        }
        if self.starts_with("/*") {
            self.comment = Some(Comment::Block { depth: 1 });
            self.index += 2;
            return;
        }
        match ch {
            '"' | '\'' => {
                let literal = self.take_quoted(ch);
                self.literals.push(literal);
            }
            '`' => self.take_template(),
            _ => self.index += 1,
        }
    }

    /// Read one position inside a comment: the comment ends at its
    /// own terminator — `-->` for an HTML comment, `*/` for a block
    /// comment, the newline for a line comment. A quote inside a
    /// comment opens NO literal.
    fn skip_comment(&mut self) {
        match self.comment {
            Some(Comment::Html) => {
                if self.starts_with("-->") {
                    self.comment = None;
                    self.index += 3;
                } else {
                    self.index += 1;
                }
            }
            Some(Comment::Block { depth }) => {
                if self.region == Region::Script && self.starts_with("/*") {
                    self.comment = Some(Comment::Block { depth: depth + 1 });
                    self.index += 2;
                } else if self.starts_with("*/") {
                    self.index += 2;
                    let remaining = depth - 1;
                    if remaining == 0 {
                        self.comment = None;
                    } else {
                        self.comment = Some(Comment::Block { depth: remaining });
                    }
                } else {
                    self.index += 1;
                }
            }
            Some(Comment::Line) => {
                if self.chars[self.index] == '\n' {
                    self.comment = None;
                }
                self.index += 1;
            }
            None => self.index += 1,
        }
    }

    /// Read the string literal delimited by `delimiter` and return
    /// its content. The opening delimiter is consumed; the literal
    /// ends at the matching delimiter, or at the end of the source.
    fn take_quoted(&mut self, delimiter: char) -> String {
        self.index += 1;
        let mut literal = String::new();
        while self.index < self.chars.len() {
            let ch = self.chars[self.index];
            if ch == '\\' {
                literal.push(ch);
                self.index += 1;
                if let Some(escaped) = self.chars.get(self.index) {
                    literal.push(*escaped);
                    self.index += 1;
                }
                continue;
            }
            if ch == delimiter {
                self.index += 1;
                break;
            }
            literal.push(ch);
            self.index += 1;
        }
        literal
    }

    /// Read the template literal delimited by backticks, and the
    /// literals of the `${…}` interpolations it holds — an
    /// interpolation is read as script code, so a verb inside one is
    /// still a verb.
    fn take_template(&mut self) {
        self.index += 1;
        let mut literal = String::new();
        while self.index < self.chars.len() {
            let ch = self.chars[self.index];
            if ch == '\\' {
                literal.push(ch);
                self.index += 1;
                if let Some(escaped) = self.chars.get(self.index) {
                    literal.push(*escaped);
                    self.index += 1;
                }
                continue;
            }
            if ch == '`' {
                self.index += 1;
                break;
            }
            if ch == '$' && self.chars.get(self.index + 1) == Some(&'{') {
                let raw = self.take_interpolation();
                literal.push_str(&raw);
                continue;
            }
            literal.push(ch);
            self.index += 1;
        }
        self.literals.push(literal);
    }

    /// Read a `${…}` interpolation: collect the literals of the
    /// expression it holds and return its raw text.
    fn take_interpolation(&mut self) -> String {
        let raw_start = self.index;
        self.index += 2;
        let expression_start = self.index;
        let mut depth = 1_usize;
        let mut expression_end = self.chars.len();
        while self.index < self.chars.len() {
            let ch = self.chars[self.index];
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    expression_end = self.index;
                    self.index += 1;
                    break;
                }
            }
            self.index += 1;
        }
        let expression: String = self.chars[expression_start..expression_end]
            .iter()
            .collect();
        let inner = Self::new(&expression, Region::Script).scan();
        self.literals.extend(inner);
        self.chars[raw_start..self.index].iter().collect()
    }

    /// Skip to the end of the current tag (`>`), reading no literal.
    fn skip_to_tag_end(&mut self) {
        while self.index < self.chars.len() && self.chars[self.index] != '>' {
            self.index += 1;
        }
        if self.index < self.chars.len() {
            self.index += 1;
        }
    }

    /// Whether the source at the cursor starts with `needle`.
    fn starts_with(&self, needle: &str) -> bool {
        let mut offset = self.index;
        for expected in needle.chars() {
            match self.chars.get(offset) {
                Some(actual) if *actual == expected => offset += 1,
                _ => return false,
            }
        }
        true
    }

    /// Whether the source at the cursor starts with `needle`,
    /// ignoring the case — an HTML tag name is case-insensitive.
    fn starts_with_ignore_case(&self, needle: &str) -> bool {
        let mut offset = self.index;
        for expected in needle.chars() {
            match self.chars.get(offset) {
                Some(actual) if actual.eq_ignore_ascii_case(&expected) => offset += 1,
                _ => return false,
            }
        }
        true
    }

    /// Whether a tag starts at the cursor (`<` followed by a
    /// letter).
    fn starts_with_alpha(&self) -> bool {
        self.chars
            .get(self.index + 1)
            .is_some_and(char::is_ascii_alphabetic)
    }
}

/// The shell-IPC verbs the surface can send — every string
/// literal the production dispatcher (`commands::parse_action`)
/// classifies as a session verb. The dispatcher's closed wire
/// set is the single source of the vocabulary; the test never
/// re-declares it.
fn sent_verbs(source: &str) -> BTreeSet<String> {
    let mut verbs = BTreeSet::new();
    for literal in string_literals(source) {
        if matches!(parse_action(&literal), Ok(BridgeAction::Verb(_))) {
            let _ = verbs.insert(literal);
        }
    }
    verbs
}

/// The JavaScript identifier tokens of a fragment — so
/// `state.rate` yields `state` and `rate`, while `btnRateUp`
/// stays one token.
fn identifiers(fragment: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut start: Option<usize> = None;
    for (index, character) in fragment.char_indices() {
        if character.is_ascii_alphanumeric() || character == '_' || character == '$' {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(begin) = start.take() {
            found.push(&fragment[begin..index]);
        }
    }
    if let Some(begin) = start {
        found.push(&fragment[begin..]);
    }
    found
}

/// The lines that PAINT user-visible text: a `textContent` or
/// `innerHTML` assignment, or an `aria-label` (a label is
/// painted too).
fn painting_lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter(|line| {
            line.contains(".textContent")
                || line.contains(".innerHTML")
                || line.contains("aria-label")
        })
        .collect()
}

/// The stored offset the deliberate-pause offer continues from.
const STORED_OFFSET_SECS: f64 = 42.0;

/// The position-comparison margin — the stored offset is copied
/// verbatim, so the comparison is exact up to the margin
/// (`clippy::float_cmp` is denied workspace-wide; the same
/// epsilon form `test_shell_ipc_us4.rs` uses).
const POSITION_EPS_SECS: f64 = 1e-9;

/// A stored reading position with the given end kind — the row
/// the offer decision is resolved from.
const fn stored_row(offset_secs: f64, end_kind: SessionEnd) -> StoredPosition {
    StoredPosition {
        offset_secs,
        first_yield_ms: 0,
        end_kind,
    }
}

/// T116 RED-FIRST: the single in-app screen must CONSUME the
/// offer shape the shell publishes — the route key, the in-app
/// route token, the window, and a semantic dialog element.
///
/// Today the surface has zero `offer` handling: no dialog
/// element, no route read, no window read. The assertion fails
/// until T117 renders the offer.
#[test]
fn the_in_app_surface_consumes_the_published_offer_shape() {
    assert!(
        STATUS_VIEW.contains("offer_route"),
        "T116: the in-app surface must read the published offer route key `offer_route` — the shell publishes it on the cold-start open payload (`lib.rs`) and in the `offer_route` command's reply, and the surface is the only surface that renders the offer (FR-009)"
    );
    assert!(
        STATUS_VIEW.contains("in_app_dialog"),
        "T116: the in-app surface must name the in-app offer route `in_app_dialog` — it is the only route that opens the visual dialog (the `lockscreen_resume` route opens NO in-app dialog, per `contracts/surface.md`)"
    );
    assert!(
        STATUS_VIEW.contains("offer_window_ms") || STATUS_VIEW.contains("window_ms"),
        "T116: the in-app surface must read the published offer window (`offer_window_ms` on the cold-start open payload, `window_ms` in the `offer_route` command's reply) — the offer's 3-second bound is the shell's, never one the surface invents (FR-009, FR-032c)"
    );
    assert!(
        STATUS_VIEW.contains("role=\"dialog\"") || STATUS_VIEW.contains("<dialog"),
        "T116: the in-app surface must carry a semantic dialog element for the offer (`role=\"dialog\"` or a `<dialog>` element) — FR-009 requires a VISUAL dialog on the single in-app screen"
    );
}

/// T116 RED-FIRST: the offer dialog offers EXACTLY TWO choices —
/// continue (`resume_at_stored`) and start over (`start_over`) —
/// and the surface sends no other verb beyond the transport and
/// rate controls it already owns.
///
/// The set is derived from the production dispatcher: every
/// string literal the surface sends that `parse_action` accepts
/// as a session verb. A third offer choice, a voice verb, or an
/// engine verb fails here.
#[test]
fn the_offer_dialog_offers_exactly_two_choices() {
    let sent = sent_verbs(STATUS_VIEW);
    let choices: BTreeSet<&str> = sent
        .iter()
        .map(String::as_str)
        .filter(|verb| !CONTROL_VERBS.contains(verb))
        .collect();
    assert_eq!(
        choices,
        BTreeSet::from(OFFER_CHOICE_VERBS),
        "T116: the in-app offer dialog must offer exactly TWO choices — continue (`resume_at_stored`, from the stored position) and start over (`start_over`, from the first spoken position) — and no other verb beyond the transport/rate controls (FR-009, `contracts/surface.md` \"Resume:\")"
    );
    for choice in OFFER_CHOICE_VERBS {
        assert!(
            choices.contains(choice),
            "T116: the offer choice `{choice}` must be reachable from the surface — the in-app screen dispatches it through the existing `sendAction` → `run_mobile_plugin` path (FR-009)"
        );
    }
}

/// T116: the offer surface paints NOTHING prohibited — no
/// document prose, no numeric rate value, no voice identity, and
/// no elapsed/remaining time (FR-011 / FR-019). The guard holds
/// today and must still hold once the dialog lands.
#[test]
fn the_offer_surface_paints_no_prohibited_field() {
    for value in RATE_VALUE_STRINGS {
        assert!(
            !STATUS_VIEW.contains(value),
            "T116: the in-app surface must never paint a numeric rate value (`{value}`) — the stepper orients by ladder position alone (FR-011, FR-012, FR-019)"
        );
    }
    for line in painting_lines(STATUS_VIEW) {
        for identifier in identifiers(line) {
            assert!(
                !PROHIBITED_PAINT_IDENTIFIERS.contains(&identifier),
                "T116: a painted line may not carry `{identifier}` — the in-app screen shows the title, the section title, the progress and the transport state, never a rate value, a voice identity, a reading time, or document prose (FR-011, FR-019); the offending line is: {line}"
            );
        }
        for word in PROHIBITED_PAINT_WORDS {
            assert!(
                !line.contains(word),
                "T116: a painted line may not carry the time word `{word}` — no elapsed/remaining time reaches a surface (FR-011, FR-019); the offending line is: {line}"
            );
        }
    }
    assert!(
        !STATUS_VIEW.contains("innerText"),
        "T116: the in-app screen never reads painted text — it is a read-only mirror of the Rust snapshot, and reading text back is the only way document prose could reach it (FR-019)"
    );
    assert!(
        !STATUS_VIEW.contains("fetch("),
        "T116: the in-app screen never loads the document body — the surface renders the Rust snapshot's display projection only, never document prose (FR-019)"
    );
}

/// T116: the Rust side of the contract — the two offer-choice
/// wires resolve to the two closed offer verbs, the shell
/// publishes the offer shape the surface consumes, and the
/// window it publishes is the FR-032c 3-second bound.
#[test]
fn the_shell_publishes_the_offer_the_surface_consumes() {
    assert!(
        matches!(
            parse_action("resume_at_stored"),
            Ok(BridgeAction::Verb(ShellVerb::ResumeAtStored))
        ),
        "the wire `resume_at_stored` must parse to the ResumeAtStored verb — the continue choice (contracts/shell-ipc.md)"
    );
    assert!(
        matches!(
            parse_action("start_over"),
            Ok(BridgeAction::Verb(ShellVerb::StartOver))
        ),
        "the wire `start_over` must parse to the StartOver verb — the start-over choice (contracts/shell-ipc.md)"
    );
    for alias in [
        "resume",
        "continue",
        "start-over",
        "restart",
        "startFromStored",
    ] {
        assert!(
            parse_action(alias).is_err(),
            "the wire set is closed: `{alias}` is not an offer choice — only `resume_at_stored` and `start_over` are (contracts/shell-ipc.md)"
        );
    }
    assert!(
        SHELL_SOURCE.contains("commands::offer_route"),
        "the shell must expose the `offer_route` command the surface reads — the route-to-surface command (FR-009, FR-044)"
    );
    assert!(
        SHELL_SOURCE.contains("\"offer_route\""),
        "the shell must publish the offer route key `offer_route` — the surface's only way to learn that an offer is open (FR-009)"
    );
    assert!(
        SHELL_SOURCE.contains("\"offer_window_ms\""),
        "the shell must publish the offer window key `offer_window_ms` on the cold-start open payload — the surface's only way to learn the window (FR-009, FR-032c)"
    );
    let three_second_window = SHELL_SOURCE.lines().any(|line| {
        let normalized = line.replace('_', "");
        normalized.contains("const") && normalized.contains("= 3000") && normalized.contains("MS")
    });
    assert!(
        three_second_window,
        "the shell's offer window must be the FR-032c 3-second bound (`OFFER_WINDOW_MS = 3000`) — the contract's bound, never another duration (FR-009, `contracts/surface.md`)"
    );
}

/// T116: only the deliberate pause routes to the in-app dialog.
/// A deliberate pause continues from the stored position and
/// opens the offer; an unexpected end opens NO in-app dialog
/// (the lock-screen resume control is the only way back, FR-009
/// / FR-031); no stored position opens nothing.
#[test]
fn only_the_deliberate_pause_routes_to_the_in_app_dialog() {
    let mut machine = OfferMachine::new();
    let open = machine.resolve_open(Some(stored_row(STORED_OFFSET_SECS, SessionEnd::Paused)));
    assert!(
        matches!(open, OpenOffer::Continue { offset_secs, .. } if (offset_secs - STORED_OFFSET_SECS).abs() <= POSITION_EPS_SECS),
        "a deliberate pause opens the continue offer at the STORED position — the continue choice resumes from there (FR-009)"
    );
    assert!(
        matches!(cold_start(open), ColdStart::AwaitOffer { .. }),
        "a deliberate pause routes to the in-app 3-second dialog — no audio begins until the offer resolves (FR-009, FR-032c)"
    );

    let mut machine = OfferMachine::new();
    let open = machine.resolve_open(Some(stored_row(
        STORED_OFFSET_SECS,
        SessionEnd::Interrupted,
    )));
    assert!(
        matches!(open, OpenOffer::ExplicitResume),
        "an unexpected end opens NO offer — the reader's explicit resume is the only start (FR-009, FR-031, FR-032d)"
    );
    assert!(
        matches!(cold_start(open), ColdStart::AwaitExplicitResume),
        "an unexpected end routes to the lock-screen resume control, never to the in-app dialog (`contracts/surface.md`: no visual dialog after kill/power-loss)"
    );

    let mut machine = OfferMachine::new();
    let open = machine.resolve_open(None);
    assert!(
        matches!(open, OpenOffer::NoOffer),
        "no stored position opens no offer (FR-010, FR-036a)"
    );
    assert!(
        matches!(cold_start(open), ColdStart::StartNow),
        "no stored position starts the reading at the first spoken position, with no dialog (FR-010)"
    );

    let mut machine = OfferMachine::new();
    let _ = machine.resolve_open(Some(stored_row(STORED_OFFSET_SECS, SessionEnd::Paused)));
    assert!(
        matches!(machine.start_over(), ExpiryOutcome::AutoStart { offset_secs } if offset_secs.abs() <= POSITION_EPS_SECS),
        "the start-over choice begins at the first spoken position — it discards the stored entry (FR-009, FR-032c)"
    );
}

/// The synthetic surface the comment-awareness control reads.
///
/// It carries the hazard the real surface could grow, in every form
/// it can take: an apostrophe inside an HTML comment, inside a CSS
/// comment, inside a script comment and inside HTML prose — each of
/// them followed by the two offer-choice verbs, so a scanner that
/// mis-pairs the delimiters loses the verbs instead of merely
/// mis-counting them. The HTML comment also carries a
/// COMMENTED-OUT ELEMENT with a quoted attribute: the one hazard a
/// prose rule alone cannot absorb, because a comment that holds
/// markup is code to a scanner that does not skip the comment — and
/// it holds MORE THAN ONE element, so that a scanner which stops at
/// the first `>` of the comment still reads the quoted attribute of
/// the second one. It also carries the two strings a scanner must
/// NOT lose: the single-quoted CSS font name (a legitimate string,
/// not a comment) and a quoted attribute value.
const COMMENTED_SAMPLE: &str = r#"<!DOCTYPE html>
<html>
<head>
  <!-- don't widen the offer: <b>Retomar</b> <button aria-label="OfferNotWired">Continuar</button> -->
  <style>
    /* don't change the reader font */
    body { font-family: system-ui, 'Segoe UI', sans-serif; }
  </style>
</head>
<body>
  <p>don't stop the reading</p>
  <button aria-label="Continuar">Continuar</button>
  <script>
    // don't send a third verb
    /* don't widen the choice */
    sendAction('resume_at_stored');
    sendAction('start_over');
    sendAction('play');
  </script>
</body>
</html>
"#;

/// T151 (F9): the literal scanner is COMMENT-AWARE — a quote inside
/// a comment opens NO literal, and the real verbs behind the
/// comments are still found.
///
/// The hazard is a SILENT one: a scanner that treats every quote in
/// the file as a delimiter pairs the apostrophe of a comment with
/// the NEXT quote, swallows the verb literals that follow and hides
/// them from the "exactly two choices" assertion. The contract then
/// passes while it no longer sees the verbs at all — weakened
/// without a single failing assertion. The control reads the
/// synthetic sample through the SAME scanner and the SAME
/// dispatcher-derived verb set the real assertion uses, so the two
/// paths cannot drift.
#[test]
fn the_literal_scanner_reads_no_literal_out_of_a_comment() {
    let literals = string_literals(COMMENTED_SAMPLE);
    let sent = sent_verbs(COMMENTED_SAMPLE);
    let choices: BTreeSet<&str> = sent
        .iter()
        .map(String::as_str)
        .filter(|verb| !CONTROL_VERBS.contains(verb))
        .collect();
    assert_eq!(
        choices,
        BTreeSet::from(OFFER_CHOICE_VERBS),
        "T151: the scanner must read the offer verbs even when a comment carries an apostrophe — a quote inside a comment opens no literal and swallows no verb; the sample offers exactly the two closed choices (FR-009)"
    );
    for text in [
        "don't",
        "widen",
        "third verb",
        "change the reader font",
        "stop the reading",
    ] {
        assert!(
            !literals.iter().any(|literal| literal.contains(text)),
            "T151: no literal may carry the comment or prose text `{text}` — a comment is not code and HTML prose is not a string"
        );
    }
    assert!(
        !literals.iter().any(|literal| literal == "OfferNotWired"),
        "T151: a COMMENTED-OUT element is not code — the scanner must read no literal out of an HTML comment, not even one that looks like a live attribute"
    );
    assert!(
        literals.iter().any(|literal| literal == "Segoe UI"),
        "T151: the scanner must keep the legitimate single-quoted CSS string — an apostrophe inside a font stack is a string, not a comment"
    );
    assert!(
        literals.iter().any(|literal| literal == "Continuar"),
        "T151: the scanner must keep the quoted attribute value — a tag attribute is code, not prose"
    );
}

/// The synthetic surface the prose-awareness control reads.
///
/// The prose apostrophe is the hazard under test; the four comment
/// syntaxes ride along so the control is ALSO sensitive to the
/// comment rules — a control that passes whether or not the comment
/// handling is there is not a control.
const PROSE_SAMPLE: &str = r#"<p>don't stop the reading</p>
<!-- don't hide the offer: <b>Retomar</b> <button aria-label="OfferNotWired">Continuar</button> -->
<style>/* don't change the reader font */</style>
<script>
// don't send a third verb
/* don't widen the choice */
sendAction('resume_at_stored');
</script>
"#;

/// T151 (F9): the literal scanner is PROSE-AWARE — an apostrophe in
/// the HTML text content opens no literal either. The prose of a
/// surface is painted, never dispatched, so a quote in it is a
/// character and not a delimiter: reading it as one would shift the
/// pairing exactly like a comment would.
///
/// The control is sensitive to BOTH rules of the scanner: drop the
/// prose rule and the verb behind the prose disappears; drop the
/// comment rule and the comment text leaks into a literal. Either
/// way the control fails.
#[test]
fn the_literal_scanner_reads_no_literal_out_of_prose() {
    let literals = string_literals(PROSE_SAMPLE);
    assert!(
        literals.iter().any(|literal| literal == "resume_at_stored"),
        "T151: an apostrophe in the HTML prose must not shift the literal pairing — the verb behind the prose is still a verb (FR-009)"
    );
    assert!(
        !literals
            .iter()
            .any(|literal| literal.contains("stop the reading")),
        "T151: the painted prose is not a string literal — a quote in the text content is a character, not a delimiter"
    );
    for text in ["change the reader font", "third verb", "widen the choice"] {
        assert!(
            !literals.iter().any(|literal| literal.contains(text)),
            "T151: no literal may carry the comment text `{text}` — a comment is not code, in the style block or in the script block alike"
        );
    }
    assert!(
        !literals.iter().any(|literal| literal == "OfferNotWired"),
        "T151: a COMMENTED-OUT element is not code — the scanner must read no literal out of an HTML comment, not even one that looks like a live attribute"
    );
}
