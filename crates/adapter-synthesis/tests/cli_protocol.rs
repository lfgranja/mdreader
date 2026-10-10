//! T040 — CLI protocol conformance tests for the FR-024
//! synthesis adapter (`crates/adapter-synthesis`), task
//! `T040` of `specs/002-fr-024-synthesis-adapter/tasks.md`
//! (Phase 5, Wave A — TEST-FIRST, BLOCKS T041).
//!
//! Spec anchors: constitution Principle I
//! (`constitution.md:246-248` — "The engine MUST be
//! drivable headless on Linux"), Principle II
//! (`constitution.md:251-253` — "Every crate exposes its
//! capability over text: arguments and stdin in, stdout
//! out, errors to stderr, never mixed. Machine-readable
//! output is available via an explicit `--format json`
//! flag and MUST be the default for scripting; a
//! human-readable format is offered alongside it"), the
//! one-code-path rule (`constitution.md:255-257` — the
//! IPC path and the CLI path exercise identical code),
//! and `AGENTS.md` ("Every crate is CLI-drivable").
//! No-document-text anchors: FR-045 and
//! `contracts/adapter-handoff.md:56` ("No embedded text
//! / document content in chunk").
//!
//! The CLI contract — what these tests specify for T041
//! to implement in `src/cli.rs` behind
//! `adapter_synthesis::cli::run` (arguments after the
//! program name; injectable readers and writers so the
//! protocol is drivable headless and testable
//! in-process, with no spawned process):
//!
//! ```text
//! adapter-synthesis <SUBCOMMAND> [OPTIONS]
//!
//! Subcommands — each success payload names the
//! capability it exercised:
//!   capabilities     declared_capabilities()
//!   chunk            split_document() — document on
//!                    stdin, or --input <path> instead
//!   format-check     check_format() — --rate-hz <u32>
//!                    --channels <u16> --width-bits <u16>
//!   delivery-check   check_delivery() — the format-check
//!                    flags plus --pcm-samples <usize>
//!                    and --duration-ms <f64>
//!   synthesize       synthesize() — document on stdin,
//!                    or --input <path> instead
//!
//! Options:
//!   --format json|human   machine-readable JSON is the
//!                         default; human is the
//!                         sibling format
//!   --input <path>        read the document from a file
//!                         instead of stdin
//!
//! Exit protocol (constitution.md:251-253):
//!   success → payload on stdout, stderr empty, Ok(())
//!   failure → stdout empty, diagnostic on stderr,
//!             Err(CliError)
//! ```
//!
//! JSON success payloads: `capabilities` →
//! `{"capability": "declared_capabilities", "ssml_level":
//! "none"|"partial"|"full", "alignment_granularity":
//! "none"|"word"|"sentence"}`; `chunk` →
//! `{"capability": "split_document", "segment_count":
//! <usize>, "segments": [{"segment_index": <u64>,
//! "line_start": <u32>, "line_end": <u32>,
//! "boundary_kind": "heading"|"paragraph", "byte_length":
//! <usize>, "resume_point": {"line_start": <u32>,
//! "line_end": <u32>}}]}` — segment metadata only,
//! never segment `content` (FR-045); `format-check` →
//! `{"capability": "check_format", "valid": <bool>}`;
//! `delivery-check` → `{"capability": "check_delivery",
//! "valid": <bool>}`.
//!
//! Red state (rule 1 — tests block the implementation):
//! the `cli` module does not exist yet, so this test
//! target does not compile until T041 lands `pub mod
//! cli` — the same unresolved-import red state the
//! T2–T5 tests were committed in (`tests/unit_failure.rs`:
//! "that unresolved-import red state is the required
//! pre-implementation position"). `env!("CARGO_BIN_EXE_…")`
//! is deliberately NOT used: that macro cannot compile
//! before a `[[bin]]` target exists, and the contract is
//! a library-level entry point, not a spawned process.
//!
//! Rust rigor (`constitution.md:460-475`; root
//! `Cargo.toml` `[lints]`, `[lints] workspace = true`):
//! no `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`/
//! `dbg!`/`println!`/`eprintln!`, no `#[allow]`, no `_ =>`
//! over a closed set. The crate declares no JSON library
//! (`crates/adapter-synthesis/Cargo.toml` carries only
//! `engine-core` and `thiserror`), so the machine-readable
//! payloads are read by a small, dependency-free JSON
//! parser local to this file — a test reader, not a
//! re-declaration of any contract type.

use proptest::prelude::*;

use adapter_synthesis::cli::run;

/// The distinctive marker embedded in every fixture and
/// generated document. FR-045
/// (`contracts/adapter-handoff.md:56`): no document
/// text may appear in any stdout payload, so this marker
/// must never appear in captured stdout.
const SENTINEL: &str = "ZKQRX-7F3A9C2E";

/// A Markdown fixture whose heading carries the sentinel.
///
/// The real `split_document` output for this document is
/// two segments: a heading block absorbing the
/// immediately following paragraph (lines 1–3, 56 UTF-8
/// bytes) and a final paragraph block (line 5, 16 UTF-8
/// bytes) — the correspondence this CLI test pins.
const FIXTURE_DOCUMENT: &str =
    "# Sentinel ZKQRX-7F3A9C2E Título\n\nParágrafo um ãçõ.\n\nParágrafo dois.\n";

/// Runs the CLI entry point in-process — the contract
/// T041 implements as `adapter_synthesis::cli::run` —
/// feeding `stdin` to the run and capturing stdout and
/// stderr into separate buffers.
///
/// Returns whether the run succeeded (the exit status:
/// `Ok` is exit 0, `Err` is non-zero) plus the captured
/// stdout and stderr as lossy UTF-8 text.
fn run_cli(arguments: &[&str], stdin: &str) -> (bool, String, String) {
    let mut stdin_bytes = stdin.as_bytes();
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let outcome = run(
        arguments.iter().map(ToString::to_string),
        &mut stdin_bytes,
        &mut stdout,
        &mut stderr,
    );
    (
        outcome.is_ok(),
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

/// Asserts the success half of the text-I/O protocol
/// (`constitution.md:251-253`): a success result and an
/// empty stderr — stdout and stderr are never mixed.
fn assert_success_protocol(succeeded: bool, stderr: &str) {
    assert!(succeeded, "the run must succeed");
    assert!(
        stderr.is_empty(),
        "stderr must be empty on success, but carried: {stderr}"
    );
}

/// Asserts the failure half of the text-I/O protocol
/// (`constitution.md:251-253`): a non-success result,
/// nothing on stdout, and the diagnostic on stderr.
fn assert_failure_protocol(succeeded: bool, stdout: &str, stderr: &str) {
    assert!(
        !succeeded,
        "the run must be a non-success result, not a silent success (SC-035)"
    );
    assert!(
        stdout.is_empty(),
        "stdout must be empty on failure — errors go to stderr, never mixed, but carried: {stdout}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr must carry the failure diagnostic"
    );
}

/// Reads `stdout` as one well-formed JSON document — the
/// machine-readable default that every success payload
/// must satisfy (`constitution.md:251-253`).
///
/// # Errors
///
/// A diagnostic naming the parse failure, so a failing
/// test reports the reason instead of panicking.
fn well_formed_json(stdout: &str) -> Result<JsonValue, Box<dyn std::error::Error>> {
    parse_json(stdout).map_err(|error| format!("stdout is not well-formed JSON: {error}").into())
}

/// Asserts that the JSON string stored under `key`
/// equals `expected`.
///
/// # Errors
///
/// When the payload does not carry a string under `key`.
fn assert_text_equals(
    payload: &JsonValue,
    key: &str,
    expected: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let actual = payload
        .field(key)
        .and_then(JsonValue::text)
        .ok_or_else(|| format!("the payload must carry the '{key}' string"))?;
    assert_eq!(actual, expected, "the '{key}' field must be '{expected}'");
    Ok(())
}

/// Asserts that the JSON number stored under `key`
/// equals `expected`, compared through `total_cmp` so no
/// floating-point comparison operator appears (the
/// `src/format_check.rs` precedent).
///
/// # Errors
///
/// When the payload does not carry a number under `key`.
fn assert_number_equals(
    payload: &JsonValue,
    key: &str,
    expected: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let actual = payload
        .field(key)
        .and_then(JsonValue::number)
        .ok_or_else(|| format!("the payload must carry the '{key}' number"))?;
    assert!(
        actual.total_cmp(&f64::from(expected)).is_eq(),
        "the '{key}' field must be {expected} but was {actual}"
    );
    Ok(())
}

/// Asserts that the payload's `segment_count` number
/// equals the length of its `segments` array, so the
/// metadata cannot disagree with the segmentation it
/// projects.
///
/// # Errors
///
/// When either field is missing, or the array is too
/// long to count.
fn segment_count_matches_the_array(payload: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let segments = payload
        .field("segments")
        .and_then(JsonValue::array)
        .ok_or("the payload must carry a segments array")?;
    let count = payload
        .field("segment_count")
        .and_then(JsonValue::number)
        .ok_or("the payload must carry a segment_count number")?;
    let length = f64::from(
        u32::try_from(segments.len()).map_err(|_| "the segments array is too long to count")?,
    );
    assert!(
        length.total_cmp(&count).is_eq(),
        "segment_count must equal the number of entries in the segments array, but {count} was declared for {length} segments"
    );
    Ok(())
}

/// Writes `document` to a uniquely named file in the
/// system temporary directory and returns its path. The
/// name mixes the process id with `case` so concurrent
/// test processes and cases never collide.
///
/// # Errors
///
/// When the temporary file cannot be written.
fn write_temp_document(
    document: &str,
    case: u32,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!(
        "adapter-synthesis-cli-{}-{case}.md",
        std::process::id()
    ));
    std::fs::write(&path, document)?;
    Ok(path)
}

/// The path as text, for an `--input` argument.
///
/// # Errors
///
/// When the path is not valid UTF-8.
fn path_as_argument(path: &std::path::Path) -> Result<&str, Box<dyn std::error::Error>> {
    path.to_str()
        .ok_or_else(|| std::io::Error::other("the temporary path is not valid UTF-8").into())
}

/// Removes a temporary document file. The result is
/// deliberately ignored: a missing file is the expected
/// end state, and the file lives in the shared system
/// temporary directory, so a removal failure is not this
/// test's concern.
fn remove_temp_document(path: &std::path::Path) {
    let _ignored = std::fs::remove_file(path);
}

/// A minimal JSON value — just enough structure to
/// assert that the CLI's machine-readable payloads parse
/// and carry the expected keys.
#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    /// The `null` literal.
    Null,
    /// A `true` or `false` literal.
    Bool(bool),
    /// A JSON number.
    Number(f64),
    /// A JSON string.
    String(String),
    /// A JSON array.
    Array(Vec<JsonValue>),
    /// A JSON object, preserving field order.
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// The value stored under `key` in an object, if
    /// this value is an object that carries the field.
    fn field(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(fields) => fields
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            JsonValue::Null
            | JsonValue::Bool(_)
            | JsonValue::Number(_)
            | JsonValue::String(_)
            | JsonValue::Array(_) => None,
        }
    }

    /// The elements, if this value is an array.
    fn array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(items) => Some(items),
            JsonValue::Null
            | JsonValue::Bool(_)
            | JsonValue::Number(_)
            | JsonValue::String(_)
            | JsonValue::Object(_) => None,
        }
    }

    /// The text, if this value is a string.
    fn text(&self) -> Option<&str> {
        match self {
            JsonValue::String(text) => Some(text),
            JsonValue::Null
            | JsonValue::Bool(_)
            | JsonValue::Number(_)
            | JsonValue::Array(_)
            | JsonValue::Object(_) => None,
        }
    }

    /// The flag, if this value is a boolean.
    fn boolean(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(flag) => Some(*flag),
            JsonValue::Null
            | JsonValue::Number(_)
            | JsonValue::String(_)
            | JsonValue::Array(_)
            | JsonValue::Object(_) => None,
        }
    }

    /// The numeric value, if this value is a number.
    fn number(&self) -> Option<f64> {
        match self {
            JsonValue::Number(value) => Some(*value),
            JsonValue::Null
            | JsonValue::Bool(_)
            | JsonValue::String(_)
            | JsonValue::Array(_)
            | JsonValue::Object(_) => None,
        }
    }
}

/// A cursor over the characters of one JSON document,
/// collected as `char`s so byte-index arithmetic can
/// never land mid-code-point.
struct JsonParser {
    /// The document's characters.
    characters: Vec<char>,
    /// The index of the next unread character.
    position: usize,
}

impl JsonParser {
    /// A parser positioned at the start of `document`.
    fn new(document: &str) -> Self {
        Self {
            characters: document.chars().collect(),
            position: 0,
        }
    }

    /// The next unread character, without consuming it.
    fn peek(&self) -> Option<char> {
        self.characters.get(self.position).copied()
    }

    /// Skips insignificant whitespace.
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.position += 1;
        }
    }

    /// Consumes `expected`, or fails with a diagnostic.
    fn consume(&mut self, expected: char) -> Result<(), String> {
        match self.peek() {
            Some(character) if character == expected => {
                self.position += 1;
                Ok(())
            }
            Some(character) => Err(format!("expected '{expected}' but found '{character}'")),
            None => Err(format!("expected '{expected}' but the input ended")),
        }
    }

    /// Parses one JSON value.
    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => Ok(JsonValue::String(self.parse_string()?)),
            Some('t') => self.parse_literal("true", JsonValue::Bool(true)),
            Some('f') => self.parse_literal("false", JsonValue::Bool(false)),
            Some('n') => self.parse_literal("null", JsonValue::Null),
            Some('-' | '0'..='9') => self.parse_number(),
            Some(character) => Err(format!("unexpected character '{character}'")),
            None => Err("expected a JSON value but the input ended".to_string()),
        }
    }

    /// Parses a fixed literal word with its value.
    fn parse_literal(&mut self, word: &str, value: JsonValue) -> Result<JsonValue, String> {
        for expected in word.chars() {
            self.consume(expected)?;
        }
        Ok(value)
    }

    /// Parses a JSON object.
    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut fields: Vec<(String, JsonValue)> = Vec::new();
        if self.peek() == Some('}') {
            self.position += 1;
            return Ok(JsonValue::Object(fields));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            fields.push((key, value));
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.position += 1;
                }
                Some('}') => {
                    self.position += 1;
                    return Ok(JsonValue::Object(fields));
                }
                Some(character) => {
                    return Err(format!("expected ',' or '}}' but found '{character}'"));
                }
                None => {
                    return Err("expected ',' or '}' but the input ended".to_string());
                }
            }
        }
    }

    /// Parses a JSON array.
    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut items: Vec<JsonValue> = Vec::new();
        if self.peek() == Some(']') {
            self.position += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            self.skip_whitespace();
            let item = self.parse_value()?;
            items.push(item);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.position += 1;
                }
                Some(']') => {
                    self.position += 1;
                    return Ok(JsonValue::Array(items));
                }
                Some(character) => {
                    return Err(format!("expected ',' or ']' but found '{character}'"));
                }
                None => {
                    return Err("expected ',' or ']' but the input ended".to_string());
                }
            }
        }
    }

    /// Parses a JSON string, resolving escape sequences.
    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut text = String::new();
        loop {
            match self.peek() {
                Some('"') => {
                    self.position += 1;
                    return Ok(text);
                }
                Some('\\') => {
                    self.position += 1;
                    let escaped = self.escape_sequence()?;
                    text.push(escaped);
                }
                Some(character) if u32::from(character) < 0x20 => {
                    return Err(format!(
                        "raw control character U+{:04X} in string",
                        u32::from(character)
                    ));
                }
                Some(character) => {
                    // The cursor MUST advance here: without it
                    // this branch re-reads the same character
                    // forever and never terminates, because it
                    // is the only branch whose happy path does
                    // not consume input.
                    self.position += 1;
                    text.push(character);
                }
                None => {
                    return Err("expected '\"' but the input ended".to_string());
                }
            }
        }
    }

    /// Parses one escape sequence after the backslash.
    fn escape_sequence(&mut self) -> Result<char, String> {
        match self.peek() {
            Some('"') => {
                self.position += 1;
                Ok('"')
            }
            Some('\\') => {
                self.position += 1;
                Ok('\\')
            }
            Some('/') => {
                self.position += 1;
                Ok('/')
            }
            Some('b') => {
                self.position += 1;
                Ok('\u{0008}')
            }
            Some('f') => {
                self.position += 1;
                Ok('\u{000C}')
            }
            Some('n') => {
                self.position += 1;
                Ok('\n')
            }
            Some('r') => {
                self.position += 1;
                Ok('\r')
            }
            Some('t') => {
                self.position += 1;
                Ok('\t')
            }
            Some('u') => {
                self.position += 1;
                self.parse_unicode_escape()
            }
            Some(character) => Err(format!("invalid escape character '{character}'")),
            None => Err("expected an escape character but the input ended".to_string()),
        }
    }

    /// Parses the four hexadecimal digits of a `\u`
    /// escape, combining a surrogate pair when a high
    /// surrogate is followed by a low one.
    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        let first = self.parse_hex_quad()?;
        if (0xD800..=0xDBFF).contains(&first) {
            if self.peek() == Some('\\') {
                let saved = self.position;
                self.position += 1;
                if self.peek() == Some('u') {
                    self.position += 1;
                    match self.parse_hex_quad() {
                        Ok(low) if (0xDC00..=0xDFFF).contains(&low) => {
                            let combined = 0x10000 + ((first - 0xD800) << 10) + (low - 0xDC00);
                            return Ok(char::from_u32(combined).unwrap_or('\u{FFFD}'));
                        }
                        _ => {
                            self.position = saved;
                        }
                    }
                } else {
                    self.position = saved;
                }
            }
            return Ok('\u{FFFD}');
        }
        if (0xDC00..=0xDFFF).contains(&first) {
            return Ok('\u{FFFD}');
        }
        Ok(char::from_u32(first).unwrap_or('\u{FFFD}'))
    }

    /// Parses exactly four hexadecimal digits as a code
    /// point.
    fn parse_hex_quad(&mut self) -> Result<u32, String> {
        let mut code_point = 0u32;
        for _ in 0..4 {
            let digit = match self.peek() {
                Some(character) if character.is_ascii_hexdigit() => {
                    self.position += 1;
                    character.to_digit(16).unwrap_or_default()
                }
                Some(character) => {
                    return Err(format!(
                        "expected a hexadecimal digit but found '{character}'"
                    ));
                }
                None => {
                    return Err("expected a hexadecimal digit but the input ended".to_string());
                }
            };
            code_point = (code_point << 4) | digit;
        }
        Ok(code_point)
    }

    /// Parses a JSON number: optional minus, integer
    /// part, optional fraction, optional exponent.
    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        if self.peek() == Some('-') {
            self.position += 1;
        }
        match self.peek() {
            Some('0') => {
                self.position += 1;
            }
            Some(character) if character.is_ascii_digit() => {
                while matches!(
                    self.peek(),
                    Some(digit) if digit.is_ascii_digit()
                ) {
                    self.position += 1;
                }
            }
            Some(character) => {
                return Err(format!("expected a digit but found '{character}'"));
            }
            None => {
                return Err("expected a digit but the input ended".to_string());
            }
        }
        if self.peek() == Some('.') {
            self.position += 1;
            if !matches!(
                self.peek(),
                Some(digit) if digit.is_ascii_digit()
            ) {
                return Err("expected a digit after '.'".to_string());
            }
            while matches!(
                self.peek(),
                Some(digit) if digit.is_ascii_digit()
            ) {
                self.position += 1;
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.position += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.position += 1;
            }
            if !matches!(
                self.peek(),
                Some(digit) if digit.is_ascii_digit()
            ) {
                return Err("expected a digit in the exponent".to_string());
            }
            while matches!(
                self.peek(),
                Some(digit) if digit.is_ascii_digit()
            ) {
                self.position += 1;
            }
        }
        let digits: String = self.characters[start..self.position].iter().collect();
        let value = digits
            .parse::<f64>()
            .map_err(|error| format!("invalid number '{digits}': {error}"))?;
        Ok(JsonValue::Number(value))
    }
}

/// Parses one complete JSON document from `document`.
///
/// # Errors
///
/// A descriptive diagnostic when `document` is not
/// exactly one JSON value.
fn parse_json(document: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(document);
    parser.skip_whitespace();
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    match parser.peek() {
        None => Ok(value),
        Some(character) => Err(format!(
            "trailing character '{character}' after the JSON value"
        )),
    }
}

// ---------------------------------------------------------------------------
// 1. Capability reachability — every public capability is
//    reachable through a named subcommand whose success
//    payload names it, and an unknown subcommand is
//    rejected (`constitution.md:246-248`; `AGENTS.md`).
// ---------------------------------------------------------------------------

/// The unconditional `(None, None)` declaration is
/// reachable through the `capabilities` subcommand, and
/// its payload names `declared_capabilities` and carries
/// both declaration keys (`FR-024c`).
#[test]
fn capabilities_subcommand_outputs_the_declaration_as_json()
-> Result<(), Box<dyn std::error::Error>> {
    let (succeeded, stdout, stderr) = run_cli(&["capabilities"], "");
    assert_success_protocol(succeeded, &stderr);
    let payload = well_formed_json(&stdout)?;
    assert_text_equals(&payload, "capability", "declared_capabilities")?;
    assert_text_equals(&payload, "ssml_level", "none")?;
    assert_text_equals(&payload, "alignment_granularity", "none")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. stdin input — a Markdown document supplied on stdin
//    produces the corresponding segmentation on stdout,
//    and the same document supplied via `--input`
//    produces byte-identical stdout: one code path, two
//    inputs (`constitution.md:255-257`).
// ---------------------------------------------------------------------------

/// The `chunk` subcommand projects the segmentation of a
/// document supplied on stdin as JSON metadata — the two
/// segments the fixture actually yields, with their
/// ranges, boundary kinds and byte lengths — and never
/// the document's prose (FR-045).
#[test]
fn chunk_subcommand_outputs_segmentation_metadata_as_json() -> Result<(), Box<dyn std::error::Error>>
{
    let (succeeded, stdout, stderr) = run_cli(&["chunk"], FIXTURE_DOCUMENT);
    assert_success_protocol(succeeded, &stderr);
    assert!(
        !stdout.contains(SENTINEL),
        "stdout must never carry document text (FR-045; contracts/adapter-handoff.md:56), but carried: {stdout}"
    );
    let payload = well_formed_json(&stdout)?;
    assert_text_equals(&payload, "capability", "split_document")?;
    segment_count_matches_the_array(&payload)?;
    let segments = payload
        .field("segments")
        .and_then(JsonValue::array)
        .ok_or("the payload must carry a segments array")?;
    assert_eq!(
        segments.len(),
        2,
        "the fixture must split into exactly two segments"
    );

    // The heading block absorbs the immediately following
    // paragraph (lines 1-3); the final paragraph stands
    // alone (line 5). Metadata only — never `content`.
    let first = segments.first().ok_or("the first segment is missing")?;
    assert_number_equals(first, "segment_index", 0)?;
    assert_number_equals(first, "line_start", 1)?;
    assert_number_equals(first, "line_end", 3)?;
    assert_text_equals(first, "boundary_kind", "heading")?;
    assert_number_equals(first, "byte_length", 56)?;
    let resume_point = first
        .field("resume_point")
        .ok_or("the first segment must carry a resume_point")?;
    assert_number_equals(resume_point, "line_start", 1)?;
    assert_number_equals(resume_point, "line_end", 3)?;

    let second = segments.get(1).ok_or("the second segment is missing")?;
    assert_number_equals(second, "segment_index", 1)?;
    assert_number_equals(second, "line_start", 5)?;
    assert_number_equals(second, "line_end", 5)?;
    assert_text_equals(second, "boundary_kind", "paragraph")?;
    assert_number_equals(second, "byte_length", 16)?;
    Ok(())
}

/// An empty document yields an empty segmentation: zero
/// segments, reported faithfully on stdout.
#[test]
fn chunk_subcommand_reports_an_empty_segmentation_for_an_empty_document()
-> Result<(), Box<dyn std::error::Error>> {
    let (succeeded, stdout, stderr) = run_cli(&["chunk"], "");
    assert_success_protocol(succeeded, &stderr);
    let payload = well_formed_json(&stdout)?;
    assert_text_equals(&payload, "capability", "split_document")?;
    segment_count_matches_the_array(&payload)?;
    let segments = payload
        .field("segments")
        .and_then(JsonValue::array)
        .ok_or("the payload must carry a segments array")?;
    assert!(
        segments.is_empty(),
        "an empty document must yield zero segments"
    );
    Ok(())
}

/// The same document supplied via `--input <path>`
/// produces byte-identical stdout to the stdin supply —
/// one code path, two inputs (`constitution.md:255-257`).
#[test]
fn chunk_subcommand_reads_the_same_document_from_stdin_and_input()
-> Result<(), Box<dyn std::error::Error>> {
    let (stdin_succeeded, stdin_stdout, stdin_stderr) = run_cli(&["chunk"], FIXTURE_DOCUMENT);
    let path = write_temp_document(FIXTURE_DOCUMENT, 1)?;
    let argument = path_as_argument(&path)?;
    let (input_succeeded, input_stdout, input_stderr) =
        run_cli(&["chunk", "--input", argument], "");
    remove_temp_document(&path);
    assert_success_protocol(stdin_succeeded, &stdin_stderr);
    assert_success_protocol(input_succeeded, &input_stderr);
    assert_eq!(
        input_stdout, stdin_stdout,
        "stdin and --input must produce byte-identical stdout"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. stdout format — machine-readable JSON is the default
//    and parses, an explicit `--format json` is accepted
//    and identical to the default, and a human-readable
//    sibling format is selectable
//    (`constitution.md:251-253`).
// ---------------------------------------------------------------------------

/// `--format json` is accepted and is the default: the
/// explicit flag produces byte-identical stdout to
/// omitting it, and the default output is well-formed
/// JSON (`constitution.md:251-253`).
#[test]
fn json_is_the_default_output_format() -> Result<(), Box<dyn std::error::Error>> {
    let (default_succeeded, default_stdout, default_stderr) = run_cli(&["chunk"], FIXTURE_DOCUMENT);
    let (explicit_succeeded, explicit_stdout, explicit_stderr) =
        run_cli(&["chunk", "--format", "json"], FIXTURE_DOCUMENT);
    assert_success_protocol(default_succeeded, &default_stderr);
    assert_success_protocol(explicit_succeeded, &explicit_stderr);
    assert_eq!(
        explicit_stdout, default_stdout,
        "the explicit --format json flag must produce byte-identical output to the default"
    );
    let payload = well_formed_json(&default_stdout)?;
    assert_text_equals(&payload, "capability", "split_document")?;
    Ok(())
}

/// The human-readable sibling format is selectable and
/// differs from the machine-readable default
/// (`constitution.md:251-253`).
#[test]
fn human_format_is_selectable_and_differs_from_the_machine_format() {
    let (json_succeeded, json_stdout, json_stderr) = run_cli(&["chunk"], FIXTURE_DOCUMENT);
    let (human_succeeded, human_stdout, human_stderr) =
        run_cli(&["chunk", "--format", "human"], FIXTURE_DOCUMENT);
    assert_success_protocol(json_succeeded, &json_stderr);
    assert_success_protocol(human_succeeded, &human_stderr);
    assert!(
        !human_stdout.is_empty(),
        "the human format must write its payload to stdout"
    );
    assert_ne!(
        human_stdout, json_stdout,
        "the human-readable sibling format must differ from the machine-readable JSON default"
    );
}

/// An unknown `--format` value is rejected: a closed
/// format set with no catch-all, routed through the
/// failure half of the protocol.
#[test]
fn an_unknown_format_is_rejected_with_nothing_on_stdout() {
    let (succeeded, stdout, stderr) = run_cli(&["chunk", "--format", "xml"], FIXTURE_DOCUMENT);
    assert_failure_protocol(succeeded, &stdout, &stderr);
}

// ---------------------------------------------------------------------------
// 4. Format verification — `check_format` and
//    `check_delivery` are each reachable through a named
//    subcommand, and a typed failure goes to stderr with
//    a non-success result and nothing on stdout
//    (`FR-024b`; `SC-035`; `constitution.md:251-253`).
// ---------------------------------------------------------------------------

/// The declared shape — mono `f32` at 22 050 Hz — is
/// accepted by the `format-check` subcommand, whose
/// payload names `check_format`.
#[test]
fn format_check_subcommand_accepts_the_declared_shape() -> Result<(), Box<dyn std::error::Error>> {
    let (succeeded, stdout, stderr) = run_cli(
        &[
            "format-check",
            "--rate-hz",
            "22050",
            "--channels",
            "1",
            "--width-bits",
            "32",
        ],
        "",
    );
    assert_success_protocol(succeeded, &stderr);
    let payload = well_formed_json(&stdout)?;
    assert_text_equals(&payload, "capability", "check_format")?;
    let valid = payload
        .field("valid")
        .and_then(JsonValue::boolean)
        .ok_or("the payload must carry a valid flag")?;
    assert!(valid, "the declared shape must be reported as valid");
    Ok(())
}

/// A conforming delivered buffer — the declared shape,
/// with the sample count its duration implies — is
/// accepted by the `delivery-check` subcommand, whose
/// payload names `check_delivery`.
#[test]
fn delivery_check_subcommand_accepts_a_conforming_buffer() -> Result<(), Box<dyn std::error::Error>>
{
    let (succeeded, stdout, stderr) = run_cli(
        &[
            "delivery-check",
            "--rate-hz",
            "22050",
            "--channels",
            "1",
            "--width-bits",
            "32",
            "--pcm-samples",
            "22050",
            "--duration-ms",
            "1000",
        ],
        "",
    );
    assert_success_protocol(succeeded, &stderr);
    let payload = well_formed_json(&stdout)?;
    assert_text_equals(&payload, "capability", "check_delivery")?;
    let valid = payload
        .field("valid")
        .and_then(JsonValue::boolean)
        .ok_or("the payload must carry a valid flag")?;
    assert!(
        valid,
        "a sample-aligned buffer at the declared shape must be reported as valid"
    );
    Ok(())
}

/// A rate deviation from the declared 22 050 Hz names a
/// typed failure routed to stderr, with a non-success
/// result and nothing on stdout (`FR-024b`; `SC-035`;
/// `EngineError::RateMismatch`).
#[test]
fn format_check_subcommand_routes_a_rate_mismatch_to_stderr() {
    let (succeeded, stdout, stderr) = run_cli(
        &[
            "format-check",
            "--rate-hz",
            "48000",
            "--channels",
            "1",
            "--width-bits",
            "32",
        ],
        "",
    );
    assert_failure_protocol(succeeded, &stdout, &stderr);
}

/// A buffer whose sample count disagrees with its
/// duration by more than half a sample names a typed
/// failure routed to stderr, with a non-success result
/// and nothing on stdout (`FR-024b`; `SC-035`).
#[test]
fn delivery_check_subcommand_routes_a_length_deviation_to_stderr() {
    let (succeeded, stdout, stderr) = run_cli(
        &[
            "delivery-check",
            "--rate-hz",
            "22050",
            "--channels",
            "1",
            "--width-bits",
            "32",
            "--pcm-samples",
            "100",
            "--duration-ms",
            "1000",
        ],
        "",
    );
    assert_failure_protocol(succeeded, &stdout, &stderr);
}

/// The shape flags are required: a `format-check`
/// invocation without them is rejected through the
/// failure half of the protocol.
#[test]
fn format_check_subcommand_requires_the_delivered_shape() {
    let (succeeded, stdout, stderr) = run_cli(&["format-check"], "");
    assert_failure_protocol(succeeded, &stdout, &stderr);
}

/// Synthesis is reachable through a named subcommand
/// and — with the real `offline_default` engine bound
/// (T044: `sherpa-onnx` 1.13.8 driving the embedded
/// Piper pt-BR `faber` voice) — delivers real audio:
/// a success result whose JSON payload names the
/// capability, the segment count and the total PCM
/// sample count, with nothing on stderr (`FR-024b`;
/// `SC-035`). The design-phase fail-closed pin
/// (unbound model → `InitFailed`, nothing on stdout)
/// does not survive binding: a well-formed document
/// must synthesize, and the payload never carries the
/// document's prose (FR-045).
#[test]
fn synthesize_subcommand_delivers_pcm_counts_as_json() -> Result<(), Box<dyn std::error::Error>> {
    let (succeeded, stdout, stderr) = run_cli(&["synthesize"], FIXTURE_DOCUMENT);
    assert_success_protocol(succeeded, &stderr);
    assert!(
        !stdout.contains(SENTINEL),
        "stdout must never carry document text (FR-045; contracts/adapter-handoff.md:56), but carried: {stdout}"
    );
    let payload = well_formed_json(&stdout)?;
    assert_text_equals(&payload, "capability", "synthesize")?;
    // The fixture splits into exactly two segments (the
    // heading block absorbs the following paragraph; the
    // final paragraph stands alone) — the same split the
    // `chunk` payload above pins.
    assert_number_equals(&payload, "segment_count", 2)?;
    // The bound engine delivers real PCM: a non-empty
    // total sample count across the synthesized segments.
    let pcm_samples = payload
        .field("pcm_samples")
        .and_then(JsonValue::number)
        .ok_or("the payload must carry a pcm_samples number")?;
    assert!(
        pcm_samples > 0.0,
        "the bound engine must deliver a non-empty PCM sample count, but the payload declared {pcm_samples}"
    );
    Ok(())
}

/// An unknown subcommand is rejected: a non-success
/// result, nothing on stdout, the diagnostic on stderr.
#[test]
fn unknown_subcommand_is_rejected_with_nothing_on_stdout() {
    let (succeeded, stdout, stderr) = run_cli(&["frobnicate"], "");
    assert_failure_protocol(succeeded, &stdout, &stderr);
}

/// A missing subcommand is rejected the same way: the
/// CLI never guesses an operation.
#[test]
fn missing_subcommand_is_rejected_with_nothing_on_stdout() {
    let (succeeded, stdout, stderr) = run_cli(&[], "");
    assert_failure_protocol(succeeded, &stdout, &stderr);
}

// ---------------------------------------------------------------------------
// 5. Property-based coverage — arbitrary documents.
//
//    `proptest` generates the document bodies: the
//    JSON-well-formedness, stderr-empty,
//    no-document-leakage and stdin/`--input`-identity
//    properties must hold for arbitrary documents, not
//    just the fixed fixture (constitution Principle III;
//    `tasks.md` rule "property-based where applicable").
// ---------------------------------------------------------------------------

// Plain `//`, not `///`: a doc comment cannot attach to a
// `proptest!` block, so `///` here would raise
// `unused_doc_comments`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn arbitrary_documents_produce_well_formed_json_without_leaking_prose(
        document in document_strategy(),
    ) {
        let (succeeded, stdout, stderr) = run_cli(&["chunk"], &document);
        prop_assert!(
            succeeded,
            "the chunk subcommand must succeed on an arbitrary document"
        );
        prop_assert!(
            stderr.is_empty(),
            "stderr must be empty on success, but carried: {stderr}",
            stderr = stderr
        );
        prop_assert!(
            !stdout.contains(SENTINEL),
            "stdout must never carry document text (FR-045; contracts/adapter-handoff.md:56)"
        );
        let payload = match parse_json(&stdout) {
            Ok(value) => value,
            Err(error) => {
                prop_assert!(
                    false,
                    "stdout must be well-formed JSON, but failed to parse: {error}",
                    error = error
                );
                JsonValue::Null
            }
        };
        prop_assert_eq!(
            payload.field("capability").and_then(JsonValue::text),
            Some("split_document"),
            "the payload must name the split_document capability"
        );
        if let Err(reason) = segment_count_matches_the_array(&payload) {
            prop_assert!(
                false,
                "the segment_count must equal the segments array length: {reason}",
                reason = reason
            );
        }
        let segments = if let Some(segments) =
            payload.field("segments").and_then(JsonValue::array)
        {
            segments
        } else {
            prop_assert!(
                false,
                "the payload must carry a segments array"
            );
            &[]
        };
        // The projected segment indices are 0-based and
        // consecutive, matching the segmentation the
        // library produces for the same document.
        for (expected_index, segment) in (0u32..).zip(segments.iter()) {
            if let Err(reason) =
                assert_number_equals(segment, "segment_index", expected_index)
            {
                prop_assert!(
                    false,
                    "the projected segmentation is not 0-based and consecutive: {reason}",
                    reason = reason
                );
            }
        }
    }
}

// Plain `//`, not `///`: a doc comment cannot attach to a
// `proptest!` block.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn arbitrary_documents_read_identically_from_stdin_and_input(
        document in document_strategy(),
    ) {
        let (stdin_succeeded, stdin_stdout, stdin_stderr) =
            run_cli(&["chunk"], &document);
        let path = match write_temp_document(&document, 2) {
            Ok(path) => path,
            Err(error) => {
                prop_assert!(
                    false,
                    "the temporary document could not be written: {error}",
                    error = error
                );
                // Unreachable: the failed assertion above
                // already ended the case, so the case ends
                // here rather than continuing without a
                // usable path.
                return Ok(());
            }
        };
        let argument = match path_as_argument(&path) {
            Ok(argument) => argument.to_string(),
            Err(error) => {
                remove_temp_document(&path);
                prop_assert!(
                    false,
                    "the temporary path is not valid UTF-8: {error}",
                    error = error
                );
                // Unreachable for the same reason as above.
                return Ok(());
            }
        };
        let (input_succeeded, input_stdout, input_stderr) =
            run_cli(&["chunk", "--input", argument.as_str()], "");
        remove_temp_document(&path);
        prop_assert!(
            stdin_succeeded,
            "the chunk subcommand must succeed on stdin"
        );
        prop_assert!(
            input_succeeded,
            "the chunk subcommand must succeed on --input"
        );
        prop_assert!(
            stdin_stderr.is_empty(),
            "stderr must be empty on the stdin run, but carried: {stderr}",
            stderr = stdin_stderr
        );
        prop_assert!(
            input_stderr.is_empty(),
            "stderr must be empty on the --input run, but carried: {stderr}",
            stderr = input_stderr
        );
        prop_assert_eq!(
            input_stdout, stdin_stdout,
            "stdin and --input must produce byte-identical stdout (constitution.md:255-257)"
        );
    }
}

// Plain `//`, not `///`: a doc comment cannot attach to a
// `proptest!` block.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn synthesis_never_leaks_document_prose_into_stdout(
        document in synthesis_document_strategy(),
    ) {
        let (succeeded, stdout, stderr) = run_cli(&["synthesize"], &document);
        prop_assert!(
            !stdout.contains(SENTINEL),
            "stdout must never carry document text, on success or on failure (FR-045; contracts/adapter-handoff.md:56)"
        );
        if succeeded {
            prop_assert!(
                stderr.is_empty(),
                "stderr must be empty on success, but carried: {stderr}",
                stderr = stderr
            );
            let payload = match parse_json(&stdout) {
                Ok(value) => value,
                Err(error) => {
                    prop_assert!(
                        false,
                        "stdout must be well-formed JSON, but failed to parse: {error}",
                        error = error
                    );
                    JsonValue::Null
                }
            };
            prop_assert_eq!(
                payload.field("capability").and_then(JsonValue::text),
                Some("synthesize"),
                "the payload must name the synthesize capability"
            );
        } else {
            // The fail-closed path (the design-phase
            // behaviour: the unbound voice model reports
            // `InitFailed`).
            prop_assert!(
                stdout.is_empty(),
                "stdout must be empty when synthesis fails closed, but carried: {stdout}",
                stdout = stdout
            );
            prop_assert!(
                !stderr.is_empty(),
                "a fail-closed synthesis must report its failure to stderr"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Builds a budget-bounded arbitrary Markdown document for the
/// synthesis no-leakage property: the same shape as
/// [`document_strategy`] — a sentinel heading followed by
/// blocks, each either a `###` heading or a paragraph of
/// multi-byte Portuguese words — with paragraphs capped at
/// 240 bytes and documents at 8 blocks, so a case synthesizes
/// at most a couple of full-budget segments.
///
/// The no-leakage property (FR-045) is structural — the
/// success payload is segment counts and `f32` sample counts,
/// and a named failure's fields are identifier-pattern only —
/// so it needs *varied* documents, not large ones. The shared
/// [`document_strategy`]'s 40-block / ~48 KB documents make
/// every case pay single-threaded ONNX inference for up to ten
/// full-budget segments, which measured >19 min for this one
/// test on this host (T044 case-count/timeout-budget rewrite;
/// `SC-009-GATE-RECORD.md` consequence (ii)). Every document
/// still carries the sentinel in its first heading, so the
/// property is exercised for real — a property that only ever
/// observes sentinel-free documents would be vacuous.
fn synthesis_document_strategy() -> impl Strategy<Value = String> {
    // One heading block: a `###` line with accented text.
    let heading = any::<u32>().prop_map(|index| format!("### Seção {index} — título ãçõ"));
    // One paragraph block: 0..240 bytes of multi-byte words.
    // Each `aãçõáê ` is 6 chars / 12 bytes, so byte length and
    // char length diverge throughout.
    let paragraph = (0u32..240u32).prop_map(|byte_budget| {
        let words = byte_budget / 12;
        "aãçõáê ".repeat(words as usize)
    });
    // Headings are rarer than paragraphs (1:3).
    let block = prop_oneof![1 => heading, 3 => paragraph];
    proptest::collection::vec(block, 2usize..8usize).prop_map(|blocks| {
        let mut document = String::from("### ");
        document.push_str(SENTINEL);
        document.push_str("\n\n");
        document.push_str(&blocks.join("\n\n"));
        document
    })
}

/// Builds an arbitrary Markdown document: a sentinel
/// heading followed by 2..40 blocks, each either a `###`
/// heading or a paragraph of multi-byte Portuguese words,
/// separated by blank lines. Every document carries the
/// sentinel in its first heading, so the no-leakage
/// property is exercised for real — a property that only
/// ever observes sentinel-free documents would be vacuous.
///
/// Reserved for the fast (non-synthesizing) properties in
/// this file — the CLI metadata tests never invoke the
/// engine, so the 40-block / ~48 KB upper bound costs
/// nothing there. Synthesizing properties use
/// [`synthesis_document_strategy`].
fn document_strategy() -> impl Strategy<Value = String> {
    // One heading block: a `###` line with accented text.
    let heading = any::<u32>().prop_map(|index| format!("### Seção {index} — título ãçõ"));
    // One paragraph block: 0..1200 bytes of multi-byte
    // words. Each `aãçõáê ` is 6 chars / 12 bytes, so
    // byte length and char length diverge throughout.
    let paragraph = (0u32..1200u32).prop_map(|byte_budget| {
        let words = byte_budget / 12;
        "aãçõáê ".repeat(words as usize)
    });
    // Headings are rarer than paragraphs (1:3).
    let block = prop_oneof![1 => heading, 3 => paragraph];
    proptest::collection::vec(block, 2usize..40usize).prop_map(|blocks| {
        let mut document = String::from("### ");
        document.push_str(SENTINEL);
        document.push_str("\n\n");
        document.push_str(&blocks.join("\n\n"));
        document
    })
}
