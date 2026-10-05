package app.mdreader

import android.app.Service
import android.content.Intent
import android.os.IBinder

/** Mirror-only Kotlin Service (FR-042 / SC-035 / Tauri Contract v1.2.0).
 * Service class structurally forced Kotlin (ActivityManager reflection);
 * playback authority remains Rust (FR-002 / FR-003). Zero transport/rate/position logic.
 */
class PlaybackService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null
    // Self-bootstraps from Application/Activity (FR-009a / FR-010)
    // Forward lifecycle events to Rust via shell-IPC (contracts/shell-ipc.md) — mirror, never computes.
}
