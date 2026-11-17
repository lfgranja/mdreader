# mdreader (Ressa)

Android media-shell TTS audiobook reader. Rust owns synthesis + PCM playback, position, buffering, and transport as the sole authority; Kotlin surface is a mirror only (SC-035); Tauri 2.12.1 is one consumer of the Rust crates.

## Design system

DESIGN-RESSOA v2.2.0 — see `DESIGN-RESSOA/CHANGELOG.md` and `DESIGN-RESSOA/VERSION`. The Ressonador Canônico seal lives at `DESIGN-RESSOA/assets/logo/seal-engine-ressonador.svg` (host-facing only; never on notification/lock-screen/in-app surfaces).

## Licença

MIT License. Copyright (c) 2026 MDMV - Laboratório de Soluções Digitais. Veja `LICENSE`.
