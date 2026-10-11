package app.mdreader

import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager

/**
 * Audio-focus request mirror (FR-012, FR-013, FR-013a, FR-014).
 *
 * Builds an [AudioFocusRequest] with AUDIOFOCUS_GAIN, USAGE_MEDIA and
 * CONTENT_TYPE_SPEECH, will-pause-when-ducked behaviour and delayed
 * grant acceptance, and listens through an OnAudioFocusChangeListener
 * that forwards each platform callback to Rust as a classified
 * shell-ipc intent string (duckable_transient, silence_demanding,
 * permanent_loss, regained).
 *
 * The platform grant answer from [start] is likewise forwarded to Rust
 * through the same sink as a FocusGrant wire string (granted, deferred,
 * refused). Unrecognised platform values on either path are forwarded
 * as named diagnostic intents (focus_change_unmapped,
 * focus_grant_unmapped), never absorbed.
 *
 * Applies nothing locally: no volume change, no pause, no resume, no
 * restore. Rust classifies and decides every outcome; this class only
 * carries the request and the callback (FR-042, SC-035).
 *
 * @param audioManager platform audio service used to file the request.
 * @param forwardIntent sink receiving one wire intent string per
 * platform callback or grant answer; the caller relays it to Rust
 * uninterpreted.
 */
class FocusMirror(
    private val audioManager: AudioManager,
    private val forwardIntent: (String) -> Unit
) {

    companion object {
        /** Wire intent for a transient loss that permits lowered audio. */
        const val INTENT_DUCKABLE_TRANSIENT = "duckable_transient"

        /** Wire intent for a transient loss that demands silence. */
        const val INTENT_SILENCE_DEMANDING = "silence_demanding"

        /** Wire intent for a permanent loss. */
        const val INTENT_PERMANENT_LOSS = "permanent_loss"

        /** Wire event for focus returned. */
        const val INTENT_REGAINED = "regained"

        /** Wire value for a granted focus request (FocusGrant::Granted). */
        const val INTENT_GRANTED = "granted"

        /** Wire value for a deferred focus grant (FocusGrant::Deferred). */
        const val INTENT_DEFERRED = "deferred"

        /** Wire value for a refused focus request (FocusGrant::Refused). */
        const val INTENT_REFUSED = "refused"

        /** Diagnostic intent for an unrecognised focus-change value; never absorbed. */
        const val INTENT_FOCUS_CHANGE_UNMAPPED = "focus_change_unmapped"

        /** Diagnostic intent for an unrecognised grant-answer value; never absorbed. */
        const val INTENT_FOCUS_GRANT_UNMAPPED = "focus_grant_unmapped"
    }

    private val focusListener = AudioManager.OnAudioFocusChangeListener { focusChange ->
        forwardIntent(
            when (focusChange) {
                AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK -> INTENT_DUCKABLE_TRANSIENT
                AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> INTENT_SILENCE_DEMANDING
                AudioManager.AUDIOFOCUS_LOSS -> INTENT_PERMANENT_LOSS
                AudioManager.AUDIOFOCUS_GAIN -> INTENT_REGAINED
                else -> INTENT_FOCUS_CHANGE_UNMAPPED
            }
        )
    }

    private var currentRequest: AudioFocusRequest? = null

    /**
     * Files the focus request, forwards the platform grant answer to
     * Rust through [forwardIntent] as a FocusGrant wire string
     * (granted, deferred, refused; focus_grant_unmapped for any other
     * platform code), and returns the platform answer verbatim. The
     * answer travels to Rust uninterpreted, where the deferred and
     * refused arms are handled (FR-013b). Makes no local decision on
     * the answer.
     *
     * @return platform grant answer (granted, delayed or failed).
     */
    fun start(): Int {
        val built = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
            .setAudioAttributes(
                AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_MEDIA)
                    .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH)
                    .build()
            )
            .setWillPauseWhenDucked(true)
            .setAcceptsDelayedFocusGain(true)
            .setOnAudioFocusChangeListener(focusListener)
            .build()
        currentRequest = built
        val answer = audioManager.requestAudioFocus(built)
        forwardIntent(
            when (answer) {
                AudioManager.AUDIOFOCUS_REQUEST_GRANTED -> INTENT_GRANTED
                AudioManager.AUDIOFOCUS_REQUEST_DELAYED -> INTENT_DEFERRED
                AudioManager.AUDIOFOCUS_REQUEST_FAILED -> INTENT_REFUSED
                else -> INTENT_FOCUS_GRANT_UNMAPPED
            }
        )
        return answer
    }

    /**
     * Releases the filed request, if any. Changes nothing else: local
     * playback state is owned by Rust alone.
     */
    fun stop() {
        val filed = currentRequest
        if (filed != null) {
            audioManager.abandonAudioFocusRequest(filed)
            currentRequest = null
        }
    }
}
