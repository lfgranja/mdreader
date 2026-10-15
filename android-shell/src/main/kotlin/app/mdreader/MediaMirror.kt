package app.mdreader

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.util.Log

/**
 * MediaSession + notification mirror (FR-003, FR-028 series,
 * FR-029, FR-028e, FR-042; SC-035).
 *
 * Mirrors the Rust-published session state onto the platform
 * MediaSession and the one media notification surface, and
 * forwards every control event back to Rust as a closed verb
 * wire string through [forwardVerb]. The mirror is derived,
 * never a second clock (SC-035 / FR-003): it holds no
 * position, rate or transport state of its own and computes
 * nothing — the only arithmetic in the class is the seconds
 * to millis unit conversion the platform APIs require
 * (PlaybackState and the notification progress report
 * millis), applied to the published position_secs and to
 * the determinate progress, never to an estimate.
 *
 * Notification content is exactly the FR-028 series: the
 * document title (metadata title else the stable filename
 * placeholder — FR-029), the most-recent-passed section
 * title (FR-028b/FR-028c/FR-028d), and the progress
 * indication (FR-028a) — never document body text, never
 * elapsed or remaining time, never rate, never voice. The
 * rate is deliberately not mirrored: the published mirror
 * state carries none (the rate ladder is a Rust concern and
 * the rate never appears on this surface), so the
 * platform's required PlaybackState speed field is set to
 * the neutral 1.0f — the field is present because the
 * platform requires it, not because the mirror knows a
 * rate. The single setPlaybackState call carries the built
 * state object, never a computed position.
 *
 * Appearance is the DESIGN-SONA palette tokens of
 * contracts/surface.md — the contract's single source
 * of truth, which resolves T044: the title `#F2F0E6`,
 * the section `#A9B8CE`, the progress white /
 * `#55688F`, the warning `#FFD400` for the warning
 * banner only, the failure `#FF4D63` for the failure
 * banner only, and the contract's green prohibited —
 * declared nowhere, used nowhere. The platform exposes
 * exactly one per-notification color channel, which
 * carries the title token; the remaining tokens are
 * declared on [MediaMirror]'s companion as the
 * mirror's styling vocabulary — constants documented
 * at their application sites, never computed and
 * never derived from the mirrored state (SC-035: the
 * reason banner's warning/failure class is Rust's
 * classification, and this mirror renders the handed
 * reason string verbatim).
 *
 * Dismissal (FR-028e) is decided by the Rust resolver
 * (player::surface::SurfaceResolver::dismissal); this
 * class applies only the platform mechanics of the
 * classification it is handed: NonDismissable renders the
 * ongoing surface — setOngoing(true), so pausing is the
 * only route to dismissal while audio plays —, AutoDismiss
 * renders the transient surface that dismisses itself at
 * the handed window (the 3 s of TRANSIENT_DISMISS_MS — a
 * stated failure reason under FR-037 or a focus-loss
 * pause reason under FR-041), and Dismissable renders the
 * resident pause surface — dismissable, never
 * auto-dismissed, its resume control surviving until the
 * reader chooses a way out.
 *
 * Control routing: the MediaSession callback forwards each
 * platform control event to Rust through [forwardVerb] as
 * the closed verb wire string (onPlay to play, onPause to
 * pause, onSkipToNext to skip_forward5s, onSkipToPrevious to
 * skip_back5s), never honoured locally. The Stop control has
 * no platform route — the session's action mask never
 * advertises it and no callback forwards it — because the
 * lock-screen control set is play, pause and skip: the Stop
 * control exists in-app only (contracts/surface.md), distinct
 * from the `stopped` transport STATE this mirror renders. The
 * rate and voice controls have no
 * MediaSession callback route, so they ride the
 * notification's action buttons as broadcast PendingIntents
 * classified at the boundary (the Residency idiom). An
 * unrecognised platform control is forwarded as the
 * diagnostic control_unmapped, never absorbed; an
 * unrecognised transport wire string renders STATE_ERROR,
 * never a healthy state.
 *
 * The exactly-one surface invariant is structural: one
 * fixed notification id, one surface reference nulled at
 * teardown, so a second media surface can never exist. The
 * swipe-away delete intent is supplied by the host — the
 * same Residency.swipeAwayIntent token the host attaches
 * to the playing surface, so that where a platform or OEM
 * build permits dismissal regardless of the ongoing flag,
 * the gesture reaches Rust and pauses first (FR-028e).
 *
 * The haptic confirmation (FR-016, contracts/earcon.md)
 * rides this same publish: Rust selects the pattern id
 * per action/trigger and publishes it with the state, and
 * [publish] renders the handed id verbatim — one distinct
 * [VibrationEffect] from a closed seven-pattern table,
 * with zero selection or computation on this side
 * (SC-035). The pattern never depends on any state this
 * class mirrors. The [Vibrator] is no audio stream: no
 * AudioTrack, no stream attribute, no focus request, so
 * the confirmation rides outside `STREAM_MUSIC` — never
 * ducked, never silenced with the media stream — which is
 * what keeps the muted/DND path of the silent
 * confirmation alive. The render is best-effort
 * ([renderHaptic]): a missing vibrator or a platform
 * refusal logs and continues, and the visual mirror this
 * same publish renders — the giant transport glyph, the
 * progress indication and the banner state change — is the
 * covering path when the haptic is fully disabled, so no
 * haptic failure ever blocks a publish.
 *
 * @param context service context used to post and dismiss
 * the media surface and to register the control receiver.
 * @param forwardVerb sink receiving one verb wire string
 * per control event; the caller relays it to Rust
 * uninterpreted.
 */
class MediaMirror(
    private val context: Context,
    private val forwardVerb: (String) -> Unit
) {

    /**
     * The progress indication (FR-028a) as published on the
     * wire — the externally-tagged T061 variant in the
     * contract's snake_case wire form: a determinate session
     * carries the published position and the marked document
     * extent (never an estimate), and an indeterminate
     * session carries no total for its whole session.
     */
    sealed class Progress {
        /** The total spoken length was known at session
         * start (FR-028a). */
        data class Determinate(
            /** The session's authoritative position, in
             * seconds — the published position_secs,
             * mirrored, never computed. */
            val positionSecs: Double,
            /** The marked document extent, in seconds —
             * never an estimate. */
            val durationSecs: Double
        ) : Progress()

        /** The total spoken length was unknown at session
         * start — the whole session stays indeterminate
         * (FR-028a). */
        object Indeterminate : Progress()
    }

    /**
     * The dismissal policy of the surface (FR-028e) — the
     * T061 classification, mirrored here only as the shape
     * the mirror consumes. The decision is Rust's
     * (player::surface::SurfaceResolver::dismissal); the
     * mirror applies the platform mechanics of whatever
     * classification it is handed.
     */
    sealed class Dismissal {
        /** The surface cannot be dismissed — the ongoing
         * state while audio plays (FR-028e: pausing is the
         * only route to dismissal while a document plays). */
        object NonDismissable : Dismissal()

        /** The surface dismisses itself after the handed
         * window — the transient-reason surface (a failure
         * reason under FR-037 or a focus-loss pause reason
         * under FR-041). */
        data class AutoDismiss(
            /** The window after which the surface dismisses
             * itself, in milliseconds. */
            val afterMs: Long
        ) : Dismissal()

        /** The surface may be dismissed by the reader but is
         * never dismissed automatically — the resident
         * surface after a pause, whose paused state and
         * resume control remain until the reader chooses a
         * way out (FR-028e). */
        object Dismissable : Dismissal()
    }

    /**
     * The Rust-published mirror state (contracts/
     * shell-ipc.md) — the wire form the mirror consumes:
     * the session's authoritative transport and position,
     * the resolved surface content, the progress indication
     * fixed at session start, the stated reason of a
     * transient surface, the dismissal classification, and
     * the published haptic pattern id. The mirror holds no
     * state of its own; every publish replaces the previous
     * render.
     */
    data class MirrorState(
        /** The transport wire string — the closed
         * five-member wire set (playing, paused, stopped,
         * completed, interrupted). */
        val transport: String,
        /** The session's authoritative position, in seconds
         * — the published position_secs, mirrored, never
         * computed. */
        val positionSecs: Double,
        /** The surface title (FR-029): the document
         * metadata title, or the filename as the stable
         * placeholder. */
        val title: String,
        /** The most-recent-passed section title
         * (FR-028b/FR-028c/FR-028d), or null before the
         * first heading and for a heading-less document —
         * never forward-looking, never document body text. */
        val sectionTitle: String?,
        /** The progress indication (FR-028a). */
        val progress: Progress,
        /** The stated reason of a transient surface — a
         * failure reason under FR-037 or a focus-loss pause
         * reason under FR-041 — or null for a surface
         * without one. The short lock-screen statement
         * (contracts/shell-ipc.md). */
        val transientReason: String?,
        /** The dismissal classification (FR-028e), decided
         * by the Rust resolver. */
        val dismissal: Dismissal,
        /** The published haptic pattern id (FR-016) — the
         * confirmation pattern Rust selected for this
         * trigger, mirrored verbatim: one of the closed 1–7
         * id table (see [HapticPattern]), published on the
         * wire's `haptic` key, or null when the publish
         * carries no confirmation. Selecting the pattern
         * per action/trigger is Rust's act; this mirror only
         * renders the handed id through the closed
         * vibration table (SC-035) — never a pattern chosen
         * from the transport, the action or any other state
         * this class mirrors. */
        val hapticId: Int?
    )

    /**
     * The closed set of published haptic pattern ids
     * (FR-016) — the seven confirmations of
     * contracts/earcon.md, enumerated here as the id codes
     * the mirror wire's `haptic` key carries. Rust selects
     * the pattern per action/trigger and publishes the id
     * with the trigger's mirror state; this enum is the
     * mirror's render vocabulary — a closed table with no
     * eighth member, no default and no fallthrough.
     *
     * The ids are fixed from the Rust `EarconKind`
     * declaration order (crates/player/src/earcon.rs:
     * Play, Pause, SkipFwd, SkipBack, Warning, Failure,
     * RateStep), so the published id and the rendered
     * pattern stay one closed set on both sides of the
     * wire.
     */
    private enum class HapticPattern(val code: Int) {
        /** Play confirmation — wire id 1. */
        Play(1),

        /** Pause confirmation — wire id 2. */
        Pause(2),

        /** Skip-forward confirmation — wire id 3. */
        SkipFwd(3),

        /** Skip-backward confirmation — wire id 4. */
        SkipBack(4),

        /** Warning confirmation — the pause-demanding
         * interruption, wire id 5. */
        Warning(5),

        /** Failure confirmation — the undeliverable voice,
         * wire id 6. */
        Failure(6),

        /** Rate-step confirmation — wire id 7. */
        RateStep(7);

        companion object {
            /**
             * The pattern for one published id, or null for
             * an id outside the closed set. A well-formed
             * publish always resolves — the published ids
             * are this closed set — so a null is the
             * fail-closed branch for a malformed wire value:
             * it drops with a diagnostic at [renderHaptic],
             * never a default pattern.
             */
            fun of(code: Int): HapticPattern? =
                values().firstOrNull { it.code == code }
        }
    }

    /** Wire verb strings of the closed verb set
     * (contracts/shell-ipc.md) and the transport wire
     * strings of the closed transport set. */
    companion object {
        /** Wire verb for the Play control — the session's
         * onPlay route and the resident surface's resume
         * control (the same closed verb; Rust applies the
         * resume semantics against the stored bookmark). */
        const val VERB_PLAY = "play"

        /** Wire verb for the Pause control (the session's
         * onPause route). */
        const val VERB_PAUSE = "pause"

        /** Wire verb for the Stop control — the in-app route only
         * (the service intent that the status_view Stop press
         * drives). The lock-screen media controls carry no Stop
         * (contracts/surface.md: the Stop control exists in-app,
         * distinct from the `stopped` transport STATE). */
        const val VERB_STOP = "stop"

        /** Wire verb for the skip-forward control (the
         * session's onSkipToNext route). */
        const val VERB_SKIP_FORWARD_5S = "skip_forward5s"

        /** Wire verb for the skip-back control (the
         * session's onSkipToPrevious route). */
        const val VERB_SKIP_BACK_5S = "skip_back5s"

        /** Wire verb for the rate-up control (a notification
         * action button — no MediaSession callback route). */
        const val VERB_RATE_UP = "rate_up"

        /** Wire verb for the rate-down control (a
         * notification action button). */
        const val VERB_RATE_DOWN = "rate_down"

        /** Wire verb for the cycle-voice-next control (a
         * notification action button; FR-018b). */
        const val VERB_CYCLE_VOICE_NEXT = "cycle_voice_next"

        /** Wire verb for the cycle-voice-prev control (a
         * notification action button; FR-018b). */
        const val VERB_CYCLE_VOICE_PREV = "cycle_voice_prev"

        /** Wire verb for resuming at the stored bookmark
         * (contracts/shell-ipc.md closed verb set; the
         * session applies the resume semantics against the
         * stored bookmark). */
        const val VERB_RESUME_AT_STORED = "resume_at_stored"

        /** Wire verb for discarding the bookmark and starting
         * over (contracts/shell-ipc.md closed verb set). */
        const val VERB_START_OVER = "start_over"

        /** Wire verb for the engine-selection control
         * (contracts/shell-ipc.md closed verb set; the
         * session answers the authoritative engine id). */
        const val VERB_CHOOSE_ENGINE = "choose_engine"

        /** Diagnostic verb for an unrecognised platform
         * control action; never absorbed. */
        const val VERB_CONTROL_UNMAPPED = "control_unmapped"

        /** Wire transport string for the playing state. */
        const val TRANSPORT_PLAYING = "playing"

        /** Wire transport string for the paused state. */
        const val TRANSPORT_PAUSED = "paused"

        /** Wire transport string for the stopped state. */
        const val TRANSPORT_STOPPED = "stopped"

        /** Wire transport string for the completed state —
         * playback reached its end, the terminal no-audio
         * state. */
        const val TRANSPORT_COMPLETED = "completed"

        /** Wire transport string for the interrupted state
         * — a live resumable state. */
        const val TRANSPORT_INTERRUPTED = "interrupted"

        /** The transient auto-dismiss window (FR-028e) —
         * the T061 constant, mirrored here for the
         * contract's documentation. The window actually
         * applied is the one handed in the dismissal
         * classification, which Rust fixes at this value. */
        const val TRANSIENT_DISMISS_MS = 3000L

        /** Notification channel of the media surface. Public
         * because the foreground holder (see [foregroundHolder])
         * posts on this same channel: one channel, one id, one
         * surface — the holder is replaced by the first publish
         * instead of joining it as a second surface. */
        const val CHANNEL_ID = "mdreader_media"

        /** Notification channel display name. */
        private const val CHANNEL_NAME = "Media"

        /** Log tag of the mirror. */
        private const val TAG = "MediaMirror"

        /** MediaSession tag — the platform's session
         * identifier for this application's playback. */
        private const val SESSION_TAG = "mdreader_media"

        /** Notification id of the single media surface. One
         * id is one surface: every publish replaces the
         * previous render instead of adding a second
         * notification, and teardown cancels exactly this
         * id. This ID is shared with the foreground service
         * (PlaybackService.FOREGROUND_ID) so the MediaStyle
         * notification IS the foreground service notification,
         * enabling lock-screen controls and screen-off
         * continuity. The resident surface (Residency) also
         * uses this ID, replacing the playing surface on
         * pause. */
        private const val NOTIFICATION_ID = 1

        /** Broadcast action of the resume control on a
         * resident-pause surface. */
        private const val ACTION_RESUME = "app.mdreader.RESUME"

        /** Broadcast action of the rate-up control. */
        private const val ACTION_RATE_UP = "app.mdreader.RATE_UP"

        /** Broadcast action of the rate-down control. */
        private const val ACTION_RATE_DOWN = "app.mdreader.RATE_DOWN"

        /** Broadcast action of the cycle-voice-next control. */
        private const val ACTION_CYCLE_VOICE_NEXT =
            "app.mdreader.CYCLE_VOICE_NEXT"

        /** Broadcast action of the cycle-voice-prev control. */
        private const val ACTION_CYCLE_VOICE_PREV =
            "app.mdreader.CYCLE_VOICE_PREV"

        /** Request code of the resume control. */
        private const val REQUEST_RESUME = 0

        /** Request code of the rate-up control. */
        private const val REQUEST_RATE_UP = 1

        /** Request code of the rate-down control. */
        private const val REQUEST_RATE_DOWN = 2

        /** Request code of the cycle-voice-next control. */
        private const val REQUEST_CYCLE_VOICE_NEXT = 3

        /** Request code of the cycle-voice-prev control. */
        private const val REQUEST_CYCLE_VOICE_PREV = 4

        /** Request code of the application launch target. */
        private const val REQUEST_LAUNCH = 5

        /**
         * The neutral PlaybackState speed — the platform's
         * required speed field set to 1.0f because the
         * published mirror state deliberately carries no
         * rate (the rate ladder is a Rust concern and the
         * rate never appears on this surface — FR-028
         * series). The field is present because the platform
         * requires it, not because the mirror knows a rate.
         */
        private const val NEUTRAL_SPEED = 1.0f

        /** Seconds per millisecond — the unit conversion
         * factor the platform APIs require (PlaybackState
         * and the notification progress report millis). */
        private const val MILLIS_PER_SECOND = 1000.0

        // ---------------------------------------------------------------
        // Haptic pattern table (FR-016, contracts/earcon.md).
        //
        // The seven closed patterns below are pairwise-
        // distinguishable by buzz duration and count, which
        // is the acceptance dimension of SC-004 (10/10 blind
        // per confirmation type, sounds on and off):
        //
        //   id  pattern     buzzes (ms)   rendered effect
        //   1   Play        30            one 30 ms one-shot
        //   2   Pause       80            one 80 ms one-shot
        //   3   SkipFwd     20 + 40       buzz 20, gap, buzz 40
        //   4   SkipBack    40 + 20       buzz 40, gap, buzz 20
        //   5   Warning     15 + 15 + 30  buzz 15, gap, buzz
        //                                     15, gap, buzz 30
        //   6   Failure     80 + 80       buzz 80, gap, buzz 80
        //   7   RateStep    10            one 10 ms one-shot
        //
        // The three singles differ by duration (30 / 80 /
        // 10); the three doubles differ by duration pair
        // ((20,40) / (40,20) / (80,80)); the warning is the
        // only triple. All render at default amplitude —
        // identity rests on duration and count alone, so no
        // pattern hides inside another. `createWaveform`
        // timings alternate buzz/gap/buzz, so every
        // multi-buzz pattern carries the fixed gap between
        // its buzzes: the gap separates, it never
        // distinguishes.
        // ---------------------------------------------------------------

        /** Play confirmation buzz — a single 30 ms
         * one-shot. */
        private const val HAPTIC_PLAY_MS = 30L

        /** Pause confirmation buzz — a single 80 ms
         * one-shot, distinct from play by duration alone. */
        private const val HAPTIC_PAUSE_MS = 80L

        /** Rate-step confirmation tick — a single 10 ms
         * one-shot, the shortest pattern of the table. */
        private const val HAPTIC_RATE_STEP_MS = 10L

        /** The inter-buzz gap of every multi-buzz pattern —
         * 20 ms, fixed. */
        private const val HAPTIC_GAP_MS = 20L

        /** Skip-forward confirmation waveform — buzz 20 ms,
         * gap, buzz 40 ms: the forward gesture renders
         * short to long. */
        private val HAPTIC_SKIP_FWD_TIMINGS =
            longArrayOf(20, HAPTIC_GAP_MS, 40)

        /** Skip-backward confirmation waveform — buzz 40 ms,
         * gap, buzz 20 ms: the backward gesture renders long
         * to short, the exact mirror of
         * [HAPTIC_SKIP_FWD_TIMINGS]. */
        private val HAPTIC_SKIP_BACK_TIMINGS =
            longArrayOf(40, HAPTIC_GAP_MS, 20)

        /** Warning confirmation waveform — buzz 15 ms, gap,
         * buzz 15 ms, gap, buzz 30 ms: the triple of the
         * pause-demanding interruption. */
        private val HAPTIC_WARNING_TIMINGS =
            longArrayOf(15, HAPTIC_GAP_MS, 15, HAPTIC_GAP_MS, 30)

        /** Failure confirmation waveform — buzz 80 ms, gap,
         * buzz 80 ms: the repeated long buzz of the
         * undeliverable voice. */
        private val HAPTIC_FAILURE_TIMINGS =
            longArrayOf(80, HAPTIC_GAP_MS, 80)

        /** `createWaveform` repeat index — play the pattern
         * once: a confirmation, never a looping alert. */
        private const val HAPTIC_NO_REPEAT = -1

        /** Label of the resume control. */
        private const val RESUME_LABEL = "Resume"

        /** Label of the rate-up control. */
        private const val RATE_UP_LABEL = "Rate +"

        /** Label of the rate-down control. */
        private const val RATE_DOWN_LABEL = "Rate -"

        /** Label of the cycle-voice-next control. */
        private const val VOICE_NEXT_LABEL = "Voice next"

        /** Label of the cycle-voice-prev control. */
        private const val VOICE_PREV_LABEL = "Voice prev"

        /**
         * The session's action mask — the closed lock-screen
         * transport control set the MediaSession exposes to the
         * platform's media controls (play, pause, skip forward
         * 5 s, skip back 5 s). Stop is deliberately absent: the
         * Stop control exists in-app only
         * (contracts/surface.md — distinct from the `stopped`
         * transport STATE, which this mirror renders). The rate
         * and voice controls are not transport actions: they ride
         * the notification's action buttons, classified at the
         * boundary.
         */
        private val CONTROL_ACTIONS: Long =
            PlaybackState.ACTION_PLAY or
                PlaybackState.ACTION_PAUSE or
                PlaybackState.ACTION_SKIP_TO_NEXT or
                PlaybackState.ACTION_SKIP_TO_PREVIOUS

        // ---------------------------------------------------------------
        // DESIGN-SONA palette tokens (contracts/surface.md,
        // "DESIGN-SONA palette tokens (single source of
        // truth, resolves T044)") — the appearance
        // vocabulary of the one media surface, declared
        // here once so the mirror's styling never invents
        // a color:
        //
        //   token            hex        applies to
        //   title            #F2F0E6    the surface title — and the
        //                               notification accent (the
        //                               platform's single color
        //                               channel, setColor)
        //   section          #A9B8CE    the section title
        //   progress track   #55688F    the progress indication
        //   progress fill    #ffffff    the progress indication
        //   warning          #FFD400    the warning banner ONLY
        //   failure          #FF4D63    the failure banner ONLY
        //   green            (prohibited) never declared,
        //                               never used
        //
        // The platform colors the notification's text fields, the
        // progress bar and the banner, and offers no per-field
        // color channel beyond the single accent: the mirror
        // therefore applies the title token through setColor (the
        // accent paints the transport glyph and the app name,
        // high-contrast on the dark lock-screen surface) and
        // declares the remaining tokens here as the contract's
        // single source of truth — documented at their
        // application sites, never approximated by a custom
        // layout the mirror would then have to own, and never
        // chosen from mirrored state. No token is derived from
        // anything the mirror receives: the class of the reason
        // banner is Rust's classification (the warning/failure
        // split of contracts/surface.md), and this mirror
        // renders the handed reason line verbatim (SC-035 —
        // styling constants only, zero computation). No marquee,
        // no body text, no rate value and no voice identity
        // ride any token: none is declared, none is colorable.
        // ---------------------------------------------------------------

        /** DESIGN-SONA title token (`#F2F0E6`) — the color of the
         *  surface title (contracts/surface.md). The value
         *  [buildSurface] hands the platform's one
         *  per-notification color channel, so the accent paints
         *  the transport glyph and the app name. */
        private const val DESIGN_SONA_TITLE = 0xFFF2F0E6.toInt()

        /** DESIGN-SONA section token (`#A9B8CE`) — the color of
         *  the section title the surface renders under the
         *  title (contracts/surface.md). The platform themes
         *  the notification's text fields, so the token is the
         *  declared contract of that field's color; the mirror
         *  never tints text (no per-field channel exists). */
        private const val DESIGN_SONA_SECTION = 0xFFA9B8CE.toInt()

        /** DESIGN-SONA progress track token (`#55688F`) — the
         *  track of the progress indication [buildSurface]
         *  reports through the platform's progress APIs
         *  (contracts/surface.md). The bar itself is
         *  system-themed; the token is the declared color of
         *  the indication's track. */
        private const val DESIGN_SONA_PROGRESS_TRACK = 0xFF55688F.toInt()

        /** DESIGN-SONA progress fill token (white) — the fill of
         *  the progress indication (contracts/surface.md), the
         *  same white the contract's progress token pair
         *  names. */
        private const val DESIGN_SONA_PROGRESS_FILL = 0xFFFFFFFF.toInt()

        /** DESIGN-SONA warning token (`#FFD400`) — the warning
         *  banner ONLY (contracts/surface.md). Reserved: the
         *  banner's class is Rust's classification and the
         *  platform's theming renders the reason line, so the
         *  token is the declared color of that one banner and
         *  no other role ever borrows it. */
        private const val DESIGN_SONA_WARNING = 0xFFFFD400.toInt()

        /** DESIGN-SONA failure token (`#FF4D63`) — the failure
         *  banner ONLY (contracts/surface.md). Reserved the
         *  same way: the failure banner of the mirror's
         *  reason line, never a control, never a glyph. The
         *  contract's prohibited green is declared nowhere. */
        private const val DESIGN_SONA_FAILURE = 0xFFFF4D63.toInt()
    }

    /** The platform MediaSession this mirror reflects state
     * into and receives control events from. Released at
     * [teardown]. */
    private val mediaSession = MediaSession(context, SESSION_TAG)

    /**
     * The session's control callback: each platform control
     * event is forwarded to Rust through [forwardVerb] as
     * the closed verb wire string the moment the platform
     * delivers it, never honoured locally. The five
     * transport controls route here; the rate and voice
     * controls route through the notification's action
     * buttons (see [surfaceActions]).
     */
    private val sessionCallback = object : MediaSession.Callback() {
        override fun onPlay() {
            forwardVerb(VERB_PLAY)
        }

        override fun onPause() {
            forwardVerb(VERB_PAUSE)
        }

        override fun onSkipToNext() {
            forwardVerb(VERB_SKIP_FORWARD_5S)
        }

        override fun onSkipToPrevious() {
            forwardVerb(VERB_SKIP_BACK_5S)
        }
    }

    /**
     * The notification-action receiver: the rate, voice and
     * resume control buttons arrive here as broadcasts, and
     * each is classified at the boundary into its closed
     * verb wire string and forwarded to Rust through
     * [forwardVerb] the moment the platform delivers it. Any
     * other platform action is forwarded as the diagnostic
     * [VERB_CONTROL_UNMAPPED], never absorbed.
     */
    private val controlReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            forwardVerb(
                when (intent.action) {
                    ACTION_RESUME -> VERB_PLAY
                    ACTION_RATE_UP -> VERB_RATE_UP
                    ACTION_RATE_DOWN -> VERB_RATE_DOWN
                    ACTION_CYCLE_VOICE_NEXT -> VERB_CYCLE_VOICE_NEXT
                    ACTION_CYCLE_VOICE_PREV -> VERB_CYCLE_VOICE_PREV
                    else -> VERB_CONTROL_UNMAPPED
                }
            )
        }
    }

    /** Handler on the main thread — the auto-dismiss
     * schedule of a transient surface (FR-028e). */
    private val handler = Handler(Looper.getMainLooper())

    /** The pending auto-dismiss of the transient surface:
     * withdraws the surface at the handed window. */
    private val autoDismiss = Runnable { withdrawSurface() }

    /**
     * The device vibrator, acquired once at the API level
     * boundary (contracts/earcon.md): the
     * [VibratorManager] route on API 31+, the legacy
     * `VIBRATOR_SERVICE` route below. Null only where the
     * platform exposes no vibrator service at all — the
     * haptic render then logs and the visual mirror covers
     * ([renderHaptic]). The vibrator is no audio stream:
     * it is never routed through the playback backend, so
     * the confirmation rides outside `STREAM_MUSIC`, is
     * never ducked and is never silenced with the media
     * stream — the muted/DND path of the silent
     * confirmation (contracts/earcon.md).
     */
    @Suppress("DEPRECATION")
    private val vibrator: Vibrator? by lazy {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            context.getSystemService(VibratorManager::class.java)
                ?.defaultVibrator
        } else {
            context.getSystemService(Context.VIBRATOR_SERVICE) as? Vibrator
        }
    }

    /**
     * The single media surface — the one notification this
     * mirror has posted (the structural form of the
     * exactly-one invariant: one reference, replaced by
     * every publish, nulled at teardown, so a second media
     * surface can never exist).
     */
    private var surface: Notification? = null

    /** Whether the control receiver is currently
     * registered. */
    private var registered = false

    /**
     * Callback to start the foreground service with the MediaStyle
     * notification on the first publish. Set by PlaybackService
     * after ShellHost.init() so the MediaStyle notification becomes
     * the foreground service notification (same ID), enabling
     * lock-screen controls and screen-off continuity.
     */
    private var foregroundStarter: ((Notification) -> Unit)? = null

    /**
     * Sets the callback to start the foreground service with the
     * MediaStyle notification. Called by PlaybackService after
     * ShellHost.init() to wire the foreground service to the
     * MediaMirror's notification.
     */
    fun setForegroundStarter(starter: (Notification) -> Unit) {
        foregroundStarter = starter
    }

    init {
        mediaSession.setFlags(
            MediaSession.FLAG_HANDLES_MEDIA_BUTTONS or
                MediaSession.FLAG_HANDLES_TRANSPORT_CONTROLS
        )
        mediaSession.setCallback(sessionCallback)
    }

    /**
     * Parses one Rust-published mirror state from its wire JSON
     * (contracts/shell-ipc.md) and renders it through [publish].
     * The wire form is this mirror's own, so the parsing lives
     * here and the host only routes the call; every field is read
     * verbatim from the published state — nothing is computed,
     * estimated, or defaulted from another value (SC-035).
     *
     * The optional `haptic` key is read verbatim as the
     * published pattern id (FR-016): the selection is Rust's
     * (the pattern id rides the same publish as the trigger,
     * so a confirmation cannot sound without the mirror) and
     * the render is the closed vibration table inside
     * [publish].
     *
     * @param mirrorJson JSON string of the MirrorState (wire form
     * per contracts/shell-ipc.md).
     * @param deleteIntent the host-supplied swipe-away delete
     * intent (see [publish]); null omits it.
     */
    fun publishJson(mirrorJson: String, deleteIntent: PendingIntent? = null) {
        try {
            val json = org.json.JSONObject(mirrorJson)
            val transport = json.getString("transport")
            val positionSecs = json.getDouble("positionSecs")
            val title = json.optString("title", "Document")
            val sectionTitle =
                if (json.has("sectionTitle") && !json.isNull("sectionTitle")) {
                    json.getString("sectionTitle")
                } else {
                    null
                }
            val transientReason =
                if (json.has("transientReason") && !json.isNull("transientReason")) {
                    json.getString("transientReason")
                } else {
                    null
                }

            // FR-016: the published haptic pattern id — Rust's
            // selection, read verbatim; absent (or null) carries
            // no confirmation.
            val hapticId =
                if (json.has("haptic") && !json.isNull("haptic")) {
                    json.getInt("haptic")
                } else {
                    null
                }

            val progressObj = json.getJSONObject("progress")
            val progress: Progress = if (progressObj.has("determinate")) {
                val det = progressObj.getJSONObject("determinate")
                Progress.Determinate(
                    positionSecs = det.getDouble("positionSecs"),
                    durationSecs = det.getDouble("durationSecs")
                )
            } else {
                Progress.Indeterminate
            }

            val dismissalObj = json.getJSONObject("dismissal")
            val dismissal: Dismissal = when (dismissalObj.keys().next()) {
                "nonDismissable" -> Dismissal.NonDismissable
                "autoDismiss" -> Dismissal.AutoDismiss(
                    afterMs = dismissalObj.getLong("autoDismiss")
                )
                "dismissable" -> Dismissal.Dismissable
                else -> Dismissal.NonDismissable
            }

            publish(
                MirrorState(
                    transport = transport,
                    positionSecs = positionSecs,
                    title = title,
                    sectionTitle = sectionTitle,
                    progress = progress,
                    transientReason = transientReason,
                    dismissal = dismissal,
                    hapticId = hapticId
                ),
                deleteIntent
            )
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse mirror JSON: ${e.message}")
        }
    }

    /**
     * Reflects one Rust-published mirror state onto the
     * MediaSession and the media surface (FR-003, FR-028
     * series, FR-028e).
     *
     * The session's playback state is the published
     * transport and position — the position being the
     * published position_secs converted to millis, a unit
     * conversion for the platform API, never a computation
     * — at the neutral speed, with the closed transport
     * action mask. The surface carries the document title,
     * the section title and the progress (FR-028 series)
     * and applies the platform mechanics of the handed
     * dismissal classification: an ongoing surface while
     * playing (setOngoing(true) — FR-028e), a transient
     * surface that dismisses itself at the handed window,
     * or a resident-pause surface that is dismissable
     * without being auto-dismissed, its resume control
     * surviving. A torn-down transport (stopped,
     * completed) withdraws the surface — the Rust resolver
     * classifies a torn-down state so that any residual
     * surface is dismissable precisely because a torn-down
     * state has nothing left to keep on the surface (the
     * FR-032f teardown dismisses the notification), and a
     * stopped document must offer no control that
     * silently begins nothing (FR-010, FR-032f); the full
     * teardown (residency, session) is the host's act
     * through [teardown].
     *
     * @param state the Rust-published mirror state (the
     * wire form of contracts/shell-ipc.md with the T061
     * surface shapes).
     * @param deleteIntent the host-supplied swipe-away
     * delete intent — the same Residency.swipeAwayIntent
     * token the host attaches to the playing surface, so
     * that where a platform or OEM build permits
     * dismissal regardless of the ongoing flag, the
     * gesture reaches Rust and pauses first (FR-028e).
     * Null omits the delete intent.
     *
     * Safe to call again: a second call replaces the
     * previous render — the same notification id, the same
     * single surface reference — and cancels any pending
     * auto-dismiss of the superseded surface.
     *
     * When the published state carries a haptic pattern id,
     * this same call renders the confirmation first: the
     * [VibrationEffect] of the published pattern, from the
     * closed table, best-effort and never blocking the
     * surface render below (FR-016; see [renderHaptic]).
     * The confirmation rides the publish itself — Rust
     * publishes trigger and pattern in one call, so a
     * haptic cannot fire without the mirror (contracts/
     * earcon.md) — and a failure of the haptic path is
     * covered by the visual render, which always happens.
     */
    fun publish(state: MirrorState, deleteIntent: PendingIntent? = null) {
        renderHaptic(state.hapticId)
        registerControlReceiver()
        mediaSession.isActive = true
        val mirrored = PlaybackState.Builder()
            .setState(
                playbackConstantOf(state.transport),
                positionMillisOf(state.positionSecs),
                NEUTRAL_SPEED
            )
            .setActions(CONTROL_ACTIONS)
            .build()
        mediaSession.setPlaybackState(mirrored)
        val manager =
            context.getSystemService(NotificationManager::class.java)
                ?: return
        ensureChannel(manager)
        if (
            state.transport == TRANSPORT_STOPPED ||
            state.transport == TRANSPORT_COMPLETED
        ) {
            withdrawSurface()
            return
        }
        handler.removeCallbacks(autoDismiss)
        val ongoing = state.dismissal is Dismissal.NonDismissable
        val built = buildSurface(state, ongoing, deleteIntent)
        // First publish: start foreground service with MediaStyle notification
        // Subsequent publishes: update via notify() (same ID replaces)
        if (surface == null && foregroundStarter != null) {
            foregroundStarter!!(built)
        } else {
            manager.notify(NOTIFICATION_ID, built)
        }
        surface = built
        val transient = state.dismissal as? Dismissal.AutoDismiss
        if (transient != null) {
            handler.postDelayed(autoDismiss, transient.afterMs)
        }
    }

    /**
     * Releases the mirror: cancels any pending
     * auto-dismiss, dismisses the one media notification,
     * unregisters the control receiver, nulls the single
     * surface reference and releases the MediaSession, so
     * zero media surfaces remain and no control event is
     * forwarded afterwards (FR-028e, FR-032f, SC-049).
     * The host invokes this when the session ends — the
     * classified teardown event — and the FR-032f document
     * end is its primary path. Applies nothing locally:
     * playback state remains owned by Rust alone (FR-042,
     * SC-035). Safe to call again: a second call releases
     * nothing.
     */
    fun teardown() {
        handler.removeCallbacks(autoDismiss)
        val manager = context.getSystemService(NotificationManager::class.java)
        if (manager != null) {
            manager.cancel(NOTIFICATION_ID)
        }
        unregisterControlReceiver()
        surface = null
        mediaSession.release()
    }

    /**
     * Withdraws the media surface: cancels any pending
     * auto-dismiss and dismisses the one notification, so
     * no media surface remains. The torn-down transports
     * (stopped, completed) withdraw here — a torn-down
     * state has nothing left to keep on the surface
     * (FR-032f, FR-010) — and the transient surface
     * withdraws itself at the handed window (FR-028e).
     * The session itself is released only at [teardown] —
     * the host's act. Safe to call again: a second call
     * cancels nothing.
     */
    private fun withdrawSurface() {
        handler.removeCallbacks(autoDismiss)
        val manager = context.getSystemService(NotificationManager::class.java)
        if (manager != null) {
            manager.cancel(NOTIFICATION_ID)
        }
        surface = null
    }

    /**
     * Renders the published haptic confirmation (FR-016):
     * the [VibrationEffect] of the published pattern id,
     * and nothing else. Selection is not this side's act —
     * Rust picks the pattern per action/trigger and the id
     * rides the mirror payload, so this method maps the
     * handed id through a closed table ([HapticPattern],
     * [effectOf]) with no pattern ever chosen from the
     * transport, the action or any other mirrored state
     * (SC-035).
     *
     * Every outcome is best-effort and none can block
     * [publish]: a null id renders nothing (a publish
     * without confirmation); an id outside the closed set
     * — impossible for a well-formed payload — logs and
     * drops rather than defaulting to some other pattern; a
     * device with no vibrator logs and continues; and any
     * platform exception is caught and logged. The visual
     * mirror — the transport glyph, the progress indication
     * and the banner state change this same publish
     * renders — is the covering path when the haptic is
     * fully disabled, so the confirmation never depends on
     * the vibrator delivering.
     *
     * @param hapticId the published pattern id, or null when
     * the publish carries no confirmation.
     */
    private fun renderHaptic(hapticId: Int?) {
        if (hapticId == null) {
            return
        }
        val pattern = HapticPattern.of(hapticId)
        if (pattern == null) {
            Log.w(
                TAG,
                "haptic pattern id $hapticId outside the closed set; dropped"
            )
            return
        }
        val device = vibrator
        if (device == null || !device.hasVibrator()) {
            Log.i(
                TAG,
                "no vibrator on this device; haptic $pattern dropped; " +
                    "the visual mirror covers"
            )
            return
        }
        try {
            device.vibrate(effectOf(pattern))
        } catch (e: Exception) {
            Log.w(TAG, "haptic $pattern not delivered: ${e.message}")
        }
    }

    /**
     * The distinct [VibrationEffect] of one closed haptic
     * pattern — the render table, fixed here and nowhere
     * else. All seven patterns are pairwise-distinguishable
     * by buzz duration and count (the table lives on
     * [MediaMirror]'s companion constants):
     *
     * | pattern   | effect |
     * | play      | one 30 ms buzz |
     * | pause     | one 80 ms buzz |
     * | skip-fwd  | buzz 20 ms + 40 ms |
     * | skip-back | buzz 40 ms + 20 ms |
     * | warning   | buzz 15 + 15 + 30 ms |
     * | failure   | buzz 80 + 80 ms |
     * | rate-step | one 10 ms tick |
     *
     * The `when` is exhaustive over the closed set with no
     * else and no default: [HapticPattern] is the whole
     * vocabulary, so no unknown pattern can reach here — one
     * handed in from the wire is dropped at [renderHaptic].
     * All effects render at default amplitude; identity
     * rests on duration and count alone.
     */
    private fun effectOf(pattern: HapticPattern): VibrationEffect =
        when (pattern) {
            HapticPattern.Play ->
                VibrationEffect.createOneShot(
                    HAPTIC_PLAY_MS, VibrationEffect.DEFAULT_AMPLITUDE
                )
            HapticPattern.Pause ->
                VibrationEffect.createOneShot(
                    HAPTIC_PAUSE_MS, VibrationEffect.DEFAULT_AMPLITUDE
                )
            HapticPattern.SkipFwd ->
                VibrationEffect.createWaveform(
                    HAPTIC_SKIP_FWD_TIMINGS, HAPTIC_NO_REPEAT
                )
            HapticPattern.SkipBack ->
                VibrationEffect.createWaveform(
                    HAPTIC_SKIP_BACK_TIMINGS, HAPTIC_NO_REPEAT
                )
            HapticPattern.Warning ->
                VibrationEffect.createWaveform(
                    HAPTIC_WARNING_TIMINGS, HAPTIC_NO_REPEAT
                )
            HapticPattern.Failure ->
                VibrationEffect.createWaveform(
                    HAPTIC_FAILURE_TIMINGS, HAPTIC_NO_REPEAT
                )
            HapticPattern.RateStep ->
                VibrationEffect.createOneShot(
                    HAPTIC_RATE_STEP_MS, VibrationEffect.DEFAULT_AMPLITUDE
                )
        }

    /**
     * The PlaybackState constant for a published transport
     * wire string — the closed five-member wire set:
     * playing mirrors STATE_PLAYING; paused and
     * interrupted mirror STATE_PAUSED (interrupted is a
     * live resumable state); stopped and completed mirror
     * STATE_STOPPED — completed is playback having reached
     * its end, the terminal no-audio state, which the
     * platform represents as stopped. Any other value
     * renders STATE_ERROR and is never absorbed: the
     * platform's error state is the rendering of a wire
     * value the closed set does not cover, never a silent
     * mapping to a healthy state.
     */
    private fun playbackConstantOf(transport: String): Int =
        when (transport) {
            TRANSPORT_PLAYING -> PlaybackState.STATE_PLAYING
            TRANSPORT_PAUSED -> PlaybackState.STATE_PAUSED
            TRANSPORT_INTERRUPTED -> PlaybackState.STATE_PAUSED
            TRANSPORT_STOPPED -> PlaybackState.STATE_STOPPED
            TRANSPORT_COMPLETED -> PlaybackState.STATE_STOPPED
            else -> PlaybackState.STATE_ERROR
        }

    /**
     * The published position converted from seconds to
     * milliseconds — the unit conversion the platform API
     * requires (PlaybackState reports its position in
     * millis). A conversion of the published position_secs
     * and nothing else: no position is computed, estimated
     * or derived from any other input (FR-003, SC-035).
     */
    private fun positionMillisOf(positionSecs: Double): Long =
        (positionSecs * MILLIS_PER_SECOND).toLong()

    /**
     * The notification action buttons of a surface
     * (contracts/shell-ipc.md): the rate and voice
     * controls ride the notification's actions as broadcast
     * PendingIntents classified at the boundary — they
     * have no MediaSession callback route — while the
     * transport controls (play, pause, stop, skip) reach
     * Rust through the MediaSession callback from the
     * lock-screen and media-control surfaces the session
     * token attaches.
     *
     * A resident pause adds the resume control — the same
     * closed verb as the session's onPlay — so the paused
     * surface keeps a control that still produces audio
     * (FR-028e: its resume control survives). A transient
     * surface carries none: it dismisses itself at the
     * window, and its resume path is the session's paused
     * state on the lock screen (FR-030).
     */
    private fun surfaceActions(dismissal: Dismissal): List<Notification.Action> {
        val rateAndVoice = listOf(
            Notification.Action.Builder(
                null, RATE_UP_LABEL,
                verbPendingIntent(ACTION_RATE_UP, REQUEST_RATE_UP)
            ).build(),
            Notification.Action.Builder(
                null, RATE_DOWN_LABEL,
                verbPendingIntent(ACTION_RATE_DOWN, REQUEST_RATE_DOWN)
            ).build(),
            Notification.Action.Builder(
                null, VOICE_NEXT_LABEL,
                verbPendingIntent(
                    ACTION_CYCLE_VOICE_NEXT,
                    REQUEST_CYCLE_VOICE_NEXT
                )
            ).build(),
            Notification.Action.Builder(
                null, VOICE_PREV_LABEL,
                verbPendingIntent(
                    ACTION_CYCLE_VOICE_PREV,
                    REQUEST_CYCLE_VOICE_PREV
                )
            ).build()
        )
        return when (dismissal) {
            is Dismissal.NonDismissable -> rateAndVoice
            is Dismissal.AutoDismiss -> emptyList()
            is Dismissal.Dismissable ->
                listOf(
                    Notification.Action.Builder(
                        null, RESUME_LABEL,
                        verbPendingIntent(ACTION_RESUME, REQUEST_RESUME)
                    ).build()
                ) + rateAndVoice
        }
    }

    /**
     * Builds the media surface: the document title
     * (FR-029), the section title — or, on a transient
     * surface, the stated reason, which is the short
     * lock-screen statement (contracts/shell-ipc.md,
     * FR-037/FR-041) —, the progress indication (FR-028a),
     * the handed dismissal mechanics (FR-028e), and the
     * surface's action buttons. The surface is declared
     * public so the title and section title reach the lock
     * screen (FR-028b), and it is a media-category surface
     * carrying the session token, so the platform's media
     * controls drive the session's closed transport
     * controls. Content is exactly the FR-028 series —
     * never document body text, never elapsed or remaining
     * time, never rate, never voice.
     *
     * Appearance: the surface is painted in the DESIGN-SONA
     * palette tokens (contracts/surface.md, the contract's
     * single source of truth, resolving T044). The platform's
     * one per-notification color channel carries the title
     * token (setColor, so the accent paints the transport
     * glyph and the app name high-contrast on the dark
     * lock-screen surface); the progress indication is
     * reported through the platform's progress APIs, whose
     * track and fill tokens (`#55688F` / white) the system
     * themes; the section and banner tokens are declared and
     * documented on the companion. No field the surface
     * carries is body text, time, rate or voice, and no
     * ticker, sub-text or marquee ever rides it.
     */
    private fun buildSurface(
        state: MirrorState,
        ongoing: Boolean,
        deleteIntent: PendingIntent?
    ): Notification {
        val launch = launchPendingIntent()
        val text = state.transientReason ?: state.sectionTitle
        val form = state.progress
        val icon =
            if (state.transport == TRANSPORT_PLAYING) {
                android.R.drawable.ic_media_play
            } else {
                android.R.drawable.ic_media_pause
            }
        fun configure(builder: Notification.Builder): Notification.Builder {
            val configured = builder
                .setSmallIcon(icon)
                // The one per-notification color channel the
                // platform exposes carries the DESIGN-SONA title
                // token (contracts/surface.md) — see the token
                // table on the companion.
                .setColor(DESIGN_SONA_TITLE)
                .setContentTitle(state.title)
                .setContentIntent(launch)
                .setOngoing(ongoing)
                .setVisibility(Notification.VISIBILITY_PUBLIC)
                .setCategory(Notification.CATEGORY_TRANSPORT)
                .setStyle(
                    Notification.MediaStyle()
                        .setMediaSession(mediaSession.sessionToken)
                )
                .setDeleteIntent(deleteIntent)
            if (text != null) {
                configured.setContentText(text)
            }
            when (form) {
                is Progress.Determinate ->
                    configured.setProgress(
                        (form.durationSecs * MILLIS_PER_SECOND).toInt(),
                        (form.positionSecs * MILLIS_PER_SECOND).toInt(),
                        false
                    )
                is Progress.Indeterminate ->
                    configured.setProgress(0, 0, true)
            }
            for (action in surfaceActions(state.dismissal)) {
                configured.addAction(action)
            }
            return configured
        }
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            configure(Notification.Builder(context, CHANNEL_ID)).build()
        } else {
            @Suppress("DEPRECATION")
            configure(Notification.Builder(context)).build()
        }
    }

    /**
     * Creates the media channel if the platform requires one and it
     * does not exist yet (API 26+). Public because the foreground
     * holder and the media surface share this mirror's channel and
     * id: one creation site keeps the channel's name and importance
     * a single source.
     */
    fun ensureChannel() {
        val manager = context.getSystemService(NotificationManager::class.java) ?: return
        ensureChannel(manager)
    }

    /**
     * Builds the foreground holder: the application label as the
     * title and nothing else — no body, no clock, no section — on
     * the media channel and the shared surface id, so the first
     * [publish] (or the resident post) replaces this holder instead
     * of adding a second surface (the exactly-one invariant). The
     * hosting service posts it synchronously on every command to
     * satisfy the platform's foreground-service window, which the
     * first mirror publish may arrive after. Rendering beyond the
     * label stays mirror-only, so the holder can carry nothing that
     * drifts from the published state (FR-028 series).
     */
    fun foregroundHolder(): Notification {
        ensureChannel()
        val label = context.packageManager
            .getApplicationLabel(context.applicationInfo)
        val launch = launchPendingIntent()
        fun build(builder: Notification.Builder): Notification =
            builder
                .setSmallIcon(android.R.drawable.ic_media_play)
                .setContentTitle(label)
                .setContentIntent(launch)
                .setOngoing(true)
                .setVisibility(Notification.VISIBILITY_PUBLIC)
                .build()
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            build(Notification.Builder(context, CHANNEL_ID))
        } else {
            @Suppress("DEPRECATION")
            build(Notification.Builder(context))
        }
    }

    /**
     * Creates the media notification channel if the platform
     * requires one and it does not exist yet (API 26+;
     * below the API-29 floor of SC-032 the platform
     * requires none).
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

    /** Registers the control receiver for the resume,
     * rate and voice control broadcasts. Safe to call
     * again: a second call registers nothing. */
    private fun registerControlReceiver() {
        if (registered) {
            return
        }
        val filter = IntentFilter(ACTION_RESUME)
        filter.addAction(ACTION_RATE_UP)
        filter.addAction(ACTION_RATE_DOWN)
        filter.addAction(ACTION_CYCLE_VOICE_NEXT)
        filter.addAction(ACTION_CYCLE_VOICE_PREV)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            context.registerReceiver(
                controlReceiver, filter, Context.RECEIVER_NOT_EXPORTED
            )
        } else {
            context.registerReceiver(controlReceiver, filter)
        }
        registered = true
    }

    /** Unregisters the control receiver. Safe to call
     * again: a second call unregisters nothing. */
    private fun unregisterControlReceiver() {
        if (!registered) {
            return
        }
        context.unregisterReceiver(controlReceiver)
        registered = false
    }

    /**
     * Builds the PendingIntent for one control broadcast:
     * an explicit in-app broadcast whose action classifies
     * the control at the boundary (contracts/shell-ipc.md
     * case mapping) — the Residency idiom.
     */
    private fun verbPendingIntent(action: String, requestCode: Int): PendingIntent {
        val intent = Intent(action).setPackage(context.packageName)
        return PendingIntent.getBroadcast(
            context, requestCode, intent, PendingIntent.FLAG_IMMUTABLE
        )
    }

    /**
     * Builds the tap target of the surface: the application
     * launch intent, so a tap opens the application — the
     * reader's route to choosing another document.
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
