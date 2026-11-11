# Bug Fix: Botão de velocidade de reprodução e usabilidade aprimorada

- **Slug**: `speed-button-ineffective-usability`
- **Fixed**: 2026-11-10T22:15:00-04:00
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

1. Corrigida a inoperância acústica da mudança de velocidade durante a reprodução ativa de áudio no motor Rust (`src-tauri/src/lib.rs`), disparando `reanchor_at` imediatamente para que a nova velocidade de síntese passe a ser audível sem necessitar de pause/play.
2. Integrado o novo ribbon visual `wave-04-aurora-breeze` atrás do monograma "R" no topo do aplicativo (`DESIGN-SONA/assets/waves/waves-top-ribbons.png`).
3. Refatorado o layout da ilha flutuante (`.bottom-island`) para uma estrutura hierárquica em dois níveis (Tier 1: controles de transporte `[↻] [⏮] [▶] [⏭]`; Tier 2: cápsula de velocidade `[−] [1.25×] [+]`), resolvendo o transbordamento horizontal para fora da tela e da área delimitada da ilha no dispositivo físico (1080px).

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/lib.rs` | modified | Dispara `reanchor_at` imediatamente ao receber `rate_up` ou `rate_down` sob `TransportState::Playing`. |
| `dist/index.html` | modified | Refatora `.bottom-island` para layout coluna/dois níveis com cápsula de velocidade estilizada, eliminando quebra de tela. |
| `src-tauri/src/status_view.html` | modified | Mantém paridade estrita com `dist/index.html`. |
| `src-tauri/src/assets/index.html` | modified | Sincronizado para empacotamento webview. |
| `src-tauri/gen/android/app/src/main/assets/index.html` | modified | Atualizado no build do Android. |
| `DESIGN-SONA/assets/waves/waves-top-ribbons.png` | modified | Atualizado com `wave-04-aurora-breeze.png`. |
| `DESIGN-SONA/assets/waves/waves-top.svg` | modified | Atualizado com `wave-04-aurora-breeze.svg`. |
| `dist/DESIGN-SONA/assets/waves/waves-top-ribbons.png` | modified | Sincronizado na distribuição frontend. |
| `src-tauri/src/assets/DESIGN-SONA/assets/waves/waves-top-ribbons.png` | modified | Sincronizado nos assets do app desktop/Android. |

## Diff Highlights

```rust
// src-tauri/src/lib.rs
if let Some(step) = rate_press_of(verb)
    && let Some(rung) = RateStepper::open(Some(previous_rate)).step(step)
{
    persist_last_rate(state, rung);
    record_rate_change(state, &snapshot, previous_rate, rung);
    // Dynamic rate change during active playback (immediate acoustic response):
    if snapshot.transport == TransportState::Playing {
        let _ = reanchor_at(state, snapshot.position_secs);
    }
}
```

```css
/* dist/index.html - Layout em dois níveis da ilha de controles */
.bottom-island {
  background: rgba(26, 35, 54, 0.78);
  backdrop-filter: blur(36px);
  -webkit-backdrop-filter: blur(36px);
  border: 1px solid rgba(255,255,255,0.18);
  box-shadow: inset 0 1px 1px rgba(255,255,255,0.22), 0 24px 50px rgba(0,0,0,0.65);
  border-radius: 36px;
  padding: 0.75rem 1.25rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  margin: 0.5rem auto 0.5rem;
  width: fit-content;
  max-width: 92%;
  flex-shrink: 0;
}
.speed-control {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.65rem;
  margin: 0;
  padding: 0.25rem 0.6rem;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 999px;
}
```

## Local Verification

- `cargo test -p src-tauri --test surface_inventory_count --test numeric_rate --test decorative_wave --test glassmorphism_contract`: **PASS** (4/4 testes verdes).
- `cargo test -p src-tauri --test test_surface_us1`: **PASS** (4/4 testes verdes).
- `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (zero lints).
- `cargo fmt --all -- --check`: **PASS**.
- `cargo tauri android build --target aarch64 --debug`: **PASS** (APK gerado com sucesso).
- Verificação em Hardware (`scripts/device-a11y-check.py` no Xiaomi Redmi Note 11, resolução 1080x2400):
  - Linha 1 (Transporte):
    - `btnRepeat`: `[217,1479][352,1617]`
    - `btnSkipBack`: `[376,1479][514,1617]`
    - `btnPlayPause`: `[539,1479][673,1617]`
    - `btnSkipForward`: `[728,1479][863,1617]`
  - Linha 2 (Velocidade compacta e perfeitamente centrada):
    - `btnRateDown`: `[332,1666][440,1776]`
    - `speedPill`: `[495,1658][613,1782]`
    - `btnRateUp`: `[638,1666][748,1776]`
  - Todos os elementos contidos com folga nas bordas da tela e da ilha flutuante (largura máxima ocupada pelos controles de velocidade = ~416px, contra os 1080px da tela).

## Deviations from Assessment

O assessment inicial previa manter todos os controles em linha única; contudo, a adição dos botões táteis dedicados de velocidade fez o conjunto exceder a largura física da tela. A solução de layout em dois níveis (transporte em cima, velocidade embaixo) resolveu completamente o problema de overflow preservando 100% da usabilidade e estética SONA.

## Follow-ups

- Executar `/speckit-bug-test slug=speed-button-ineffective-usability` para consolidar o relatório final de testes de regressão.
