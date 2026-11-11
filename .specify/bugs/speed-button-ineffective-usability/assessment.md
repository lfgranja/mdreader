# Bug Assessment: Botão de velocidade de reprodução não muda nada e usabilidade deficiente

- **Slug**: `speed-button-ineffective-usability`
- **Created**: 2026-11-10T21:28:00-04:00
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

> "Botão de velocidade de reprodução não muda nada, não afeta em nada a reprodução, além da usabilidade dele estar bastante ruim."

## Symptom

1. **Inoperância Acústica da Velocidade:** Ao clicar no controle de velocidade durante a reprodução ativa de áudio (ou antes dela), a velocidade da voz neural não é alterada perceptivelmente no áudio emitido em hardware real, ou apenas persiste o valor no `redb` sem re-ancorar/alterar a taxa de playback no stream ativo de áudio.
2. **Deficiência de Usabilidade (UI/UX):**
   - Os botões de ajuste fino `btnRateDown` (`−`) e `btnRateUp` (`+`) e o indicador de passos `speedDots` estão ocultados com `opacity: 0; pointer-events: none; width: 0; height: 0;` no CSS do `dist/index.html` e `src-tauri/src/status_view.html`.
   - O único elemento visível para controle de taxa é a pílula `#speedPill` (`1.25×`), que ao ser clicada tenta alternar ciclicamente a taxa (`(rateIndex + 1) % LADDER.length`), porém com lógica inconsistente: `sendAction(rateIndex === 0 ? 'rate_down' : 'rate_up')`, falhando em navegar decrescente e com área de toque subótima.
   - O feedback sonoro (`EarconKind::RateStep`) e a confirmação visual no HUD não comunicam claramente o estado selecionado ao usuário.

## Reproduction

1. Abrir o aplicativo Ressoa e iniciar a leitura de um documento ou diretório Markdown.
2. Com o áudio tocando, clicar na pílula de velocidade (`speedPill`).
3. Observar que:
   - Apenas `rate_up` é acionado consecutivamente até saturar em `4.0x`, sem permitir redução direta ou reversão rápida.
   - Os botões `+` e `−` não existem visualmente para o usuário.
   - Se o áudio já estiver no buffer pré-sintetizado, a velocidade do buffer em reprodução não se ajusta dinamicamente ou não invalida/re-sintetiza o trecho com a nova taxa.

## Suspected Code Paths

- `dist/index.html:1243-1251` / `src-tauri/src/status_view.html:1243-1251`: lógica do clique em `speedPill` que apenas despacha `rateIndex === 0 ? 'rate_down' : 'rate_up'`, causando comportamento cíclico incorreto e confuso.
- `dist/index.html:314-331`: regras de CSS que tornam `speed-btn` (`btnRateDown` e `btnRateUp`) e `speed-dots` invisíveis (`opacity: 0`, `width: 0`), prejudicando severamente a usabilidade e a acessibilidade.
- `src-tauri/src/lib.rs:4412-4428`: no handler de `after_execute`, comandos `rate_up` e `rate_down` persistem a taxa em `persist_last_rate` e registram no log, mas diferentemente do `skip`, **não re-ancoram nem reiniciam o stream** de áudio quando em `TransportState::Playing`, deixando o buffer pré-sintetizado na velocidade antiga tocando até o final do segmento/documento.
- `crates/player/src/rate.rs`: máquina de estados da escada `PlaybackRate` (`R0_75` a `R4_0`).

## Root Cause Hypothesis

**Confiança: Alta.**
Há duas causas interligadas:
1. **Lado Backend (Motor Rust):** Quando o usuário aciona `rate_up` ou `rate_down` durante `Playing`, `src-tauri/src/lib.rs` atualiza o estado interno e grava no banco `redb`, mas **não aplica a nova taxa ao áudio corrente**. Como o pipeline de áudio já tem buffers de PCM sintetizados na taxa anterior, o usuário continua ouvindo na velocidade antiga até que uma nova síntese ocorra, dando a nítida impressão de que "o botão não muda nada".
2. **Lado Frontend (Usabilidade & Design):** A interface ocultou os controles intuitivos `+` e `−` e agrupou toda a interação em um único clique no elemento `#speedPill`, forçando um ciclo unidirecional confuso (`rateIndex = (rateIndex + 1) % LADDER.length`) com despacho inconsistente de ações para o backend Tauri.

## Proposed Remediation

**Preferred**:
1. **Frontend (Usabilidade SONA):**
   - Restaurar a visibilidade e usabilidade tátil dos botões `btnRateDown` (`−`) e `btnRateUp` (`+`) dentro da ilha inferior com dimensões adequadas (alvo de toque WCAG ≥ 48x48px), permitindo incremento e decremento diretos e previsíveis.
   - Tornar o clique na pílula `#speedPill` um alternador rápido de velocidades comuns (ex: ciclo entre `1.0x -> 1.25x -> 1.5x -> 2.0x -> 1.0x`) ou abrir um menu/popover tátil rápido com as opções da escada.
2. **Backend (Ajuste Imediato de Áudio em Playback):**
   - Em `src-tauri/src/lib.rs`, quando um `rate_up` ou `rate_down` for executado com sucesso e o transporte estiver em `TransportState::Playing`, invocar a invalidação e re-ancoragem imediata (`reanchor_at(state, snapshot.position_secs)`) ou ajuste dinâmico do relógio/stream, garantindo que o ouvinte perceba a mudança de velocidade instantaneamente no áudio.

**Files likely to change**:
- `dist/index.html`
- `src-tauri/src/status_view.html`
- `src-tauri/gen/android/app/src/main/assets/index.html`
- `src-tauri/src/lib.rs`
- `src-tauri/tests/test_surface_us1.rs` (ou teste específico de rate change em reprodução)

**Tests to add or update**:
- Teste de integração em `src-tauri` garantindo que a emissão de `rate_up`/`rate_down` durante `Playing` dispara o recálculo do áudio na nova taxa sem congelamento do transporte.
- Teste de regressão visual/DOM comprovando que os controles de velocidade estão interativos e visíveis com `aria-label` correto.

## Risks & Considerations

- **Custo Computacional da Re-síntese:** Re-ancorar na mudança de taxa exige sintetizar o novo segmento na nova velocidade; graças à **Spec 017** (`AudioChunkCache` indexado por `(fingerprint, rate)`), trechos já sintetizados na nova taxa carregarão instantaneamente (<50ms).
- **Invariante de Contraste C-H:** Qualquer ajuste de CSS na ilha de controles deve respeitar os testes do Surface Audit (`C-H1` a `C-H9`) e os contrastes mínimos do design SONA.

## Open Questions

- Nenhuma impeditiva. O fluxo segue para correção e testes automatizados.
