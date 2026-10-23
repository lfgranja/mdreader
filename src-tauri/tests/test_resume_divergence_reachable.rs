//! T160 — RED-FIRST property test: the resume's reachability after a
//! jump (feature 004-ressoa-dogfood-build, task T160).
//!
//! THE FINDING (tasks.md F1, MEDIUM). The resume path refused a play
//! whose queued audio no longer named the stored place — "the queued
//! audio does not match the resume place" — and the refusal left
//! two driving-time operations unreachable: a play after a jump (the
//! jump moved the stored place while the queue stood still) and a
//! live start-over away from the first spoken place. The cold-start
//! open was the only recourse.
//!
//! THE CONTRACT.
//!
//! * The reader's play REACHES the stored place: when the queued
//!   audio has diverged, the correct stretch is RE-SYNTHESISED from
//!   the stored place, and the reading continues there.
//! * The wrong stretch is NEVER played: the diverged queue is
//!   DROPPED before the re-synthesis, so the fail-closed default
//!   stands (FR-007).
//! * When the stretch cannot be re-synthesised, the refusal stands:
//!   the reading stays silent rather than play the wrong words under
//!   the stored place.
//!
//! MECHANISM. The session's own half is runtime: the closed
//! state transition table is public, so the jump, the pause and
//! the reader's play are exercised against the REAL table, and the
//! place the reader is told is read from the snapshot's own
//! rendering. The bridge's half is a source reading: the resume
//! path, the re-anchor and the fail-closed refusal are read out of
//! the production source and asserted as invariants — the same
//! discipline `src-tauri/tests/test_resume_restarts_audio.rs` uses.
//!
//! SHELL-VOCABULARY NOTE. This file is outside the shell-purity
//! gate's vocabulary allowlist, so it names no shell-vocabulary word
//! anywhere — not even in prose, not even in a comment. The
//! session's own words are spoken of as "the state", "the reading's
//! place" and "the jump" instead. Keep it that way: the gate greps
//! the WHOLE file, and a word introduced here fails the gate for
//! every other task.

use std::collections::BTreeMap;

use player::cli::{PlaybackSession, ShellVerb};
use proptest::prelude::*;

/// The bridge — the file that owns the resume path.
const BRIDGE: &str = include_str!("../src/lib.rs");

/// The production part of the bridge: everything before its first
/// inline test module.
fn production_source() -> &'static str {
    let mut offset = 0usize;
    for line in BRIDGE.split_inclusive('\n') {
        let trimmed = line.trim();
        let opens_a_test_module = trimmed.starts_with("mod ")
            && (trimmed.contains("_tests") || trimmed.contains(" tests"))
            && trimmed.ends_with('{');
        if opens_a_test_module {
            return &BRIDGE[..offset];
        }
        offset += line.len();
    }
    BRIDGE
}

/// The declared functions of the production source, with their
/// bodies.
fn fn_bodies(source: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut index = 0usize;
    while let Some(found) = source[index..].find("fn ") {
        let at = index + found;
        let name_start = at + 3;
        let name_end = name_start
            + source[name_start..]
                .find(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
                .unwrap_or(source.len() - name_start);
        let name = source[name_start..name_end].to_string();
        let Some(brace) = source[name_end..].find('{') else {
            break;
        };
        let open = name_end + brace;
        let Some(close) = matching_brace(source, open) else {
            break;
        };
        let _inserted = out.insert(name, source[open..=close].to_string());
        index = close;
    }
    out
}

/// The brace that closes the one at `open`.
const fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index = skip_string(bytes, index);
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
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

/// The end of the string literal that starts at `start`.
const fn skip_string(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    let mut escaped = false;
    while index < bytes.len() {
        if escaped {
            escaped = false;
        } else if bytes[index] == b'\\' {
            escaped = true;
        } else if bytes[index] == b'"' {
            return index + 1;
        }
        index += 1;
    }
    index
}

// ===========================================================================
// The properties.
// ===========================================================================

// T160 RED-FIRST: the reader's play REACHES the stored place after a
// jump.
//
// The jump moves the stored place while the queue stands still, so
// the queued audio no longer names the stored place. The session's
// own half is the closed transition table: the jump is accepted
// while the reading plays, the pause retains the place the jump
// moved it to, and the reader's play is ACCEPTED and lands on the
// playing state with that same place (FR-007).
//
// The place is read from the snapshot's own rendering: the two
// snapshots differ in the state's name alone, so the place the
// reader is told after the resume is the place the jump stored.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_the_play_after_a_jump_reaches_the_stored_place(elapsed in 0.0f64..3_600.0) {
        let mut session = PlaybackSession::new();
        let started = session.execute(ShellVerb::Play);
        prop_assert!(started.is_ok(), "a fresh session must start playing");
        session.on_audible_progress(elapsed);
        let before_jump = format!("{:?}", session.snapshot());

        // The jump: the stored place moves, the queue stands still.
        let jumped = session.execute(ShellVerb::SkipForward);
        prop_assert!(
            jumped.is_ok(),
            "a playing reading must accept the jump (the closed state transition table)"
        );
        // The pause: the place the jump moved to is retained.
        let paused = session.execute(ShellVerb::Pause);
        prop_assert!(paused.is_ok(), "a playing reading must accept the pause");
        let at_pause = format!("{:?}", session.snapshot());
        prop_assert_ne!(
            &before_jump,
            &at_pause,
            "the jump must move the stored place: the reading sat at {} and the jump left it at {}",
            before_jump,
            at_pause
        );

        // The reader's play: accepted, on the playing state, at the
        // stored place.
        let resumed = session.execute(ShellVerb::ResumeAtStored);
        prop_assert!(
            resumed.is_ok(),
            "the reader's play MUST be accepted after a jump (FR-007): the reading's place is \
             {at_pause} and the play must reach it, not refuse the reader"
        );
        let after = format!("{:?}", session.snapshot());
        prop_assert!(
            after.contains("Playing"),
            "the reader's play must land the reading on the playing state, the snapshot reads {}",
            after
        );
        prop_assert_eq!(
            at_pause.replace("Paused", "Playing"),
            after.clone(),
            "the reader's play MUST reach the stored place (FR-007): the reading paused at {} and \
             the play left it at {}",
            at_pause,
            after
        );
    }
}

/// T160: the resume path REACHES the stored place when the queued
/// audio has diverged — it re-synthesises the correct stretch
/// instead of refusing the reader's play — and the wrong stretch is
/// never played.
#[test]
fn the_resume_path_re_anchors_the_stored_place() {
    let bodies = fn_bodies(production_source());
    let Some(resume) = bodies.get("resume_class_verb") else {
        assert!(
            bodies.contains_key("resume_class_verb"),
            "T160: the production declares no resume path"
        );
        return;
    };
    assert!(
        resume.contains("restart_backend_stream("),
        "T160: the resume path does not restart the backend's stream on the queued audio — the \
         fast path (the queued audio still names the stored place) must stay (FR-005)"
    );
    assert!(
        resume.contains("reanchor_at("),
        "T160: the resume path does not re-anchor: the reader's play after a jump MUST reach the \
         stored place, not refuse it (FR-007)"
    );
    assert!(
        resume.contains("refuse("),
        "T160: the fail-closed default must stand: when the stretch cannot be re-synthesised, \
         the reading stays silent rather than play the wrong words under the stored place \
         (FR-007)"
    );

    let Some(reanchor) = bodies.get("reanchor_at") else {
        assert!(
            bodies.contains_key("reanchor_at"),
            "T160: the production declares no re-anchor — the reader's play after a jump must \
             reach the stored place (FR-007)"
        );
        return;
    };
    assert!(
        reanchor.contains("stop_backend("),
        "T160: the re-anchor does not drop the diverged queue — the wrong stretch must NEVER be \
         played (FR-007)"
    );
    assert!(
        reanchor.contains("play_document("),
        "T160: the re-anchor does not re-synthesise the correct stretch — the reader's play must \
         reach the stored place (FR-007)"
    );
    let drop_at = reanchor.find("stop_backend(").unwrap_or(usize::MAX);
    let synth_at = reanchor.find("play_document(").unwrap_or(usize::MAX);
    assert!(
        drop_at < synth_at,
        "T160: the re-anchor re-synthesises BEFORE it drops the diverged queue — the wrong \
         stretch must never be played (FR-007)"
    );
}
