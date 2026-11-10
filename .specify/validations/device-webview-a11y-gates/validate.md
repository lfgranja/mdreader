# Validation: device-webview-a11y-gates (Ressoa/mdreader)

- **Date**: 2026-11-09
- **Device**: 2201117TY / `192.168.100.102:39217` (adb), package `app.mdreader` debug
- **Status**: partially verified (observational where privileged automation unavailable)

## Evidence

| Gate/check | Result | Evidence |
|---|---|---|
| Android debug backend/app functional (user-confirmed) | pass (user report) | User confirmed áudio/TTS funciona abrindo arquivo no Ressoa; not re-measured acoustically here. |
| Frontend module syntax (regression guard for picker `SyntaxError`) | pass | `scripts/check-frontend-module-syntax.sh dist/index.html` → OK. |
| Device WebView/CDP smoke | pass | `scripts/device-webview-smoke.py 192.168.100.102:39217` → ALL PASS (`no_runtime_exception_on_load`, `title_click_opens_documentsui` → `com.android.documentsui...PickActivity`, `tauri_bridge_present`, `no_fatal_js_logcat`, CDP `a11y_tree_available` 36 nodes, `title_card_a11y_label`). Note: module-scoped JS functions are not globals; behavior is the contract. |
| Accessibility tree/labels (WebView + UIAutomator) | pass (observational) | `scripts/device-a11y-check.py 192.168.100.102:39217` → `cdp_a11y_tree_nonempty`, `title_picker_name_present`, `uiautomator_dump_ok`; controls expose `content-desc` (Tocar/pausar, Reiniciar, Parar e limpar posição, Avançar/Retroceder, Ajustar/Diminuir/Aumentar velocidade). |
| TalkBack automation enabled/exercised | **blocked — needs explicit authorization/capability** | `enabled_accessibility_services` has no TalkBack; `cmd accessibility` available but `settings put secure ...` returned `Bad arguments` / secure-settings restriction observed. Script does **not** change secure settings. Requires explicit user/device authorization or an authorized automation path (e.g. device owner/profile/test harness) to enable and automate TalkBack gestures/focus. |
| Host Rust workspace tests (`cargo nextest --workspace`) | **blocked in this host config (partial progress)** | Initial failures: missing `clang` linker (`.cargo/config.toml` forced clang; host has gcc) and `MirrorState.synthesizing` missing in `player`/`src-tauri` tests — fixed (`player/src/surface.rs` serde-skip false, test constructors, `frontend_mirror` call sites). `cargo test -p player --test test_earcon_mirror` (8 pass), `--test test_shell_ipc_us4` (5 pass), `-p src-tauri --test test_progress_label` (4 pass) with `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=gcc`. Full workspace run interrupted/slow in session copies; rerun with `linker=gcc` and capture authoritative result. |
| Picker bug `title-open-picker-device-102` | verified | `.specify/bugs/title-open-picker-device-102/{assessment,fix,test}.md`, commit-ready source+assets synced, APK rebuilt and device-verified. |

## Open blockers (cannot be completed without explicit authorization/capability)

1. **TalkBack automation**: enablement/toggle and full TalkBack gesture automation on this non-rooted/user device restricted by secure-settings permissions — needs explicit authorization or supported test harness.
2. **“100% funcionalidades implementadas pelo Ressoa com 100% cobertura de testes automatizada passando”**: requires authoritative feature inventory + complete automated suite across host/unit/integration/device/E2E/audio/accessibility and all gates green; not fully evidenced in this session. Current evidence is partial and targeted.

## Artifacts

- `scripts/check-frontend-module-syntax.sh`
- `scripts/device-webview-smoke.py`
- `scripts/device-a11y-check.py`
- `.specify/bugs/title-open-picker-device-102/*`
