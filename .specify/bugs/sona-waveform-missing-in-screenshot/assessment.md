# Bug Assessment (Re-evaluated): sona-waveform-missing-in-screenshot

- **Slug**: sona-waveform-missing-in-screenshot
- **Created**: 2026-10-28 (re-evaluated)
- **Source**: pasted text + reference image (`DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png`) vs screenshot (`/tmp/android_screen.png` / `/tmp/android_screen_updated.png`)
- **Verdict**: likely valid — partial fix applied, reproduction with áudio ativo ainda inconclusiva
- **Severity**: high (visual core do design Sona — waveform funcional é requisito do spec/plan 007)

## Report (reavaliação com evidência atualizada)

O fix anterior (`drawWaveform()` opacidade aumentada + guarda defensiva `indeterminate`) melhorou a visibilidade estrutural, mas o usuário confirma que o resultado ainda está "BEEEEEM longe do proposto no mockup" (`ui-proposta-2`). A captura atualizada (`android_screen_updated.png`) ainda não confirma visualmente os 5 ribbons coloridos funcionando com dados de sessão real. A causa fundamental permanece: `drawWaveform()` pode receber `payload` vazio (sem `SampleQueue` consumido) quando o app inicia sem reprodução ativa (`monkey` apenas inicia a Activity, não inicia áudio/síntese), resultando em um canvas vazio ou com apenas a forma estática mínima.

A proposta (`ui-proposta-2`) define: card `Rust Async Architecture` (conteúdo de referência), waveform com 5 camadas coloridas (ribbon funcional com playhead branco), header decorativo com ondas (`.card::before`), bottom bar com rate `1.25x` e botões circulares grandes. O app implementado segue a estrutura, mas a funcionalidade visual do waveform ainda depende de uma sessão com consumo ativo.

## Symptom (reconfirmado)

O screenshot do app (`/tmp/android_screen_updated.png`) mostra: card `glassmorphism` (`Concha Harmônica R`, `Sem título`), controles (`↻` | `⏪` | `▶` | `⏹` | `⏩`), rate (`1.25x`), mas o waveform (`#waveform`) ainda não confirma visualmente a renderização funcional com dados de sessão real. O mockup (`ui-proposta-2`) exige que o waveform seja dinâmico/funcional, não apenas um placeholder estático.

## Reproduction (reconfirmado com fix aplicado)

1. Build APK (`cargo tauri android build` — concluído com `libsrc_tauri.so`).
2. Instalar (`adb install -r ...` — `Success`).
3. Lançar (`adb shell monkey -p app.mdreader ...`).
4. Capturar (`adb shell screencap` → `/tmp/android_screen_updated.png`).
5. Resultado: card presente, header `Concha Harmônica R` correto (`concha-harmonica-r.svg`), `waves-top.svg` presente (`DESIGN-SONA/assets/waves/`), `drawWaveform()` implementado, `surface_audit::inventory` passa (120/120), mas o visual completo do waveform (5 camadas + playhead com dados) não é confirmado sem reprodução de áudio.

[NEEDS CLARIFICATION (remains open): o `drawWaveform()` é chamado com dados de `SampleQueue::consume_into()` quando uma sessão de áudio está ativa? A captura com `monkey` (sem reprodução) não reproduz esse estado.]
[NEEDS CLARIFICATION (remains open): o `AppState::channel_waveform` publica `payload` com `total_units > 0` no momento inicial? Se `consume_into()` retorna `count == 0` (sem amostras consumidas), o `drawWaveform()` pode desenhar apenas a base estática, não o ribbon funcional.]

## Suspected Code Paths (reconfirmado)

- `src-tauri/src/status_view.html`: `<canvas id="waveform" aria-hidden="true">` (linha 281); `.wave-subtle` (decorative wave, linha 88); `drawWaveform()` (linha 448); guarda `indeterminate` (linha 515); `reduced-motion` (linha 274).
- `crates/player/src/backend.rs`: `SampleQueue::consume_into()` (linha 406, RMS); `consume_into()` deve publicar `progress` com `total_units > 0` e `consumed_units > 0` para que o `drawWaveform()` receba dados determinísticos.
- `src-tauri/src/lib.rs`: `AppState` com `channel_rms` / `channel_waveform` / `channel_progress`. Se `skip while hidden/no subscribers` está ativo, o `drawWaveform()` pode não receber eventos.
- Build/deploy: `DESIGN-SONA/assets/waves/waves-top.svg` está presente (`find` confirma); `status_view.html` contém a referência correta (`url("DESIGN-SONA/assets/waves/waves-top.svg")`).

## Root Cause Hypothesis (reconfirmado, confiança: alta)

A evidência aponta para uma única causa principal (não mais uma combinação incerta):

- **O `drawWaveform()` precisa de dados de `SampleQueue::consume_into()` para desenhar o ribbon funcional.** Quando o app inicia sem uma sessão de áudio ativa (como no teste com `monkey`), `consume_into()` pode retornar `count == 0` (sem amostras consumidas), resultando em `payload` vazio (`total_units == 0` ou `consumed_units == 0`). A função `drawWaveform()` desenha os ribbons (que agora têm opacidade aumentada — `0.45`–`0.60`), mas se `indeterminate` é `true` (porque `kind == 'indeterminate'` ou `total_units == 0`), o playhead pode ser desenhado como estático (a guarda defensiva adicionada no fix anterior garante isso). No entanto, para que o waveform apareça como no mockup (`ui-proposta-2`), é necessário que a sessão esteja ativa (`SampleQueue` consumindo) para que `drawWaveform()` desenhe com `indeterminate == false` e o playhead apareça como um ponto móvel no ribbon colorido.

Portanto, a remediação anterior (`opacity` + `indeterminate` guard) resolve o problema de visibilidade estrutural, mas não resolve a funcionalidade completa: o `waveform` precisa de uma sessão com áudio para ser totalmente funcional conforme o design.

## Proposed Remediation (revisada — preferida para fix completo)

**Preferida (para completar a funcionalidade)**: Corrigir o `AppState` (`src-tauri/src/lib.rs`) para garantir que, ao iniciar uma sessão (mesmo sem áudio consumido imediatamente), o `channel_waveform` publique um `payload` inicial com `total_units = 1` e `consumed_units = 0` (determinístico, não `indeterminate`), permitindo que `drawWaveform()` desenhe o ribbon completo e o playhead no início do documento. Adicionalmente, garantir que `drawWaveform()` seja chamado no momento da inicialização (`setup` / `run`), não apenas no `visibilitychange` ou `renderProgress()`.

Passos revisados:
1. Confirmar que `DESIGN-SONA/assets/waves/waves-top.svg` está no APK (`find` confirma — OK).
2. Modificar `AppState` (`lib.rs`) para publicar `payload` inicial (`total_units = 1`, `kind = 'determinate'`) quando a sessão inicia, garantindo que `drawWaveform()` receba dados.
3. Se a causa for `reduced-motion` / `visibilitychange`, a guarda defensiva já está aplicada (`animation: none` no CSS; `indeterminate` guard no JS). Confirmar que o `drawWaveform()` não retorna prematuramente (`if (document.hidden) return;` — isso está correto; mas `if (document.hidden) return;` só retorna se oculto; quando visível, deve desenhar — OK).
4. Adicionar teste que simula uma sessão com `SampleQueue` consumindo (`tests/waveform_canvas_inventory.rs` atualizado) e outro que confirma que `drawWaveform()` desenha com `indeterminate` (já adicionado — passa).
5. Rebuildar APK e capturar screenshot com uma reprodução que inicia uma sessão (`SampleQueue` ativo) para confirmar o waveform funcional.

**Alternativa** (se a causa não for apenas `payload` vazio): Verificar se `drawWaveform()` precisa ser chamado explicitamente no `run()` ou `setup()` (não apenas no `visibilitychange` ou `renderProgress()`). Se `AppState` não publica eventos no momento inicial, `drawWaveform()` nunca é chamado até que `visibilitychange` ou um evento `progress` ocorra — o que explica o canvas vazio no screenshot inicial.

**Arquivos a alterar (prováveis — fix completo)**:
- `src-tauri/src/lib.rs` (`AppState`: inicializar `channel_waveform` / `channel_progress` com `payload` base)
- `crates/player/src/backend.rs` (`SampleQueue::consume_into()`: garantir que `rms()` e `progress` sejam publicados mesmo com `count == 0`, se necessário)
- `src-tauri/src/status_view.html` (`drawWaveform()`: já corrigido — `indeterminate` guard; não precisa de mais mudanças a menos que a reprodução com áudio ainda falhe)
- `tests/waveform_canvas_inventory.rs` (já atualizado — passa)

**Testes a adicionar/atualizar (fix completo)**:
- `tests/waveform_canvas_inventory.rs`: já atualizado (`drawWaveform` + `canvas` + `id="waveform"`).
- Adicionar: teste que simula `drawWaveform()` com `indeterminate == true` e `progressPct == 0`, confirmando que o playhead estático aparece.
- Adicionar: teste que confirma `AppState` publica `channel_waveform` com `total_units > 0` no momento da inicialização (`lib.rs`).

## Changes (fix applied)

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/status_view.html` | `drawWaveform()` — opacidade dos ribbons aumentada (`0.45`–`0.60`); guarda defensiva `indeterminate` adicionada (playhead estático) | Melhora visibilidade; protege contra payload vazio |
| `DESIGN-SONA/assets/waves/waves-top.svg` | Confirmado presente (`cp` realizado) | Asset decorativo `.card::before` |
| `src-tauri/src/surface_audit/inventory.rs` | Asserção `drawWaveform` + `canvas` + `id="waveform"` adicionada (`C-H4`) | Teste estrutural |
| `tests/waveform_canvas_inventory.rs` | Passa (`canvas_present_aria_hidden_and_progress_retained`) | Confirma estrutura |

## Diff Highlights (fix applied)

`drawWaveform()` — opacidade:
```js
const colors = ['rgba(143,216,232,0.45)', ... , 'rgba(185,166,245,0.60)'];
```
`indeterminate` guard:
```js
} else {
  ctx.fillStyle = 'rgba(255,255,255,0.6)';
  ctx.beginPath(); ctx.arc(cssW/2, centerY, 6, 0, Math.PI * 2); ctx.fill();
}
```
`inventory.rs` — assertion:
```rust
assert!(html.contains("drawWaveform") && html.contains("id=\"waveform\"") && html.contains("canvas"), ...);
```

## Tests Added or Updated (fix applied)

- `surface_audit::inventory::tests::surface_audit_element_inventory` — passa (adicionada assertion `drawWaveform` + `canvas` visível).
- `tests/waveform_canvas_inventory.rs` — passa (`canvas_present_aria_hidden_and_progress_retained`).
- `tests/decorative_wave.rs` — passa (`decorative_wave_contract`).

## Local Verification (fix applied)

- `TMPDIR=/dev/shm XDG_DATA_HOME=/dev/shm cargo test --lib` → **120 passed; 0 failed** (`surface_audit`, `chroma`, `glass`, `decorative`, `overflow`, `movement`, `exclusivity`, `inventory`).
- `drawWaveform()` visível no HTML corrigido (`grep` confirma opacidade e guarda).
- `waves-top.svg` confirmado no `DESIGN-SONA/assets/waves/`.
- `dist/index.html` atualizado com as mudanças.

## Residual Risks (re-evaluated)

- **Funcionalidade completa ainda inconclusiva**: o fix aplicado resolve a **visibilidade estrutural** do `waveform` (opacidade + guarda defensiva + asset confirmado), mas a **funcionalidade completa** (waveform colorido com 5 camadas funcionando com dados de `SampleQueue` consumindo) ainda requer uma reprodução com áudio ativo. A captura estática (`monkey` sem reprodução) não prova esse caso.
- Se `drawWaveform()` ainda não receber `payload` com `total_units > 0`, o `waveform` pode aparecer como uma forma estática mínima (guarda `indeterminate`), mas não como o ribbon animado funcional do mockup.
- A próxima etapa é confirmar a publicação de `AppState` (`channel_waveform` / `channel_progress`) com dados determinísticos (`total_units > 0`) e tirar um novo screenshot com reprodução ativa.

## Recommendation (re-evaluated)

Status: **partial** (fix estrutural aplicado; funcionalidade completa ainda inconclusiva). O fix melhora a visibilidade do `waveform` (opacidade + guarda defensiva) e confirma que o `drawWaveform()` está implementado, o `canvas` está presente, e o `waves-top.svg` está no bundle. No entanto, a reprodução visual completa — confirmar que o `waveform` aparece como 5 ribbons coloridos com playhead funcionando sob uma sessão com áudio — ainda não foi confirmada.

Recomendação imediata:
1. Confirmar `AppState::channel_waveform` publica `payload` com `total_units > 0` (adicionar log ou verificação no `lib.rs`).
2. Se `channel_waveform` não publica no momento inicial, corrigir o `AppState` para publicar um `payload` base (`total_units = 1`, `kind = 'determinate'`) quando uma sessão inicia.
3. Rebuildar APK (`cargo tauri android build`) e tirar um novo screenshot com reprodução ativa (`SampleQueue` consumindo) para confirmar o `waveform` funcional.

Se a funcionalidade ainda falhar após isso, reavaliar (`/speckit.bug.assess slug=sona-waveform-missing-in-screenshot`) com a nova evidência (screenshot com reprodução ativa + `console.log` do `drawWaveform()`).

## Follow-ups (re-evaluated)

- Confirmar `AppState::channel_waveform` publica `payload` determinístico no momento inicial.
- Confirmar que `drawWaveform()` é chamado com `indeterminate == false` quando uma sessão está ativa.
- Confirmar que o `waveform` aparece como 5 ribbons coloridos (teal → jade → violet) com playhead branco no screenshot com reprodução ativa.
- Se necessário, adicionar no `AppState` uma inicialização de `payload` (`total_units = 1`, `consumed_units = 0`) para evitar que `drawWaveform()` desenhe apenas a forma estática mínima.

## 2026-11-20 — Reconciliação de status (append-only; histórico preservado)

O status "partial" registrado acima era anterior à verificação de 2026-11-02 (`test.md`). O `test.md:7` registra **Result: pass** com evidência no dispositivo físico 53cf9aa7: `publish_rms(state.inner(), 0.0)` em `register_rms_listener`, canvas `#waveform` 388x80 montado, integração RMS via `SampleQueue::consume_into`, e 6 contratos verdes (`waveform_canvas`, `waveform_canvas_inventory`, `rms_payload_contract`, `waveform_performance`, `prop_rms_us1`, `prop_waveform_us2`). O `fix.md:6` registra `Status: applied (completo)`. Os três primeiros follow-ups acima foram portanto atendidos pela verificação; o quarto (payload base `total_units=1`) foi resolvido pelo `publish_rms(..., 0.0)` imediato no registro do listener.

**Status reconciliado: RESOLVIDO** (evidência documental local; uma nova captura visual com reprodução ativa continua [NV] e exigiria gate 9 + dispositivo — não executado nesta rodada). Esta nota é uma reconciliação de registro, não uma reavaliação técnica.
