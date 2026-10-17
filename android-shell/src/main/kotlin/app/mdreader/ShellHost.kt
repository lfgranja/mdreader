package app.mdreader

import android.app.Notification
import android.content.Context
import android.media.AudioManager
import android.util.Log

/**
 * Shell host — the single construction site of the four platform
 * mirrors (FR-002, FR-003; SC-035).
 *
 * Builds [MediaMirror], [Residency], [FocusMirror] and
 * [RoutingMirror] exactly once against the application context, each
 * with a sink that relays its wire string to Rust uninterpreted as an
 * integer channel code through [postShellVerb] / [postShellIntent].
 * The host applies nothing locally and decides nothing: every wire
 * string the mirrors classify travels to Rust, where the session and
 * its machines own every outcome (FR-042).
 *
 * The channel carries fieldless verbs and intents as integer codes
 * only (the numeric vocabulary of `shell_jni.rs code_to_action`):
 * object graphs never cross the boundary. Device wires arrive from
 * [RoutingMirror] as JSON shapes carrying an identifier; the host
 * relays the fieldless fallback code and withholds the identifier
 * (never logged, never forwarded) — the Rust side answers with the
 * same fallback it serves over IPC (FR-016 / FR-017a).
 *
 * Native binding: the two `external` members resolve against the
 * already-loaded application library — the host performs no
 * `System.loadLibrary` of its own, the symbol lookup spans the loaded
 * libraries of the process. While the library is absent (host-side
 * runs, pre-bootstrap calls) the link error is caught and recorded
 * as a diagnostic log; an event is never dropped silently and never
 * crashes the caller.
 *
 * Wires with no bridge counterpart (the focus-grant answers
 * `granted | deferred | refused` and the `focus_change_unmapped` /
 * `focus_grant_unmapped` diagnostics) are recorded as diagnostic logs
 * and dropped — the same wires the IPC classifier rejects, so the two
 * paths agree on what has no counterpart.
 */
object ShellHost {

    /** Channel codes and wire constants of this host — members of
     * the host object itself (an `object` has no companion). */
        private const val TAG = "ShellHost"

        /** Channel code of the Play verb. */
        const val CODE_PLAY = 1

        /** Channel code of the Pause verb. */
        const val CODE_PAUSE = 2

        /** Channel code of the Stop verb. */
        const val CODE_STOP = 3

        /** Channel code of the forward-jump verb. */
        const val CODE_SKIP_FORWARD_5S = 4

        /** Channel code of the backward-jump verb. */
        const val CODE_SKIP_BACK_5S = 5

        /** Channel code of the tempo-up verb. */
        const val CODE_RATE_UP = 6

        /** Channel code of the tempo-down verb. */
        const val CODE_RATE_DOWN = 7

        /** Channel code of the cycle-voice-next verb. */
        const val CODE_CYCLE_VOICE_NEXT = 8

        /** Channel code of the cycle-voice-prev verb. */
        const val CODE_CYCLE_VOICE_PREV = 9

        /** Channel code of the resume-at-stored verb. */
        const val CODE_RESUME_AT_STORED = 10

        /** Channel code of the start-over verb. */
        const val CODE_START_OVER = 11

        /** Channel code of the choose-engine verb. */
        const val CODE_CHOOSE_ENGINE = 12

        /** Channel code of the duckable-transient intent. */
        const val CODE_DUCKABLE_TRANSIENT = 13

        /** Channel code of the silence-demanding intent. */
        const val CODE_SILENCE_DEMANDING = 14

        /** Channel code of the permanent-loss intent. */
        const val CODE_PERMANENT_LOSS = 15

        /** Channel code of the focus-returned intent. */
        const val CODE_REGAINED = 16

        /** Channel code of the becoming-noisy intent. */
        const val CODE_BECOMING_NOISY = 17

        /** Channel code of the device-appeared intent. */
        const val CODE_DEVICE_APPEARED = 18

        /** Channel code of the device-disappeared intent. */
        const val CODE_DEVICE_DISAPPEARED = 19

        /** Channel code of the app-foregrounded intent. */
        const val CODE_APP_FOREGROUNDED = 20

        /** Channel code of the app-backgrounded intent. */
        const val CODE_APP_BACKGROUNDED = 21

        /** Channel code of the resident-stop intent. */
        const val CODE_RESIDENT_STOP = 22

        /** Channel code of the swipe-away intent. */
        const val CODE_SWIPE_AWAY = 23

        /** Channel code of the document-end intent. */
        const val CODE_DOCUMENT_END = 24

        /** Channel code of the control-unmapped diagnostic. */
        const val CODE_CONTROL_UNMAPPED = 25

        /** Channel code of the lifecycle-unmapped diagnostic. */
        const val CODE_LIFECYCLE_UNMAPPED = 26

        /** Channel code of the read-only inventory query. */
        const val CODE_DIAGNOSE = 27

        /** Opening of the device-appeared JSON wire: the tag as
         * the key, the identifier withheld (never logged, never
         * forwarded — the fieldless fallback rides instead). */
        private const val DEVICE_APPEARED_PREFIX = "{\"device_appeared\""

        /** Opening of the device-disappeared JSON wire. */
        private const val DEVICE_DISAPPEARED_PREFIX = "{\"device_disappeared\""

    /** The media-session and surface mirror; built at [init]. */
    private var media: MediaMirror? = null

    /** The resident-lifecycle mirror; built at [init]. */
    private var resident: Residency? = null

    /** The audio-focus mirror; built at [init]. */
    private var focus: FocusMirror? = null

    /** The routing mirror; built at [init]. */
    private var routing: RoutingMirror? = null

    /** Whether [init] has completed. */
    @Volatile
    private var initialised = false

    /**
     * Builds the four mirrors against the application context and
     * starts the routing observation. Safe to call again: a second
     * call builds nothing.
     *
     * The service host invokes this from its creation path — the
     * service itself is created by the platform from the
     * Application/Activity entry points declared in the manifest, so
     * no explicit construction call from Rust or the frontend precedes
     * it. Each mirror receives the host sink matching its kind
     * (verbs to [forwardVerb], intents to [forwardIntent]).
     *
     * @param context any context; the application context is held.
     * @return true when every mirror is built, false when the audio
     * service is unavailable (recorded as a diagnostic log; the
     * already-built mirrors still forward).
     */
    fun init(context: Context): Boolean {
        if (initialised) {
            return true
        }
        val appContext = context.applicationContext
        media = MediaMirror(appContext, ::forwardVerb)
        resident = Residency(appContext, ::forwardIntent)
        val audio = appContext.getSystemService(AudioManager::class.java)
        if (audio == null) {
            Log.w(TAG, "Audio service unavailable; focus and routing mirrors unwired")
            initialised = true
            return false
        }
        focus = FocusMirror(audio, ::forwardIntent)
        val routes = RoutingMirror(appContext, audio, ::forwardIntent)
        routing = routes
        routes.start()
        initialised = true
        return true
    }

    /**
     * Releases the routing observation. The service host invokes this
     * from its destruction path. Safe to call again: a second call
     * releases nothing.
     */
    fun release() {
        routing?.stop()
        routing = null
        focus = null
        resident = null
        media = null
        initialised = false
    }

    /**
     * Sets the foreground starter callback on the MediaMirror.
     * Called by PlaybackService after init() to wire the foreground
     * service to the MediaMirror's MediaStyle notification.
     */
    fun setMediaMirrorForegroundStarter(starter: (Notification) -> Unit) {
        media?.setForegroundStarter(starter)
    }

    /**
     * Builds the media mirror's foreground holder — the application
     * label alone, on the mirror's own channel and the shared
     * surface id — or null when the mirror is unbuilt. The service
     * host posts it synchronously on every command so the platform's
     * foreground-service window is always satisfied; the first
     * mirror publish replaces it.
     */
    fun foregroundHolder(): Notification? {
        return media?.foregroundHolder()
    }

    /**
     * Relays one closed verb wire string to Rust as its channel code.
     * Unknown values are recorded as diagnostic logs and dropped —
     * never absorbed, never crashing the caller.
     *
     * @param wire one `MediaMirror.VERB_*` wire string.
     */
    fun forwardVerb(wire: String) {
        val code = when (wire) {
            MediaMirror.VERB_PLAY -> CODE_PLAY
            MediaMirror.VERB_PAUSE -> CODE_PAUSE
            MediaMirror.VERB_STOP -> CODE_STOP
            MediaMirror.VERB_SKIP_FORWARD_5S -> CODE_SKIP_FORWARD_5S
            MediaMirror.VERB_SKIP_BACK_5S -> CODE_SKIP_BACK_5S
            MediaMirror.VERB_RATE_UP -> CODE_RATE_UP
            MediaMirror.VERB_RATE_DOWN -> CODE_RATE_DOWN
            MediaMirror.VERB_CYCLE_VOICE_NEXT -> CODE_CYCLE_VOICE_NEXT
            MediaMirror.VERB_CYCLE_VOICE_PREV -> CODE_CYCLE_VOICE_PREV
            MediaMirror.VERB_RESUME_AT_STORED -> CODE_RESUME_AT_STORED
            MediaMirror.VERB_START_OVER -> CODE_START_OVER
            MediaMirror.VERB_CHOOSE_ENGINE -> CODE_CHOOSE_ENGINE
            MediaMirror.VERB_CONTROL_UNMAPPED -> CODE_CONTROL_UNMAPPED
            else -> {
                Log.w(TAG, "Dropping verb with no channel code: $wire")
                return
            }
        }
        postVerb(code)
    }

    /**
     * Relays one classified intent wire string to Rust as its channel
     * code. Device wires arrive as JSON shapes; the host relays the
     * fieldless fallback code and withholds the identifier. Wires with
     * no bridge counterpart (the grant answers and the unmapped-focus
     * diagnostics) and unknown values are recorded as diagnostic logs
     * and dropped — never absorbed, never crashing the caller. The
     * identifier of a device wire is never logged.
     *
     * @param wire one `FocusMirror.INTENT_*`,
     * `RoutingMirror.INTENT_*` or `Residency.INTENT_*` wire string.
     */
    fun forwardIntent(wire: String) {
        val code = when {
            wire == FocusMirror.INTENT_DUCKABLE_TRANSIENT -> CODE_DUCKABLE_TRANSIENT
            wire == FocusMirror.INTENT_SILENCE_DEMANDING -> CODE_SILENCE_DEMANDING
            wire == FocusMirror.INTENT_PERMANENT_LOSS -> CODE_PERMANENT_LOSS
            wire == FocusMirror.INTENT_REGAINED -> CODE_REGAINED
            wire == FocusMirror.INTENT_APP_FOREGROUNDED -> CODE_APP_FOREGROUNDED
            wire == FocusMirror.INTENT_APP_BACKGROUNDED -> CODE_APP_BACKGROUNDED
            wire == RoutingMirror.INTENT_BECOMING_NOISY -> CODE_BECOMING_NOISY
            wire == Residency.INTENT_RESIDENT_STOP -> CODE_RESIDENT_STOP
            wire == Residency.INTENT_SWIPE_AWAY -> CODE_SWIPE_AWAY
            wire == Residency.INTENT_DOCUMENT_END -> CODE_DOCUMENT_END
            wire == Residency.INTENT_LIFECYCLE_UNMAPPED -> CODE_LIFECYCLE_UNMAPPED
            wire.startsWith(DEVICE_APPEARED_PREFIX) -> CODE_DEVICE_APPEARED
            wire == RoutingMirror.INTENT_DEVICE_APPEARED -> CODE_DEVICE_APPEARED
            wire.startsWith(DEVICE_DISAPPEARED_PREFIX) -> CODE_DEVICE_DISAPPEARED
            wire == RoutingMirror.INTENT_DEVICE_DISAPPEARED -> CODE_DEVICE_DISAPPEARED
            else -> {
                Log.w(TAG, "Dropping intent with no bridge counterpart: $wire")
                return
            }
        }
        postIntent(code)
    }

    /**
     * Files the audio-focus request: registration mechanics only. The
     * platform grant answer travels to Rust through [forwardIntent]
     * uninterpreted (a wire with no bridge counterpart is recorded
     * there as a diagnostic), and no local decision follows it. The
     * service host invokes this when a Play command arrives; Rust owns
     * what the answer means.
     */
    fun requestFocus() {
        val mirror = focus
        if (mirror == null) {
            Log.w(TAG, "Focus request unwired; Play forwarded without a filed request")
            return
        }
        mirror.start()
    }

    /**
     * Releases the filed focus request, if any; changes nothing else.
     * The service host invokes this when a Stop command arrives.
     * A pause keeps the filed request held.
     */
    fun abandonFocus() {
        focus?.stop()
    }

    /**
     * Posts one verb code to Rust. A missing native library is
     * recorded as a diagnostic log; the event is never dropped
     * silently and never crashes the caller.
     */
    private fun postVerb(code: Int) {
        try {
            postShellVerb(code)
        } catch (error: UnsatisfiedLinkError) {
            Log.w(TAG, "Verb channel unavailable (code=$code): ${error.message}")
        }
    }

    /**
     * Posts one intent code to Rust. A missing native library is
     * recorded as a diagnostic log; the event is never dropped
     * silently and never crashes the caller.
     */
    private fun postIntent(code: Int) {
        try {
            postShellIntent(code)
        } catch (error: UnsatisfiedLinkError) {
            Log.w(TAG, "Intent channel unavailable (code=$code): ${error.message}")
        }
    }

    /**
     * Native entry of the verb channel: one fieldless verb as its
     * integer code. Implemented in Rust (`shell_jni.rs`); resolves
     * against the already-loaded application library.
     */
    external fun postShellVerb(code: Int)

    /**
     * Native entry of the intent channel: one fieldless intent as its
     * integer code. Implemented in Rust (`shell_jni.rs`); resolves
     * against the already-loaded application library.
     */
    external fun postShellIntent(code: Int)

    /**
     * Publishes a mirror state to the MediaMirror from Rust via JNI.
     * Called from Rust's publish path to update the MediaSession and
     * media notification on the lock screen.
     *
     * Routing only: the wire JSON is the media mirror's own form
     * (contracts/shell-ipc.md) and is parsed by
     * [MediaMirror.publishJson] — the mirror owns its wire form and
     * this host applies nothing locally. The swipe-away token rides
     * the resident lifecycle (FR-028e): the host attaches the same
     * intent the resident surface carries to the playing surface, so
     * where a platform or OEM build permits dismissal regardless of
     * the ongoing flag, the gesture reaches Rust and pauses first.
     * The `deleteIntentJson` parameter is reserved for a
     * Rust-supplied token; the bridge is live, but until Rust
     * supplies one the host ignores it and attaches its own
     * swipe-away intent.
     *
     * @param mirrorJson JSON string of the MirrorState (wire form per contracts/shell-ipc.md).
     * @param deleteIntentJson reserved for a Rust-supplied swipe-away
     * token; currently ignored — the host attaches its own.
     */
    @JvmStatic
    fun publishMirror(mirrorJson: String, deleteIntentJson: String?) {
        val mirror = media ?: return
        val deleteIntent = resident?.swipeAwayIntent()
        mirror.publishJson(mirrorJson, deleteIntent)
    }
}
