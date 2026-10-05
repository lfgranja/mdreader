package app.mdreader

/** Routing / device-removal mirror (FR-016 / FR-016a / FR-027).
 * Forwards `onAudioDevicesRemoved` + `BECOMING_NOISY` to Rust; holds ≤1 s pause bound.
 */
class RoutingMirror {
}
