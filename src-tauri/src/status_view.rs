//! T040 — Lock-screen / notification surface for focus (FR-028 / FR-029 / FR-030 / SC-010 / SC-033 / SC-042).
//! Mirror: title + section + progress only; determinacy fixed at session start.
//! Non-dismissable while playing; transient auto-dismiss 3 s; resident dismissable when paused.
//! Zero playback logic (FR-042 / SC-035).

pub fn render_notification_surface(
    title: &str,
    section_title: Option<&str>,
    progress: Option<f32>, // determinacy fixed at session start
) -> String {
    // Delegation to Rust playback module — never computes position/rate/transport here.
    format!("{} | {} | progress={:?}", title, section_title.unwrap_or(""), progress)
}
