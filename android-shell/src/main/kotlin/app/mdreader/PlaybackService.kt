package app.mdreader

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
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
 * Notification rendering stays mirror-only: the media surface is
 * published by [MediaMirror.publish] from Rust-published state, and
 * the resident surface by [Residency]. The foreground holder below
 * carries the application label alone — no body, no clock, no
 * section — and shares the resident surface id, so the first resident
 * post replaces it instead of adding a second surface. Content beyond
 * the label must never be added here (FR-028 series).
 */
class PlaybackService : Service() {

    companion object {
        private const val TAG = "PlaybackService"
        private const val CHANNEL_ID = "mdreader_playback"

        /** Service intent action for the Play command. */
        const val ACTION_PLAY = "app.mdreader.PLAY"

        /** Service intent action for the Pause command. */
        const val ACTION_PAUSE = "app.mdreader.PAUSE"

        /** Service intent action for the Stop command. */
        const val ACTION_STOP = "app.mdreader.STOP"

        /** Intent extra carrying the document source for [ACTION_PLAY]. */
        const val EXTRA_DOCUMENT = "document"

        /** Intent extra carrying the clock reading for [ACTION_PLAY]. */
        const val EXTRA_POSITION = "position"

        /**
         * Foreground holder id — the same id as the resident surface
         * ([Residency]), so the resident post replaces this holder
         * instead of adding a second surface.
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
        ensureChannel()
        ShellHost.init(this)
    }

    override fun onDestroy() {
        ShellHost.release()
        super.onDestroy()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
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
                startForegroundHolder()
            }
            ACTION_PAUSE -> {
                Log.d(TAG, "Pause")
                ShellHost.forwardVerb(MediaMirror.VERB_PAUSE)
            }
            ACTION_STOP -> {
                Log.d(TAG, "Stop")
                ShellHost.forwardVerb(MediaMirror.VERB_STOP)
                ShellHost.abandonFocus()
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
     * Holds the foreground mechanics with the application label alone
     * until the first mirror publish or resident post replaces this
     * holder (same id — one surface, never two). Carries no body, no
     * clock, no section: rendering stays mirror-only.
     */
    private fun startForegroundHolder() {
        val holder = foregroundPlaceholder()
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

    private fun ensureChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val manager = getSystemService(NotificationManager::class.java) ?: return
            if (manager.getNotificationChannel(CHANNEL_ID) == null) {
                val channel = NotificationChannel(
                    CHANNEL_ID,
                    "Playback",
                    NotificationManager.IMPORTANCE_LOW
                )
                manager.createNotificationChannel(channel)
            }
        }
    }

    /**
     * Builds the foreground holder: the application label as the title
     * and nothing else. The tap target opens the application — the
     * reader's route to choosing another document.
     */
    private fun foregroundPlaceholder(): Notification {
        val label = applicationInfo.loadLabel(packageManager).toString()
        val launch = packageManager.getLaunchIntentForPackage(packageName)?.let {
            PendingIntent.getActivity(
                this, 0, it,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )
        }
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
                .setSmallIcon(android.R.drawable.ic_media_play)
                .setContentTitle(label)
                .setContentIntent(launch)
                .setOngoing(true)
                .setVisibility(Notification.VISIBILITY_PUBLIC)
                .build()
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
                .setSmallIcon(android.R.drawable.ic_media_play)
                .setContentTitle(label)
                .setContentIntent(launch)
                .setOngoing(true)
                .setVisibility(Notification.VISIBILITY_PUBLIC)
                .build()
        }
    }
}
