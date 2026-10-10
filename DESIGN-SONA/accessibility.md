# DESIGN-SONA/accessibility.md — Acessibilidade / Eyes-Free / Muda (Sona Inspirado)

**Versão:** 1.0.0 · **Data:** 2026-10-06
> Aplicável a todo `DESIGN-SONA/`. Base: `DESIGN/accessibility.md` (B — eyes-free, earcon, 48dp, não-WCAG-declarado). Mudanças: gramática sonora expandida com terminologia Sona (Adagio/Andante/Allegro/Sforzando/Ostinato/Crescendo/Aria/Unison); synesthesia como licença para visualização sonora; mutismo como dignidade (não deficiência); acesso não depende de áudio, mas áudio é redundante.

---

## 1. Contrato de acessibilidade — mesmo do B, expandido

**Não é WCAG/TalkBack.** É "operável sem olhos, confirmado pelo ouvido" (FR-018a / SC-008). Nenhum requisito de acessibilidade visual está adicionado sem emenda de spec.

**Mudança Sona:** o earcon não é apenas "2 notas" — é uma **linguagem musical completa** (ver §3). Isso não aumenta a complexidade; aumenta a **distintividade**, o que é útil para diferenciar estados sem depender de cor.

---

## 2. Earcon — Gramática musical Sona (expandido do B)

Referência: `Universe:Sona` (terminologia musical: Adagio, Andante, Allegro, Sforzando, Ostinato, Crescendo, Aria, Unison); `Sona/Audio`; `Sona/Trivia` (DJ Sona = 3h de som, primeira campeã totalmente baseada em música).

| Ação | Termo Sona | Som (Rust 22050 Hz mono f32) | Duração / Característica |
|---|---|---|---|
| Play / Resume | **Adagio** | C5 → E5 ascendente, suave | 120 ms/nota; curador |
| Pause | **Andante** | E5 → C5 descendente | 120 ms/nota; leve |
| Skip ± | **Allegro** | C5 → E5 + tick breve | Confirmação de salto |
| Rate change | **Sforzando** | Tick 1000 Hz / 40 ms | Acente preciso |
| Warning 3 s | **Ostinato** | Duplo tick A5 (60 + 80 ms) | Contagem regressiva |
| Failure | **Sforzando** + **Crescendo** (interrompido) | D3 quadrado 200 ms + breve repetição | Falha — claramente diferente do aviso |
| Section boundary | **Unison** | Tick 880 Hz / 60 ms | Estrutural, leve |
| Resume offer (3 s) | **Aria / Unison** | Soft 2 notas + fala de estado | Suave, de apoio |

**Regra:** todos os earcons são sintetizados pelo engine — nenhum asset externo, nenhuma dependência de áudio do sistema Android. Isso é a mesma regra do B, apenas com uma gramática mais rica.

---

## 3. Synesthesia — Visualização da voz (opcional, decorativa, não informativa)

**Referência:** `Universe:Sona` — *"Sona is able to see music as a visual stimulus."* A licença de que som pode ser visto é documentada pela Riot.

**Aplicação no design:**
- **In-app (opcional):** uma faixa de 3–5 px na borda do progresso, com cor turquesa (`#8FD8E8`) que varia sutilmente com o nível de RMS (não com o estado — só com a intensidade). É decorativa; não é necessária; não substitui a barra de progresso.
- **Nunca na notificação:** a notificação não tem espaço para decoração; a faixa é só in-app.
- **Regra:** se a faixa existe, é apenas uma "visualização da voz" de Sona — não é canal de informação (não mostra posição, não mostra estado). Se o usuário não a vê, o sistema funciona igual.

---

## 4. Muda — Não é deficiência, é dignidade

**Referência:** `Universe:Sona` — Sona é muda; comunica por sinais (sign language com família); a linha é *"My music is my voice."*

**Implicação para o design:**
- O sistema funciona **com ou sem som**. Se o usuário desativa o earcon, a interface ainda opera: cor + forma + texto + posição continuam. **Não há dependência sonora.**
- Isso é o oposto de "earcon é obrigatório" — o earcon é obrigatório para quem está dirigindo (FR-018a), mas o sistema não falha se o usuário o desativa.
- Se o usuário é mudinho (não fala, mas ouve), o earcon ainda funciona — ele confirma por som, não por fala.
- Se o usuário é surdo (não ouve), a interface ainda funciona — ele confirma por visão (cor + forma + texto) e por gesto (swipe da razão).

---

## 5. Confirmação por voz (TTS do sistema, não conteúdo do documento)

Quando o estado muda (play → pause → resume → skip), pode haver uma breve confirmação por voz (TTS do Android, não do documento):
- *"Leitura pausada — outro app usando o áudio"* (FR-041, razão de pausa).
- *"Leitura retomada"* (FR-032c, resume offer).
- **Nunca** fala o conteúdo do documento — respeita o invariante FR-028 (não body text na notificação / superfície pública).

---

## 6. Touch targets / gestos

- **In-app:** ≥ 48 dp (M3); espaço ≥ 8 dp; razão scroll por swipe (gesto, não leitura de texto que escapa).
- **Notificação:** controles da plataforma (MediaStyle) — nenhum target custom desenhado (B — disciplina institucional).
- **Gestos de "sign language" (opcional, inspirado por Sona):** swipe de 3 dedos para pausa (não exige olhar); swipe de 2 dedos para pular ±5 s; swipe vertical longo para mudar taxa (um rung por gesto, não por lista). **Não implementado no escopo atual — documento como extensão acessível futura.**

---

## 7. Testes (mesmo do B, com nota Sona adicional)

- **Blind-test (SC-008):** confirmar que cada earcon (Adagio, Andante, Allegro, Sforzando, Ostinato, Unison) é distintivo sem olhar. O earcon de "Aviso 3s" (Ostinato) deve ser distinto do "Falha" (Sforzando) — a frequência e a duração devem ser claramente diferentes.
- **Colorblind:** testar com simulação de deuteranopia (como Riot faz) — a distinção entre turquesa (`#8FD8E8`) e dourado (`#E8C46A`) deve ser perceptível por forma/posição, não só por cor; se não for, adicionar símbolo (ex.: ponto vs barra vs posição).
- **Mudo:** testar com earcon desativado — confirmar que cor + forma + texto + posição continuam operáveis.
- **Matrix:** Pixel / Samsung / Xiaomi (mesmo do B).

---

*Referências:* `DESIGN/accessibility.md` (B base); `Universe:Sona` (§6c — synesthesia); `Sona/Trivia` (mudez, sign language, sound); `wiki.leagueoflegends.com/en-us/Sona/Audio`; `m3.material.io` touch targets; `developer.android.com` (MediaStyle, AudioAttributes — USAGE_MEDIA / USAGE_NOTIFICATION).
