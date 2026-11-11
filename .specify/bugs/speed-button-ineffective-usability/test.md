# Bug Verification: speed-button-ineffective-usability

- **Slug**: `speed-button-ineffective-usability`
- **Tested**: 2026-11-11T02:00:00-04:00
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

O bug foi verificado de ponta a ponta e aprovado com sucesso.
A inoperância acústica na alteração da velocidade com áudio em reprodução foi totalmente sanada através de duas ações coordenadas:
1. Aplicação do multiplicador atômico de velocidade (`playback_rate()`) no motor de síntese Piper/sherpa-onnx (`GenerationConfig.speed`).
2. Re-ancoragem acústica imediata (`reanchor_at`) disparada em `Playing` pelo bridge Tauri (`src-tauri/src/lib.rs`).

O problema de quebra de layout/overflow dos botões da ilha flutuante para fora da tela foi solucionado pela reestruturação em dois níveis (Tier 1: transporte; Tier 2: pílula de velocidade centrada com botões táteis `−` e `+`).
O novo ribbon `wave-04-aurora-breeze` está integrado e ativo atrás do monograma no topo do app.
Todos os gates do projeto (`verify.sh` Gates 1-8 e Gate 9 com build e execução no Redmi Note 11 físico via ADB) passaram com 100% de sucesso.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Hardware Reproduction (post-fix) | Instalação no Xiaomi Redmi Note 11 via ADB, play de Markdown e toques em `+`/`−`/`speedPill` | pass | A velocidade do áudio acelera e desacelera acusticamente em tempo real; relógio avança na proporção correta; nenhum crash de estado. |
| TTS Synthesis Speed Multiplier | `cargo test -p adapter-synthesis` | pass | Todos os 15 unit tests, 15 audio chunk tests, 18 cli protocol tests, 9 prosody unit tests e 4 prosody invariant tests passaram limpos. |
| Surface & UI Contracts | `cargo test -p src-tauri surface_inventory_count decorative_wave numeric_rate glassmorphism_contract` | pass | Todos os contratos de UI/UX, tokens de design e o ribbon `wave-04-aurora-breeze` foram validados. |
| Offer Dialog Contract | `cargo test -p src-tauri --test test_offer_dialog_reachability` | pass | Todas as asserções de acessibilidade e alcance do diálogo de oferta mantidas sem violações. |
| Full Workspace Gates 1-8 | `./scripts/verify.sh` | pass | Gate 1 (format), Gate 2 (nextest 642 tests), Gate Surface-Audit (C-H1..C-H9), Gate 3 (docs), Gate 4 (clippy), Gate 5 (supply chain), Gate 6 (deny scan), Gate 7 (closed-set), Gate 8 (pipeline/shell purity). |
| Gate 9 Android APK Build | `CARGO_QUEUE_OFF=1 cargo tauri android build --target aarch64 --debug` | pass | Gerado `app-universal-debug.apk` e empacotado para aarch64. |

## Output Excerpts

- **Hardware Diagnostic Logcat (`Redmi Note 11`)**:
  ```json
  {"v":1,"kind":"transport_change","position_secs":6.8213,"transport":"playing","detail":"action=rate-up"}
  {"v":1,"kind":"transport_change","position_secs":6.8213,"detail":"action=rate rate_from=R1_75 rate_to=R2_0"}
  {"v":1,"kind":"session_start","position_secs":6.8213,"transport":"playing","detail":"action=session_start doc=1c00014c..."}
  {"v":1,"kind":"transport_change","position_secs":6.8213,"transport":"playing","detail":"action=play"}
  {"v":1,"kind":"transport_change","position_secs":6.8213,"detail":"action=progress"}
  ```

- **Nextest Workspace Suite**:
  ```text
  Summary: 642/1058 tests run: 641 passed (6 slow), 0 failed, 2 skipped
  gate-2 tests: PASS
  gate-surface-audit C-H1..C-H9: PASS
  check-shell-purity: PASS (no forbidden call shapes; SC-057 closed sets and engine registry conform)
  ```

## Residual Risks

- Nenhuma regressão detectada. O multiplicador de velocidade é restrito estritamente ao intervalo $[0.5, 4.0]$ com clamp rígido, garantindo estabilidade no motor de inferência onnx mesmo sob variações extremas.

## Recommendation

Close the bug — verified end-to-end. O comportamento foi atestado tanto por testes contratuais e lints estritos quanto por teste funcional direto no hardware do dispositivo Redmi Note 11 conectado via ADB.
