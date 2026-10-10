# DESIGN.md — Design System Mestre: mdreader / ressona.dev

**Design system:** Candidato **B — Sinalização** (CTB / DIN 1451 / Semáforo)
**Versão:** 1.0.0 · **Data:** 2026-10-06 · **Status:** especificado (notificação nativa MediaStyle; in-app Svelte 5 webview planejado, ainda não existe)
**Produto:** leitor de áudio de documentos técnicos — notificação Android MediaStyle + status view in-app mínima (spec `001-android-media-shell`)

**Cadeia de autoridade:**
`RELATORIO_BRAND_PRODUTO.md` (escolha de B + paleta corrigida, 2026-10-06) → `DESIGN_SYSTEMS_5_CANDIDATOS.md` §B (tese, paleta, riscos) → `DESIGN/tokens.md` v1.0.0 (arquitetura de tokens — vivo) → **este arquivo** (índice mestre).

**Normativas vinculantes (aplicam-se a todo o diretório `DESIGN/`):**
- Constituição mdreader **v1.2.0 §VII — Closed Sets, No Catch-Alls** (redação vigente v1.3.0; o Princípio VII está inalterado desde v1.0.0): o conjunto de cores é um *closed set*; adicionar variante = edição coordenada + bump semântico + update de docs.
- Constituição mdreader **Princípio VIII — Driving-First (NON-NEGOTIABLE)**: nenhuma operação exige olhar para a tela; o áudio é a autoridade, a cor é redundância confirmatória.
- **CTB** — Código de Trânsito Brasileiro, Lei 9.503/1997: semântica de sinal (amarelo = atenção transitória; vermelho = proibição / perda permanente; azul = informação obrigatória).
- **DIN 1451**: disciplina de sinalização — tipografia funcional, legível à distância, sem decoração.
- **Material 3 (M3)**: apenas **48 dp** de target e a estrutura **MediaStyle**. B **não** usa M3 color roles (`primary`/`secondary`/`tertiary`) — as funções de cor vêm do CTB, não do M3.
- **Android 16 Live Updates** (`Notification.ProgressStyle`): mecanismo sancionado de progresso; proíbe `customContentView` — reforça a declaração SC-029.
- **WCAG 2.2**: SC 1.4.3 (texto ≥ **4.5:1**), SC 1.4.11 (não-texto ≥ **3:1**), SC 1.4.1 (cor nunca é o único meio).
- Spec `001-android-media-shell` (família FR-028, FR-041, SC-010, SC-032, SC-033/033b, SC-035, SC-042) e `VERIFICATION.md`.

---

## 1. Decisão: por que B

Cinco candidatos foram compilados (`DESIGN_SYSTEMS_5_CANDIDATOS.md`). **B — Sinalização** foi escolhido por evidência, não por gosto:

1. **Mais próximo do SC-029 sem quebrar a plataforma** — ponto pulsante amarelo + TTL 3 s (in-app) + razão estática + earcon (notificação) é a resposta mais forte ao requisito de texto rolante dentro do que `Notification`/`RemoteViews` permitem.
2. **Sobrevivência OEM alta** — MediaStyle padrão, nenhum `RemoteViews` customizado; nada de gradiente para o OEM reinterpretar.
3. **Disciplina de closed set mais forte** ("cor como função regulada") — alinhamento direto ao Princípio VII da constituição.
4. **Contraste verificável em todos os primitivos** (mínimo 3.01:1, computado contra `night`).
5. **Nenhum risco de confusão com "Sona"** (campeã de LoL) — `Sona` permanece como engine interno, fora da marca pública.

Por que não os outros: **A** é honesto sobre o SC-029, mas exige força in-app (agulha RMS real) além do escopo do spec 001; **C** é institucionalmente sólida, mas seu ciano-identidade não tem função semântica regulada; **D** é o mais forte in-app, mas monocromático não entrega o código de sinal; **E** transforma progresso em gradiente decorativo, com risco de expansão de conjunto e `cool` 3.12:1 marginal.

> **Rastreabilidade do token SC-029:** na família de documentos de design (`RELATÓRIO_BRAND_PRODUTO.md`, `DESIGN_SYSTEMS_5_CANDIDATOS.md`, `DESIGN/tokens.md`), **SC-029** designa a restrição de texto rolante (requisito de origem no spec: **FR-041**). Em `specs/001-android-media-shell/VERIFICATION.md`, o token SC-029 enumera a *rate ladder* (FR-019) — convenções distintas. Este documento segue a convenção da família de design.

---

## 2. Princípios do sistema

1. **Conjunto fechado, sem catch-all.** O conjunto de cores é um closed set com a mesma disciplina dos enums do workspace (rate ladder, transport, focus outcomes, error kinds). Sem `Other`, sem `Unknown`, sem braço padrão.
2. **Cor como função regulada, nunca decoração.** Cada primitivo tem exatamente uma função permitida e uma lista explícita de proibições (§5, `tokens.md` §4/§7). Não existe cor de marca, cor de "atenção suave" ou cor de estado implícito.
3. **Nenhuma animação.** As únicas duas exceções nomeadas: o **ponto amarelo pulsante** (alpha 1.2 s ease-in-out, que morre no TTL 3 s) e o **scroll real in-app** (conduzido pelo usuário — nunca marquee automático). Toda mudança de estado é instantânea; nenhuma transição, fade ou barra colorida animada.
4. **Disciplina institucional (CTB / DIN 1451).** Sinalização é infraestrutura: uma geometria, uma voz, uma função por cor; sobrevive de 4K px a 16 px; nenhuma inovação visual, só consistência.
5. **Eyes-free por design (Princípio VIII).** Cor é redundância confirmatória; o áudio é a autoridade. Nenhuma informação viaja apenas por cor (WCAG 2.2 SC 1.4.1) — sempre forma/posição + earcon.
6. **Repouso = ausência de cor.** Não existe verde no conjunto. O estado "ok" é simplesmente `paint`/`dim` sobre `night`. Proibido criar "verde de sucesso" ou "verde de playing".

---

## 3. Índice do diretório `DESIGN/`

| Arquivo | Status | Conteúdo |
|---|---|---|
| `DESIGN.md` | **vivo (este)** | Mestre: decisão, princípios, índice, paletas, resumo de tokens/components, acessibilidade, marca, riscos, histórico |
| `tokens.md` | vivo, v1.0.0 | Arquitetura de tokens em 3 camadas; paleta corrigida com contraste computado (fórmula WCAG); regras do conjunto fechado (§4); variante única `dark` (§5); earcons (§6); matriz uso/proibido (§7) |
| `components.md` | especificado (escopo fixado neste mestre) | Specs detalhados por componente — **NotificationSurface**, **StatusView**, **RateView**, **FocusMirror**: layout, regras de conteúdo, determinate/indeterminate, non-dismissable, targets 48 dp, estados e transições permitidas |
| `accessibility.md` | especificado | Contrato eyes-free: earcons (forma, síntese Rust 22050 Hz mono f32), confirmação audível, alvos WCAG 2.2, condução, matriz de redundância cor ↔ áudio |
| `brand.md` | especificado | `ressona.dev` (marca pública), Engine `Sona` (interno), `mdreader` (workspace técnico), F-Droid, direção de logo (onda + documento), tom de voz |
| `risks.md` | especificado | Registro de riscos e desvios declarados: SC-029, OEM recolor, minSdk, light mode, throttling de pulso, Live Updates |

---

## 4. As cinco paletas (resumo)

| Candidato | Tese | Paleta (resumo) | Veredito |
|---|---|---|---|
| A — MEDIDOR | VU-meter / Ableton / SMuFL | `#0B0C0E` · âmbar `#E8B04B` · verde `#5FD0A0` · vermelho `#E5534B` | honesto sobre SC-029; força in-app além do escopo |
| **B — SINALIZAÇÃO** | **CTB / DIN 1451 / Semáforo** | **`#0A0A0A` · `#FFFFFF` · `#C9CDD2` · `#FFD400` · `#FF4D63` · `#3D8BE8` · `#5D5D5D`** | **ESCOLHIDO** |
| C — EMISSORA | NRK / DR / Yle (broadcast) | `#0E1116` · ciano `#5AC8FA` · coral `#FF7A59` | sólida; sem função de sinal regulada |
| D — EDITORIAL | Müller-Brockmann / iA Writer | `#101010` · `#F5F2EC` · vermelho `#C8553D` só falha | forte in-app; sem código de sinal |
| E — ESPECTRAL | Duotone / Audacity | gradiente cool `#2D5F8B` → hot `#E8A33D` | decoração; risco de expansão; cool 3.12:1 marginal |

Detalhes, notas de tipografia e riscos por candidato: `DESIGN_SYSTEMS_5_CANDIDATOS.md`.

---

## 5. Arquitetura de tokens (resumo)

Três camadas; valores completos, contraste computado e CSS em `tokens.md` §1–§3.

**Camada Primitive — 7 tokens** (valores brutos, sem significado funcional; contraste computado contra `night`):

| Token | Hex | Contraste vs `night` | Função única |
|---|---|---|---|
| `--md-night` | `#0A0A0A` | — (surface base) | fundo de **toda** superfície |
| `--md-paint` | `#FFFFFF` | 19.8:1 | tinta principal (título, razão estática, fill) |
| `--md-dim` | `#C9CDD2` | 12.39:1 | tinta secundária (seção, metadados, taxa) |
| `--md-warning` | `#FFD400` | 13.83:1 | **único** aviso — sempre transitório, TTL 3 s |
| `--md-prohibit` | `#FF4D63` | 6.12:1 | **única** falha / perda permanente |
| `--md-info` | `#3D8BE8` | 5.71:1 | **única** informação não-crítica |
| `--md-track` | `#5D5D5D` | 3.01:1 | trilho de progresso — **proibido em texto** |

**Camada Semantic — 8 tokens** (intenção, não valor): `surface.canvas`, `text.primary`, `text.secondary`, `signal.warning`, `signal.failure`, `signal.info`, `progress.track`, `progress.fill`.

**Camada Component — 10 tokens** (existe somente onde há superfície real):
`--md-notif-title`, `--md-notif-section`, `--md-notif-progress-track`, `--md-notif-progress-fill` (NotificationSurface) · `--md-status-dot-warning`, `--md-status-dot-failure`, `--md-status-info` (StatusView) · `--md-rate-value` (RateView) · `--md-focus-loss`, `--md-focus-duck` (FocusMirror).

**Regra de ouro semântica — repouso = ausência de cor.** Progresso é sempre monocromático (`paint` sobre `track`): **posição ≠ estado** — a barra nunca é colorida. Paleta corrigida em 2026-10-06: `#FF4D63` (6.12:1), `#3D8BE8` (5.71:1), `#5D5D5D` (3.01:1) — se uma cor falha na verificação, a cor é corrigida, nunca a regra.

---

## 6. Componentes principais

### 6.1 NotificationSurface (Android MediaStyle — nativa)

- **Permitido:** título do documento, título da seção atual (mais recente precedente; nenhum antes do primeiro heading), progresso, controles de transporte.
- **Progresso:** determinate **iff** o total é conhecido no início da sessão; senão indeterminate por toda a sessão (SC-033b/FR-028a) — nenhuma troca de determinância mid-session, nunca total estimado. Fill branco sobre trilho `track`.
- **Proibido:** body text (SC-010: zero instâncias), tempo decorrido/remanescente, taxa, nome de voz.
- **Comportamento:** non-dismissable enquanto toca (`ongoing`); pausa → dismissível; fallback definido se OEM permitir dismiss durante playback.
- **Transporte:** play, pause, resume, skip ±5.0 s (clamped), Stop — do lock screen, sem desbloquear (Princípio VIII).
- **Visibilidade:** declara `visible`, nunca redige, obedece a preferência system-wide de ocultação (sem workaround).
- **Restrições de plataforma:** template MediaStyle obrigatória; sem `RemoteViews` customizado; Live Updates (Android 16) proíbem `customContentView`; limite de ~252 dp para superfície customizada reforça o uso de template. Fluxo de permissão de notificação é fora de escopo (consequência declarada, sem prompt projetado).

### 6.2 StatusView (in-app, grade 2-campo)

- Razão estática **≤ 40 caracteres pt-BR** em `text.primary`; seção atual em `text.secondary`.
- **Ponto amarelo pulsante** (alpha 1.2 s) com **TTL 3 s obrigatório** — a resposta visual do SC-029; após o TTL, volta à ausência de cor.
- Dot vermelho persistente para falha/perda permanente (sempre com earcon + razão escrita); chip azul para info não-crítica (routing, foco temporário).
- **Faixa de razão com scroll real** — SC-029 atendido apenas aqui (o usuário conduz o scroll; nunca marquee).

### 6.3 RateView (in-app)

- Escada fechada `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`; um passo por pressão, saturante nas pontas; sem `Other`/`Unknown`.
- Stretch preserva pitch (operação de contrato do engine; ≤ ±3 cents @ 2.0×). A taxa **só aparece in-app**, nunca na notificação.

### 6.4 FocusMirror (in-app)

- `*_MAY_DUCK` → ~20 % de duck (ganho [0.15, 0.25]), leitura continua (ADR-001 — desvio deliberado do best-practice Android, registrado na constituição).
- Perda permanente (call/alarm) → pausa + razão, **sem auto-resume**; a razão expira em 3 s em toda parte (FR-041/FR-037a); falha real persiste (FR-037).
- `BECOMING_NOISY` / `onAudioDevicesRemoved` → pausa dentro do bound de ≤ 1 s.
- Ganho de foco é **evento** (direção), não um quarto resultado.

**Fronteiras (não-metas):** sem screen de settings, sem widget, sem temas além de `dark`, sem animações além das duas nomeadas (Princípio 3), sem light mode. Detalhes, estados e layouts: `components.md`.

---

## 7. Acessibilidade / eyes-free

- **Earcon de confirmação:** 2 notas, **C5→E5 ascendente**; inversão para negativo — ação aceite (play/pause/next/rate).
- **Earcon de aviso/pausa (SC-029):** **descendente, 3 notas** — acompanhado da razão estática com TTL 3 s.
- **Síntese 100 % Rust:** 22050 Hz mono f32, nenhum asset externo, nenhuma dependência de áudio de notificação.
- Toda superfície que usa `signal.*` tem par auditivo correspondente; se a cor existe sem earcon, é defeito.
- **Targets ≥ 48 dp** (M3). **Contraste:** texto ≥ 4.5:1 (SC 1.4.3); não-texto ≥ 3:1 (SC 1.4.11) — `track` (3.01:1) é o único token abaixo de 4.5:1 e é proibido em texto.
- Cor nunca é o único canal (SC 1.4.1): forma + posição + earcon. Contrato detalhado: `accessibility.md`.

---

## 8. Marca e integração

- **`ressona.dev`** — marca de divulgação (RDAP 404 = livre): landing, F-Droid, comunidade.
- **Engine `Sona`** — nome do motor interno (código), **nunca** marca pública (evita associação com LoL).
- **`mdreader`** — workspace/repo/package técnico (`app.mdreader`); mantido e confirmado.
- **F-Droid** — distribuição confirmada (não Play Store); copyleft é permitido pela licence policy da constituição.
- **Direção de logo:** símbolo simples (onda + documento); tipografia clara. **Tom:** técnico, direto, sem hype; não-invasivo — o produto rejeita teleprompter e body text no lock-screen, e a marca reflete isso. Detalhes: `brand.md`.

---

## 9. Restrições e riscos principais

1. **SC-029 — desvio declarado (não oculto).** Texto rolante é impossível em `Notification`/`RemoteViews` (sem `selected`, template MediaStyle obrigatória, Live Updates proíbem `customContentView`). Solução: earcon descendente 3 notas + razão estática ≤ 40 chars pt-BR + TTL 3 s + ponto pulsante in-app + scroll real in-app. Declarar o descumprimento é honestidade, não falha — o relatório do defeito é obrigatório.
2. **OEM recolor.** MediaStyle padrão **reduz** (não elimina) a chance de recoloração da paleta. Matriz obrigatória de verificação: **Pixel / Samsung / Xiaomi** (gate 9 foi verificado em Xiaomi/API 33).
3. **minSdk 29 vs manifesto 24.** O piso do produto é minSdk 29 (SC-032); o scaffold pode gerar manifest com 24. Corrigir em `gen/android/.../AndroidManifest.xml`; `check-manifest.sh` afirma `minSdk(Version) 29`; re-verificar a cada upgrade de Tauri (editar o manifest diretamente é o caminho sancionado — `mobile::update_manifest` não existe).
4. **Conjuntos fechados.** Cores, rate ladder, transport, focus outcomes, error kinds — adicionar variante = edição coordenada + bump semântico + docs (Princípio VII); nenhum `Other`/`Unknown`/catch-all.
5. **Light mode: não especificado.** `dark` é a única variante; light exige reverificação completa dos 7 primitivos contra um novo surface + matriz OEM refeita.
6. **Throttling do pulso in-app.** OEM/API podem throttlar o alpha; na matriz, verificar. Se throttled, earcon + razão estática carregam a informação — a cor é redundância.
7. **Android 16 Live Updates.** Adotar `Notification.ProgressStyle` exige reverificar a paleta contra o render da plataforma e refazer a matriz OEM.

Registro completo: `risks.md`.

---

## 10. Referências normativas

- **CTB** — Código de Trânsito Brasileiro, Lei 9.503/1997 (semântica de sinal).
- **DIN 1451** (disciplina de sinalização alemã).
- **Material 3** — color roles (rejeitados por B), targets 48 dp, estrutura MediaStyle.
- **Android 16 Live Updates** — `Notification.ProgressStyle`; proibição de `customContentView`.
- **WCAG 2.2** — SC 1.4.1, SC 1.4.3 (4.5:1), SC 1.4.11 (3:1 não-texto).
- **Constituição mdreader v1.2.0 §VII** (Princípio VII — Closed Sets, No Catch-Alls; vigente v1.3.0, princípio inalterado) e Princípio VIII (Driving-First).
- **Spec `001-android-media-shell`** — `spec.md` (FR-028 família, FR-041, FR-019), `VERIFICATION.md`, `checklists/notification-surface.md`.
- **`RELATORIO_BRAND_PRODUTO.md`** (escolha de B, paleta corrigida, estratégia de marca confirmada) e **`DESIGN_SYSTEMS_5_CANDIDATOS.md`**.

---

## 11. Histórico de versão

| Versão | Data | Mudança |
|---|---|---|
| 1.0.0 | 2026-10-06 | Especificação inicial do mestre: candidato B adotado; paleta corrigida (`#FF4D63` 6.12:1, `#3D8BE8` 5.71:1, `#5D5D5D` 3.01:1); `tokens.md` vivo; `components.md`, `accessibility.md`, `brand.md`, `risks.md` com escopo fixado; SC-029 declarado com solução earcon + razão estática + TTL 3 s; única variante `dark`. |

*Fontes: `RELATORIO_BRAND_PRODUTO.md` · `DESIGN_SYSTEMS_5_CANDIDATOS.md` · `DESIGN/tokens.md` v1.0.0 · `.specify/memory/constitution.md` (Princípios VII e VIII) · `.agents/product-marketing.md` (marca `ressona.dev`, Engine `Sona`, workspace `mdreader`, F-Droid) · `specs/001-android-media-shell/` (spec, VERIFICATION, checklists).*
