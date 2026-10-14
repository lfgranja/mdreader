package app.mdreader

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build

/**
 * Resident lifecycle + teardown mirror (FR-009, FR-009a, FR-032f;
 * SC-043, SC-049).
 *
 * Wires the resident-service lifecycle: the resident notification host
 * while a resumable session exists, the resident-stop control, the
 * document-end teardown, and the swipe-away forwarding. Every control and
 * teardown event is forwarded to Rust as a classified shell-ipc intent
 * string (`resident_stop`, `swipe_away`, `document_end`) through
 * [forwardIntent]; an unrecognised platform lifecycle value is forwarded
 * as the diagnostic `lifecycle_unmapped`, never absorbed.
 *
 * The exactly-one invariant (FR-009a; SC-043, SC-049) is structural: a
 * single resident host reference — the one persistent resident
 * notification — held while a resumable session exists and nulled at
 * teardown, so exactly one resident component and one ongoing
 * notification exist while a session is resumable and zero remain after
 * the document ends. The 10/10 device-trial counts are owned by T053 and
 * T083, not by this class.
 *
 * Applies nothing locally and decides nothing. The resident-stop press
 * preserves the stored bookmark while the transport Stop discards it
 * (FR-009a against FR-032), a swipe-away while playing pauses first under
 * the FR-028e fallback, and the document end tears residency down
 * (FR-032f): every one of those outcomes is classified and decided by the
 * Rust session machine (T058–T060), and this class only carries the
 * platform lifecycle and the controls (FR-042, SC-035). No position, rate
 * or transport state is held or computed here.
 *
 * @param context service context used to post and dismiss the resident
 * notification and to register the lifecycle receiver.
 * @param forwardIntent sink receiving one wire intent string per platform
 * lifecycle event; the caller relays it to Rust uninterpreted.
 */
class Residency(
    private val context: Context,
    private val forwardIntent: (String) -> Unit
) {

    /** Wire intent strings of the closed residency-intent set
     * (contracts/shell-ipc.md). */
    companion object {
        /** Wire intent for the resident-stop control pressed (FR-009a):
         * the reader ended residency from the resident notification. Rust
         * preserves the stored bookmark — the session stays resumable —
         * and ends residency, distinct from the transport Stop that
         * discards it (FR-032). */
        const val INTENT_RESIDENT_STOP = "resident_stop"

        /** Wire intent for the resident surface swiped away (FR-009a,
         * FR-028e): the reader dismissed a notification surface. Rust
         * pauses first under the FR-028e fallback when audio was still
         * playing, preserves the bookmark, then ends residency. */
        const val INTENT_SWIPE_AWAY = "swipe_away"

        /** Wire intent for the document end (FR-032f): the teardown event
         * the session machine records as the session-end reason and
         * through which any open resume offer is closed, a document that
         * has reached its end being mutually exclusive with an offer
         * awaiting a reader choice (FR-032c, FR-032e). */
        const val INTENT_DOCUMENT_END = "document_end"

        /** Diagnostic intent for an unrecognised platform lifecycle
         * value; never absorbed. */
        const val INTENT_LIFECYCLE_UNMAPPED = "lifecycle_unmapped"

        private const val CHANNEL_ID = "mdreader_resident"
        private const val CHANNEL_NAME = "Resident"

        /** Notification id of the single resident surface. The resident
         * component is the playback service itself (FR-009a), so this is
         * the id the host service uses for its foreground notification:
         * one id is one surface, and posting here replaces the playing
         * surface with the resident one instead of adding a second
         * notification (SC-043, SC-049). */
        private const val NOTIFICATION_ID = 1

        private const val ACTION_RESIDENT_STOP = "app.mdreader.RESIDENT_STOP"
        private const val ACTION_SWIPE_AWAY = "app.mdreader.SWIPE_AWAY"

        private const val REQUEST_RESIDENT_STOP = 0
        private const val REQUEST_SWIPE_AWAY = 1
        private const val REQUEST_LAUNCH = 2

        private const val RESUME_LABEL = "Resume"
        private const val RESIDENT_STOP_LABEL = "Stop"
        private const val RESIDENT_CONTENT_TEXT = "Paused"
    }

    /**
     * The platform lifecycle receiver: the resident-stop control and the
     * swipe-away both arrive here as broadcasts, and each is forwarded to
     * Rust through [forwardIntent] the moment the platform delivers it.
     * Any other platform lifecycle value is forwarded as the diagnostic
     * [INTENT_LIFECYCLE_UNMAPPED], never absorbed.
     */
    private val lifecycleReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            forwardIntent(
                when (intent.action) {
                    ACTION_RESIDENT_STOP -> INTENT_RESIDENT_STOP
                    ACTION_SWIPE_AWAY -> INTENT_SWIPE_AWAY
                    else -> INTENT_LIFECYCLE_UNMAPPED
                }
            )
        }
    }

    /**
     * The single resident host — the one persistent resident notification
     * while a resumable session exists (FR-009a). The structural form of
     * the exactly-one invariant (SC-043, SC-049): one reference, nulled
     * at teardown, so no second resident surface can ever exist.
     */
    private var residentHost: Notification? = null

    /** Whether the lifecycle receiver is currently registered. */
    private var registered = false

    /**
     * Establishes the resident host while a resumable session exists:
     * registers the lifecycle receiver and posts the one persistent
     * resident notification — the paused state and the resume control,
     * asserting no elapsed time, remaining time, section title or
     * progress (FR-009a) — and holds it as the single resident host. The
     * notification carries the resident-stop control (forwarded as
     * [INTENT_RESIDENT_STOP]) and the swipe-away delete intent (forwarded
     * as [INTENT_SWIPE_AWAY]); the resident surface is dismissable rather
     * than ongoing-flagged, so the reader's ways out of residency are the
     * resident-stop control, a swipe-away, or document end (FR-009a,
     * FR-028e).
     *
     * @param title document title for the resident surface, resolved from
     * the Rust state mirror (FR-029) and supplied by the host because
     * this class holds no document state.
     * @param resumeIntent PendingIntent for the resume control, supplied
     * by the host from the transport surface: the press is delivered to
     * Rust as the resume command, so a Play press from this notification
     * still produces audio without the application being opened (SC-043).
     * The control is wired here, never honoured.
     *
     * Safe to call again: a second call while resident is a no-op, so a
     * second resident surface can never exist (FR-009a).
     */
    fun enter(title: String, resumeIntent: PendingIntent) {
        if (residentHost != null) {
            return
        }
        val manager = context.getSystemService(NotificationManager::class.java) ?: return
        ensureChannel(manager)
        registerLifecycleReceiver()
        val resident = buildResidentNotification(title, resumeIntent)
        manager.notify(NOTIFICATION_ID, resident)
        residentHost = resident
    }

    /**
     * Records the document end and tears residency down (FR-032f):
     * forwards [INTENT_DOCUMENT_END] to Rust — the classified teardown
     * event, through which the session machine records the session-end
     * reason and closes any open resume offer (FR-032c, FR-032e) — then
     * releases the resident host. The stored position is cleared
     * Rust-side (FR-032b); this class holds none to clear. Teardown at
     * document end is the spec-mandated consequence rather than a
     * decision: a completed document has no resumable position, so
     * nothing is left resident and no resume is offered anywhere
     * (FR-032e, FR-032f, SC-045, SC-049).
     */
    fun documentEnd() {
        forwardIntent(INTENT_DOCUMENT_END)
        // Keep-current-only trim on the document-end path: every
        // other snapshot in the directory is deleted
        // (contracts/ingest.md).
        DocIngest.trimToCurrent(context)
        teardown()
    }

    /**
     * Releases the resident host: dismisses the one persistent resident
     * notification, unregisters the lifecycle receiver and nulls the
     * single host reference, so zero resident surfaces remain (FR-032f,
     * SC-049). The host invokes this when Rust commands the end of
     * residency — the classified outcome of an [INTENT_RESIDENT_STOP] or
     * [INTENT_SWIPE_AWAY] event, or a transport Stop that discarded the
     * bookmark (FR-009a, FR-032) — and [documentEnd] invokes it for the
     * FR-032f path. Stopping the service itself is the host's act; this
     * class only releases the surface. Applies nothing locally: no pause,
     * no resume, no transport change (FR-042, SC-035). Safe to call
     * again: a second call releases nothing.
     */
    fun teardown() {
        if (residentHost == null) {
            return
        }
        val manager = context.getSystemService(NotificationManager::class.java)
        if (manager != null) {
            manager.cancel(NOTIFICATION_ID)
        }
        unregisterLifecycleReceiver()
        residentHost = null
    }

    /**
     * Establishes the swipe-away forwarding and returns its PendingIntent
     * for a notification delete intent: the lifecycle receiver is
     * registered — the delivery path for the broadcast — and the
     * PendingIntent that classifies the dismissal as [INTENT_SWIPE_AWAY]
     * is returned. The resident surface carries the intent (see
     * [enter]); the host attaches the same intent to the playing surface
     * — ongoing-flagged and non-dismissable while audio plays (FR-028e) —
     * so that where a platform or OEM build permits dismissal regardless
     * of the ongoing flag, the gesture reaches Rust and pauses first
     * rather than leaving audio without a control surface (FR-028e). The
     * stored position is preserved either way, a Rust-side decision.
     *
     * Safe to call again: a second call registers nothing new and returns
     * the same PendingIntent token.
     */
    fun swipeAwayIntent(): PendingIntent {
        registerLifecycleReceiver()
        return lifecyclePendingIntent(ACTION_SWIPE_AWAY, REQUEST_SWIPE_AWAY)
    }

    /**
     * Creates the resident notification channel if the platform requires
     * one and it does not exist yet (API 26+; below the API-29 floor of
     * SC-032 the platform requires none).
     */
    private fun ensureChannel(manager: NotificationManager) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            if (manager.getNotificationChannel(CHANNEL_ID) == null) {
                manager.createNotificationChannel(
                    NotificationChannel(
                        CHANNEL_ID, CHANNEL_NAME,
                        NotificationManager.IMPORTANCE_LOW
                    )
                )
            }
        }
    }

    /** Registers the lifecycle receiver for the resident-stop and
     * swipe-away broadcasts. Safe to call again: a second call registers
     * nothing. */
    private fun registerLifecycleReceiver() {
        if (registered) {
            return
        }
        val filter = IntentFilter(ACTION_RESIDENT_STOP)
        filter.addAction(ACTION_SWIPE_AWAY)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            context.registerReceiver(
                lifecycleReceiver, filter, Context.RECEIVER_NOT_EXPORTED
            )
        } else {
            context.registerReceiver(lifecycleReceiver, filter)
        }
        registered = true
    }

    /** Unregisters the lifecycle receiver. Safe to call again: a second
     * call unregisters nothing. */
    private fun unregisterLifecycleReceiver() {
        if (!registered) {
            return
        }
        context.unregisterReceiver(lifecycleReceiver)
        registered = false
    }

    /**
     * Builds the resident notification: the paused state and the resume
     * control, with no elapsed time, remaining time, section title or
     * progress (FR-009a), the resident-stop control, and the swipe-away
     * delete intent. The surface is dismissable — not ongoing-flagged —
     * because pausing is the route to dismissal while a session is
     * resumable (FR-028e), and it is declared public so the paused state
     * and the document title reach the lock screen (FR-028b).
     */
    private fun buildResidentNotification(
        title: String,
        resumeIntent: PendingIntent
    ): Notification {
        val swipeIntent = swipeAwayIntent()
        val stopIntent = lifecyclePendingIntent(
            ACTION_RESIDENT_STOP, REQUEST_RESIDENT_STOP
        )
        val resume = Notification.Action.Builder(null, RESUME_LABEL, resumeIntent).build()
        val stop = Notification.Action.Builder(null, RESIDENT_STOP_LABEL, stopIntent).build()
        val launch = launchPendingIntent()
        fun configure(builder: Notification.Builder): Notification.Builder =
            builder
                .setSmallIcon(android.R.drawable.ic_media_pause)
                .setContentTitle(title)
                .setContentText(RESIDENT_CONTENT_TEXT)
                .setContentIntent(launch)
                .setVisibility(Notification.VISIBILITY_PUBLIC)
                .setDeleteIntent(swipeIntent)
                .addAction(resume)
                .addAction(stop)
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            configure(Notification.Builder(context, CHANNEL_ID)).build()
        } else {
            @Suppress("DEPRECATION")
            configure(Notification.Builder(context)).build()
        }
    }

    /**
     * Builds the PendingIntent for one lifecycle broadcast: an explicit
     * in-app broadcast whose action classifies the event at the boundary
     * (contracts/shell-ipc.md case mapping).
     */
    private fun lifecyclePendingIntent(action: String, requestCode: Int): PendingIntent {
        val intent = Intent(action).setPackage(context.packageName)
        return PendingIntent.getBroadcast(
            context, requestCode, intent, PendingIntent.FLAG_IMMUTABLE
        )
    }

    /**
     * Builds the tap target of the resident surface: the application
     * launch intent, so a tap opens the application — the reader's route
     * to choosing another document, since a completed or stopped document
     * offers no resume (FR-032f).
     */
    private fun launchPendingIntent(): PendingIntent? =
        context.packageManager
            .getLaunchIntentForPackage(context.packageName)
            ?.let {
                PendingIntent.getActivity(
                    context, REQUEST_LAUNCH, it,
                    PendingIntent.FLAG_IMMUTABLE
                )
            }
}
