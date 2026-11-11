# Bug Fix: Botão de velocidade de reprodução e usabilidade aprimorada

- **Slug**: `speed-button-ineffective-usability`
- **Fixed**: 2026-11-10T21:54:00-04:00
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Corrigida a inoperância acústica da mudança de velocidade durante a reprodução ativa de áudio no motor Rust (`src-tauri/src/lib.rs`), restaurada a visibilidade e usabilidade tátil dos botões de ajuste fino de velocidade (`btnRateDown` e `btnRateUp`) no design SONA, e integrado o novo ribbon visual `wave-04-aurora-breeze` atrás do monograma "R" no topo do aplicativo.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/lib.rs` | modified | Dispara `reanchor_at` imediatamente ao receber `rate_up` ou `rate_down` sob `TransportState::Playing`. |
| `dist/index.html` | modified | Restaura botões táteis `.speed-btn` (`38x38px`), `gap: 0.4rem`, ciclo inteligente no `#speedPill`. |
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
/* dist/index.html */
.speed-btn {
  opacity: 1;
  pointer-events: auto;
  width: 38px;
  height: 38px;
  font-size: 1.15rem;
  background: rgba(255,255,255,0.08);
  border: 1px solid rgba(255,255,255,0.12);
  color: var(--sona-voice);
  border-radius: 50%;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s, transform 0.08s;
}
```

## Local Verification

- `cargo test -p src-tauri --test surface_inventory_count --test numeric_rate --test decorative_wave --test glassmorphism_contract`: **PASS** (4/4 testes verdes).
- `cargo test -p src-tauri --test test_surface_us1`: **PASS** (4/4 testes verdes).
- `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (zero lints).
- `cargo fmt --all -- --check`: **PASS**.
- `cargo tauri android build --target aarch64 --debug`: **PASS** (APK gerado com sucesso).
- Verificação em Hardware (`scripts/device-a11y-check.py` no Xiaomi Redmi Note 11):
  - `btnRateDown` detectado com limites `[690,1573][797,1680]` e `content-desc="Diminuir velocidade"`.
  - `btnRateUp` detectado com limites `[962,1573][1069,1680]` e `content-desc="Aumentar velocidade"`.
  - `speedPill` posicionado centralmente com limites `[827,1564][948,1688]` e `content-desc="Ajustar velocidade"`.

## Deviations from Assessment

Nenhuma. A remediação seguiu exatamente a proposta preferida e incluiu a adoção do ribbon `wave-04-aurora-breeze` solicitada pelo usuário.

## Follow-ups

- Executar `/speckit-bug-test slug=speed-button-ineffective-usability` para consolidar o relatório final de testes de regressão.
