# mdreader (Ressa)

Android media-shell TTS audiobook reader. Rust owns synthesis + PCM playback, position, buffering, and transport as the sole authority; Kotlin surface is a mirror only (SC-035); Tauri 2.12.1 is one consumer of the Rust crates.

## Design system

DESIGN-SONA v2.2.0 — see `DESIGN-SONA/CHANGELOG.md` and `DESIGN-SONA/VERSION`. The Ressonador Canônico seal lives at `DESIGN-SONA/assets/logo/seal-engine-ressonador.svg` (host-facing only; never on notification/lock-screen/in-app surfaces).