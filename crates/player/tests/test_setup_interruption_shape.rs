//! T002e — BLOCKING shape test for interruption reasons (red before T004).
//! Asserts the 8-reason closed set (FR-005) and stub contracts.

use player::interruption::{InterruptionReason, reason_line};

#[test]
fn interruption_enum_has_exactly_8_variants_no_other() {
    let all: Vec<InterruptionReason> = vec![
        InterruptionReason::FocusDenied,
        InterruptionReason::FocusDelayed,
        InterruptionReason::PermanentLoss,
        InterruptionReason::TransientPaused,
        InterruptionReason::DeviceRemoved,
        InterruptionReason::VoiceUnavailable,
        InterruptionReason::FormatUndeliverable,
        InterruptionReason::DecodeFailed,
    ];
    assert_eq!(all.len(), 8, "expected exactly 8 interruption reasons, no Other/Unknown");
}

#[test]
fn interruption_enum_exhaustive_no_other_variant() {
    fn take(r: InterruptionReason) -> &'static str {
        match r {
            InterruptionReason::FocusDenied => "FocusDenied",
            InterruptionReason::FocusDelayed => "FocusDelayed",
            InterruptionReason::PermanentLoss => "PermanentLoss",
            InterruptionReason::TransientPaused => "TransientPaused",
            InterruptionReason::DeviceRemoved => "DeviceRemoved",
            InterruptionReason::VoiceUnavailable => "VoiceUnavailable",
            InterruptionReason::FormatUndeliverable => "FormatUndeliverable",
            InterruptionReason::DecodeFailed => "DecodeFailed",
        }
    }
    assert_eq!(take(InterruptionReason::DecodeFailed), "DecodeFailed");
}

#[test]
fn reason_line_exhaustive_no_catchall_and_word_bound() {
    let cases = [
        (InterruptionReason::FocusDenied, "Refused"),
        (InterruptionReason::FocusDelayed, "Deferred"),
        (InterruptionReason::PermanentLoss, "Stopped"),
        (InterruptionReason::TransientPaused, "Paused"),
        (InterruptionReason::DeviceRemoved, "Disconnected"),
        (InterruptionReason::VoiceUnavailable, "Voice missing"),
        (InterruptionReason::FormatUndeliverable, "Format bad"),
        (InterruptionReason::DecodeFailed, "Decode error"),
    ];
    for (r, expected) in cases {
        let line = reason_line(&r);
        assert_eq!(line, expected,
            "reason_line({:?}) expected final contract label {:?}; stub is incomplete (red before T004)",
            r, expected);
        let words: Vec<&str> = line.split_whitespace().collect();
        assert!(words.len() <= 8,
            "reason_line({:?}) exceeds 8 words: {:?} ({} words)", r, line, words.len());
    }
}

#[test]
fn reason_line_has_no_catchall_match_arm() {
    let src = include_str!("../src/interruption.rs");
    assert!(!src.contains("_ =>"),
        "interruption.rs contains a `_ =>` catch-all arm — exhaustive closed set requires none");
}
