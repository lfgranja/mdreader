package app.mdreader

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.media.AudioDeviceCallback
import android.media.AudioDeviceInfo
import android.media.AudioManager
import android.os.Build

/**
 * Routing / device-removal mirror (FR-016, FR-016a, FR-017,
 * FR-017a, FR-039, FR-042; SC-035).
 *
 * Wires the two named removal mechanisms of FR-016a and
 * forwards each platform callback to Rust as a classified
 * shell-ipc routing intent, through the same [forwardIntent]
 * sink the focus mirror uses:
 *
 * - the **primary** mechanism — a receiver registered for
 *   [AudioManager.ACTION_AUDIO_BECOMING_NOISY] — forwards
 *   [INTENT_BECOMING_NOISY] on every removal it covers
 *   (FR-016a);
 * - the **secondary** mechanism — an [AudioDeviceCallback]
 *   (API 23, below the API-29 floor of SC-032) — forwards
 *   [INTENT_DEVICE_DISAPPEARED] for every output device the
 *   platform reports removed (FR-016, FR-016a) and
 *   [INTENT_DEVICE_APPEARED] for every output device the
 *   platform reports added (FR-017a), each intent carrying
 *   the device identifier derived from the platform's own
 *   identification ([AudioDeviceInfo] type and address).
 *
 * The two mechanisms are independent: each callback is
 * forwarded on its own the moment the platform delivers it,
 * neither waits for the other, and no removal passes
 * unhandled because the platform happened to deliver only
 * one of them (FR-016a). Device events are forwarded per
 * device, so a route change covering several devices reaches
 * Rust as one intent per device, each recorded Rust-side as
 * an output-device event (FR-039).
 *
 * Applies nothing locally: no pause, no transport change,
 * no position, rate or gain logic, no skip — the one-second
 * pause bound and every routing decision (pause, follow, no
 * change) are owned by the Rust routing machine (T046), and
 * the privacy class of a device is classified Rust-side
 * from its identifier (FR-017a). This class only classifies
 * the platform callback into the closed routing-intent set
 * and forwards it (FR-042, SC-035).
 *
 * @param context service context used to register and
 * unregister the becoming-noisy receiver.
 * @param audioManager platform audio service used to
 * register and unregister the audio-device callback.
 * @param forwardIntent sink receiving one wire intent
 * string per platform callback (`becoming_noisy`,
 * `{"device_appeared":{"device_id":"…"}}`,
 * `{"device_disappeared":{"device_id":"…"}}`); the caller
 * relays it to Rust uninterpreted.
 */
class RoutingMirror(
    private val context: Context,
    private val audioManager: AudioManager,
    private val forwardIntent: (String) -> Unit
) {

    /** Wire intent strings of the closed routing-intent set
     * (contracts/shell-ipc.md). */
    companion object {
        /** Wire intent for the primary removal signal (FR-016a):
         * the output is about to become noisy. */
        const val INTENT_BECOMING_NOISY = "becoming_noisy"

        /** Wire intent for an output device appearing (FR-017a). */
        const val INTENT_DEVICE_APPEARED = "device_appeared"

        /** Wire intent for an output device disappearing
         * (FR-016, FR-016a). */
        const val INTENT_DEVICE_DISAPPEARED = "device_disappeared"
    }

    /**
     * The primary mechanism (FR-016a): receives
     * `ACTION_AUDIO_BECOMING_NOISY` and forwards
     * [INTENT_BECOMING_NOISY] to Rust. Only the noisy action
     * is accepted — no other action belongs to the routing
     * vocabulary, so an unmatched action is never forwarded.
     */
    private val becomingNoisyReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            if (AudioManager.ACTION_AUDIO_BECOMING_NOISY == intent.action) {
                forwardIntent(INTENT_BECOMING_NOISY)
            }
        }
    }

    /**
     * The secondary mechanism (FR-016a) and the appearance
     * side of FR-017a: each output device the platform adds
     * or removes is forwarded per device, independently, as
     * its own [INTENT_DEVICE_APPEARED] /
     * [INTENT_DEVICE_DISAPPEARED] intent. Only sink devices
     * are routed — the routing vocabulary carries output
     * devices only, so a source-only device is not an
     * output-route change and is not forwarded.
     */
    private val deviceCallback = object : AudioDeviceCallback() {
        override fun onAudioDevicesAdded(devices: Array<AudioDeviceInfo>) {
            for (device in devices) {
                if (device.isSink) {
                    forwardIntent(
                        deviceIntent(INTENT_DEVICE_APPEARED, deviceIdOf(device))
                    )
                }
            }
        }

        override fun onAudioDevicesRemoved(devices: Array<AudioDeviceInfo>) {
            for (device in devices) {
                if (device.isSink) {
                    forwardIntent(
                        deviceIntent(
                            INTENT_DEVICE_DISAPPEARED,
                            deviceIdOf(device)
                        )
                    )
                }
            }
        }
    }

    /** Whether both platform signals are currently registered. */
    private var registered = false

    /**
     * Registers both FR-016a mechanisms with the platform:
     * the becoming-noisy receiver (primary) and the
     * audio-device callback (secondary, plus appearances per
     * FR-017a). Device callbacks are delivered on the main
     * thread (null handler) and each is forwarded
     * independently to Rust through [forwardIntent]. Safe to
     * call again: a second call re-registers nothing.
     */
    fun start() {
        if (registered) {
            return
        }
        val noisyFilter = IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            context.registerReceiver(
                becomingNoisyReceiver, noisyFilter, Context.RECEIVER_NOT_EXPORTED
            )
        } else {
            context.registerReceiver(becomingNoisyReceiver, noisyFilter)
        }
        audioManager.registerAudioDeviceCallback(deviceCallback, null)
        registered = true
    }

    /**
     * Unregisters both mechanisms, in reverse registration
     * order. After [stop] no platform callback is forwarded
     * until [start] runs again. Unregisters nothing else and
     * applies nothing locally — playback state remains owned
     * by Rust alone (FR-042, SC-035). Safe to call again: a
     * second call unregisters nothing.
     */
    fun stop() {
        if (!registered) {
            return
        }
        audioManager.unregisterAudioDeviceCallback(deviceCallback)
        context.unregisterReceiver(becomingNoisyReceiver)
        registered = false
    }

    /**
     * Derives the identifier of an output device from the
     * platform's own identification — the [AudioDeviceInfo]
     * type, joined with the hardware address when the
     * platform reports one (Bluetooth routes) and the type
     * alone when it does not (wired routes). The identifier
     * travels with the appeared/disappeared intents; the
     * privacy class is classified Rust-side from it
     * (FR-017a).
     */
    private fun deviceIdOf(device: AudioDeviceInfo): String {
        val address = device.address
        return if (address.isEmpty()) {
            device.type.toString()
        } else {
            "${device.type}:$address"
        }
    }

    /**
     * Builds the wire intent string for one output-device
     * event: the intent tag as the JSON key and the device
     * identifier as its payload — the shell-ipc routing
     * intent shape (`{"device_appeared":{"device_id":"…"}}`).
     * The privacy class is deliberately absent: Rust
     * classifies it from the identifier (FR-017a).
     */
    private fun deviceIntent(tag: String, deviceId: String): String =
        "{\"$tag\":{\"device_id\":\"$deviceId\"}}"
}
