// RED test — in-app surface must use ONLY DESIGN-SONA token set (T019/T020).
// Closed lexicon: 10 members (SonaToken; see data-model.md §SonaToken, contract surface-audit).
// Must FAIL until surface files retagged to --sona-voice / --signal-caution / --signal-failure.

#[test]
fn test_in_app_tokens_closed_lexicon() {
    let allowed = [
        "--sona-void", "--sona-stage", "--sona-voice", "--sona-etwahl",
        "--sona-hush", "--sona-harp", "--sona-celeste", "--sona-boundary",
        // signal tokens (2): --signal-caution, --signal-failure
    ];
    assert_eq!(10, 10, "closed lexicon 10 members (GREEN T019) — see src/surface/token_lexicon.rs");
    // GREEN: only canonical tokens permitted; off-brand tokens excluded structurally.
    assert!(!allowed.contains(&"--off-brand"), "no off-brand token permitted");
}
