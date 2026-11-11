# Bug Verification: speed-button-ineffective-usability

- **Slug**: `speed-button-ineffective-usability`
- **Tested**: 2026-11-10T22:16:00-04:00
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: pass

## Summary

Verificação concluída no dispositivo físico Android de referência (`adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp`, Xiaomi Redmi Note 11, MIUI Android 13, tela 1080x2400).
A inoperância acústica ao alterar a velocidade com áudio em reprodução foi corrigida através do re-anchor dinâmico imediato (`reanchor_at(state, snapshot.position_secs)`) em `src-tauri/src/lib.rs`.
O problema de quebra de layout/overflow dos botões para fora da tela e da ilha flutuante foi corrigido com a reorganização da `.bottom-island` em estrutura hierárquica de dois níveis (Tier 1: controles de transporte; Tier 2: cápsula de velocidade centrada).
O ribbon `wave-04-aurora-breeze` foi adotado com sucesso como background decorativo no topo.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Dynamic Rate Re-anchor | `src-tauri/src/lib.rs:4412` | pass | Ao pressionar `rate_up`/`rate_down` durante `TransportState::Playing`, dispara `reanchor_at` na posição exata para resposta acústica imediata. |
| Two-Tier Layout & Bounds | `python3 scripts/device-a11y-check.py` | pass | Todos os botões perfeitamente contidos dentro da largura da tela (1080px) e dentro da cápsula delimitada. |
| Element Coordinates (Device) | UIAutomator dump no Redmi Note 11 | pass | `btnRateDown` `[332,1666][440,1776]`, `speedPill` `[495,1658][613,1782]`, `btnRateUp` `[638,1666][748,1776]`. Largura do bloco de velocidade = ~416px. |
| Surface Inventory Count | `cargo test -p src-tauri --test surface_inventory_count` | pass | C-H1 a C-H9 satisfeitos; 11 elementos nomeados na árvore sem quebras contratuais. |
| Numeric Rate Contract | `cargo test -p src-tauri --test numeric_rate` | pass | Passos nominais [0.75×, 1.0×, 1.25×, 1.5×, 2.0×] validados no stepper e UI. |
| Aurora Breeze Ribbon | `cargo test -p src-tauri --test decorative_wave` | pass | `wave-04-aurora-breeze` integrado em `DESIGN-SONA/assets/waves/waves-top-ribbons.png` e empacotado. |
| Glassmorphism Contract | `cargo test -p src-tauri --test glassmorphism_contract` | pass | Tokens de background, backdrop-filter e bordas preservados na `.bottom-island`. |
| Android Build & Install | `cargo tauri android build --target aarch64 --debug` | pass | APK universal debug gerado, transmitido via ADB e ativo em primeiro plano. |

## Residual Risks

- Em documentos extremamente longos, re-anchors consecutivos e frenéticos de velocidade em fração de segundo descartam o buffer pré-sintetizado para regenerar na taxa selecionada; o debounce natural do clique e o motor offline sherpa-onnx respondem com baixa latência sem causar instabilidade no estado do player.

## Recommendation

**Pass** — Bug fechado com sucesso (`bug.test: pass`). O botão de velocidade responde tátil e acusticamente, os botões estão esteticamente integrados sem transbordamento de layout, e o ribbon `aurora-breeze` confere acabamento visual de alto padrão.
