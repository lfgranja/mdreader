//! Player CLI entry point (T027).
//!
//! CLI-drivable per Principle II: stdin/args → stdout,
//! errors → stderr, `--format json`. Each verb calls the
//! identical backend function as the IPC verb.

#![expect(
    clippy::multiple_crate_versions,
    reason = "every duplicate is third-party-forced at newest-release pins: ndk 0.9.0 (directly and via cpal 0.18.2) carries thiserror 1.0.69 and jni-sys 0.3.1 — whose published 0.3.1 release itself depends on jni-sys 0.4.1 — while cpal's jni 0.22.4 carries thiserror 2.0.21 and jni-sys 0.4.1; cargo cannot unify across semver majors or a crate with its own re-export, crates.io shows no newer versions, and supply-chain governance for these transitive duplicates belongs to cargo-deny, not this lint"
)]

use clap::{Parser, Subcommand};
use player::cli::{PlaybackSession, ShellVerb};
use player::earcon::{EarconDescriptor, EarconKind};
use serde_json::Value;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(name = "player")]
#[command(about = "Playback engine CLI — mirror of IPC verbs")]
struct Cli {
    /// Subcommand.
    #[command(subcommand)]
    command: Commands,
    /// Output format.
    #[arg(long, default_value = "json", global = true)]
    format: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Play.
    Play,
    /// Pause.
    Pause,
    /// Stop.
    Stop,
    /// Skip forward 5 s.
    SkipForward,
    /// Skip backward 5 s.
    SkipBack,
    /// Rate up one step.
    RateUp,
    /// Rate down one step.
    RateDown,
    /// Resume at stored position.
    ResumeAtStored,
    /// Start over from 0.0.
    StartOver,
    /// Read current position.
    Position,
    /// Read current state.
    State,
    /// Read current engine tier.
    EngineTier,
    /// Earcon PCM descriptor (T030).
    #[command(subcommand)]
    Earcon(EarconCommands),
    /// Content-offset progress payload (T030b): the machine-readable
    /// progress shape the lock-screen and in-app surfaces share,
    /// with the frozen contract label.
    Progress {
        /// Parsed-content units consumed (content-offset, never
        /// audio-duration).
        #[arg(long, required_unless_present = "indeterminate")]
        consumed: Option<u64>,
        /// Total parsed-content units known at session start.
        #[arg(long, required_unless_present = "indeterminate")]
        total: Option<u64>,
        /// Current section index `k` (1-based; 0 with zero sections).
        #[arg(long, required_unless_present = "indeterminate")]
        k: Option<u32>,
        /// Total sections `n` (0 with zero sections).
        #[arg(long, required_unless_present = "indeterminate")]
        n: Option<u32>,
        /// The indeterminate form: no determinable percent. Zero
        /// sections (`--k 0 --n 0`) resolves here too — the whole
        /// session stays indeterminate, never a flip.
        #[arg(long, conflicts_with_all = ["consumed", "total", "k", "n"])]
        indeterminate: bool,
    },
}

/// Earcon subcommands — one per `EarconKind`.
#[derive(Subcommand, Debug)]
enum EarconCommands {
    /// Play confirmation.
    Play,
    /// Pause confirmation.
    Pause,
    /// Skip forward confirmation.
    #[command(name = "skip-fwd")]
    SkipFwd,
    /// Skip backward confirmation.
    #[command(name = "skip-back")]
    SkipBack,
    /// Warning earcon.
    Warning,
    /// Failure earcon.
    Failure,
    /// Rate step confirmation.
    Rate,
}

/// The usage diagnostic of the progress branch — the failure half of
/// the CLI protocol: the diagnostic to stderr, exit 2, nothing on
/// stdout (FR-005). `-> !` lets the let-else's `else` arm diverge
/// exactly as the inline exit did.
///
/// `clippy::print_stderr` is suppressed the sanctioned way —
/// `#[expect(lint, reason = "...")]`, never an `allow` and never a
/// weakened lint table — because the workspace denies printing in
/// every target, binaries included.
#[expect(
    clippy::print_stderr,
    reason = "the progress branch's usage error must reach the operator on stderr while stdout stays reserved for the progress line (FR-005); main has no other channel to state it on"
)]
fn refuse_progress_usage() -> ! {
    eprintln!(
        "usage: player progress (--consumed <u> --total <u> --k <k> --n <n> | --indeterminate)"
    );
    std::process::exit(2);
}

#[expect(
    clippy::too_many_lines,
    reason = "the CLI dispatch is the binary's single closed match over the clap command tree; each arm wires one subcommand to the session API and splitting it would separate the usage-error exits from their arms"
)]
fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut session = PlaybackSession::new();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    match cli.command {
        Commands::Play => {
            let _ = session.execute(ShellVerb::Play);
        }
        Commands::Pause => {
            let _ = session.execute(ShellVerb::Pause);
        }
        Commands::Stop => {
            let _ = session.execute(ShellVerb::Stop);
        }
        Commands::SkipForward => {
            let _ = session.execute(ShellVerb::SkipForward);
        }
        Commands::SkipBack => {
            let _ = session.execute(ShellVerb::SkipBackward);
        }
        Commands::RateUp => {
            let _ = session.execute(ShellVerb::RateUp);
        }
        Commands::RateDown => {
            let _ = session.execute(ShellVerb::RateDown);
        }
        Commands::ResumeAtStored => {
            let _ = session.execute(ShellVerb::ResumeAtStored);
        }
        Commands::StartOver => {
            let _ = session.execute(ShellVerb::StartOver);
        }
        Commands::Position => {
            let snap = session.snapshot();
            writeln!(
                out,
                "{{\"transport\":\"{:?}\",\"position_secs\":{}}}",
                snap.transport, snap.position_secs
            )?;
            return Ok(());
        }
        Commands::State => {
            let snap = session.snapshot();
            writeln!(out, "{{\"transport\":\"{:?}\"}}", snap.transport)?;
            return Ok(());
        }
        Commands::EngineTier => {
            writeln!(out, "{{\"engine_tier\":\"offline_default\"}}")?;
            return Ok(());
        }
        Commands::Progress {
            consumed,
            total,
            k,
            n,
            indeterminate,
        } => {
            // The payload: exactly {progress, label, section_title}
            // — the machine-readable progress contract
            // (contracts/surface.md "CLI surface"; Principle II: the
            // Tauri IPC drives this same code).
            let progress = if indeterminate {
                player::surface::Progress::Indeterminate
            } else {
                // clap's required_unless_present guarantees the four
                // values are present on this branch; the tuple
                // destructure states that as the closed requirement it
                // is (no catch-all arm), and the failure half of the
                // protocol — diagnostic to stderr, nothing on stdout —
                // answers a flag combination clap cannot produce.
                let (Some(consumed_units), Some(total_units), Some(k), Some(n)) =
                    (consumed, total, k, n)
                else {
                    refuse_progress_usage();
                };
                // The zero-sections case (k = 0, n = 0) resolves
                // indeterminate — the whole session stays
                // indeterminate, never a flip.
                if k == 0 && n == 0 {
                    player::surface::Progress::Indeterminate
                } else {
                    player::surface::Progress::Determinate {
                        consumed_units,
                        total_units,
                        k,
                        n,
                    }
                }
            };
            let label = player::surface::progress_label(&progress);
            let payload = serde_json::json!({
                "progress": progress,
                "label": label,
                "section_title": Value::Null,
            });
            let json = serde_json::to_string(&payload)?;
            writeln!(out, "{json}")?;
            return Ok(());
        }
        Commands::Earcon(earcon_cmd) => {
            let kind = match earcon_cmd {
                EarconCommands::Play => EarconKind::Play,
                EarconCommands::Pause => EarconKind::Pause,
                EarconCommands::SkipFwd => EarconKind::SkipFwd,
                EarconCommands::SkipBack => EarconKind::SkipBack,
                EarconCommands::Warning => EarconKind::Warning,
                EarconCommands::Failure => EarconKind::Failure,
                EarconCommands::Rate => EarconKind::RateStep,
            };
            let descriptor = EarconDescriptor::for_kind(kind);
            // Output as JSON (--format json is the only supported format per contract)
            // serde_json::Error converts into io::Error through its From impl,
            // so the failure goes to stderr through the error channel — never
            // a catch-all ErrorKind::Other (closed-set gate, SC-057).
            let json = serde_json::to_string(&descriptor)?;
            writeln!(out, "{json}")?;
            return Ok(());
        }
    }
    let snap = session.snapshot();
    writeln!(
        out,
        "{{\"transport\":\"{:?}\",\"position_secs\":{},\"rate\":{:?},\"engine_id\":\"{}\"}}",
        snap.transport, snap.position_secs, snap.rate, snap.engine_id
    )?;
    Ok(())
}
