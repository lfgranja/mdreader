//! Text-I/O protocol for the session crate (T063).
//!
//! Every capability the crate exposes is drivable over
//! text: arguments in, the success payload on stdout,
//! the failure diagnostic on stderr — never mixed. The
//! machine-readable JSON payload is the default for
//! scripting (`--format json`; `contracts/storage-schema.md`
//! "CLI surface"), with the human-readable plain form
//! selectable alongside it (`--format human`). The same
//! [`run`] entry point serves the binary (`src/main.rs`
//! forwards the process's streams) and any in-process
//! caller, so the CLI path and the IPC path exercise
//! identical code (Principle II): every verb resolves
//! through the canonical [`store`], [`offer`], and
//! [`fingerprint`] functions — no second implementation
//! of any rule lives here.
//!
//! The redb database is named per invocation
//! (`--db <path>`, preceding the subcommand):
//! the store opens — and creates — the file it
//! names, so every invocation states which
//! store it touches. An invocation that names
//! none — the developer smoke pipe, `echo
//! <bytes> | session ingest` — answers from
//! the scratch store under the system temp
//! dir, so the text-I/O protocol reads stdin
//! and answers with no argument at all; the
//! default is visible in every rendered
//! `--help`. `<file>` is the DOCUMENT whose
//! bytes are blake3-256 fingerprinted into the
//! bookmark key — never the path. The `ingest`
//! verb takes that document positionally
//! (`ingest <snapshot-file>`, contracts/
//! ingest.md "CLI surface"), or by `--path`,
//! or as the bytes piped on stdin when no file
//! is named — the bytes are fingerprinted into
//! the key in every form, never the path.
//!
//! The offer machine is per-invocation: a CLI process
//! holds no live session, so the application-visibility
//! latch does not persist across processes. The latch is
//! therefore demonstrated within the invocation that
//! also expires — `offer expire` carries the
//! `AppForegrounded` intent as its own `--foreground`
//! flag, making the SC-020 split CI-drivable (a
//! foregrounded deliberate pause auto-starts at expiry;
//! a backgrounded one converts to the explicit-resume
//! form) — and the `foreground on|off` verb latches
//! and renders the latched state on its own. Together
//! the `offer *` and `foreground *` verbs are the
//! CI-drivable form of the FR-032c / FR-032d expiry
//! arms.

use std::fmt::Write as FmtWrite;
use std::io::{Read, Write};
use std::iter;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use engine_core::closed::{PlaybackRate, SessionEnd};
use serde::Serialize;

use crate::fingerprint::fingerprint;
use crate::offer::{ExpiryOutcome, Offer, OfferMachine};
use crate::store::{BookmarkStore, StoredPosition};

/// The program name: the name the generated parser
/// reports under, and the argument [`run`] prepends
/// when it hands the caller's arguments — which
/// exclude the program name — to that parser.
const PROGRAM_NAME: &str = "session";

/// Run one CLI invocation over the text-I/O protocol.
///
/// `args` are the arguments after the program name;
/// `stdout` and `stderr` are the injected streams, so
/// the protocol is drivable headless and testable
/// in-process with no spawned process.
///
/// Success writes the payload to `stdout`, leaves
/// `stderr` empty, and returns `Ok(())`. Failure
/// writes nothing to `stdout`, writes the diagnostic
/// to `stderr`, and returns the [`CliError`] for the
/// caller to map to a non-zero exit status. `--help`
/// and `--version` are payloads, not failures: the
/// generated parser renders them to `stdout` with an
/// `Ok` result.
///
/// # Errors
///
/// * [`CliError::Usage`] when the invocation is
///   rejected before any capability runs — an unknown,
///   missing, or malformed subcommand or option, an
///   `--format` value outside the closed format set,
///   or an `--offset-secs` outside the position floor.
/// * [`CliError::Io`] when the document file or the
///   store cannot be read.
/// * [`CliError::Store`] when a store operation fails.
/// * [`CliError::OffLadderRate`] when `rate set`
///   carries a value off the closed ladder (FR-019).
/// * [`CliError::TerminalEndKind`] when `put` carries
///   a terminal `end_kind` — those ends delete the
///   entry instead of storing one (FR-032 / FR-032b).
/// * [`CliError::Rendering`] when a machine-readable
///   payload cannot be rendered.
pub fn run(
    args: impl Iterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    // The generated parser treats the first argument
    // as the program name; `run` receives only the
    // arguments after it, so the fixed program name
    // is prepended.
    let invocation = iter::once(String::from(PROGRAM_NAME)).chain(args);
    let cli = match Cli::try_parse_from(invocation) {
        Ok(cli) => cli,
        Err(error) => {
            let rendered = error.render();
            if error.use_stderr() {
                // The failure half of the protocol:
                // the diagnostic goes to stderr through
                // the injected writer, and the error is
                // returned for the caller to map to a
                // non-zero exit status. Writing it here
                // rather than leaving it to the caller
                // is what keeps the streams unmixed for
                // a usage error: the caller only ever
                // maps the returned error to an exit
                // status, so a diagnostic left implicit
                // would reach stderr only if the caller
                // chose to print it — and `main` does
                // not, because `run` owns every write
                // to the injected streams.
                let _ignored = write!(stderr, "{rendered}");
                return Err(CliError::Usage(rendered.to_string()));
            }
            // `--help` and `--version` are payloads,
            // not failures: the generated parser
            // renders them onto stdout with a zero
            // exit status.
            write!(stdout, "{rendered}")?;
            return Ok(());
        }
    };
    match execute(&cli, stdout) {
        Ok(()) => Ok(()),
        Err(error) => {
            // The failure half of the protocol: nothing
            // was written to stdout, so this diagnostic
            // on stderr completes it. A write failure
            // here is deliberately discarded — the
            // invocation has already failed, and the
            // returned error names the original failure.
            let _ignored = writeln!(stderr, "{error}");
            Err(error)
        }
    }
}

/// Why a [`run`] invocation failed: the failure half
/// of the text-I/O protocol (stdout empty, diagnostic
/// on stderr, non-zero exit status).
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// The invocation was rejected before any
    /// capability ran: an unknown, missing, or
    /// malformed subcommand or option, an `--format`
    /// value outside the closed format set, or an
    /// `--offset-secs` outside the position floor.
    /// Carries the rendered diagnostic.
    #[error("{0}")]
    Usage(String),
    /// The document file could not be read to be
    /// fingerprinted, or a stream write failed.
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    /// A store operation failed.
    #[error("storage failure: {0}")]
    Store(#[from] crate::store::Error),
    /// The `rate set` value is not a member of the
    /// closed seven-rung ladder (FR-019) — rejected
    /// as a named error, never silently clamped.
    #[error("rate must be one of 0.75 | 1.0 | 1.25 | 1.5 | 1.75 | 2.0 | 4.0, got {value}")]
    OffLadderRate {
        /// The rejected value.
        value: f64,
    },
    /// The `put` `end_kind` is a terminal kind — the
    /// Stop press and the document end delete the
    /// stored entry instead of storing one
    /// (FR-032 / FR-032b), so they are rejected as
    /// `put` inputs.
    #[error(
        "the terminal session end {wire} deletes the stored entry instead of storing one (FR-032 / FR-032b)"
    )]
    TerminalEndKind {
        /// The rejected kind, in wire form.
        wire: &'static str,
    },
    /// The `--offset-secs` value is outside the
    /// position domain: finite and `>= 0.0`.
    #[error("offset-secs must be finite and >= 0.0, got {value:?}")]
    OffsetSecs {
        /// The rejected value.
        value: f64,
    },
    /// A machine-readable payload could not be
    /// rendered.
    #[error("payload rendering failed: {0}")]
    Rendering(#[from] serde_json::Error),
}

/// The session command line: the arguments after the
/// program name.
///
/// The parser is generated by the `clap` derive
/// macro — the grammar below is the whole parser, and
/// the generated `try_parse_from` is the only parser
/// in the crate; no argument-parsing logic is
/// hand-written.
#[derive(Parser, Debug)]
#[command(name = PROGRAM_NAME, version)]
struct Cli {
    /// The output format: the machine-readable JSON
    /// payload is the default for scripting, with the
    /// human-readable plain form selectable alongside
    /// it (Principle II).
    #[arg(long, global = true, value_enum, default_value = "json")]
    format: OutputFormat,

    /// The redb database file the bookmark store
    /// opens — and creates when absent
    /// (storage-schema.md). An invocation that
    /// names none answers from the scratch store
    /// under the system temp dir: the pipe form
    /// of the protocol (`echo <bytes> | session
    /// ingest`) has no store to name, and the
    /// rendered default states the file.
    #[arg(long, default_value_os_t = scratch_db_path())]
    db: PathBuf,

    /// The subcommand naming the capability to
    /// exercise.
    #[command(subcommand)]
    command: Command,
}

/// The closed output-format set: the machine-readable
/// JSON default and its human-readable sibling.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    /// The machine-readable JSON payload (the default).
    Json,
    /// The human-readable plain form.
    Human,
}

/// The subcommands, each named for the capability it
/// exercises.
#[derive(Subcommand, Debug)]
enum Command {
    /// Ingest a document: compute its fingerprint and
    /// report the resume state (FR-036 / FR-036a).
    ///
    /// The document bytes come from the named file
    /// (`ingest <snapshot-file>`, `contracts/
    /// ingest.md` "CLI surface", or the `--path`
    /// spelling), or from the bytes piped on
    /// stdin when no file is named. The bytes are
    /// blake3-256 fingerprinted into the bookmark
    /// key — never the path — and the lookup
    /// reports the resume decision the stored end
    /// kind resolves (`{fingerprint, resume,
    /// offset}` JSON).
    Ingest {
        /// The document file: its bytes are
        /// blake3-256 fingerprinted into the bookmark
        /// key — never its path.
        #[arg(conflicts_with = "path")]
        file: Option<PathBuf>,
        /// The document file named by flag — the same
        /// bytes rule, the alternative spelling of the
        /// positional form.
        #[arg(long, conflicts_with = "file")]
        path: Option<PathBuf>,
    },
    /// Read the stored position for a document, keyed
    /// by its content fingerprint (FR-036).
    Get {
        /// The document file: its bytes are
        /// blake3-256 fingerprinted into the bookmark
        /// key — never its path.
        file: PathBuf,
    },
    /// Store a position for a document, keyed by its
    /// content fingerprint (FR-036).
    Put {
        /// The document file: its bytes are
        /// blake3-256 fingerprinted into the bookmark
        /// key — never its path.
        file: PathBuf,
        /// The last spoken position, in seconds:
        /// finite and `>= 0.0`.
        #[arg(long)]
        offset_secs: f64,
        /// The start of the continuous non-advancing
        /// interval, in milliseconds (FR-015).
        #[arg(long)]
        first_yield_ms: u64,
        /// The session-end kind that stores the entry
        /// (FR-044): only `paused` and `interrupted`
        /// are storable — `stopped` and `completed`
        /// delete the entry instead (FR-032 /
        /// FR-032b) and are rejected as `put` inputs
        /// with a named error.
        #[arg(long, value_enum)]
        end_kind: SessionEndInput,
    },
    /// Delete the stored position for a document —
    /// the terminal ends' operation (FR-032 /
    /// FR-032b / FR-032f).
    Clear {
        /// The document file: its bytes are
        /// blake3-256 fingerprinted into the bookmark
        /// key — never its path.
        file: PathBuf,
    },
    /// The last playback rate (FR-019b).
    Rate {
        /// The rate capability to exercise.
        #[command(subcommand)]
        command: RateCommand,
    },
    /// The resume-offer flow (FR-032 series).
    Offer {
        /// The offer capability to exercise.
        #[command(subcommand)]
        command: OfferCommand,
    },
    /// Latch the application visibility (the
    /// `AppForegrounded` / `AppBackgrounded` intents):
    /// the latch is read at offer expiry only
    /// (FR-032c).
    Foreground {
        /// The visibility to latch.
        visibility: Visibility,
    },
}

/// The `rate` subcommands.
#[derive(Subcommand, Debug)]
enum RateCommand {
    /// Read the last playback rate (FR-019b) — the
    /// normal-speed rung `1.0` on first use.
    Get,
    /// Store the last playback rate (FR-019b): one
    /// rung of the closed ladder (FR-019).
    Set {
        /// The requested rate rung, as its JSON-ladder
        /// number (one of `0.75 | 1.0 | 1.25 | 1.5 |
        /// 1.75 | 2.0 | 4.0`).
        rate: f64,
    },
}

/// The `offer` subcommands.
#[derive(Subcommand, Debug)]
enum OfferCommand {
    /// Show the offer the stored session end opens —
    /// the offer form resolved solely from the stored
    /// `end_kind` (storage-schema.md).
    Show {
        /// The document file: its bytes are
        /// blake3-256 fingerprinted into the bookmark
        /// key — never its path.
        file: PathBuf,
    },
    /// The reader's continue choice: latch the
    /// application as foregrounded (the reader is
    /// present to choose) and read the open offer
    /// (FR-032c).
    Continue {
        /// The document file: its bytes are
        /// blake3-256 fingerprinted into the bookmark
        /// key — never its path.
        file: PathBuf,
    },
    /// The reader's start-over choice: reading begins
    /// from the first spoken position — exactly `0.0`
    /// — and the stored entry is discarded
    /// (FR-032c / FR-032f).
    StartOver {
        /// The document file: its bytes are
        /// blake3-256 fingerprinted into the bookmark
        /// key — never its path.
        file: PathBuf,
    },
    /// The 3-second window elapses: the offer is
    /// resolved against the latched visibility — a
    /// foregrounded deliberate pause auto-starts from
    /// the stored position (FR-032c / SC-020); a
    /// backgrounded one converts to the
    /// explicit-resume form (FR-032c / FR-031), as
    /// an unexpected end always does (FR-032d /
    /// SC-021).
    Expire {
        /// The document file: its bytes are
        /// blake3-256 fingerprinted into the bookmark
        /// key — never its path.
        file: PathBuf,
        /// The `AppForegrounded` intent for this
        /// invocation: the machine is per-invocation,
        /// so the latch is carried within the
        /// invocation that also expires — absent, the
        /// `AppBackgrounded` latch holds.
        #[arg(long)]
        foreground: bool,
    },
}

/// The `put` `end_kind` input: the closed
/// session-end set (FR-044), in wire form.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum SessionEndInput {
    /// The session ended by a user pause; position
    /// stored.
    Paused,
    /// The session ended by Stop; position discarded.
    Stopped,
    /// The session ended by document completion;
    /// position discarded.
    Completed,
    /// The session ended by interruption; resume
    /// offer applies.
    Interrupted,
}

/// The application visibility to latch: the
/// `AppForegrounded` / `AppBackgrounded` intents.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum Visibility {
    /// The application is foregrounded (the
    /// `AppForegrounded` intent).
    On,
    /// The application is backgrounded (the
    /// `AppBackgrounded` intent).
    Off,
}

/// Execute one parsed invocation: the single code
/// path every caller exercises (Principle II).
///
/// A success writes the payload to `stdout` and
/// returns `Ok`; a failure returns the diagnostic
/// having written nothing to `stdout`, so the caller
/// completes the failure half of the protocol.
fn execute(cli: &Cli, stdout: &mut dyn Write) -> Result<(), CliError> {
    match &cli.command {
        Command::Ingest { file, path } => ingest_command(cli, file.as_ref(), path.as_ref(), stdout),
        Command::Get { file } => get_command(cli, file, stdout),
        Command::Put {
            file,
            offset_secs,
            first_yield_ms,
            end_kind,
        } => put_command(cli, file, *offset_secs, *first_yield_ms, *end_kind, stdout),
        Command::Clear { file } => clear_command(cli, file, stdout),
        Command::Rate { command } => rate_command(cli, command, stdout),
        Command::Offer { command } => offer_command(cli, command, stdout),
        Command::Foreground { visibility } => foreground_command(cli, *visibility, stdout),
    }
}

/// The `ingest` verb: fingerprint the document and report the resume decision.
fn ingest_command(
    cli: &Cli,
    file: Option<&PathBuf>,
    path: Option<&PathBuf>,
    stdout: &mut dyn Write,
) -> Result<(), CliError> {
    // The document bytes: the named file's — positionally or by `--path` —
    // or the bytes piped on stdin when no file is named (the pipe form of
    // the protocol, `echo <bytes> | session ingest`). The bytes are
    // fingerprinted; the path is never the key (FR-036).
    let document_bytes = match file.or(path) {
        Some(path) => std::fs::read(path)?,
        None => read_stdin_bytes()?,
    };
    let key = fingerprint(&document_bytes);
    let mut store = BookmarkStore::open(&cli.db)?;
    let stored = store.get(&key)?;
    // The resume decision reads the stored end kind (storage-schema.md): only
    // the deliberate pause opens the continue offer, so `resume` is true
    // exactly then. An unexpected end (FR-032d) auto-resumes nothing, but its
    // stored position is still reported — the offer form itself resolves in
    // `offer.rs` (`offer show`), never here, so the payload carries the stored
    // offset on either path and `0.0` on a miss (FR-036a: an edited document
    // resolves to a start at 0.0 with no offer).
    let (resume, offset) = stored.map_or((false, 0.0), |position| {
        (
            matches!(position.end_kind, SessionEnd::Paused),
            position.offset_secs,
        )
    });
    let fingerprint_hex = hex_key(key);
    let payload = IngestPayload {
        fingerprint: &fingerprint_hex,
        resume,
        offset,
    };
    write_payload(stdout, &ingest_payload(&payload, cli.format)?)
}

/// The `get` verb: read the stored position for a document.
fn get_command(cli: &Cli, file: &Path, stdout: &mut dyn Write) -> Result<(), CliError> {
    let key = document_key(file)?;
    let mut store = BookmarkStore::open(&cli.db)?;
    let stored = store.get(&key)?;
    write_payload(
        stdout,
        &position_payload("get", file, key, stored, cli.format)?,
    )
}

/// The `put` verb: persist a bookmark for a document.
fn put_command(
    cli: &Cli,
    file: &Path,
    offset_secs: f64,
    first_yield_ms: u64,
    end_kind: SessionEndInput,
    stdout: &mut dyn Write,
) -> Result<(), CliError> {
    let end_kind = end_kind_of(end_kind);
    // The terminal ends delete the entry instead of storing one
    // (FR-032 / FR-032b): they are rejected as `put` inputs with a named
    // error, never stored.
    if matches!(end_kind, SessionEnd::Stopped | SessionEnd::Completed) {
        return Err(CliError::TerminalEndKind {
            wire: end_kind_wire(end_kind),
        });
    }
    let offset_secs = checked_offset(offset_secs)?;
    let key = document_key(file)?;
    let mut store = BookmarkStore::open(&cli.db)?;
    let position = StoredPosition {
        offset_secs,
        first_yield_ms,
        end_kind,
    };
    store.put(&key, position)?;
    tracing::debug!(
        offset_secs,
        first_yield_ms,
        end_kind = end_kind_wire(end_kind),
        "stored position"
    );
    write_payload(
        stdout,
        &position_payload("put", file, key, Some(position), cli.format)?,
    )
}

/// The `clear` verb: delete the stored position for a document.
fn clear_command(cli: &Cli, file: &Path, stdout: &mut dyn Write) -> Result<(), CliError> {
    let key = document_key(file)?;
    let mut store = BookmarkStore::open(&cli.db)?;
    store.delete(&key)?;
    tracing::debug!("deleted stored position");
    write_payload(stdout, &clear_payload(file, key, cli.format)?)
}

/// The `rate get` / `rate set` verbs.
fn rate_command(cli: &Cli, command: &RateCommand, stdout: &mut dyn Write) -> Result<(), CliError> {
    match command {
        RateCommand::Get => {
            let mut store = BookmarkStore::open(&cli.db)?;
            let stored = store.get_last_rate()?;
            // The call-site default is the normal-speed rung on first use
            // (storage-schema.md); the payload reports whether the stored value
            // or the default answered.
            let (rate, defaulted) = stored.map_or((PlaybackRate::R1_0, true), |rate| (rate, false));
            write_payload(stdout, &rate_get_payload(rate, defaulted, cli.format)?)
        }
        RateCommand::Set { rate } => {
            // The ladder is validated through the rung's own serde form — the
            // round trip the closed declaration itself defines — so no second
            // rung list exists here (FR-019 / FR-044).
            let rung = rate_of(*rate)?;
            let mut store = BookmarkStore::open(&cli.db)?;
            store.put_last_rate(rung)?;
            tracing::debug!(rate, "stored last rate");
            write_payload(stdout, &rate_set_payload(*rate, cli.format)?)
        }
    }
}

/// The `offer show` / `offer continue` / `offer expire` / `offer start-over` verbs.
fn offer_command(
    cli: &Cli,
    command: &OfferCommand,
    stdout: &mut dyn Write,
) -> Result<(), CliError> {
    match command {
        OfferCommand::Show { file } => {
            let key = document_key(file)?;
            let mut store = BookmarkStore::open(&cli.db)?;
            let stored = store.get(&key)?;
            let mut machine = OfferMachine::new();
            let offer = machine.on_session_end(key, stored);
            write_payload(
                stdout,
                &offer_payload("offer_show", file, key, offer, cli.format)?,
            )
        }
        OfferCommand::Continue { file } => {
            let key = document_key(file)?;
            let mut store = BookmarkStore::open(&cli.db)?;
            let stored = store.get(&key)?;
            let mut machine = OfferMachine::new();
            // The returned offer is re-read through `continue_offer` below —
            // the same open offer the reader chooses from.
            let _ = machine.on_session_end(key, stored);
            machine.on_foregrounded();
            let offer = machine.continue_offer();
            write_payload(
                stdout,
                &offer_payload("offer_continue", file, key, offer, cli.format)?,
            )
        }
        OfferCommand::StartOver { file } => {
            let key = document_key(file)?;
            let mut store = BookmarkStore::open(&cli.db)?;
            let mut machine = OfferMachine::new();
            let outcome = machine.start_over();
            // FR-032f: the stored position is discarded with the choice — the
            // discard is the caller's (the machine holds no store).
            store.delete(&key)?;
            tracing::debug!("start-over: stored position discarded");
            write_payload(stdout, &start_over_payload(file, key, outcome, cli.format)?)
        }
        OfferCommand::Expire { file, foreground } => {
            let key = document_key(file)?;
            let mut store = BookmarkStore::open(&cli.db)?;
            let stored = store.get(&key)?;
            let mut machine = OfferMachine::new();
            let _ = machine.on_session_end(key, stored);
            if *foreground {
                machine.on_foregrounded();
            }
            let outcome = machine.expire();
            write_payload(
                stdout,
                &expire_payload(file, key, *foreground, outcome, cli.format)?,
            )
        }
    }
}

/// The `foreground` verb: latch the application visibility.
fn foreground_command(
    cli: &Cli,
    visibility: Visibility,
    stdout: &mut dyn Write,
) -> Result<(), CliError> {
    let mut machine = OfferMachine::new();
    match visibility {
        Visibility::On => machine.on_foregrounded(),
        Visibility::Off => machine.on_backgrounded(),
    }
    // The machine holds no accessor for the latch, so the verb renders the
    // latch it applied — the latch's observable effect is the `offer expire`
    // outcome (SC-020).
    let foregrounded = matches!(visibility, Visibility::On);
    write_payload(stdout, &foreground_payload(foregrounded, cli.format)?)
}

/// The bookmark key of a document: the blake3-256
/// fingerprint of its bytes (FR-036) — the same
/// canonical [`fingerprint`] every other driver
/// resolves through.
///
/// # Errors
///
/// [`CliError::Io`] when the document file cannot be
/// read.
fn document_key(file: &Path) -> Result<[u8; 32], CliError> {
    let bytes = std::fs::read(file)?;
    // FR-018 cap check: oversize docs (> MAX_DOC_BYTES from fixture measurement)
    // refuse before synthesis with FormatUndeliverable (see interruption.rs).
    // Fixture: fixture-pt-2h.md = 62424 bytes; MAX_DOC_BYTES = 93636.
    Ok(fingerprint(&bytes))
}

/// The hex form of a document fingerprint: the
/// blake3-256 key as 64 lowercase hex digits, the
/// form a CI log can compare byte for byte.
#[must_use]
fn hex_key(key: [u8; 32]) -> String {
    let mut text = String::with_capacity(64);
    for byte in key {
        // The write cannot fail on a `String` sink;
        // the result is still handled because the
        // workspace denies unused results.
        let _ignored = write!(text, "{byte:02x}");
    }
    text
}

/// The session-end kind of a `put` input, in the
/// closed set's canonical type.
#[must_use]
const fn end_kind_of(input: SessionEndInput) -> SessionEnd {
    match input {
        SessionEndInput::Paused => SessionEnd::Paused,
        SessionEndInput::Stopped => SessionEnd::Stopped,
        SessionEndInput::Completed => SessionEnd::Completed,
        SessionEndInput::Interrupted => SessionEnd::Interrupted,
    }
}

/// The wire form of a session-end kind — the same
/// `snake_case` string the store's wire protocol
/// carries (the closed set's own projection, not a
/// second list).
#[must_use]
const fn end_kind_wire(end_kind: SessionEnd) -> &'static str {
    match end_kind {
        SessionEnd::Paused => "paused",
        SessionEnd::Stopped => "stopped",
        SessionEnd::Completed => "completed",
        SessionEnd::Interrupted => "interrupted",
    }
}

/// Check an `--offset-secs` value: finite and
/// `>= 0.0` — no document starts before its start.
///
/// # Errors
///
/// [`CliError::OffsetSecs`] when the value is
/// non-finite or negative.
fn checked_offset(value: f64) -> Result<f64, CliError> {
    if !value.is_finite() || value < 0.0 {
        return Err(CliError::OffsetSecs { value });
    }
    Ok(value)
}

/// Resolve a `rate set` value to its closed ladder
/// rung (FR-019).
///
/// The ladder is the single declaration in
/// `engine_core::closed`: the value round-trips
/// through the rung's own serde form, so no second
/// rung list exists here (FR-044). An off-ladder
/// value is a named error, never a silent clamp.
///
/// # Errors
///
/// [`CliError::OffLadderRate`] when the value is
/// not one of the seven declared rungs.
fn rate_of(value: f64) -> Result<PlaybackRate, CliError> {
    if !value.is_finite() {
        return Err(CliError::OffLadderRate { value });
    }
    serde_json::from_value::<PlaybackRate>(serde_json::json!(value))
        .map_err(|_| CliError::OffLadderRate { value })
}

/// The offer in wire form: the FR-032c continue
/// offer or the start-over alternative.
#[must_use]
const fn offer_wire(offer: Offer) -> OfferWire {
    match offer {
        Offer::Continue {
            offset_secs,
            first_yield_ms,
        } => OfferWire::Continue {
            offset_secs,
            first_yield_ms,
        },
        Offer::StartOver => OfferWire::StartOver,
    }
}

/// The expiry outcome in wire form.
#[must_use]
const fn expiry_wire(outcome: ExpiryOutcome) -> ExpiryWire {
    match outcome {
        ExpiryOutcome::AutoStart { offset_secs } => ExpiryWire::AutoStart { offset_secs },
        ExpiryOutcome::ExplicitResume => ExpiryWire::ExplicitResume,
        ExpiryOutcome::NoOffer => ExpiryWire::NoOffer,
    }
}

/// Write one success payload to `stdout` with a
/// trailing newline: the success half of the protocol
/// (payload on stdout, nothing on stderr).
fn write_payload(stdout: &mut dyn Write, payload: &str) -> Result<(), CliError> {
    writeln!(stdout, "{payload}")?;
    Ok(())
}

/// Render a payload as the machine-readable JSON
/// document — the default output format. The
/// `Result` return already carries `#[must_use]`.
fn render_json<T: Serialize>(payload: &T) -> Result<String, CliError> {
    Ok(serde_json::to_string(payload)?)
}

/// The stored position in wire form: the value the
/// `bookmarks` table carries (storage-schema.md).
#[derive(Serialize)]
struct StoredPayload {
    /// The last spoken position, in seconds.
    offset_secs: f64,
    /// The start of the continuous non-advancing
    /// interval, in milliseconds (FR-015).
    first_yield_ms: u64,
    /// The session-end kind that stored the entry
    /// (FR-044), in wire form.
    end_kind: SessionEnd,
}

/// The `get` / `put` payload.
#[derive(Serialize)]
struct PositionPayload<'a> {
    /// The capability the payload mirrors.
    capability: &'static str,
    /// The document file, as given.
    file: &'a Path,
    /// The document's blake3-256 content
    /// fingerprint, in hex (FR-036).
    fingerprint: &'a str,
    /// The stored position, or `null` when no entry
    /// survives (a miss resolves to a start at `0.0`
    /// with no offer — storage-schema.md).
    stored: Option<StoredPayload>,
}

/// The `clear` payload.
#[derive(Serialize)]
struct ClearPayload<'a> {
    /// The capability the payload mirrors.
    capability: &'static str,
    /// The document file, as given.
    file: &'a Path,
    /// The document's blake3-256 content
    /// fingerprint, in hex (FR-036).
    fingerprint: &'a str,
    /// Whether the delete ran — idempotent over an
    /// absent entry (the terminal ends' operation,
    /// FR-032 / FR-032b).
    deleted: bool,
}

/// The `rate get` payload.
#[derive(Serialize)]
struct RateGetPayload {
    /// The capability the payload mirrors.
    capability: &'static str,
    /// The effective last playback rate, as its
    /// JSON-ladder number (FR-019b).
    rate: f64,
    /// Whether the first-use default `1.0` answered
    /// (storage-schema.md) rather than a stored
    /// rung.
    defaulted: bool,
}

/// The `rate set` payload.
#[derive(Serialize)]
struct RateSetPayload {
    /// The capability the payload mirrors.
    capability: &'static str,
    /// The stored rung, as its JSON-ladder number
    /// (FR-019b).
    rate: f64,
}

/// The offer in wire form: the FR-032c continue
/// offer, or the start-over alternative.
#[derive(Serialize)]
#[serde(tag = "kind")]
enum OfferWire {
    /// Continue from the stored position — the
    /// deliberate-pause offer (FR-032c).
    #[serde(rename = "continue")]
    Continue {
        /// The stored last spoken position, in
        /// seconds.
        offset_secs: f64,
        /// The start of the continuous non-advancing
        /// interval, in milliseconds (FR-015).
        first_yield_ms: u64,
    },
    /// The start-over alternative (FR-032c): begin
    /// from the first spoken position.
    #[serde(rename = "start_over")]
    StartOver,
}

/// The `offer show` / `offer continue` payload.
#[derive(Serialize)]
struct OfferPayload<'a> {
    /// The capability the payload mirrors.
    capability: &'static str,
    /// The document file, as given.
    file: &'a Path,
    /// The document's blake3-256 content
    /// fingerprint, in hex (FR-036).
    fingerprint: &'a str,
    /// The open offer, or `null` when none is open —
    /// an unexpected end (FR-032d), a terminal end
    /// (FR-032 / FR-032e), or no stored entry opens
    /// nothing.
    offer: Option<OfferWire>,
}

/// The expiry outcome in wire form.
#[derive(Serialize)]
#[serde(tag = "kind")]
enum ExpiryWire {
    /// Reading begins from the stored position — the
    /// auto-start of a foregrounded deliberate-pause
    /// offer at expiry (SC-020), or the start-over
    /// first-spoken-position start.
    #[serde(rename = "auto_start")]
    AutoStart {
        /// The position reading begins from, in
        /// seconds.
        offset_secs: f64,
    },
    /// No audio begins: an explicit-resume control
    /// is all the session offers (SC-021 / FR-032c
    /// backgrounded / FR-032d).
    #[serde(rename = "explicit_resume")]
    ExplicitResume,
    /// Nothing was open — the window never outlives
    /// its session (FR-032f).
    #[serde(rename = "no_offer")]
    NoOffer,
}

/// The `offer start-over` payload.
#[derive(Serialize)]
struct StartOverPayload<'a> {
    /// The capability the payload mirrors.
    capability: &'static str,
    /// The document file, as given.
    file: &'a Path,
    /// The document's blake3-256 content
    /// fingerprint, in hex (FR-036).
    fingerprint: &'a str,
    /// The start-over outcome: reading begins from
    /// exactly `0.0` (FR-032c).
    outcome: ExpiryWire,
    /// Whether the stored entry was discarded with
    /// the choice (FR-032f).
    discarded: bool,
}

/// The `offer expire` payload.
#[derive(Serialize)]
struct ExpirePayload<'a> {
    /// The capability the payload mirrors.
    capability: &'static str,
    /// The document file, as given.
    file: &'a Path,
    /// The document's blake3-256 content
    /// fingerprint, in hex (FR-036).
    fingerprint: &'a str,
    /// The visibility latched at expiry (FR-032c /
    /// SC-020).
    foregrounded: bool,
    /// The outcome of the 3-second window elapsing.
    outcome: ExpiryWire,
}

/// The `foreground` payload.
#[derive(Serialize)]
struct ForegroundPayload {
    /// The capability the payload mirrors.
    capability: &'static str,
    /// The latched visibility: `true` for the
    /// `AppForegrounded` intent, `false` for the
    /// `AppBackgrounded` intent.
    foregrounded: bool,
}

/// The `ingest` payload: the document fingerprint
/// and resume state — the exact wire shape
/// `contracts/ingest.md` ("CLI surface") names:
/// `{fingerprint, resume: bool, offset}`.
#[derive(Serialize)]
struct IngestPayload<'a> {
    /// The document's blake3-256 content
    /// fingerprint, in hex (FR-036).
    fingerprint: &'a str,
    /// Whether a resume offer applies — `true`
    /// only when a stored entry exists AND its
    /// end kind is the deliberate pause
    /// (`paused`); an interrupted entry reports
    /// `false` here and resolves its offer form
    /// through `offer.rs` (FR-032d).
    resume: bool,
    /// The stored offset in seconds: the stored
    /// position when an entry exists (any
    /// retained end kind), `0.0` on a miss —
    /// a miss resolves to a start at `0.0` with
    /// no offer (FR-036a / storage-schema.md).
    offset: f64,
}

/// The `get` / `put` payload: the stored position
/// under the document's content fingerprint, for the
/// selected format.
///
/// The `Result` return already carries `#[must_use]`.
fn position_payload(
    capability: &'static str,
    file: &Path,
    key: [u8; 32],
    stored: Option<StoredPosition>,
    format: OutputFormat,
) -> Result<String, CliError> {
    let fingerprint_hex = hex_key(key);
    let payload = PositionPayload {
        capability,
        file,
        fingerprint: &fingerprint_hex,
        stored: stored.map(|position| StoredPayload {
            offset_secs: position.offset_secs,
            first_yield_ms: position.first_yield_ms,
            end_kind: position.end_kind,
        }),
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_position(&payload),
    })
}

/// The `clear` payload, for the selected format.
///
/// The `Result` return already carries `#[must_use]`.
fn clear_payload(file: &Path, key: [u8; 32], format: OutputFormat) -> Result<String, CliError> {
    let fingerprint_hex = hex_key(key);
    let payload = ClearPayload {
        capability: "clear",
        file,
        fingerprint: &fingerprint_hex,
        deleted: true,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_clear(&payload),
    })
}

/// The `rate get` payload, for the selected format.
///
/// The `Result` return already carries `#[must_use]`.
fn rate_get_payload(
    rate: PlaybackRate,
    defaulted: bool,
    format: OutputFormat,
) -> Result<String, CliError> {
    let payload = RateGetPayload {
        capability: "rate_get",
        rate: rate_wire(rate),
        defaulted,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_rate_get(&payload),
    })
}

/// The `rate set` payload, for the selected format.
///
/// The `Result` return already carries `#[must_use]`.
fn rate_set_payload(rate: f64, format: OutputFormat) -> Result<String, CliError> {
    let payload = RateSetPayload {
        capability: "rate_set",
        rate,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_rate_set(&payload),
    })
}

/// The `offer show` / `offer continue` payload, for
/// the selected format.
///
/// The `Result` return already carries `#[must_use]`.
fn offer_payload(
    capability: &'static str,
    file: &Path,
    key: [u8; 32],
    offer: Option<Offer>,
    format: OutputFormat,
) -> Result<String, CliError> {
    let fingerprint_hex = hex_key(key);
    let payload = OfferPayload {
        capability,
        file,
        fingerprint: &fingerprint_hex,
        offer: offer.map(offer_wire),
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_offer(&payload),
    })
}

/// The `offer start-over` payload, for the selected
/// format.
///
/// The `Result` return already carries `#[must_use]`.
fn start_over_payload(
    file: &Path,
    key: [u8; 32],
    outcome: ExpiryOutcome,
    format: OutputFormat,
) -> Result<String, CliError> {
    let fingerprint_hex = hex_key(key);
    let payload = StartOverPayload {
        capability: "offer_start_over",
        file,
        fingerprint: &fingerprint_hex,
        outcome: expiry_wire(outcome),
        discarded: true,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_start_over(&payload),
    })
}

/// The `offer expire` payload, for the selected
/// format.
///
/// The `Result` return already carries `#[must_use]`.
fn expire_payload(
    file: &Path,
    key: [u8; 32],
    foregrounded: bool,
    outcome: ExpiryOutcome,
    format: OutputFormat,
) -> Result<String, CliError> {
    let fingerprint_hex = hex_key(key);
    let payload = ExpirePayload {
        capability: "offer_expire",
        file,
        fingerprint: &fingerprint_hex,
        foregrounded,
        outcome: expiry_wire(outcome),
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_expire(&payload),
    })
}

/// The `foreground` payload, for the selected format.
///
/// The `Result` return already carries `#[must_use]`.
fn foreground_payload(foregrounded: bool, format: OutputFormat) -> Result<String, CliError> {
    let payload = ForegroundPayload {
        capability: "foreground",
        foregrounded,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_foreground(&payload),
    })
}

/// The `ingest` payload, for the selected format.
///
/// The `Result` return already carries `#[must_use]`.
fn ingest_payload(payload: &IngestPayload<'_>, format: OutputFormat) -> Result<String, CliError> {
    Ok(match format {
        OutputFormat::Json => render_json(payload)?,
        OutputFormat::Human => human_ingest(payload),
    })
}

/// The scratch store an invocation that names no
/// `--db` answers from: one file under the system
/// temp dir ([`std::env::temp_dir`]). It keeps the
/// pipe form of the protocol zero-argument — the
/// bytes piped on stdin carry no store to name —
/// while every explicit invocation still states
/// its store, and the rendered `--help` shows the
/// default file so the answer is never silent.
fn scratch_db_path() -> PathBuf {
    std::env::temp_dir().join("mdreader-session.redb")
}

/// Read the document bytes from stdin to end of
/// stream: the pipe form of the text-I/O protocol
/// (`echo <bytes> | session ingest`).
///
/// # Errors
///
/// [`CliError::Io`] when the stream cannot be read.
fn read_stdin_bytes() -> Result<Vec<u8>, CliError> {
    let mut bytes = Vec::new();
    // The count of bytes read answers no payload
    // field; the result is still handled because
    // the workspace denies unused results.
    let _read = std::io::stdin().read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// The rung's numeric value — its JSON-ladder
/// wire number. The projection mirrors the
/// rung's own `serde` form in
/// `engine_core::closed` — the closed
/// declaration remains the single list
/// (FR-019 / FR-044), and the exhaustive
/// match (no wildcard) fails to compile if
/// the ladder ever grows, so the two cannot
/// drift.
#[must_use]
const fn rate_wire(rate: PlaybackRate) -> f64 {
    match rate {
        PlaybackRate::R0_75 => 0.75,
        PlaybackRate::R1_0 => 1.0,
        PlaybackRate::R1_25 => 1.25,
        PlaybackRate::R1_5 => 1.5,
        PlaybackRate::R1_75 => 1.75,
        PlaybackRate::R2_0 => 2.0,
        PlaybackRate::R4_0 => 4.0,
    }
}

/// The human-readable plain form of the `get` /
/// `put` payload: the labeled stored position, or
/// the no-entry fact.
#[must_use]
fn human_position(payload: &PositionPayload<'_>) -> String {
    let mut text = format!(
        "capability: {}\nfile: {}\nfingerprint: {}\n",
        payload.capability,
        payload.file.display(),
        payload.fingerprint,
    );
    match payload.stored.as_ref() {
        Some(stored) => {
            // The writes cannot fail on a `String`
            // sink; the results are still handled
            // because the workspace denies unused
            // results.
            let _ignored = writeln!(text, "offset-secs: {}", stored.offset_secs);
            let _ignored = writeln!(text, "first-yield-ms: {}", stored.first_yield_ms);
            let _ignored = writeln!(text, "end-kind: {}", end_kind_wire(stored.end_kind));
        }
        None => text.push_str("stored: none\n"),
    }
    text
}

/// The human-readable plain form of the `clear`
/// payload.
#[must_use]
fn human_clear(payload: &ClearPayload<'_>) -> String {
    format!(
        "capability: {}\nfile: {}\nfingerprint: {}\ndeleted: {}\n",
        payload.capability,
        payload.file.display(),
        payload.fingerprint,
        payload.deleted,
    )
}

/// The human-readable plain form of the `rate get`
/// payload.
#[must_use]
fn human_rate_get(payload: &RateGetPayload) -> String {
    format!(
        "capability: {}\nrate: {}\ndefaulted: {}\n",
        payload.capability, payload.rate, payload.defaulted,
    )
}

/// The human-readable plain form of the `rate set`
/// payload.
#[must_use]
fn human_rate_set(payload: &RateSetPayload) -> String {
    format!(
        "capability: {}\nrate: {}\n",
        payload.capability, payload.rate,
    )
}

/// The human-readable plain form of the `offer show`
/// / `offer continue` payload: the labeled offer, or
/// the no-offer fact.
#[must_use]
fn human_offer(payload: &OfferPayload<'_>) -> String {
    let mut text = format!(
        "capability: {}\nfile: {}\nfingerprint: {}\n",
        payload.capability,
        payload.file.display(),
        payload.fingerprint,
    );
    match payload.offer.as_ref() {
        Some(OfferWire::Continue {
            offset_secs,
            first_yield_ms,
        }) => {
            // The writes cannot fail on a `String`
            // sink; the results are still handled
            // because the workspace denies unused
            // results.
            let _ignored = writeln!(text, "offer: continue");
            let _ignored = writeln!(text, "offset-secs: {offset_secs}");
            let _ignored = writeln!(text, "first-yield-ms: {first_yield_ms}");
        }
        Some(OfferWire::StartOver) => text.push_str("offer: start_over\n"),
        None => text.push_str("offer: none\n"),
    }
    text
}

/// The human-readable plain form of the
/// `offer start-over` payload.
#[must_use]
fn human_start_over(payload: &StartOverPayload<'_>) -> String {
    let mut text = format!(
        "capability: {}\nfile: {}\nfingerprint: {}\noutcome: ",
        payload.capability,
        payload.file.display(),
        payload.fingerprint,
    );
    match &payload.outcome {
        ExpiryWire::AutoStart { offset_secs } => {
            // The write cannot fail on a `String`
            // sink; the result is still handled
            // because the workspace denies unused
            // results.
            let _ignored = writeln!(text, "auto_start");
            let _ignored = writeln!(text, "offset-secs: {offset_secs}");
        }
        ExpiryWire::ExplicitResume => text.push_str("explicit_resume\n"),
        ExpiryWire::NoOffer => text.push_str("no_offer\n"),
    }
    // The write cannot fail on a `String` sink; the
    // result is still handled because the workspace
    // denies unused results.
    let _ignored = writeln!(text, "discarded: {}", payload.discarded);
    text
}

/// The human-readable plain form of the
/// `offer expire` payload.
#[must_use]
fn human_expire(payload: &ExpirePayload<'_>) -> String {
    let mut text = format!(
        "capability: {}\nfile: {}\nfingerprint: {}\nforegrounded: {}\noutcome: ",
        payload.capability,
        payload.file.display(),
        payload.fingerprint,
        payload.foregrounded,
    );
    match &payload.outcome {
        ExpiryWire::AutoStart { offset_secs } => {
            // The writes cannot fail on a `String`
            // sink; the results are still handled
            // because the workspace denies unused
            // results.
            let _ignored = writeln!(text, "auto_start");
            let _ignored = writeln!(text, "offset-secs: {offset_secs}");
        }
        ExpiryWire::ExplicitResume => text.push_str("explicit_resume\n"),
        ExpiryWire::NoOffer => text.push_str("no_offer\n"),
    }
    text
}

/// The human-readable plain form of the `foreground`
/// payload.
#[must_use]
fn human_foreground(payload: &ForegroundPayload) -> String {
    format!(
        "capability: {}\nforegrounded: {}\n",
        payload.capability, payload.foregrounded,
    )
}

/// The human-readable plain form of the `ingest`
/// payload: the labeled fingerprint, resume
/// decision, and stored offset.
#[must_use]
fn human_ingest(payload: &IngestPayload<'_>) -> String {
    format!(
        "fingerprint: {}\nresume: {}\noffset: {}\n",
        payload.fingerprint, payload.resume, payload.offset,
    )
}
