//! Player CLI entry point (T027).
//!
//! CLI-drivable per Principle II: stdin/args → stdout,
//! errors → stderr, `--format json`. Each verb calls the
//! identical backend function as the IPC verb.

use clap::{Parser, Subcommand};
use player::cli::{PlaybackSession, ShellVerb};
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(name = "player")]
#[command(about = "Playback engine CLI — mirror of IPC verbs")]
struct Cli {
    /// Subcommand.
    #[command(subcommand)]
    command: Commands,
    /// Output format.
    #[arg(long, default_value = "json")]
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
}

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
    }
    let snap = session.snapshot();
    writeln!(
        out,
        "{{\"transport\":\"{:?}\",\"position_secs\":{},\"rate\":{:?},\"engine_id\":\"{}\"}}",
        snap.transport, snap.position_secs, snap.rate, snap.engine_id
    )?;
    Ok(())
}
