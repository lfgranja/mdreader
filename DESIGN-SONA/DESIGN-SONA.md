# DESIGN-SONA.md — Design System Mestre: Sona (Influência / Inspirado pela personagem)

**Versão:** 1.0.0 · **Data:** 2026-10-06 · **Status:** especificado (novo diretório, inspirado pelo candidato B, adaptado à personalidade Sona)
**Referência:** `DESIGN.md` (B — Sinalização), `DESIGN_SYSTEMS_5_CANDIDATOS.md` §B, `RELATORIO_BRAND_PRODUTO.md` (escolha de B, marca ressona.dev, Engine Sona interno)
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

**Mudança de direção:** em vez de disciplinar severamente o semáforo (amarelo só 3s, verde proibido, vermelho só falha), esta versão incorpora a **suavidade**, **musicalidade** e **cura** de Sona dentro dos mesmos limites técnicos. Isso significa:
- A paleta é **mais clara, mais luminosa, com tons de azul-turquesa, dourado suave, violeta pastel** — derivados das cores documentadas de Sona (turquoise, gold, purple, pearl, cream) adaptados para contraste AA contra `#0B0F1A` (fundo OLED escuro).
- O **earcon é expandido** com a terminologia musical de Sona (Adagio, Andante, Allegro, Sforzando, Ostinato, Crescendo, Aria, Unison) — não apenas "2 notas".
- A **notificação** mantém a estrutura de B (título + seção + progresso + estado) mas o acento é **turquesa suave** (não amarelo severo) para fluxo; o **amarelo** é reservado apenas para o aviso de pausa (3s); o **vermelho** só falha; o **violeta claro** é apenas para o "Song of Celerity" (velocidade/rate) — nunca para estado crítico.
- O **in-app** ganha uma **faixa musical sutil** (3–5 px) inspirada na faixa de envelope do candidato E, mas com **cor suave e não gradiente agressivo** — uma rampa de turquesa a dourado, nunca vermelho/âmbar contestando.

---

## 3. Paleta Sona (adaptada para dark/OLED, contraste AA)

| Token | Hex | Papel | Contraste s/ `#0B0F1A` | Notas / Fonte |
|---|---|---|---|---|
| `--sona-void` | `#0B0F1A` | Fundo base (OLED) | — | Derivado do azul escuro de Sona, adaptado para preto com leve sombra azul |
| `--sona-stage` | `#141B2D` | Superfície elevada (cards, notificação expandida) | — | Um passo acima do void, mesmo tom |
| `--sona-voice` | `#F2F0E6` | Tinta principal (texto) | **16.75:1** | Quase branco, tom quente — ecoa `#f5edae` (cream) adaptado para legibilidade |
| `--sona-hush` | `#A9B8CE` | Texto secundário (seção, dim) | **9.51:1** | Periwinkle do azul da família documentada |
| `--sona-etwahl` | `#8FD8E8` | **Acento primário** — progresso ativo, estado fluxo, focus ring, logo vivo | **11.98:1** | Turquesa de Sona (`#2d63a1` adaptado); **único acento de identidade** |
| `--sona-harp` | `#E8C46A` | **Acento secundário** — taxa ativa, ênfase de seção, ponto de estado suave | **11.42:1** | Ouro de Sona (`#c9b373` adaptado); usado com moderação |
| `--sona-celeste` | `#B9A6F5` | **Acento terciário** — Song of Celerity / rate / velocidade (não estado crítico) | **8.96:1** | Violeta de Sona (`#cb97e3` adaptado); **nunca** usado para falha |
| `--sona-aurora` | `#3D2B8A` | Sombra profunda (não usado como cor de texto; só sombra de fundo, elevação) | — | Azul profundo; decorativo |
| `--sona-boundary` | `#55688F` | Hairline / dividers | **3.43:1** | Limiar 3:1 não-texto; marginal; usar `--sona-etwahl` para anel de focus |

**Nota de contraste:** todos os textos principais passam 4.5:1; `--sona-boundary` passa 3:1 (não-texto); `--sona-etwahl` e `--sona-harp` passam 3:1 como objetos gráficos (progress fill, focus ring). Nenhuma cor inventada fora das documentadas de Sona.

---

## 4. Regra do conjunto fechado (adaptada de B, softened para Sona)

A disciplina de B (cor como função, conjunto fechado) é mantida, mas a **paleta é ampliada** para incluir turquesa, dourado e violeta — todos dentro do documento de Sona. A regra não é "menos cores"; é "cada cor tem uma função, nenhuma função usa outra cor, e nenhuma cor é usada para decoração":

- **Turquesa (`#8FD8E8`)** — progresso ativo, fluxo, identidade, focus ring. Nunca usado para falha.
- **Dourado (`#E8C46A`)** — taxa ativa, seção de destaque, ponto de aviso suave. Nunca usado para falha crítica.
- **Violeta (`#B9A6F5`)** — rate / velocidade (Song of Celerity). Nunca usado para falha ou aviso de 3s.
- **Amarelo (`#FFD400`)** — **exclusivo** para aviso de pausa que expira (3s) — mantido do B, porque é a cor de aviso universal no CTB.
- **Vermelho (`#FF4D63`)** — **exclusivo** para falha / perda permanente. Nunca para transição.
- **Branco (`#FFFFFF`)** — texto, progresso de fluxo, referência de posição.
- **Dim (`#A9B8CE`)** — seção, metadados, texto secundário.

**Regra de expansão:** adicionar uma cor nova (ex.: rosa de "Spites" de Sona) = edição coordenada + bump semântico + docs. Mas a paleta atual já contém 3 acentos + 1 aviso + 1 falha — suficiente para o sistema.

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

## 7. Marca — Integração com ressona.dev / Engine Sona

- **Logo:** círculo + linha horizontal (lente de semáforo) — mantido do B, mas com **ponto turquesa `#8FD8E8`** em vez de amarelo (a marca de divulgação usa a cor da identidade, não a cor do aviso; o amarelo `#FFD400` é só estado de aviso). 
- **Wordmark:** "ressona" em Roboto Flex, cor `#F2F0E6`, acento turquesa no ponto do círculo.
- **Engine Sona:** nome do motor interno (`crates/engine-core`); quando o usuário lê o código ou docs internos, vê "Engine Sona"; quando vê a marca, vê "ressona" — separação clara, nenhuma confusão com LoL.
- **F-Droid:** a marca é compatível (não exige Google, não exige Play Store).

---

## 8. Riscos específicos do design Sona (adicionais ao R2–R8 do B)

- **R9 — Confusão com LoL:** se o usuário associar "Sona" ao jogo por experiência prévia, a marca pode ser percebida como "produto de jogo". Mitigação: a marca pública é `ressona.dev`, não `Sona`; o engine é confinado ao código; o site explica a referência (música, cura, harmonia) sem mencionar LoL.
- **R10 — Paleta muito clara / baixa saturação:** `#8FD8E8`, `#B9A6F5`, `#E8C46A` são cores de alta luminosidade. Em alguns monitores OLED, pode parecer "lavado" — mas o contraste é alto (11.98:1, 8.96:1, 11.42:1), então não é problema de legibilidade, é de percepção. Mitigação: usar `#0B0F1A` como fundo profundo para criar contraste visual, não apenas contraste de luminância.
- **R11 — Risco de "muito bonito, pouco funcional":** a paleta é suave, musical, elegante — pode ser percebida como "revista" se não for acompanhada de disciplina funcional. Mitigação: o design system é explícito em "não é revista" (referência ao risco D do B); o in-app é grade funcional (2 campos, ribbon-marker, taxa); a notificação é operacional (texto, progresso, cor de estado).
- **R12 — Dependência de som:** a personagem é musical, mas o produto deve funcionar sem som (muta = dignidade). Mitigação: todos os estados têm forma + posição + texto; earcon é redundante, não substitutiva.

---

*Referências do design Sona:* `leagueoflegends.com/en-us/champions/sona/`; `wiki.leagueoflegends.com/en-us/Universe:Sona`; `Sona/Audio`; `Sona/Trivia`; `brand.riotgames.com/en-us/league-of-legends/color`; `DESIGN.md` (B — Sinalização); `DESIGN_SYSTEMS_5_CANDIDATOS.md` §E (Espectral / duotone — referência a Audacity/sonificação); `spec.md` FR-028/FR-041/SC-029/SC-035; `VERIFICATION.md` 184.
