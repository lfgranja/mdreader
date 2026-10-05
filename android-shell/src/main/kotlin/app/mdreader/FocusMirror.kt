package app.mdreader

/** Audio-focus request mirror (FR-013 / FR-013a / FR-014 / FR-017).
 * Classifies interrupter request; forwards to Rust. No pause/duck logic in Kotlin.
 */
class FocusMirror {
    // Forbidden shapes excluded by design (T013 / T081): never `.position =` / `.rate =`
}
