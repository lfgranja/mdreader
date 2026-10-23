# DESIGN-SONA/components.md — Componentes: B base + Sona adaptado

**Versão:** 2.0.0 · **Data:** 2026-10-16 · **Status:** homologado
> Referência: `DESIGN/components.md` (B — Sinalização), `briefing-logo-ressoa-design-sona.md` (Briefing Oficial v2.0.0). Mudanças: tríade de ondas sonoras Sona (Azul, Verde, Roxo) integrada ao componente BrandLogo; ponto amarelo pulsante (aviso 3s) mantido; progresso monocromático (`#FFFFFF` fill sobre `#55688F` trilho); notificação sem custom RemoteViews e sem marca; in-app grade 2-campo com logo oficial no cabeçalho; scroll real de razão (SC-029 atendido só in-app).

---

## 1. NotificationSurface (MediaStyle — superfície real, igual ao B)

**Não muda de B** — a superfície é controlada pelo Android, não pelo design. A mudança é na **paleta aplicada**:

| Elemento | Token Sona | Uso | Regra |
|---|---|---|---|
| Título doc | `--sona-voice` (`#F2F0E6`) | Metadado / nome arquivo estável | Nenhum body text; FR-029 / SC-030 |
| Seção atual | `--sona-hush` (`#A9B8CE`) | Último heading passado | Vazio antes do 1º heading; limpa ao parar |
| Progresso | `--sona-voice` fill sobre `--sona-boundary` (`#55688F`) | Determinado sse total conhecido no início | Indeterminado = ciclo parcial; nunca estimado; nunca muda mid-session |
| Estado fluxo | `--sona-etwahl` (`#8FD8E8`) + forma barra cheia | Nenhum corpo de texto; posição indica | Não usará verde (repouso = ausência) |
| Estado caution (3s) | `--sona-etwahl` + ponto amarelo pulsante + `#FFD400` | Aviso de pausa que expira; auto-dismiss 3s | Só amarelo; nunca vermelho; nunca azul; nunca turquesa para aviso |
| Estado failure | `--sona-fail` (`#FF4D63`) + `#FFD400` não aparece | Falha / perda permanente | Só vermelho; nunca amarelo; nunca turquesa |
| Estado info | `--sona-info` (`#3D8BE8`) — opcional | Taxa / detalhes — **só in-app** (SC-027) | Nunca na notificação |

**Regra Absoluta:** O logotipo ou as ondas sonoras **NUNCA aparecem na Notificação MediaStyle ou Lock-Screen** (FR-028 / FR-029). O espaço pertence exclusivamente ao título do documento e ao transporte.

---

## 2. PlayerView (A Lente Sinestésica da Sona — Modo Padrão / Default)

**Papel Arquitetural:** Componente raiz de visualização do tocador no smartphone e desktop (`src/lib/components/player/LenteSinestesicaPlayer.svelte`).

### Especificação de Svelte 5 (Runes):
- **Estado de Transporte:** `$state<TransportState>('idle')` (`'flowing'`, `'paused'`, `'error'`).
- **Nível RMS de Áudio:** `$derived<number>(audioEngine.currentRms)` (alimenta a microfita superior de 4px).
- **Taxa Discreta:** `$state<RateRung>(1.0)` restrita estritamente ao conjunto fechado `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`.
- **Modo de Tela:** `$state<'lente' | 'cockpit'>('lente')`.

### Anatomia Visual:
1. **Fita Sinestésica de Borda Superior (4px):**
   - Altura: 4px contínuos no topo do viewport.
   - Gradiente volumétrico tricolor: Turquesa (`#8FD8E8`), Jade (`#5AD6A8`) e Violeta (`#B9A6F5`).
   - Reatividade: A opacidade e a ondulação sutil respondem ao envelope de potência RMS (22050 Hz). Totalmente desativada em repouso.
2. **Cabeçalho Minimalista:**
   - Canto superior esquerdo: Marca Primária oficial **A Concha Harmônica ("R")** em 24dp com halo suave.
   - Wordmark institucional: `ressoa` em caixa-baixa, fonte sans-serif humanista, tom marfim acústico (`#F2F0E6`).
   - Canto superior direito: Indicador técnico discreto `22050 Hz · mono` (botão de abertura do modal de telemetria) e botão de alternância para o **Modo Cockpit** (ícone HUD).
3. **Card Central de Metadados (Frosted Glass & Doppelrand):**
   - Fundo: `rgba(20, 27, 45, 0.65)` com `backdrop-filter: blur(24px)`.
   - Borda: Dupla borda *Doppelrand* (externa 1px `rgba(255,255,255,0.06)`, interna 1px `rgba(143,216,232,0.15)`).
   - Título da Obra: 22sp, peso 600, marfim acústico (`#F2F0E6`), limitada a 2 linhas com elipse.
   - Seção / Capítulo Atual: 14sp, peso 500, tom periwinkle (`#A9B8CE`), precedido pelo ícone de índice (ao tocar, abre o *OutlineEditorialDrawer*).
   - **Invariante Inviolável (`FR-028` / `SC-030`):** Nenhum parágrafo ou corpo de texto do documento é exibido.
4. **Mostrador Acústico Central (Fusão com Selo do Ressonador):**
   - Mostrador circular rebaixado com o **Selo do Ressonador Canônico** no centro.
   - Anel de preenchimento de buffer em turquesa (`#8FD8E8`) e dourado (`#E8C46A`), indicando a saúde do pre-roll de 60s.
   - Playhead com nó perolado de marfim acústico.
5. **Ilha Flutuante de Transporte (*Floating Playback Pill*):**
   - Posição: Fixa na base inferior, margem de 24dp (*Thumb Zone* ergonômica).
   - Fundo: Vidro fosco profundo com borda anodizada.
   - Controles:
     - Salto para trás: `-15s` (48×48dp).
     - Botão Central de Transporte: `Play / Pause` (64×64dp) com transição de cor semafórica (turquesa quando tocando; repouso/pausa neutro).
     - Salto para frente: `+30s` (48×48dp).
     - Seletor de Taxa: Pílula discreta exibindo a velocidade ativa em dourado (ex: `1.25x`), com toque cíclico nos 7 rungs.

---

## 3. CockpitEyesFreeView (Modo Condução & Mobilidade Eyes-Free)

**Papel Arquitetural:** Interface de alto contraste e segurança periférica veicular (`src/lib/components/player/CockpitEyesFreeView.svelte`).

- **Acionamento:** Botão HUD no cabeçalho ou gesto de pinça para fora (*pinch-to-expand*).
- **Alvos de Toque Massivos:** Todos os botões possuem dimensões mínimas de 72×72dp (excedendo a recomendação de 48dp da W3C/Android).
- **Halo Central de Fluxo (180dp):**
   - Estado Tocando: Anel espesso e luminoso em turquesa (`#8FD8E8`) pulsando com respiração lenta.
   - Centro: Botão massivo de Pause com alto contraste (fundo `#0B0F1A` recortado).
- **Aviso Temporal de Pausa (3.0s TTL / `SC-029`):**
   - Ao ocorrer pausa transitória (perda temporária de áudio focus, ligação, `BECOMING_NOISY`), o anel turquesa transmuta instantaneamente para **Âmbar Pulsante (`#FFD400`)**.
   - Cronômetro central decrescente: Texto gigante `3.0s` decrescendo em tempo real até 0.0s com barra de preenchimento radial regressiva.
   - Ao expirar os 3.0s, o aviso encerra de acordo com a especificação formal, retornando ao repouso sem auto-resume indevido.
- **Botão de Retorno:** Toque no canto superior para recolher de volta à Lente Sinestésica.

---

## 4. OutlineEditorialDrawer (Sumário Suíço & Ribbon-Marker)

**Papel Arquitetural:** Painel lateral deslizante de navegação macro e bookmarks (`src/lib/components/outline/OutlineEditorialDrawer.svelte`).

- **Estética:** Josef Müller-Brockmann / Racionalismo Suíço. Grade pura sem ornamentos supérfluos.
- **Fita Física de Marfim (*Ribbon-Marker*):**
   - Uma linha vertical de 3px em marfim acústico (`#F2F0E6`) na borda esquerda da lista de capítulos.
   - O comprimento e a posição da fita mapeiam visualmente a localização do leitor no documento completo.
   - **Regra:** Não usa porcentagem ("62%"), apenas posições estruturais concretas ("Capítulo 4 de 12").
- **Lista de Seções:**
   - Tipografia limpa com indentação hierárquica (H1, H2, H3).
   - Seção ativa destacada com traço de marfim e indicador sutil em turquesa.
   - Bookmarks persistidos automaticamente por hash BLAKE3 (edição do arquivo invalida o marcador sem falhas silenciosas).

---

## 5. TelemetryMatrixModal (Diagnóstico & Engenharia Rust)

**Papel Arquitetural:** Modal técnico de inspeção profunda do pipeline nativo de áudio (`src/lib/components/diag/TelemetryMatrixModal.svelte`).

- **Estética:** Otl Aicher DIN 1972 / German Engineering.
- **Centro:** Malha geométrica de ressonância de Chladni contendo o **Selo do Ressonador Canônico**.
- **Métricas Expostas (tempo real via canal Tauri IPC):**
   - Pipeline de Áudio: `AAudio · cpal 0.18.2 · 22050 Hz · mono f32`.
   - Fingerprint do Documento: Hash criptográfico `BLAKE3: a3f9...c812`.
   - Latência de Buffer: Tempo de entrega da fatia PCM (ex: `42 ms`).
   - Capacidade de Pre-roll: Limite estrito de 60s preenchido pelo sintetizador.
   - Registro de Eventos: Últimas 10 linhas do log diagnóstico local append-only (`JSONL`), com filtro por severidade.
- **Acionamento:** Toque na tag de amostragem no topo da tela ou atalho de teclado `Ctrl+Shift+D`.

---

## 5. Earcon — Gramática Sona (música como acesso)

Expansão do earcon do B usando a terminologia musical de Sona (`Universe:Sona`; `Sona/Audio`):

| Ação | Termo Sona | Som | Característica |
|---|---|---|---|
| Play / Resume | **Adagio** (lento, suave) | C5 → E5 ascendente, 120ms/nota, suave | Curador, não agressivo |
| Pause | **Andante** (médio, leve) | E5 → C5 descendente, 120ms | Leve |
| Skip ± | **Allegro** (rápido) | C5 → E5 + tick breve | Confirmação |
| Rate change | **Sforzando** (acento) | Tick 1000Hz / 40ms | Preciso |
| Warning 3s | **Ostinato** (motivo repetido) | Duplo tick A5, 60ms + 80ms | Contagem regressiva |
| Failure | **Sforzando** + **Crescendo** (interrompido) | D3 quadrado 200ms + breve repetição | Falha, não aviso |
| Section boundary | **Unison** (em uníssono) | Tick 880Hz / 60ms | Leve, estrutural |

**Regra:** sintetizado pelo Rust (22050 Hz mono f32); nenhum asset externo; obrigatória para todo controle enquanto dirigindo.

---

## 6. Componentes de Identidade e Ícones de Sistema

| Componente | Asset de Produção | Regra de Renderização |
|---|---|---|
| `BrandLogoCanonical` | `DESIGN-SONA/assets/logo/logo-canonical.svg` | ViewBox `0 0 100 100`. Anel marfim `#F2F0E6`, 3 ondas (Azul, Verde, Roxo) com gap de 3.5px, nó central `(50,50)`. |
| `BrandLogoPureWaves` | `DESIGN-SONA/assets/logo/logo-waves-pure.svg` | 3 fitas acústicas do *Etwahl* para avatares e rodapés institucionais. |
| `AndroidAdaptiveIcon` | `DESIGN-SONA/assets/logo/ic_launcher_monochrome.xml` | Foreground contido na Safe Zone circular de 66 dp do canvas de 108 dp; silhueta plana recortada para Material You. |

---

*Referências:* `DESIGN/components.md` (B — componente base); `DESIGN-SONA.md` §5–7 (mudanças); `briefing-logo-ressoa-design-sona.md` (Briefing Oficial v2.0.0); `spec.md` FR-018/FR-028/FR-041/SC-029/SC-035; `VERIFICATION.md` 184.

