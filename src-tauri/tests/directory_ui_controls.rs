//! Structural contract for spec 023 — directory UI controls (RED first).
//!
//! The bridge already exposes `pick_directory`, `previous_chapter` and
//! `next_chapter`; these tests prove the canonical status view actually
//! reaches them and keeps every served bundle byte-identical.

const VIEW: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/status_view.html"));

#[test]
fn directory_picker_button_is_wired() {
    assert!(
        VIEW.contains("id=\"btnPickDirectory\""),
        "T229: directory picker button markup must exist in status_view.html"
    );
    assert!(
        VIEW.contains("aria-label=\"Selecionar diretório\""),
        "T229: directory picker button needs an accessible label"
    );
    assert!(
        VIEW.contains("btnPickDirectory.addEventListener('click'"),
        "T229: directory picker click handler must be wired"
    );
    assert!(
        VIEW.contains("btnPickDirectory.addEventListener('keydown'"),
        "T229: directory picker keyboard handler must be wired"
    );
    assert!(
        VIEW.contains("sendAction('pick_directory')"),
        "T229: the button must dispatch the existing pick_directory action"
    );
}

#[test]
fn directory_selected_listener_refreshes_chapter_state() {
    assert!(
        VIEW.contains("tauri://channel/directory-selected"),
        "T230: the view must listen for the directory-selected shell event"
    );
    assert!(
        VIEW.contains("directory-selected', (event) => {"),
        "T230: listener body must be present and consume the directory-selected payload"
    );
    assert!(
        VIEW.contains("payload.chapterFilename"),
        "T230: the listener must read the chapter filename from the payload"
    );
    assert!(
        VIEW.contains("refreshStatusView();"),
        "T230: the listener must refresh chapter title/count from the shell state"
    );
}

#[test]
fn chapter_nav_buttons_dispatch_existing_actions() {
    assert!(
        VIEW.contains("id=\"btnPrevChapter\""),
        "T234: previous chapter button markup must exist"
    );
    assert!(
        VIEW.contains("id=\"btnNextChapter\""),
        "T234: next chapter button markup must exist"
    );
    assert!(
        VIEW.contains("sendAction('previous_chapter')"),
        "T234: previous button must dispatch the existing bridge action"
    );
    assert!(
        VIEW.contains("sendAction('next_chapter')"),
        "T234: next button must dispatch the existing bridge action"
    );
    assert!(
        VIEW.contains("aria-label=\"Capítulo anterior\""),
        "T234: previous button needs an accessible label"
    );
    assert!(
        VIEW.contains("aria-label=\"Próximo capítulo\""),
        "T234: next button needs an accessible label"
    );
}

#[test]
fn chapter_nav_uses_hidden_attribute_not_display_none() {
    assert!(
        VIEW.contains("id=\"chapterNav\""),
        "T234: chapter navigation group must exist"
    );
    assert!(
        VIEW.contains("hidden"),
        "T234: chapter navigation must use the hidden attribute"
    );
    assert!(
        VIEW.contains("chapterNav.hidden"),
        "T234: visibility must be toggled from chapter metadata in JS"
    );
    assert!(
        !VIEW.contains("#chapterNav { display: none"),
        "T234: a display:none corpse would break the C-H4 surface audit"
    );
}

#[test]
fn all_served_bundles_are_byte_identical() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let canonical = std::fs::read(format!("{manifest}/src/status_view.html")).unwrap_or_default();
    for rel in [
        "src/assets/index.html",
        "gen/android/app/src/main/assets/index.html",
        "../dist/index.html",
        "dist/index.html",
    ] {
        let served = std::fs::read(format!("{manifest}/{rel}")).unwrap_or_default();
        assert_eq!(
            served, canonical,
            "FR-DUI-005: {rel} drifted from the canonical status view"
        );
    }
}
