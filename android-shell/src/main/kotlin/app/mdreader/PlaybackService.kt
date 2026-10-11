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

/** PlaybackService — thin Kotlin shell (Tauri Platform Contract).
 *
 *  Service class structurally forced Kotlin (ActivityManager reflection).
 *  Playback authority remains Rust. Zero transport/rate/position logic.
 */
class PlaybackService : Service() {

    companion object {
        private const val TAG = "PlaybackService"
        private const val CHANNEL_ID = "mdreader_playback"
        const val ACTION_PLAY = "app.mdreader.PLAY"
        const val ACTION_PAUSE = "app.mdreader.PAUSE"
        const val ACTION_STOP = "app.mdreader.STOP"
        const val EXTRA_DOCUMENT = "document"
        const val EXTRA_POSITION = "position"

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
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val action = intent?.action ?: return START_STICKY
        when (action) {
            ACTION_PLAY -> {
                val document = intent.getStringExtra(EXTRA_DOCUMENT) ?: ""
                val position = intent.getDoubleExtra(EXTRA_POSITION, 0.0)
                Log.d(TAG, "Play: len=${document.length} pos=$position")
                val notification = buildNotification("mdreader", "Tocando...")
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    startForeground(1, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK)
                } else {
                    startForeground(1, notification)
                }
                // TODO: forward to Rust via run_mobile_plugin when bridge lands.
                // Rust owns synthesis + PCM playback; Kotlin only mirrors state.
            }
            ACTION_PAUSE -> {
                Log.d(TAG, "Pause")
            }
            ACTION_STOP -> {
                Log.d(TAG, "Stop")
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                    stopForeground(STOP_FOREGROUND_REMOVE)
                } else {
                    @Suppress("DEPRECATION")
                    stopForeground(true)
                }
                stopSelf()
            }
        }
        return START_STICKY
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

    private fun buildNotification(title: String, text: String): Notification {
        val launch = packageManager.getLaunchIntentForPackage(packageName)?.let {
            PendingIntent.getActivity(
                this, 0, it,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )
        }
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
                .setSmallIcon(android.R.drawable.ic_media_play)
                .setContentTitle(title)
                .setContentText(text)
                .setContentIntent(launch)
                .setOngoing(true)
                .setVisibility(Notification.VISIBILITY_PUBLIC)
                .build()
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
                .setSmallIcon(android.R.drawable.ic_media_play)
                .setContentTitle(title)
                .setContentText(text)
                .setContentIntent(launch)
                .setOngoing(true)
                .setVisibility(Notification.VISIBILITY_PUBLIC)
                .build()
        }
    }
}
