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
    assert_eq!(
        all.len(),
        8,
        "expected exactly 8 interruption reasons, no Other/Unknown"
    );
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
        (
            InterruptionReason::FocusDenied,
            "Foco de áudio negado — pausado",
        ),
        (InterruptionReason::FocusDelayed, ""),
        (InterruptionReason::PermanentLoss, "Áudio perdido — pausado"),
        (
            InterruptionReason::TransientPaused,
            "Pausado por interrupção — retome para continuar",
        ),
        (
            InterruptionReason::DeviceRemoved,
            "Fones removidos — pausado",
        ),
        (
            InterruptionReason::VoiceUnavailable,
            "Voz indisponível — leitura não pode iniciar",
        ),
        (
            InterruptionReason::FormatUndeliverable,
            "Formato do documento não pode ser lido",
        ),
        (
            InterruptionReason::DecodeFailed,
            "Falha ao decodificar áudio — leitura parada",
        ),
    ];
    for (r, expected) in cases {
        let line = reason_line(&r);
        assert_eq!(
            line, expected,
            "reason_line({r:?}) expected final contract label {expected:?}"
        );
        // ≤8 words (split by whitespace) for non-empty lines
        if !line.is_empty() {
            let words: Vec<&str> = line.split_whitespace().collect();
            assert!(
                words.len() <= 8,
                "reason_line({:?}) exceeds 8 words: {:?} ({} words)",
                r,
                line,
                words.len()
            );
        }
    }
}

#[test]
fn reason_line_has_no_catchall_match_arm() {
    let src = include_str!("../src/interruption.rs");
    assert!(
        !src.contains("_ =>"),
        "interruption.rs contains a `_ =>` catch-all arm — exhaustive closed set requires none"
    );
}

#[test]
fn is_persistent_matches_undeliverable_set() {
    assert!(!InterruptionReason::FocusDenied.is_persistent());
    assert!(!InterruptionReason::FocusDelayed.is_persistent());
    assert!(!InterruptionReason::PermanentLoss.is_persistent());
    assert!(!InterruptionReason::TransientPaused.is_persistent());
    assert!(!InterruptionReason::DeviceRemoved.is_persistent());
    assert!(InterruptionReason::VoiceUnavailable.is_persistent());
    assert!(InterruptionReason::FormatUndeliverable.is_persistent());
    assert!(InterruptionReason::DecodeFailed.is_persistent());
}

#[test]
fn reason_ttl_ms_transient_3000_persistent_none() {
    assert_eq!(InterruptionReason::FocusDenied.reason_ttl_ms(), Some(3000));
    assert_eq!(InterruptionReason::FocusDelayed.reason_ttl_ms(), Some(3000));
    assert_eq!(
        InterruptionReason::PermanentLoss.reason_ttl_ms(),
        Some(3000)
    );
    assert_eq!(
        InterruptionReason::TransientPaused.reason_ttl_ms(),
        Some(3000)
    );
    assert_eq!(
        InterruptionReason::DeviceRemoved.reason_ttl_ms(),
        Some(3000)
    );
    assert_eq!(InterruptionReason::VoiceUnavailable.reason_ttl_ms(), None);
    assert_eq!(
        InterruptionReason::FormatUndeliverable.reason_ttl_ms(),
        None
    );
    assert_eq!(InterruptionReason::DecodeFailed.reason_ttl_ms(), None);
}
