# DESIGN/brand.md — Integração da marca ressona.dev (B — Sinalização)

**Versão:** 1.0.0 · **Data:** 2026-10-06 · **Status:** integrado ao master `DESIGN.md`

> Aplicável a todo `DESIGN/`. Regra: marca é referência, não conteúdo. Nenhum texto de documento aparece na marca; a marca não substitui título do documento na notificação.

---

## 1. Nome e origem

- **Workspace técnico:** `mdreader` — mantido, não renomeado. É o nome do repositório, do package Kotlin (`app.mdreader`), do workspace Cargo, do Tauri `tauri.conf.json`. Nunca será usado como marca de divulgação.
- **Marca de divulgação:** `ressona.dev` — confirmado livre pelo RDAP do registry autoritativo (`pubapi.registry.google` → HTTP 404 com `errorCode: 404`; `.org`/`.com.br` tomados por terceiros; `ressona.es` e `ressona.com.br` são sites de terceiros — não são a marca do projeto).
- **Name of engine:** `Engine Sona` — confinado aos crates `engine-core` / `player`; nunca aparece na UI de superfície, nunca na notificação, nunca no site; referência interna de código.
- **Por que não `Sona` como marca pública:** `Sona` é campeã de LoL (Sona, a Curandeira). Usar como marca de divulgação cria risco de confusão de marca + associação ao jogo que não é intenção do produto. `Sona` como engine evita esse risco por confinamento.

---

## 2. Logo (direção visual — baseada no candidato B)

**Forma:** círculo aberto + linha horizontal que atravessa o círculo (lente de semáforo sobre via — referência ao CTB / sinalização). Monocromático sobre transparente (`#0A0A0A` lado escuro; `#F5F2EC` lado claro se necessário futuro).

**Acento (único):** amarelo `#FFD400` — usado apenas no ponto de estado (o "ponto amarelo pulsante" do candidato B). Se o ponto não está ativo (fluxo), o logo é monocromático.

**Tipografia do wordmark:** Roboto Flex, peso médio, 20sp mínimo (in-app); na notificação não há wordmark (só título do doc + seção + progresso — a marca não aparece na lock screen, conforme FR-028: "no body text, no other substitution").

**Escalas:** 16px (favicon / chip de estado) → 4K (landing / web). A forma deve ser legível em 16px sem redesenho — por isso é simples (círculo + linha, não curva complexa).

---

## 3. Paleta da marca (repetição do token B, para referência)

| Foco | Hex | Uso na marca |
|---|---|---|
| Fundo / luz | `#0A0A0A` / `#F5F2EC` | Site / in-app / notificação (escuro primário) |
| Tinta | `#FFFFFF` | Texto, progresso, logo monocromático |
| Dim | `#C9CDD2` | Subtítulos, seções secundárias |
| **Acento único** | `#FFD400` | **Apenas o ponto de estado / logo vivo** — nunca como fundo, nunca como texto |
| Falha | `#FF4D63` | Só para estado de erro (não é acento da marca) |
| Info | `#3D8BE8` | Só informativo (não é acento da marca) |

**Regra:** a marca usa **um** acento (`#FFD400`). A falha e o info são funções do sistema, não da marca.

---

## 4. Voz / Tom / Posicionamento

- **Categoria:** Leitor de áudio de documentos técnicos (não podcast, não audiolivro, não revista).
- **Personalidade:** Técnica, confiável, simples, respeitosa com a atenção do usuário.
- **Tom:** Direto. Não hype. Não gamificação. "Operável por sentimento, confirmado pelo ouvido" — a voz reflete isso.
- **Referência cultural:** CTB/DIN (sinalização brasileira), NRK/DR (disciplinaridade institucional), M3 (estrutura técnica). **Não** referência a gaming (LoL Sona), **não** referência a entretenimento.
- **F-Droid:** a marca deve ser compatível com distribuição open-source — não exige assinatura Google, não exige conta de desenvolvedor Apple, não tem taxa de 30%.

---

## 5. Aplicações (onde a marca vive)

| Superfície | Marca aparece? | Regra |
|---|---|---|
| **Notificação MediaStyle** | **NÃO** | O título é do documento (FR-029); a marca não substitui conteúdo do documento. A notificação é operada pelo Android, não pelo design da marca. |
| **In-app status view** | **SIM — mínimo** | Logo no topo (16 dp) + palavra "ressona" (12 sp, `#C9CDD2`) — nunca no fluxo de leitura; apenas no cabeçalho da status view. |
| **Site / Landing** | **SIM — completo** | Logo grande + wordmark + descrição + links para repo + F-Droid + docs. |
| **F-Droid / Repo** | **SIM** | Nome do repo pode ser `ressona` (ou `ressona/leitura`); descrição menciona `mdreader` como workspace técnico; `Engine Sona` mencionado apenas em docs internos. |
| **Earcon / Áudio** | **NÃO** | O earcon é sintético (Rust 22050 Hz); não há "som de marca". Se futuramente houver, seria uma variação do earcon existente, não um asset separado. |

---

## 6. O que a marca NÃO faz (deliberado)

- **Não é uma revista** (não há design editorial na marca — o editorial é candidato D, não B).
- **Não é uma personagem** (não há mascote, não há animação de personagem — o ponto amarelo é estado, não mascote).
- **Não é uma música** (não há jingle; apenas earcon de estado).
- **Não é uma identidade de luxo** (não usar serif premium, não usar gradientes, não usar sombra decorativa — o candidato B é disciplina institucional).
- **Não confunde com LoL** ("Engine Sona" é confinado; a marca pública é "ressona").

---

*Referências:* `DESIGN.md` §9; `DESIGN_SYSTEMS_5_CANDIDATOS.md` §B-9; `RELATORIO_BRAND_PRODUTO.md` §8 (Top 5: Leitora, DocSona, Audileitura, LeituraAud, Sona — com notas de risco); `spec.md` FR-029 (título vem de metadado, não de marca); `.agents/product-marketing.md` (marca = referência, não conteúdo).
