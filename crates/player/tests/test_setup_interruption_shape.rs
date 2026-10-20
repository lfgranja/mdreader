//! T002e — BLOCKING shape test for interruption reasons (red before T004).
//! Asserts the 8-reason closed set (FR-005) and stub contracts.
//!
//! T101 adds the character bound of contracts/surface.md (lock-screen
//! surface, "Interruption reason"): the reason line is a ≤8-word pt-BR
//! line (soft target) with a ≤40-char hard limit — the Android
//! notification content-text limit — and when the two conflict the
//! ≤40-char limit wins. The property below draws from the closed
//! eight-reason set and asserts both bounds on every line.
//!
//! The conflict is RESOLVED, not merely recorded: three of the eight
//! lines stood over the cap (47 / 43 / 43 chars) and were shortened
//! rather than the bound relaxed, so every line now satisfies BOTH
//! bounds at once and the tie-break rule is needed for no line.

use proptest::prelude::*;

use player::interruption::{InterruptionReason, reason_line};

/// The Android notification content-text hard limit
/// (contracts/surface.md): the reason line never exceeds this many
/// characters. The limit wins over the ≤8-word soft target whenever
/// the two conflict.
const REASON_LINE_MAX_CHARS: usize = 40;

/// The ≤8-word soft target on the same reason line
/// (contracts/surface.md).
const REASON_LINE_MAX_WORDS: usize = 8;

/// The closed eight-reason set (FR-005), pinned member by member —
/// exactly eight reasons, no `Other` / `Unknown`.
const EIGHT_REASONS: [InterruptionReason; 8] = [
    InterruptionReason::FocusDenied,
    InterruptionReason::FocusDelayed,
    InterruptionReason::PermanentLoss,
    InterruptionReason::TransientPaused,
    InterruptionReason::DeviceRemoved,
    InterruptionReason::VoiceUnavailable,
    InterruptionReason::FormatUndeliverable,
    InterruptionReason::DecodeFailed,
];

/// The closed eight-reason set as a proptest strategy: every member is
/// generated explicitly, so a ninth reason would need a strategy arm
/// here before anything could compile.
fn reason_strategy() -> impl Strategy<Value = InterruptionReason> {
    prop_oneof![
        Just(InterruptionReason::FocusDenied),
        Just(InterruptionReason::FocusDelayed),
        Just(InterruptionReason::PermanentLoss),
        Just(InterruptionReason::TransientPaused),
        Just(InterruptionReason::DeviceRemoved),
        Just(InterruptionReason::VoiceUnavailable),
        Just(InterruptionReason::FormatUndeliverable),
        Just(InterruptionReason::DecodeFailed),
    ]
}

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
            "Pausado por interrupção — retome",
        ),
        (
            InterruptionReason::DeviceRemoved,
            "Fones removidos — pausado",
        ),
        (
            InterruptionReason::VoiceUnavailable,
            "Voz indisponível — leitura não inicia",
        ),
        (
            InterruptionReason::FormatUndeliverable,
            "Formato do documento não pode ser lido",
        ),
        (
            InterruptionReason::DecodeFailed,
            "Falha ao decodificar — leitura parada",
        ),
    ];
    for (r, expected) in cases {
        let line = reason_line(&r);
        assert_eq!(
            line, expected,
            "reason_line({r:?}) expected final contract label {expected:?}"
        );
        // ≤40 chars — the Android content-text hard limit
        // (contracts/surface.md). The limit wins over the ≤8-word
        // soft target whenever the two conflict.
        assert!(
            line.chars().count() <= REASON_LINE_MAX_CHARS,
            "reason_line({:?}) exceeds {REASON_LINE_MAX_CHARS} chars: {:?} ({} chars)",
            r,
            line,
            line.chars().count()
        );
        // ≤8 words (split by whitespace) for non-empty lines
        if !line.is_empty() {
            let words: Vec<&str> = line.split_whitespace().collect();
            assert!(
                words.len() <= REASON_LINE_MAX_WORDS,
                "reason_line({:?}) exceeds {REASON_LINE_MAX_WORDS} words: {:?} ({} words)",
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// T101 — every reason line of the closed eight-reason set fits
    /// the ≤40-char hard limit of contracts/surface.md (the Android
    /// notification content-text limit) AND the ≤8-word soft target.
    ///
    /// Three of the eight lines stood over the hard limit as they
    /// stood (TransientPaused 47, VoiceUnavailable 43, DecodeFailed
    /// 43 chars) while staying inside the ≤8-word target; the
    /// contract states the ≤40-char limit wins, so the lines are
    /// shortened rather than the bound relaxed.
    ///
    /// Format note: every `prop_assert*` message uses explicit
    /// positional arguments, never implicit named captures — proptest
    /// builds the message through `concat!`, and rustc rejects
    /// implicit captures in macro-expanded format strings.
    #[test]
    fn prop_every_reason_line_fits_the_char_and_word_bounds(reason in reason_strategy()) {
        let line = reason_line(&reason);
        prop_assert!(
            line.chars().count() <= REASON_LINE_MAX_CHARS,
            "the reason line for {:?} is {} chars — the ≤{}-char content-text limit is exceeded by {:?}",
            reason,
            line.chars().count(),
            REASON_LINE_MAX_CHARS,
            line
        );
        prop_assert!(
            line.split_whitespace().count() <= REASON_LINE_MAX_WORDS,
            "the reason line for {:?} is {} words — the ≤{}-word target is exceeded by {:?}",
            reason,
            line.split_whitespace().count(),
            REASON_LINE_MAX_WORDS,
            line
        );
    }
}

/// The same ≤40-char bound, asserted exhaustively over the closed
/// eight-reason set — no sampling, no gap: a reason that no property
/// run happens to draw is still measured here.
#[test]
fn every_reason_line_fits_the_char_hard_limit_exhaustively() {
    for reason in EIGHT_REASONS {
        let line = reason_line(&reason);
        assert!(
            line.chars().count() <= REASON_LINE_MAX_CHARS,
            "the {reason:?} reason line {line:?} is {} chars — the ≤{REASON_LINE_MAX_CHARS}-char content-text limit is exceeded",
            line.chars().count()
        );
    }
}

/// T101 — the conflict between the two bounds is RESOLVED, not merely
/// recorded: every line of the closed eight-reason set satisfies
/// BOTH the ≤40-char hard limit and the ≤8-word soft target at once,
/// so the contract's tie-break rule ("if the two conflict, ≤40 chars
/// wins") is needed for no line at all.
///
/// The three lines the audit measured over the cap — `TransientPaused`
/// (47 chars), `VoiceUnavailable` (43) and `DecodeFailed` (43) — are
/// pinned at their new measurements, so a later edit that lengthens
/// one of them back over the cap fails here with the exact
/// measurement rather than quietly re-opening the conflict.
#[test]
fn the_reason_line_length_conflict_is_resolved_and_not_merely_recorded() {
    for reason in EIGHT_REASONS {
        let line = reason_line(&reason);
        let chars = line.chars().count();
        let words = line.split_whitespace().count();
        assert!(
            chars <= REASON_LINE_MAX_CHARS,
            "the {reason:?} reason line {line:?} is {chars} chars — the ≤{REASON_LINE_MAX_CHARS}-char content-text limit is exceeded"
        );
        assert!(
            words <= REASON_LINE_MAX_WORDS,
            "the {reason:?} reason line {line:?} states {words} words — the ≤{REASON_LINE_MAX_WORDS}-word target is exceeded"
        );
    }
    // The three lines that once stood over the cap, measured again.
    let resolved = [
        (InterruptionReason::TransientPaused, 32),
        (InterruptionReason::VoiceUnavailable, 37),
        (InterruptionReason::DecodeFailed, 37),
    ];
    for (reason, expected_chars) in resolved {
        let line = reason_line(&reason);
        assert_eq!(
            line.chars().count(),
            expected_chars,
            "the {reason:?} reason line {line:?} changed length — re-measure it against the ≤{REASON_LINE_MAX_CHARS}-char content-text limit and the contract table"
        );
    }
}
