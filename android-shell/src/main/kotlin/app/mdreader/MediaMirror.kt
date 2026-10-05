package app.mdreader

/** MediaSession mirror only — never a second source of truth (FR-003 / FR-042).
 * Reflects Rust playback module state; computes nothing. Notification content:
 * title + most-recent-passed section title + progress (FR-028 / FR-029).
 */
class MediaMirror {
    // Delegation targets only — shapes/getters, never computation.
    // No `.position =`, `.rate =`, `seekTo(`, `gain *=`, `skip(` per T081 forbidden-shape scan.
}
