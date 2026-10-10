# DESIGN-SONA/tokens.md — Token Architecture: Sona-Inspired (B base, softened)

**Versão:** 1.0.0 · **Data:** 2026-10-06
> Base: `DESIGN/tokens.md` v1.0.0 (B — Sinalização, paleta corrigida, 3 camadas). Mudança: paleta adaptada à personalidade Sona (turquesa, dourado, violeta pastel) mantendo contraste AA, conjunto fechado, apenas dark.

---

## 1. Camada Primitive (hex, sem significado até a semantic)

| Token | Hex | Uso | Contraste s/ `#0B0F1A` | Nota / Fonte |
|---|---|---|---|---|
| `--sona-void` | `#0B0F1A` | Fundo base (OLED dark, leve sombra azul) | — | Derivado do azul-escuro de Sona; adaptado para preto com sombra azul |
| `--sona-stage` | `#141B2D` | Superfície elevada (cards, notificação expandida) | — | Um passo acima; mesma família |
| `--sona-voice` | `#F2F0E6` | Texto primário (título doc, body in-app) | **16.75:1** | Ecoa `#f5edae` (cream da comunidade); adaptado para AAA |
| `--sona-hush` | `#A9B8CE` | Texto secundário (seção, metadados, dim) | **9.51:1** | Periwinkle; passa AA a todos os tamanhos |
| `--sona-etwahl` | `#8FD8E8` | **Acento primário** — progresso, fluxo, focus ring, logo vivo | **11.98:1** | Turquesa de Sona (documentado `Universe:Sona`, azul/turquesa); adaptado para AA |
| `--sona-harp` | `#E8C46A` | **Acento secundário** — taxa ativa, ênfase, ponto de estado suave | **11.42:1** | Ouro de Sona (`#c9b373` adaptado); usado com moderação |
| `--sona-celeste` | `#B9A6F5` | **Acento terciário** — rate / Song of Celerity / velocidade | **8.96:1** | Violeta de Sona (`#cb97e3` adaptado); **não usado para falha/aviso** |
| `--sona-boundary` | `#55688F` | Hairline / dividers / contornos | **3.43:1** | Limiar 3:1 não-texto; marginal; usar `--sona-etwahl` para ring de focus |

**Regra de contraste verificada:** todos os textos ≥ 4.5:1; todos os objetos gráficos (fill, ring, track) ≥ 3:1. Nenhum token fora da lista acima é permitido sem edição coordenada + bump + docs (§VII da constituição).

---

## 2. Conjunto Fechado — Regra de Uso (adaptada do B)

- **Turquesa (`#8FD8E8`)** — progresso ativo, fluxo, identificação de marca, focus ring. **Nunca** falha, **nunca** aviso 3s, **nunca** info geral.
- **Dourado (`#E8C46A`)** — taxa ativa, destaque de seção, ponto de estado suave. **Nunca** falha, **nunca** aviso.
- **Violeta (`#B9A6F5`)** — rate / velocidade (Song of Celerity). **Nunca** falha, **nunca** aviso, **nunca** fluxo.
- **Amarelo (`#FFD400`)** — **exclusivo** aviso de pausa que expira (3 s). **Nunca** outro estado. **Nunca** progresso.
- **Vermelho (`#FF4D63`)** — **exclusivo** falha / perda permanente. **Nunca** transição, **nunca** aviso.
- **Branco (`#FFFFFF`)** — texto, progresso de fluxo (preenchimento), referência de posição.
- **Dim (`#C9CDD2`)** — seção, metadados, texto secundário.
- **Verde (`#5FD0A0`) — PROIBIDO.** Repouso = ausência de cor, não verde. Nenhum verde como "tocando".

---

## 3. Camada Semantic (mapeamento 1:1)

| Semantic | Primário | Função | Componente |
|---|---|---|---|
| `--surface-canvas` | `#0B0F1A` | Fundo universal | Notificação, in-app, site |
| `--surface-raised` | `#141B2D` | Card / notificação expandida | StatusView, landing |
| `--text-primary` | `#F2F0E6` | Título doc, corpo | Notificação, in-app |
| `--text-secondary` | `#A9B8CE` | Seção, metadados | Notificação, in-app |
| `--signal-flow` | `#FFFFFF` / `#8FD8E8` | Progresso preenchido + estado fluxo | Notificação, in-app |
| `--signal-caution` | `#FFD400` | Aviso 3s | Notificação (ponto pulsante), in-app |
| `--signal-failure` | `#FF4D63` | Falha permanente | Notificação, in-app |
| `--signal-info` | `#3D8BE8` | Informativo (taxa, detalhes) | In-app apenas (SC-027) |
| `--signal-track` | `#55688F` | Trilha de progresso | Notificação, in-app |

---

## 4. Camada Component (mapeamento aos componentes reais)

| Componente | Token primário | Token secundário | Regra |
|---|---|---|---|
| `NotificationSurface` (MediaStyle) | `--signal-flow` (barra `#FFFFFF`) | `--signal-track` (`#55688F`) | Sem custom RemoteViews; estado por cor + forma + texto |
| `StatusView` in-app | `--text-primary` + `--signal-etwahl` (progress) | `--signal-caution` (dot amarelo pulsante) | Grade 2-campo; scroll de razão real (SC-029 atendido) |
| `RateStepper` | `--signal-harp` (ativo) / `--signal-fog` (inativo) | — | Ladder fechado; nunca na notificação |
| `FocusMirror` / `RoutingMirror` | Nenhum token visual próprio — estado refletido por `--signal-caution` (transitória) / `--signal-failure` (permanente) | — | Sem componente visual adicional |

---

## 5. Variantes

- **Default / Dark:** `#0B0F1A` + tokens acima. **Único modo especificado.**
- **Light:** **não especificado.** Se requerido no futuro: requer re-verificação completa (não apenas inversão). Pré-requisitos: (a) manutenção dos contrastes de texto ≥ 4.5:1; (b) `#FFD400` ainda deve ser visível como aviso; (c) `#8FD8E8` ainda deve ser visível como fluxo; (d) `#FF4D63` ainda deve ser visível como falha; (e) `#B9A6F5` ainda deve ser visível como rate; (f) nenhum token novo inventado.

---

## 6. Acessibilidade — Earcon Sona (música como acesso)

O earcon usa a terminologia musical de Sona (Adagio, Andante, Allegro, Sforzando, Ostinato, Crescendo, Aria, Unison) como a **gramática sonora** do sistema. Não é apenas "2 notas" — é uma linguagem:

| Termo Sona | Ação | Som (Rust 22050 Hz) | Notas |
|---|---|---|---|
| **Adagio** | Play / Resume | C5 → E5 ascendente, suave (120 ms/nota) | Curador, não agressivo |
| **Andante** | Pause | E5 → C5 descendente (120 ms/nota) | Leve, não abrupto |
| **Allegro** | Skip ± / Rate change | C5 → E5 + tick breve (40 ms) | Confirmação de salto |
| **Sforzando** | Falha / Recusa | D3 quadrado 200 ms + repetição | Força, curto — claramente falha |
| **Ostinato** | Aviso 3 s | Duplo tick A5 (60 ms + 80 ms intervalo) | Contagem regressiva |
| **Crescendo** | Escalation (engine/voice change) | Crescendo de 3 notas, crescente | Não usado para falha (não confundir) |
| **Aria / Unison** | Confirmação / Duet | Soft, consonant, 2 notas suaves | Suporte, não alerta |

**Regra:** earcon é obrigatório para todo controle enquanto dirigindo (FR-018a). Nenhum controle funciona sem confirmação auditiva.

---

## 7. Referências normativas de cada token

- `#8FD8E8`: `Universe:Sona` (§4) — turquoise; `brand.riotgames.com/en-us/league-of-legends/color` (chroma "Turquoise"); `r/sonamains` (turquoise extrato); wiki chroma "Turquoise".
- `#E8C46A`: `Universe:Sona` (§4) — gold; `r/sonamains` (gold family `#c9b373`); LoL brand `Okra #C4A15B` (terceiro); adaptado para AA.
- `#B9A6F5`: `Universe:Sona` (§4) — violet / Amethyst ring for E; wiki chroma "Amethyst".
- `#0B0F1A`: adaptado do azul-escuro documentado (`#0B0C0E` do B) para OLED profundo.
- `#FFD400`: CTB / semáforo amarelo (avisos); mantenho do B como único aviso.
- `#FF4D63`: CTB / semáforo vermelho (proibição/perda permanente); mantenho do B.
