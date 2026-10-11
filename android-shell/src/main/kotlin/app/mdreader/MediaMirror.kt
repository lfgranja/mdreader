package app.mdreader

/** MediaSession mirror only — never a second source of truth (FR-003 / FR-042).
 * Reflects Rust playback module state; computes nothing. Notification content:
 * title + most-recent-passed section title + progress (FR-028 / FR-029).
 */
class MediaMirror {
    // Delegation targets only — shapes/getters, never computation; see T081 for the denied shape list in check-shell-purity.sh
}
