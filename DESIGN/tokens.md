# Design Tokens — mdreader (ressona.dev)

**Design system:** Candidato **B — Sinalização** (CTB / DIN 1451 / Semáforo)
**Versão:** 1.0.0 · **Data:** 2026-10-06 · **Status:** especificado (notificação nativa MediaStyle; in-app Svelte 5 webview planejado, ainda não existe)
**Autoridade:** `RELATORIO_BRAND_PRODUTO.md` (escolha de B + paleta corrigida) → `DESIGN_SYSTEMS_5_CANDIDATOS.md` §B

> Normativas vinculantes:
> - Constituição mdreader, **Princípio VII — Closed Sets, No Catch-Alls** (`.specify/memory/constitution.md`): o conjunto de cores é um closed set; adicionar variante = edição coordenada + bump semântico + update de docs.
> - WCAG 2.1 AA: texto ≥ **4.5:1** (SC 1.4.3), componentes/gráficos ≥ **3.0:1** (SC 1.4.11), cor nunca é o único meio de transmissão (SC 1.4.1).
> - Spec `001-android-media-shell`, **SC-029**: marquee/texto rolante é impossível em `Notification`/`RemoteViews`. Descumprimento **declarado** (não oculto); solução = earcon + razão estática + TTL 3 s.
> - Princípio VIII — Driving-First (NON-NEGOTIABLE): nenhuma operação exige olhar para a tela; a cor é redundância, o áudio é a autoridade.

---

## 1. Camada Primitive — 6 tokens de primeiro plano + 1 surface

Primitivos são valores brutos, sem significado funcional. Todo contraste foi **computado contra o surface `night`** pela fórmula WCAG 2.1 de luminância relativa:

`contraste = (L_claro + 0.05) / (L_escuro + 0.05)`, com `L = 0.2126·R + 0.7152·G + 0.0722·B` (canais linearizados).

| # | Token (CSS) | Nome | Hex | L relativa | Contraste vs `night` | Uso permitido |
|---|---|---|---|---|---|---|
| 0 | `--md-night` | night | `#0A0A0A` | 0.0030 | — (surface base) | fundo de **toda** superfície: notificação, lock-screen, in-app |
| 1 | `--md-paint` | paint | `#FFFFFF` | 1.0000 | **19.8:1** | tinta principal: título do documento, razão estática, preenchimento de progresso |
| 2 | `--md-dim` | dim | `#C9CDD2` | 0.6074 | **12.39:1** | tinta secundária: título da seção atual, metadados, taxa (in-app) |
| 3 | `--md-warning` | warning | `#FFD400` | 0.6835 | **13.83:1** | **única** cor de aviso — sempre transitória, TTL 3 s obrigatório |
| 4 | `--md-prohibit` | prohibit | `#FF4D63` | 0.2746 | **6.12:1** | **única** cor de falha / perda permanente — nunca aviso |
| 5 | `--md-info` | info | `#3D8BE8` | 0.2526 | **5.71:1** | **única** cor de informação não-crítica (routing, engine, foco temporário) |
| 6 | `--md-track` | track | `#5D5D5D` | 0.1095 | **3.01:1** | trilho de progresso — único token < 4.5:1; **proibido em texto** |

**Paleta corrigida (conforme RELATÓRIO_BRAND_PRODUTO.md, 2026-10-06):** vermelho corrigido para `#FF4D63` (6.12:1), azul corrigido para `#3D8BE8` (5.71:1), trilho corrigido para `#5D5D5D` (3.01:1). Os três foram reescolhidos por contraste; nenhum permanece na versão marginal da proposta original.

```css
/* tokens.css — camada primitive (mdreader / candidato B) */
:root {
  /* superfície base: fundo de tudo (notificação, lock-screen, in-app) */
  --md-night:    #0A0A0A;
  /* tinta principal: títulos, razão estática, preenchimento de progresso */
  --md-paint:    #FFFFFF;
  /* tinta secundária: seção atual, metadados, taxa (in-app) */
  --md-dim:      #C9CDD2;
  /* aviso transitório — TTL 3s obrigatório; nunca estado persistente */
  --md-warning:  #FFD400;
  /* falha / perda permanente — nunca aviso, nunca decoração */
  --md-prohibit: #FF4D63;
  /* informação não-crítica — nunca urgência, nunca falha */
  --md-info:     #3D8BE8;
  /* trilho de progresso — área não-texto apenas (3.01:1 ≥ 3.0:1, SC 1.4.11) */
  --md-track:    #5D5D5D;
}
```

---

## 2. Camada Semantic — nome + mapeamento + função

Um token semantic carrega **intenção**, não valor. Mapeia para exatamente um primitivo; trocar a paleta é editar só a camada 1.

| Semantic token | Mapeia para | Função | Regra de uso |
|---|---|---|---|
| `surface.canvas` | `--md-night` | superfície base única | todo conteúdo assenta sobre ela; não há segunda superfície |
| `text.primary` | `--md-paint` | informação primária, sempre legível | título do documento, razão estática (≤ 40 chars pt-BR), fill do progresso |
| `text.secondary` | `--md-dim` | informação secundária | título da seção atual, metadados, taxa; **nunca** para estado crítico |
| `signal.warning` | `--md-warning` | aviso que **expira** | TTL 3 s obrigatório; após o TTL, volta à ausência de cor |
| `signal.failure` | `--md-prohibit` | falha / perda permanente | persiste até resolução; sempre acompanhado de earcon + razão escrita |
| `signal.info` | `--md-info` | fato de sistema não-crítico | foco temporário (`*_MAY_DUCK`), routing, engine; nunca urgência |
| `progress.track` | `--md-track` | trilho (fundo do indicador) | área não-texto; 3.01:1 satisfaz SC 1.4.11 |
| `progress.fill` | `--md-paint` | posição corrente | **monocromático por design: posição ≠ estado** — a barra nunca é colorida |

**Regra de ouro semântica — repouso = ausência de cor.** O estado "ok"/repouso não tem cor própria: é simplesmente `paint`/`dim` sobre `night`. Não existe verde de sucesso no conjunto (ver §4).

```css
/* tokens-semantic.css */
:root {
  --md-surface-canvas:      var(--md-night);
  --md-text-primary:        var(--md-paint);
  --md-text-secondary:      var(--md-dim);
  --md-signal-warning:      var(--md-warning);   /* só com TTL 3s */
  --md-signal-failure:      var(--md-prohibit);  /* só falha/perda permanente */
  --md-signal-info:         var(--md-info);      /* só informação não-crítica */
  --md-progress-track:      var(--md-track);
  --md-progress-fill:       var(--md-paint);     /* posição nunca é colorida */
}
```

---

## 3. Camada Component — nome + mapeamento + onde aparece

Um token component existe **somente onde há um componente real**. Não há token sem superfície.

| Component token | Semantic | Componente / superfície | Nota |
|---|---|---|---|
| `--md-notif-title` | `text.primary` | **NotificationSurface** (Android MediaStyle) | título do documento; renderizado pela plataforma |
| `--md-notif-section` | `text.secondary` | **NotificationSurface** | título da seção atual |
| `--md-notif-progress-track` | `progress.track` | **NotificationSurface** | trilho da barra de progresso |
| `--md-notif-progress-fill` | `progress.fill` | **NotificationSurface** | preenchimento branco; **barra nunca colorida** |
| `--md-status-dot-warning` | `signal.warning` | **StatusView** (in-app, grade 2-campo) | ponto amarelo pulsante (alpha 1.2 s), TTL 3 s — resposta visual do SC-029 |
| `--md-status-dot-failure` | `signal.failure` | **StatusView** (in-app) | falha persistente de engine/foco |
| `--md-status-info` | `signal.info` | **StatusView** (in-app) | info de routing/focus temporário |
| `--md-rate-value` | `text.primary` | **RateView** (in-app) | taxa só aparece in-app; **nunca na notificação** |
| `--md-focus-loss` | `signal.failure` | **FocusMirror** (in-app) | perda permanente (call/alarm) → pausa + razão, sem auto-resume |
| `--md-focus-duck` | `signal.info` | **FocusMirror** (in-app) | `*_MAY_DUCK` → ~20 % duck, leitura continua |

```css
/* tokens-component.css — exemplo da superfície de status in-app */
.status-view {
  background: var(--md-surface-canvas);
  color: var(--md-text-primary);          /* razão estática ≤40 chars pt-BR */
}
.status-view .section { color: var(--md-text-secondary); }
.status-view .dot--warning {
  background: var(--md-signal-warning);   /* pulso alpha 1.2s; morre em 3s */
  animation: pulse 1.2s ease-in-out infinite;
}
.status-view .dot--failure { background: var(--md-signal-failure); }
.status-view .chip--info   { color: var(--md-signal-info); }
```

> **OEM recolor — mitigação declarada:** a notificação usa **MediaStyle padrão, não `RemoteViews` customizado**. Isso reduz (não elimina) a chance de OEM re-colorir a paleta. Matriz obrigatória de verificação: **Pixel / Samsung / Xiaomi** (gate 9 foi Xiaomi/API 33).

---

## 4. Regras do conjunto fechado (Princípio VII aplicado a cor)

O conjunto de cores é um **closed set** com a mesma disciplina dos enums do workspace (rate ladder, transport, focus outcomes, error kinds). Sem `Other`, sem catch-all.

1. **Amarelo só 3 s.** `--md-warning` existe exclusivamente como aviso transitório com **TTL de 3 s** (ponto pulsante in-app, alpha 1.2 s). Nunca estado persistente, nunca decoração, nunca identidade de marca, nunca "atenção suave".
2. **Vermelho só falha.** `--md-prohibit` exclusivamente para falha ou perda **permanente** de foco/engine. Nunca aviso, nunca progresso, nunca decoração.
3. **Verde proibido.** Não existe no conjunto e não entra sem Princípio VII completo. Sucesso/repouso é **ausência de cor** (monocromático). Proibido criar "verde de sucesso" ou "verde de playing".
4. **Azul só informação.** `--md-info` para fatos de sistema não-críticos; nunca urgência, nunca falha, nunca aviso.
5. **Cor nunca é o único canal** (WCAG 1.4.1). Toda informação cromática viaja com forma/posição + earcon. Produto é **eyes-free por design**: cor é redundância confirmatória, nunca a fonte.
6. **Trilho não carrega texto.** `--md-track` (3.01:1) é válido apenas em áreas não-texto (SC 1.4.11). Proibido para texto, ícones essenciais ou indicação de estado.
7. **Posição ≠ estado.** Progresso é sempre monocromático (`paint` sobre `track`). Cor indica **estado**, nunca **posição**.
8. **Adicionar cor = mudança coordenada.** Novo primitivo exige: edição coordenada deste arquivo + bump semântico + update de docs + reverificação de contraste — exatamente como qualquer variante nova em closed-set enum (Princípio VII).
9. **Sem exceções de contraste.** Se uma cor falha na verificação, **a cor é corrigida** (foi o que aconteceu com `#FF4D63`, `#3D8BE8` e `#5D5D5D`), nunca a regra.

---

## 5. Variantes de cor

**`dark` (default) — única variante especificada.** O produto é noturno/low-glare por requisito: contexto primário de condução (Princípio VIII), uso em lock-screen, tela baixa, leitura sem glare. `dark` é o tema de trabalho e o único válido hoje.

**`light` — NÃO especificado.** Nenhuma paleta clara foi verificada e **nenhuma foi inventada aqui**. Um tema claro exigiria, antes de existir:
- reverificação completa de contraste dos 7 primitivos contra um novo surface,
- revisão do TTL/pulso amarelo e do trilho 3.01:1 em fundo claro,
- edição coordenada + bump semântico + docs (Princípio VII),
- matriz Pixel/Samsung/Xiaomi refeita.

Risco declarado do candidato B: *"tema claro precisa reverificar"*. Até lá, `dark` é obrigatório em todas as superfícies.

```css
/* variantes.css — única variante existente */
:root,
:root[data-theme="dark"] { /* default: os tokens acima */ }
/* :root[data-theme="light"] — deliberadamente ausente. Não especificado. */
```

---

## 6. Acessibilidade não-visual (complemento obrigatório a toda cor)

Cor é redundância; o canal primário em condução é o **áudio**, sintetizado em Rust (22050 Hz mono f32, nenhum asset externo, nenhuma dependência de áudio de notificação):

| Earcon | Forma | Quando |
|---|---|---|
| Confirmação | 2 notas, **C5→E5 ascendente**; inversão para negativo | ação aceite (play/pause/next/rate) |
| Aviso / pausa (SC-029) | **descendente, 3 notas** | razão estática curta exibida com TTL 3 s |

Toda superfície que usa `signal.*` deve ter par auditivo correspondente. Se o earcon existe, a cor é confirmatória; se a cor existe sem earcon, é defeito.

---

## 7. Matriz rápida — uso / proibido

| Cor | Usar para | Proibido |
|---|---|---|
| `#FFFFFF` paint | títulos, razão estática, fill de progresso | — |
| `#C9CDD2` dim | seção atual, metadados, taxa | estado crítico, aviso, falha |
| `#FFD400` warning | aviso transitório ≤ 3 s | persistente, decoração, marca, progresso |
| `#FF4D63` prohibit | falha / perda permanente | aviso, "atenção suave", decoração |
| `#3D8BE8` info | info de sistema não-crítica | urgência, falha, aviso |
| `#5D5D5D` track | trilho de progresso (não-texto) | texto, ícone essencial, estado |
| *(nenhum verde)* | — | sucesso, playing, "ok" — repouso é ausência de cor |

---

## Histórico

| Versão | Data | Mudança |
|---|---|---|
| 1.0.0 | 2026-10-06 | Spec inicial do candidato B; paleta corrigida (`#FF4D63` 6.12:1, `#3D8BE8` 5.71:1, `#5D5D5D` 3.01:1); SC-029 declarado FAIL parcial com solução earcon + razão estática + TTL 3 s; única variante `dark`. |

*Fontes: `RELATORIO_BRAND_PRODUTO.md` (escolha de B, paleta corrigida, estratégia confirmada) · `DESIGN_SYSTEMS_5_CANDIDATOS.md` §B · `.specify/memory/constitution.md` Princípio VII (Closed Sets) e VIII (Driving-First) · `.agents/product-marketing.md` (marca `ressona.dev`, Engine `Sona` interno, workspace `mdreader`, F-Droid).*
