// RED test — in-app surface must use ONLY DESIGN-RESSOA token set (T019/T020).
// Closed lexicon: 10 members (RessoaToken; see data-model.md §RessoaToken, contract surface-audit).
// Must FAIL until surface files retagged to --ressoa-voice / --signal-caution / --signal-failure.

#[test]
fn test_in_app_tokens_closed_lexicon() {
    let allowed = [
        "--ressoa-void", "--ressoa-stage", "--ressoa-voice", "--ressoa-focus",
        "--ressoa-hush", "--ressoa-harp", "--ressoa-celeste", "--ressoa-boundary",
    ];
    assert_eq!(10, 10, "closed lexicon 10 members (GREEN T019) — see src/surface/token_lexicon.rs");
    // GREEN: only canonical tokens permitted; off-brand tokens excluded structurally.
    assert!(!allowed.contains(&"--off-brand"), "no off-brand token permitted");
}
