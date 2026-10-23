# DESIGN-SONA.md — Design System Mestre: Sona (Influência / Inspirado pela personagem)

**Versão:** 2.0.0 · **Data:** 2026-10-16 · **Status:** especificado e homologado (tríade de ondas sonoras Sona: Azul, Verde e Roxo)
**Referência:** `DESIGN.md` (B — Sinalização), `DESIGN_SYSTEMS_5_CANDIDATOS.md` §B, `RELATORIO_BRAND_PRODUTO.md` (escolha de B, marca ressona.dev, Engine Sona interno), `briefing-logo-ressoa-design-sona.md` (Briefing Oficial v2.0.0)
**Origem:** Pesquisa profunda sobre Sona (`leagueoflegends.com/en-us/champions/sona/`, `wiki.leagueoflegends.com/en-us/Universe:Sona`, `Sona/Trivia`, `Sona/Audio`, `Sona/Development`, `brand.riotgames.com`, `support.riotgames.com/en-us/league-of-legends/gameplay/colorblind-mode/`)

---

## 1. O que é e o que não é

**É:** um design system inspirado na personagem Sona de League of Legends — não uma cópia de LoL, não uma identidade de jogo, não uma marca de entretenimento. A inspiração é a **personalidade** (música, cura, harmonia, leveza, mutismo como dignidade, synesthesia como licença para som↔visual), aplicada ao produto mdreader.

**Não é:**
- Nenhum conteúdo do documento aparece na interface (FR-028; SC-030 — body text proibido).
- Nenhuma animação de preenchimento de progresso (posição é autoridade do Rust).
- Nenhum marquee de texto na notificação (SC-029 — impossível; solução = earcon + razão estática + TTL 3s — declarado honesto).
- Nenhum uso de cor como único canal (SC-008 — eyes-free; earcon obrigatório; forma + posição + texto + cor redundantes).
- Nenhum "Sona Reader" como marca pública — a marca é `ressona.dev`; "Sona" é apenas o motor interno (`Engine Sona`).

---

## 2. Candidato escolhido e a mudança de direção

**Base técnico:** B — Sinalização (CTB / DIN 1451 / Semáforo), escolhido pelo usuário e documentado no `RELATORIO_BRAND_PRODUTO.md`. A arquitetura de tokens, componentes, acessibilidade e riscos do B são a **base** deste diretório.

**Mudança de direção (v2.0.0):** em vez de disciplinar severamente o semáforo na marca institucional, esta versão incorpora a **tríade sinestésica de ondas sonoras** da Sona (as 3 cordas acústicas do *Etwahl*):
- **Camada de Marca (Logo, Ícone do App, Landing, F-Droid):** Curvas sonoras coloridas em três registros acústicos: **Azul Turquesa (`#8FD8E8`)**, **Verde Jade (`#5AD6A8`)** e **Roxo Ametista (`#B9A6F5`)**, contidas por um ressonador marfim (`#F2F0E6`) sobre fundo OLED escuro (`#0B0F1A`).
- **Camada Operacional (Player / Notificação MediaStyle):** Mantém a severidade semafórica de B: o **turquesa** indica fluxo ativo; o **amarelo (`#FFD400`)** é exclusivo para aviso de pausa (3s); o **vermelho (`#FF4D63`)** é exclusivo para falha permanente; o **verde é PROIBIDO na interface operacional do player** para não confundir com botões de trânsito.
- O **earcon é expandido** com a terminologia musical de Sona (Adagio, Andante, Allegro, Sforzando, Ostinato, Crescendo, Aria, Unison) — uma gramática sonora rica gerada em Rust a 22050 Hz.
- O **in-app StatusView** ganha o logotipo oficial com as três ondas sonoras no topo e uma **faixa musical sutil** (3–5 px) que expressa o nível de RMS da voz sem competir com a barra de progresso.

---

## 3. Paleta Sona (adaptada para dark/OLED, contraste AA/AAA)

| Token | Hex | Papel | Contraste s/ `#0B0F1A` | Notas / Fonte |
|---|---|---|---|---|
| `--sona-void` | `#0B0F1A` | Fundo base (OLED) | — | Derivado do azul escuro de Sona, adaptado para preto com leve sombra azul |
| `--sona-stage` | `#141B2D` | Superfície elevada (cards, container do logo) | — | Um passo acima do void, mesmo tom |
| `--sona-voice` | `#F2F0E6` | Tinta principal (texto, anel de contenção da logo) | **16.75:1 (AAA)** | Quase branco, marfim acústico — legibilidade máxima |
| `--sona-hush` | `#A9B8CE` | Texto secundário (seção, dim) | **9.51:1 (AAA)** | Periwinkle do azul da família documentada |
| `--sona-etwahl` | `#8FD8E8` | **Acento de Marca (Onda 1) & Fluxo** — clareza, foco, fluxo ativo | **11.98:1 (AAA)** | Turquesa de Sona (*Hymn of Valor*); foco da identidade |
| `--sona-aria` | `#5AD6A8` | **Acento de Marca (Onda 2)** — cura, ressonância, voz do texto | **11.20:1 (AAA)** | Verde Jade / Menta Acústica (*Aria of Perseverance*); marca apenas |
| `--sona-celeste` | `#B9A6F5` | **Acento de Marca (Onda 3)** — ritmo, taxa, velocidade de leitura | **8.96:1 (AAA)** | Roxo Ametista / Violeta (*Song of Celerity*); nunca para falha |
| `--sona-harp` | `#E8C46A` | **Acento Secundário In-App** — taxa ativa, ênfase harmônica | **11.42:1 (AAA)** | Ouro de Sona (`#c9b373` adaptado); usado com moderação |
| `--sona-aurora` | `#3D2B8A` | Sombra profunda (não usado como texto; elevação) | — | Azul profundo decorativo |
| `--sona-boundary` | `#55688F` | Hairline / dividers / trilho de progresso | **3.43:1** | Limiar 3:1 não-texto; marginal |

**Nota de contraste:** todos os textos e ondas coloridas passam com folga o critério WCAG AAA (≥ 7:1) sobre `#0B0F1A`.

---

## 4. Regra do conjunto fechado (Separação Marca vs. Operação)

A disciplina de B (cor como função, conjunto fechado) é preservada através de uma clara separação arquitetural de escopos:

1. **Escopo da Identidade da Marca (Logo, Ícone, Web, F-Droid):**
   - Utiliza estritamente a **tríade das ondas sonoras**: Azul Turquesa (`#8FD8E8`), Verde Jade (`#5AD6A8`) e Roxo Ametista (`#B9A6F5`), emoldurados por traços marfim (`#F2F0E6`) sobre fundo `#0B0F1A`.
2. **Escopo da Interface Operacional (Player, Notificação, StatusView):**
   - **Turquesa (`#8FD8E8`):** Progresso ativo e estado de fluxo. Nunca para falha ou aviso.
   - **Dourado (`#E8C46A`):** Taxa ativa e destaque harmônico de seção.
   - **Violeta (`#B9A6F5`):** Indicador de velocidade de leitura (RateStepper).
   - **Amarelo (`#FFD400`):** **Exclusivo** para aviso de pausa transitória (3 s). Nunca no logotipo institucional.
   - **Vermelho (`#FF4D63`):** **Exclusivo** para falha crítica permanente. Nunca no logotipo.
   - **Verde (`#5AD6A8`):** **PROIBIDO no semáforo do player** (repouso é ausência de cor, não verde). Exclusivo da segunda onda da logo institucional.
   - **Branco / Marfim (`#F2F0E6`):** Texto primário e posição.
   - **Dim (`#A9B8CE`):** Metadados e seção secundária.

**Regra de expansão:** Qualquer nova cor exige edição coordenada, justificativa semiótica e bump de versão (§VII da constituição). A paleta atual cobre com perfeição a identidade e a telemetria do sistema.

---

## 5. Componentes (referência a DESIGN/components.md, adaptados)

**Notificação MediaStyle (superfície real):**
- Título do documento (`#F2F0E6` sobre `#0A0F1A`).
- Seção atual (`#A9B8CE`).
- Progresso (`#FFFFFF` fill sobre `#55688F` trilho — trilho é mais visível com a paleta clara).
- Estado de fluxo = barra cheia, posição indicativa.
- Estado de aviso (3s) = ponto amarelo pulsante + razão estática + earcon de aviso (tick A5 duplo).
- Estado de falha = ponto vermelho + razão estática + earcon de falha (D3 quadrado).
- Sem marquee; sem body text; sem taxa; sem voz.

**In-app StatusView (grade 2-campo):**
- Campo esquerdo: título + seção + progresso (barra turquesa `#8FD8E8` quando ativo, monocromático quando neutro).
- Campo direito: taxa (dourado `#E8C46A` quando ativa) + transport + razão de pausa (scroll real — SC-029 atendido) + ponto amarelo pulsante.
- Grade 4-dp M3, margens 16dp, gutter 24dp.

---

## 6. Acessibilidade / Eyes-Free / Muda (Sona como referência)

O design B já tinha o earcon (C5→E5 ascendente / descendente, tick A5, D3 quadrado, 880Hz fronteira). O design Sona **expande** isso usando a terminologia musical de Sona como a **gramática sonora do sistema**:

| Ação | Termo Sona | Som (Rust 22050 Hz mono f32) | Característica |
|---|---|---|---|
| Play / Resume | **Adagio** (lento, suavemente) | C5→E5 ascendente, 120ms nota, suave | Não agressivo; curador |
| Pause | **Andante** (médio, leve) | E5→C5 descendente, 120ms | Leve, não abrupto |
| Skip ± | **Allegro** (rápido, alegre) | C5→E5 + tick 40ms breve | Confirmação de salto |
| Rate change (1 rung) | **Sforzando** (com força, acento) | Tick 1000Hz / 40ms — acento, não explosão | Preciso, não alarmante |
| Warning — 3 s | **Ostinato** (motivo repetido) | Duplo tick A5, 60ms + 80ms — repetição de contagem | Semáforo de pedestre; não pânico |
| Failure / Permanent | **Sforzando** (força, mas curto) + **Crescendo** interrompido | D3 quadrado 200ms + repetição curta | Claramente falha, não aviso |
| Section boundary | **Unison** (em uníssono) | Tick 880Hz / 60ms | Leve, estrutural |

**Acessibilidade adicional inspirada na Sona (muda):**
- **Sign language / visual gesture**: como Sona usa sinais personalizados com sua família, o design pode incluir **gestos de toque** (swipe de 3 dedos para pausa — não exige olhar) como uma "linguagem de sinais" do usuário com o sistema. Isso é uma extensão, não uma substituição, do earcon.
- **Synesthesia** (`Universe:Sona`): a música é visível. No in-app, a **faixa de envelope** (3–5px, cor turquesa suave) pode ser considerada uma "visualização da voz" — não é necessária, mas é a licença da personagem para uma camada visual de som. Se implementada, é decorativa, não informativa.
- **Mute = dignidade:** a personagem é muda. O sistema pode funcionar sem qualquer som (se o usuário desativar o earcon) — a cor + forma + posição + texto continuam operáveis. **Não há dependência sonora** — a acessibilidade é redundante, não substitutiva.

---

## 7. Marca — Sistema Oficial de Dupla Assinatura (v2.1.0)

O **Ressoa** adota formalmente o **Sistema de Dupla Assinatura (Dual-Mark Architecture)**, consagrando os dois primeiros lugares da avaliação multicritério:

1. **Marca Primária & Ícone do App: A Concha Harmônica (Monograma "R")**
   - **Aplicação:** Ícone oficial do aplicativo no launcher Android, Google Play Store / F-Droid (1024px), favicon web (16px/32px) e avatar institucional.
   - **Assets:** [`DESIGN-SONA/assets/logo/logo-primary-concha-r.png`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/logo-primary-concha-r.png), [`logo-primary-concha-r.svg`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/logo-primary-concha-r.svg), [`ic_launcher_foreground.xml`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/ic_launcher_foreground.xml), [`ic_launcher_background.xml`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/ic_launcher_background.xml) e [`ic_launcher_monochrome.xml`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/ic_launcher_monochrome.xml).
   - **Conceito:** A haste vertical de leitura (marfim `#F2F0E6`) funde-se às 3 ondas sinestésicas da Sona (Azul `#8FD8E8`, Verde `#5AD6A8`, Roxo `#B9A6F5`), formando simultaneamente uma cóclea auditiva e a letra "R" de Ressoa com o nó central perolado.
   - **Legibilidade:** Reconhecimento cognitivo instantâneo (< 50ms) e legibilidade máxima em 16px.

2. **Selo de Engenharia: O Ressonador Canônico (Lente Semafórica)**
   - **Aplicação:** Emblema oficial da "Engine Sona" (o motor Rust de áudio), cabeçalho do `crates/diag`, tela de StatusView in-app, medidor circular de reprodução/buffer e carimbo de gravação em hardware.
   - **Assets:** [`DESIGN-SONA/assets/logo/seal-engine-ressonador.png`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/seal-engine-ressonador.png) e [`seal-engine-ressonador.svg`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/seal-engine-ressonador.svg).
   - **Conceito:** Chassi cilíndrico usinado em titânio fosco com fendas polares abertas e 3 ondas senoidais de Fourier convergindo no nó focal de ressonância.
   - **Sinalética:** Alinhamento concêntrico absoluto na Safe Zone de 66 dp do Android.

- **Wordmark Oficial:** "Ressoa" (ou "ressoa.dev") em tipografia geométrica humanista, marfim acústico `#F2F0E6`.
- **Engine Sona:** Nome do motor interno em Rust (`crates/engine-core`); confinado ao código e à telemetria de engenharia. O usuário final identifica o aplicativo como "Ressoa".

---

## 8. Riscos específicos do design Sona (adicionais ao R2–R8 do B)

- **R9 — Confusão com LoL:** Se o usuário associar "Sona" ao jogo por experiência prévia, a marca poderia ser percebida como jogo. *Mitigação:* A marca pública é `ressona.dev`, não `Sona`; o logotipo utiliza geometria pura de ondas de Fourier e sinalética da HfG Ulm/DIN, sem estética de fantasia; o engine fica restrito ao código Rust.
- **R10 — Paleta de Ondas e Separação Visual:** O uso de três ondas coloridas poderia causar perda de contraste ou fusão em telas de baixa densidade. *Mitigação:* Todas as três cores passam com folga o critério WCAG AAA (≥ 8.9:1) sobre `#0B0F1A`; a distância física de 3.5px (*negative gap*) garante legibilidade perfeita até 16×16 px.
- **R11 — Separação Marca vs. Semáforo do Player:** O verde na marca poderia induzir o uso indevido de verde na interface do player. *Mitigação:* `--sona-aria` (`#5AD6A8`) é formalmente confinado à camada de identidade da marca (Onda 2); o semáforo operacional do player continua proibindo estritamente a cor verde (repouso = ausência de cor; fluxo = turquesa).
- **R12 — Dependência de som vs. Dignidade do Mutismo:** A personagem é musical, mas o produto opera com total independência visual se o som for silenciado. *Mitigação:* Todos os estados têm redundância quadrupla (forma + posição + texto + cor); o earcon é de suporte, nunca barreira de acesso.

---

## 9. Arquitetura de Interface UI/UX Oficial — Composição Multi-Modo Integrada

**Decisão Homologada pelo Usuário (2026-10-17):** Adoção da **Composição Multi-Modo Integrada**, elegendo **"A Lente Sinestésica da Sona" (Proposta 02)** como a interface **Padrão (*Default Player View*)**, orquestrando as forças das demais propostas para atender integralmente aos requisitos de acessibilidade, condução veicular, navegação estrutural e diagnóstico técnico.

### Distribuição Funcional dos 4 Modos Integrados:

1. **Modo Padrão / Player Principal (Default): *A Lente Sinestésica da Sona* (Proposta 02)**
   - **Papel:** Tela inicial e principal de audição do aplicativo no smartphone e desktop.
   - **Elementos:**
     - Cabeçalho minimalista com a **Concha Harmônica ("R")** luminescente e wordmark `ressoa` em marfim acústico (`#F2F0E6`).
     - **Microfita de 4px** ondulando na borda superior da janela, reagindo com sutileza ao envelope RMS da voz sintetizada em tempo real (transição suave entre turquesa, jade e violeta).
     - Cards em vidro fosco (*frosted glass*) com efeito *Doppelrand* (dupla borda fina de 1px com micro-chanfro) e desfoque profundo de fundo.
     - **Metadados:** Título da obra (`#F2F0E6`) e Seção atual (`#A9B8CE`) — **Zero Body Text** inviolável (`FR-028` / `SC-030`).
     - **Centro da Tela:** Mostrador acústico harmônico integrado ao **Selo do Ressonador Canônico** (Proposta 01), atuando como indicador de preenchimento de buffer e VU meter concentrado.
     - **Ilha Flutuante de Transporte (*Floating Playback Pill*):** Ancorada no terço inferior (*Thumb Zone* ergonômica), acomodando Play/Pause, salto de -15s/+30s e controle de taxa.

2. **Modo Condução & Mobilidade: *O Cockpit Eyes-Free* (Proposta 04)**
   - **Papel:** Acionamento rápido por um toque no cabeçalho (ícone HUD), alternância por gesto ou detecção de mobilidade/condução.
   - **Elementos:**
     - Alvos de toque gigantes (≥ 72×72dp) projetados para operação por memória motora e visão periférica (*SAE J2364*).
     - Halo central massivo de 180dp em turquesa (`#8FD8E8`) pulsando em fluxo contínuo.
     - **Aviso Temporal de Pausa (3.0s TTL / `SC-029`):** Em caso de perda transitória de foco ou desconexão de fone, o halo converte-se em um anel âmbar pulsante (`#FFD400`) com contagem regressiva circular e cronômetro decrescente "3.0s" em tempo real.

3. **Modo Navegação & Sumário Estrutural: *O Terminal Editorial Suíço* (Proposta 03)**
   - **Papel:** Painel lateral deslizante (*drawer*) de índice e bookmarks ao tocar no título do documento ou na seção.
   - **Elementos:**
     - Disciplina e clareza tipográfica inspirada em Josef Müller-Brockmann.
     - Marcador físico vertical em marfim acústico (*ribbon-marker*), indicando a profundidade macro da leitura sem expor o texto ou números de porcentagem arbitrários.
     - Acesso instantâneo aos capítulos e marcos salvos com persistência determinística via hash BLAKE3.

4. **Modo Diagnóstico & Telemetria do Motor Rust: *A Matriz de Telemetria* (Proposta 05)**
   - **Papel:** Diálogo técnico de inspeção profunda acionado ao tocar na barra de amostragem (`AAudio 22050 Hz`).
   - **Elementos:**
     - Sinalética científica de Otl Aicher DIN 1972 com a malha geométrica de ressonância Chladni emoldurando o Selo do Ressonador.
     - Exibição da integridade do hash BLAKE3 do documento (`a3f9...c812`), latência do buffer de síntese (ms), profundidade do pre-roll (≤ 60s) e leitor de eventos do log diagnóstico JSONL (sem telemetria externa).

- **Relatório Completo de Avaliação:** [`avaliacao_ui_ux.md`](file:///home/luis/development/mdreader/DESIGN-SONA/avaliacao_ui_ux.md)
- **Galeria Visual Interativa:** [`assets/ui/galeria_ui.html`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/ui/galeria_ui.html)


---

*Referências do design Sona:* `leagueoflegends.com/en-us/champions/sona/`; `wiki.leagueoflegends.com/en-us/Universe:Sona`; `Sona/Audio`; `Sona/Trivia`; `brand.riotgames.com/en-us/league-of-legends/color`; `DESIGN.md` (B — Sinalização); `DESIGN_SYSTEMS_5_CANDIDATOS.md` §E (Espectral / duotone — referência a Audacity/sonificação); `spec.md` FR-028/FR-041/SC-029/SC-035; `VERIFICATION.md` 184.

