# DESIGN-SONA/components.md — Componentes: B base + Sona adaptado

**Versão:** 1.0.0 · **Data:** 2026-10-06
> Referência: `DESIGN/components.md` (B — Sinalização). Mudanças: paleta turquesa/dourado/violeta; ponto amarelo pulsante (aviso 3s) mantido; progresso monocromático (`#FFFFFF` fill sobre `#55688F` trilho); notificação sem custom RemoteViews; in-app grade 2-campo com faixas suaves; scroll real de razão (SC-029 atendido só in-app).

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
| Estado failure | `--sona-prohibit` (`#FF4D63`) + `#FFD400` não aparece | Falha / perda permanente | Só vermelho; nunca amarelo; nunca turquesa |
| Estado info | `--sona-info` (`#3D8AEB`) — opcional | Taxa / detalhes — **só in-app** (SC-027) | Nunca na notificação |

**Nota de mudança Sona:** o acento primário é turquesa (`#8FD8E8`) — mais suave que o amarelo do B, mas ainda claro (11.98:1). O amarelo é reservado exclusivamente ao aviso de 3s. O vermelho é exclusivo à falha. Isso mantém a disciplina do B com uma paleta mais leve.

---

## 2. StatusView in-app — Grade 2-campo (Müller-Brockmann adaptado)

**Layout (mesmo B, paleta Sona):**
- Fundo `#0B0F1A`; cards `#141B2D`; texto `#F2F0E6`; secundário `#A9B8CE`; progresso turquesa `#8FD8E8` (ativo) / `#55688F` (trilho); taxa dourado `#E8C46A` (ativo) / `#A9B8CE` (inativo); razão com scroll real (SC-029 atendido); ponto amarelo pulsante (`#FFD400`, alpha 1.2s) quando razão expira.

**Grid:** campo esquerdo (~62%) = título + seção + progresso + ribbon-marker (posição no documento como fita física, referência D/B); campo direito (~38%) = taxa + transport + razão + ponto amarelo.

**Ribbon-marker:** a única ornamentação. Uma linha vertical de `#F2F0E6` (ou `#8FD8E8` quando ativa) na borda esquerda, com comprimento proporcional à posição no documento. **Nunca** porcentagem — apenas "Seção 3 · As cartas na mesa".

---

## 3. RateStepper (controle de taxa)

**Mesma regra do B:** escada fechada `[0.75, 1.0, 1.25, 1.50, 1.75, 2.0, 4.0]` — uma pressão = um rung; saturação nas extremidades. **Nunca** na notificação (SC-027). Taxa mostrada só in-app, em dourado (`#E8C46A`) quando ativa.

---

## 4. FocusMirror / RoutingMirror / Residencies

**Sem componente visual próprio** — reflete estado via cor + forma + texto da notificação/in-app (mesma regra do B). Perda transitória = ponto amarelo + razão estática; perda permanente = ponto vermelho + razão estática; regresso = ponto turquesa + resume offer (3 s).

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

*Referências:* `DESIGN/components.md` (B — componente base); `DESIGN-SONA.md` §5–7 (mudanças); `spec.md` FR-018/FR-028/FR-041/SC-029/SC-035; `VERIFICATION.md` 184.
