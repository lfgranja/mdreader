package app.mdreader

import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.util.Log

/**
 * PlaybackService — thin Kotlin shell (Tauri Platform Contract).
 *
 * Service class structurally forced Kotlin (ActivityManager
 * reflection): the platform creates it from the Application/Activity
 * entry points declared in the manifest, which is the self-bootstrap
 * the Rust `register_android_plugin` body documents. Playback
 * authority remains Rust. Zero command, tempo, or clock logic here:
 * every control action arriving as a service intent is relayed to Rust
 * through [ShellHost.forwardVerb] as its closed verb wire the moment
 * the platform delivers it, never honoured locally.
 *
 * The document and clock extras of [ACTION_PLAY] are observed only
 * (lengths for the debug log, never content): the integer-code channel
 * is fieldless by design, so the document itself reaches Rust over the
 * IPC `run_mobile_plugin` call carrying it — the service never
 * interprets, splits, or synthesizes anything.
 *
 * Foreground residency (T022): the service posts its holder —
 * the application label alone, no body, no clock, no section — on
 * every command, on the same channel and id as the media mirror's
 * surface, so the first [MediaMirror.publish] replaces the holder
 * instead of adding a second surface. Posting synchronously is what
 * keeps the residency hold in screen-off playback: a service started
 * with `startForegroundService` must call `startForeground` within
 * the platform window, and the first mirror publish may arrive after
 * it. The holder carries no render of its own — rendering stays
 * mirror-only (FR-028 series).
 *
 * The media surface is published by [MediaMirror.publish] from
 * Rust-published state and the resident surface by [Residency]; this
 * class holds neither. Dismissal of the playing surface is refused
 * by the mirror's ongoing flag (FR-028e), and the lock-screen
 * control set the media session exposes is play, pause, skip
 * forward/back — the Stop control exists in-app only
 * (contracts/surface.md), which is why [ACTION_STOP] rides this
 * service intent and never the platform's media controls.
 */
class PlaybackService : Service() {

    companion object {
        private const val TAG = "PlaybackService"

        /** Service intent action for the Play command. */
        const val ACTION_PLAY = "app.mdreader.PLAY"

        /** Service intent action for the Pause command. */
        const val ACTION_PAUSE = "app.mdreader.PAUSE"

        /** Service intent action for the Stop command — the in-app
         * route of the Stop control (contracts/surface.md); the
         * lock-screen media controls never carry Stop. */
        const val ACTION_STOP = "app.mdreader.STOP"

        /** Intent extra carrying the document source for [ACTION_PLAY]. */
        const val EXTRA_DOCUMENT = "document"

        /** Intent extra carrying the clock reading for [ACTION_PLAY]. */
        const val EXTRA_POSITION = "position"

        /**
         * Foreground holder id — the same id as the media surface
         * ([MediaMirror]) and the resident surface ([Residency]), so
         * each post replaces the previous one instead of adding a
         * second notification: one id is one surface.
         */
        private const val FOREGROUND_ID = 1

        /**
         * Starts the playback service for one document. The document
         * and clock travel as intent extras for observation; the Play
         * verb itself is relayed fieldless to Rust on delivery.
         */
        fun startPlayback(context: Context, document: String, positionSecs: Double = 0.0) {
            val intent = Intent(context, PlaybackService::class.java).apply {
                action = ACTION_PLAY
                putExtra(EXTRA_DOCUMENT, document)
                putExtra(EXTRA_POSITION, positionSecs)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        ShellHost.init(this)
        // Wire the foreground service to MediaMirror's MediaStyle notification
        // so the MediaStyle notification IS the foreground service notification
        // (same ID), enabling lock-screen controls and screen-off continuity.
        ShellHost.setMediaMirrorForegroundStarter { notification ->
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                startForeground(
                    FOREGROUND_ID, notification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK
                )
            } else {
                startForeground(FOREGROUND_ID, notification)
            }
        }
    }

    override fun onDestroy() {
        ShellHost.release()
        super.onDestroy()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // The holder posts before anything else: the platform's
        // foreground-service window runs from this callback, and the
        // first mirror publish may arrive after it. Same id and
        // channel as the media surface, so the publish replaces this
        // holder — one surface, never two.
        startForegroundHolder()

        val command = intent ?: run {
            Log.w(TAG, "Command with no intent; nothing relayed")
            return START_STICKY
        }
        val action = command.action
        if (action == null) {
            Log.w(TAG, "Command with no action; nothing relayed")
            return START_STICKY
        }
        when (action) {
            ACTION_PLAY -> {
                val document = command.getStringExtra(EXTRA_DOCUMENT) ?: ""
                val position = command.getDoubleExtra(EXTRA_POSITION, 0.0)
                Log.d(TAG, "Play: len=${document.length} pos=$position")
                ShellHost.requestFocus()
                ShellHost.forwardVerb(MediaMirror.VERB_PLAY)
                // Foreground service already holds the placeholder above;
                // MediaMirror.publish() replaces it with the MediaStyle
                // notification on the first Rust-published state.
            }
            ACTION_PAUSE -> {
                Log.d(TAG, "Pause")
                ShellHost.forwardVerb(MediaMirror.VERB_PAUSE)
            }
            ACTION_STOP -> {
                Log.d(TAG, "Stop")
                ShellHost.forwardVerb(MediaMirror.VERB_STOP)
                ShellHost.abandonFocus()
                // Keep-current-only trim on the Stop path: every other
                // snapshot in the directory is deleted (contracts/ingest.md).
                DocIngest.trimToCurrent(this)
                stopForegroundAndSelf()
            }
            else -> {
                Log.w(TAG, "Unknown service action; relaying diagnostic")
                ShellHost.forwardVerb(MediaMirror.VERB_CONTROL_UNMAPPED)
            }
        }
        return START_STICKY
    }

    /**
     * Posts the foreground holder: the application label alone — no
     * body, no clock, no section — built by the media mirror on its
     * own channel and the shared surface id, so the first resident
     * or media post replaces this holder instead of adding a second
     * surface (one id is one surface). Safe to call again: a second
     * call replaces the holder with itself.
     */
    private fun startForegroundHolder() {
        val holder = ShellHost.foregroundHolder()
        if (holder == null) {
            Log.w(TAG, "Mirror unbuilt; foreground holder unavailable")
            return
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                FOREGROUND_ID, holder,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK
            )
        } else {
            startForeground(FOREGROUND_ID, holder)
        }
    }

    /** Stops the foreground state and the service (the Stop path). */
    private fun stopForegroundAndSelf() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            stopForeground(STOP_FOREGROUND_REMOVE)
        } else {
            @Suppress("DEPRECATION")
            stopForeground(true)
        }
        stopSelf()
    }
}
