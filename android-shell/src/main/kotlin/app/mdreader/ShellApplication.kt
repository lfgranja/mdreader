package app.mdreader

import android.app.Activity
import android.app.Application
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.util.Log

/**
 * ShellApplication — the process bootstrap of the Android shell
 * (FR-009a, FR-010, FR-021).
 *
 * The manifest declares this class as the application entry point, so
 * it runs once per process start, before any Activity or Service. It
 * does exactly three things, and decides nothing:
 *
 *  1. Builds the four platform mirrors through [ShellHost.init] — the
 *     same call the service host makes, kept idempotent there — so
 *     the media session exists before the first command.
 *  2. Starts the resident [PlaybackService], whose creation path
 *     posts the foreground holder (the application label alone). The
 *     platform may refuse a foreground start when the process was
 *     launched in the background (API 31+); the refusal is recorded
 *     as a diagnostic log and nothing else happens — the stated
 *     reason surfaces through the shell gates, never as a phantom
 *     session (FR-010).
 *  3. Files the notification consent request (FR-021): on API 33+
 *     the platform notification permission is a runtime consent; the
 *     first resumed Activity is asked once, and the answer is left to
 *     the platform — no local decision follows it.
 */
class ShellApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        ShellHost.init(this)
        startResidentShell()
        registerActivityLifecycleCallbacks(notificationConsent)
    }

    /**
     * Starts the resident shell service. A platform refusal (API 31+
     * background start restriction) is recorded as a diagnostic log:
     * no foreground holder exists thereafter, and the app-open path
     * re-attempts the start on the next process launch.
     */
    private fun startResidentShell() {
        try {
            val intent = Intent(this, PlaybackService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                startForegroundService(intent)
            } else {
                startService(intent)
            }
        } catch (error: Exception) {
            Log.w(TAG, "Resident shell start refused by the platform: ${error.message}")
        }
    }

    /**
     * Files the notification consent request once, from the first
     * resumed Activity (the platform requires an Activity to ask).
     * The answer is observed by the platform's own state; the shell
     * voices no opinion about it here (FR-021: a declared, revocable
     * consent).
     */
    private val notificationConsent = object : ActivityLifecycleCallbacks {

        private var filed = false

        override fun onActivityResumed(activity: Activity) {
            if (filed || Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
                return
            }
            filed = true
            val granted = activity.checkSelfPermission(POST_NOTIFICATIONS) ==
                PackageManager.PERMISSION_GRANTED
            if (!granted) {
                activity.requestPermissions(
                    arrayOf(POST_NOTIFICATIONS),
                    REQUEST_NOTIFICATION_CONSENT,
                )
            }
        }

        override fun onActivityCreated(activity: Activity, state: Bundle?) {}

        override fun onActivityStarted(activity: Activity) {}

        override fun onActivityPaused(activity: Activity) {}

        override fun onActivityStopped(activity: Activity) {}

        override fun onActivitySaveInstanceState(activity: Activity, state: Bundle) {}

        override fun onActivityDestroyed(activity: Activity) {}
    }

    private companion object {
        private const val TAG = "ShellApplication"

        /** Platform notification consent (API 33+). */
        private const val POST_NOTIFICATIONS = "android.permission.POST_NOTIFICATIONS"

        /** Request code of the one notification consent ask. */
        private const val REQUEST_NOTIFICATION_CONSENT = 701
    }
}
