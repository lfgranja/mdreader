# Relatório Consolidado: Marca, Produto e Monetização para mdreader
> Projeto: mdreader (workspace Rust + Android media-shell)  
> Data: 2026-10-06  
> Contexto: projeto open source em fase greenfield (spec 001-android-media-shell não finalizada; código ainda sendo estruturado).

---

## 1. Identidade atual do projeto (do código / specs)

| Aspecto | Evidência |
|---|---|
| **Workspace / repo** | `mdreader` (dir `/home/luis/development/mdreader`) |
| **Crates Rust** | `player`, `session`, `diag`, `engine-core`, `doc-structure` (nenhum nome de marca) |
| **App Android / Kotlin** | `package app.mdreader` (PlaybackService, FocusMirror, RoutingMirror, MediaMirror, Residency) |
| **Tauri / web** | `<title>mdreader — Playback Shell</title>` |
| **Spec ativa** | `specs/001-android-media-shell/` — shell de áudio para leitura de documentos (markdown?) com notificação lock-screen, engine de voz (22050 Hz, pt-BR voices), fingerprint `blake3`, bookmark persistente, focus mirror, routing mirror |
| **Público estimado** | Leitores de documentos digitais (possivelmente motoristas, acessibilidade, consumo de conteúdo técnico escrito); nicho de áudio de documentos |
| **Diferenciais técnicos** | Notificação lock-screen com título + seção atual + progresso (sem body text, sem tempo decorrido); engine-computação apenas em Rust; zero playback logic em Kotlin (SC-035); diagnostic record JSONL 30 dias / 10 MB; rate ladder fechada; engine IDs no registry da constitution |

**Conclusão rápida de identidade:** ainda não existe uma **marca de produto** separada — apenas nome técnico `mdreader` usado como package/name de workspace e app. Isso é típico de greenfield, mas já começa a criar dependência (package Kotlin, spec paths, Tauri title, repo name).

---

## 2. Resposta direta: "Precisamos de um nome de produto/brand/marca?"

**Sim, mas com timing e escopo definidos — não precisamos de uma marca de luxo agora, precisamos de um nome de referência antes do lançamento público.**

Como o projeto é **open source de nicho técnico** (leitura de áudio de documentos com engine de voz, focus, notificação lock-screen), a marca serve a três propósitos distintos:

1. **Referência técnica** — já existe (`mdreader`). Manter como nome do workspace/repo é aceitável, mas não é uma marca de divulgação.
2. **Referência de lançamento / divulgação** — quando for para GitHub público, Product Hunt (se aplicável), domínio, app store, blog, comunidade — um nome de marca separa o "produto que as pessoas falam" do "pacote de código".
3. **Referência de monetização / freemium (se houver)** — se no futuro houver uma versão paga (voz premium, bookmark cloud, múltiplas engines, sincronização), a marca precisa ser consistente em páginas de preço, paywall e e-mail.

**Regra de ouro extraída das skills:** a `product-marketing` indica que a primeira ação é criar `.agents/product-marketing.md` (posicionamento, ICP, problema, diferenciação). A `brand-naming` diz que não se deve gerar nomes sem contexto; a `launch` diz que uma marca clara é necessária antes de qualquer canal "borrowed" (podcast, colaboração, imprensa). A `pricing` confirma que para open source o preço inicial é "um bet para aprender", não uma decisão de marca — mas a marca ainda precisa existir para a página de adoção.

---

## 3. Achados das skills de Brand (brand-naming, brand-identity, brandkit, brand-guidelines)

> Fontes: `/home/luis/.agents/skills/brand-naming/SKILL.md`, `brand-identity`, `brandkit`, `brand-guidelines`; também `skills find brand` (heuristic: 347K installs `brandkit`, 100K `brand-guidelines`, 1.8K `brand-naming`).

### 3.1 Quando criar / atualizar
- **Antes do lançamento público** (não antes de escrever código). O `brand-naming` recomenda: se o usuário tem uma descrição breve e pergunta "o que chamar", vá para **Geração**; se já tem nomes, vá para **Avaliação**. No nosso caso, temos descrição do spec, mas nenhum candidato — portanto, **geração é o modo correto**.
- O `brand-identity` exige contexto: nome, categoria, personalidade (3–5 palavras), audiência, posicionamento (premium / acessível / técnico), referências visuais, o que evitar.

### 3.2 Direções de naming recomendadas (extraídas do workflow brand-naming)

Para um produto de **leitura de áudio de documentos / acessibilidade / nicho técnico**, as direções estratégicas são:

| Direção | Lógica | Exemplos (não definitivos) | Risco |
|---|---|---|---|
| **Descritiva técnica** | Diz exatamente o que faz; fácil de buscar | `DocSona`, `LeituraAud`, `VozDoc`, `AudioDoc`, `MarkdownVoice` | Pode ser genérico; risco de conflito com nomes de domínio |
| **Abstrata / poética** | Evoca som, voz, leitura, documento; mais memorável | `Sona` (som), `EcoLeitura`, `Aurora` (luz/voz), `VentoLecto`, `Murmúrio` | Requer mais esforço de posicionamento; pode ser confuso sem contexto |
| **Híbrida / prefixo-sufixo** | Combina palavra técnica + palavra de voz | `Leitora` (pt-BR), `DocLê`, `VozLeitura`, `Audileitura`, `LeituraViva` | Equilíbrio; mais fácil de verificar disponibilidade |
| **Personal / marca do autor** | Se for projeto pessoal do fundador (luis?) | `LuisLeitura`, `MdLê` | Limita expansão; mas é aceitável para open source de nicho |

**Recomendação do workflow:** gerar 3–5 direções com 3–4 nomes cada, depois avaliar em tabela (clareza, memória, som, visual, risco, nota geral). **Não devo impor uma escolha agora**, mas posso propor uma estratégia de seleção.

### 3.3 Verificação de disponibilidade (obrigatório antes de registrar)

O `brand-naming` e `brand-identity` destacam: verifique **GitHub org/repo**, **domínio** (se for usar .com/.org/.dev), **app store** (Android / iOS), **PyPI / crates.io / npm** se aplicável, **INPI / USPTO / EUIPO** se for registrar. Para open source de nicho, a ordem de prioridade é:

1. **GitHub repo / org** (já ocupado? `mdreader` pode ser genérico — verifique se há repos com esse nome)
2. **Domínio `.org`** (para open source) ou `.dev` / `.app`
3. **Nome do package Kotlin** (`app.mdreader`) — se mudar a marca, pode manter o package técnico e trocar apenas o `label` no manifesto / Tauri
4. **App Store / Play Store** — o nome deve ser único o suficiente para busca

> **Nota prática:** se você quer manter `mdreader` como nome técnico do repo/workspace, é aceitável. Mas se quiser divulgar como "Leitora" (exemplo), o repo pode ser `mdreader/leitura` ou `leitura` com o workspace renomeado gradualmente.

### 3.4 Posicionamento recomendado (baseado no spec + brand-identity)

- **Categoria:** Leitor de áudio de documentos / acessibilidade digital / produtividade de leitura
- **Personalidade:** Técnica, confiável, simples, respeitosa com a atenção do leitor (não invasiva — o spec rejeita teleprompter no lock-screen, rejeita body text, rejeita tempo decorrido; a marca deve refletir essa "não-invasão")
- **Tom:** Direto, sem hype; para quem lê enquanto dirige, trabalha, ou precisa de acesso auditivo a conteúdo técnico
- **Visual (brand-identity):** Minimalista, tipografia clara, paleta monocromática com acento de cor fria (azul-acinzentado, verde-água) — refletindo "técnico" e "calmo"; evitar cores quentes de entretenimento; logotipo pode ser letra + símbolo simples (ex: nota ou onda + documento)

---

## 4. Achados das skills de Produto / Marketing / Monetização (product-marketing, pricing, launch, paywalls)

### 4.1 Product-marketing (contexto de produto)

- **Arquivo esperado:** `.agents/product-marketing.md` (não existe ainda).
- **Seção obrigatória antes de qualquer outra skill:** Produto, audiência, problemas, diferenciação, concorrência.
- **Aplicação ao mdreader:**
  - **Problema:** Leitores de documentos longos (markdown, PDFs, artigos) precisam de uma forma de consumo auditiva sem perda de posição, com notificação clara no lock-screen, sem invasão de privacidade (não mostra texto do documento).
  - **Diferenciação:** Rejeita teleprompter, rejeita body text no lock-screen, usa fingerprint `blake3` (não nome de arquivo), engine-única-autoridade (Rust, não Kotlin), focus mirror, diagnostic record privado.
  - **ICP:** Leitores de conteúdo técnico que consomem em contextos de baixa atenção (motoristas, trabalhadores manuais, pessoas com baixa visão), desenvolvedores que querem ler docs enquanto trabalham.

**Recomendação:** crie `.agents/product-marketing.md` agora, antes de qualquer campanha. Isso é o que `product-marketing` exige como primeiro passo.

### 4.2 Pricing / Monetização (skill `pricing` — instalada)

- **Modelo para open source de nicho:** não há preço obrigatório. O `pricing` recomenda: "On day one you don't have a price to optimize — you have a bet to place." Para open source, o "preço" pode ser:
  - **$0** — código livre, página de adoção clara, README excelente, build simples (`cargo build --workspace`).
  - **Freemium (futuro)** — se houver uma versão "Pro" com features pagas (múltiplas vozes premium, sincronização de bookmark entre dispositivos, exportação de diagnostic record, API de engine), o preço deve ser definido apenas depois que usuários reais pedirem.
  - **Doações / sponsorship / GitHub Sponsors** — aceitável para projetos de nicho técnico.
- **Métrica de valor:** não é "por usuário" neste caso (não é SaaS multi-tenant necessariamente), mas pode ser "por documento processado" ou "por hora de áudio" se futuramente houver serviço de cloud.
- **Regra de ouro:** não ancorar no preço do concorrente; ancorar no valor percebido (tempo economizado, acesso auditivo a conteúdo que antes era apenas visual).

### 4.3 Launch (skill `launch` — instalada)

- **Framework ORB:** Owned (GitHub repo, blog, e-mail, comunidade Discord/Slack), Rented (Twitter/X, LinkedIn, Reddit, YouTube), Borrowed (podcasts de acessibilidade, blogs de leitores, comunidades de desenvolvimento Rust/Android).
- **Para open source de nicho técnico:**
  - **Owned:** repositório GitHub com README claro, releases com changelog, uma landing page simples (GitHub Pages), lista de e-mail para atualizações (não para marketing agressivo).
  - **Borrowed:** participar de podcasts ou artigos sobre "leitura auditiva de docs"; colaborar com projetos de acessibilidade (ex: screen reader communities); escrever um post técnico explicando por que Rust é a autoridade única para playback (reflete o spec SC-035).
- **Momento ideal:** somente quando o spec 001 estiver implementado, o app rodar em dispositivo (gate 9 do AGENTS.md: `cargo tauri android build --target aarch64`), e houver pelo menos uma demonstração de áudio real.
- **Checklist (resumo da skill):** definir canal owned (1–2), escolher 1–2 plataformas rented onde a audiência está (provavelmente GitHub Discussions / Reddit / Twitter para devs), planejar 2–3 borrowed (guest posts / podcasts), criar waitlist/early access se quiser exclusividade, construir momentum com releases incrementais (não um "big bang").

### 4.4 Paywalls / Upgrade (skill `paywalls` — instalada)

- **Quando usar:** apenas se houver modelo freemium. O `paywalls` diz: "Value Before Ask" — o usuário deve sentir valor antes de ver a tela de upgrade.
- **Para mdreader:** se no futuro houver uma versão paga, o ponto de upgrade natural é **após a primeira leitura concluída com sucesso** (quando o usuário já experimentou o "aha moment" — notificação lock-screen, posição salva, áudio funcionando, focus respeitado). Nunca antes.
- **Componentes necessários:** headline focado no benefício, demonstração de valor ("Com Pro você pode salvar bookmarks entre dispositivos"), comparação de planos, preço simples (anual vs mensal), social proof ("X leitores usam").
- **Respeito ao "não":** não bloquear a leitura se o usuário recusar; mantenha a versão gratuita funcional para construir confiança.

---

## 5. Síntese consolidada: o que fazer agora

### 5.1 Sobre a marca/nome

| Pergunta | Resposta consolidada |
|---|---|
| **Precisamos de nome agora?** | **Sim — um nome de referência** para repo, pacote Kotlin e divulgação, mas não precisa ser definitivo de luxo. `mdreader` já serve como técnico. Se quiser divulgar, crie um nome de produto separado. |
| **Qual o risco de adiar?** | Alto se você já usa `app.mdreader` no Kotlin e `mdreader` nos specs; mudar depois é refatoração de pacote, manifesto e documentação. Defina agora, mesmo que seja provisório. |
| **Que tipo de nome?** | Para nicho técnico, recomendo **híbrida / descritiva** (ex: `LeituraAud`, `DocSona`, `Leitora`) — fácil de buscar, fácil de pronunciar, reflete o propósito sem ser genérico. Evite nomes totalmente abstratos se não houver orçamento de marketing. |
| **Como escolher?** | Siga o workflow `brand-naming`: (1) contexto (já temos), (2) estratégia breve (posicionamento técnico/acessível, não invasivo), (3) gerar 3 direções × 3 nomes, (4) avaliar tabela, (5) top 5. **Não escolha por gosto — escolha por clareza + disponibilidade + som.** |

### 5.2 Sobre monetização

- **Agora:** $0. Código aberto. README em pt-BR e en (se aplicável). Build simples. Nenhum paywall.
- **Depois (pós-spec, pós-device):** testar freemium apenas se houver pedido real. A `pricing` diz: "pick a price you can learn from" — ou seja, se for colocar preço, escolha um valor que permita aprender (ex: $4.99/mês ou $29/ano) e revise após 3 meses de feedback.

### 5.3 Sobre marketing / lançamento

- **Agora:** não lancar. O `launch` é para "quando estamos prestes a enviar" (gate 9 do verificador). A fase atual é **especificação + implementação**.
- **Próximo marco:** quando `spec.md` for implementado e `build --target aarch64` passar, aí sim: criar `.agents/product-marketing.md`, definir nome de marca final, verificar domínio/GitHub, criar landing page simples, planejar lançamento com ORB (Owned primeiro, Borrowed depois).

---

## 6. Próximos passos concretos (checklist)

- [ ] **Brand / Nome (esta semana)**
  - [ ] Confirmar se `mdreader` permanecera como nome técnico do repo/workspace (recomendado: sim)
  - [ ] Se quiser divulgar com outro nome: definir estratégia breve (descritiva / híbrida / abstrata) conforme `brand-naming`
  - [ ] Gerar 3 direções × 3 nomes (exemplos para início: `Leitora`, `DocSona`, `VozDoc`, `Audileitura`, `LeituraAud`)
  - [ ] Verificar disponibilidade: GitHub org/repo, domínio `.org`, Play Store / App Store, crates/io
  - [ ] Se registrar, fazer verificação INPI / USPTO se aplicável (Brasil)
- [ ] **Produto / Contexto (antes de qualquer lançamento)**
  - [ ] Criar `.agents/product-marketing.md` com: overview, ICP (leitores de docs, motoristas, acessibilidade), problemas, diferenciação, concorrência (outros leitores de áudio, screen readers, apps de áudio de texto)
- [ ] **Monetização (após primeiro usuário real)**
  - [ ] Decidir: $0 (open source puro) ou freemium futuro (definir features pagas: múltiplas vozes, bookmark cloud, exportação de record, API)
  - [ ] Se freemium: usar `pricing` para definir 2–3 tiers apenas após feedback; usar `paywalls` apenas após "aha moment"
- [ ] **Lançamento (após gate 9)**
  - [ ] Criar landing page simples (GitHub Pages ou site estático)
  - [ ] Definir canais Owned (GitHub Discussions, e-mail simples), Rented (1–2 plataformas onde devs de audi/leitura estão), Borrowed (guest post / podcast técnico)
  - [ ] Planejar lançamento incremental (release v0.1.0 → release com notificação lock-screen → release com focus mirror) — não um big bang

---

## 7. Referências das skills utilizadas

| Skill | Caminho / Instalação | Uso neste relatório |
|---|---|---|
| `brand-naming` | `/home/luis/.agents/skills/brand-naming/` (local) | Workflow de geração/avaliação de nomes; direções estratégicas; verificação de disponibilidade |
| `brand-identity` | `/home/luis/.agents/skills/brand-identity/` (local) | Posicionamento visual, personalidade, paleta, tipografia, referências |
| `brandkit` | `/home/luis/.agents/skills/brandkit/` (local) | Criação de board de identidade visual (se necessário depois) |
| `brand-guidelines` | `/home/luis/.agents/skills/brand-guidelines/` (local) | Aplicação de diretrizes visuais (se usar marca Anthropic ou outra) |
| `product-marketing` | `/home/luis/.agents/skills/product-marketing/` (instalada via `npx skills add coreyhaines31/marketingskills@product-marketing`) | Criação de `.agents/product-marketing.md`; contexto de produto, ICP, diferenciação |
| `pricing` | `/home/luis/.agents/skills/pricing/` (instalada via `npx skills add`) | Estratégia de preços para open source / freemium; $10/$100/$1.000 rule; valor vs custo |
| `launch` | `/home/luis/.agents/skills/launch/` (instalada via `npx skills add`) | Framework ORB; momentos de lançamento; checklist de go-to-market |
| `paywalls` | `/home/luis/.agents/skills/paywalls/` (instalada via `npx skills add`) | Quando e como criar upgrade screens; respeito ao "não"; ponto de upgrade natural |

---

*Relatório elaborado com uso de múltiplos subagentes/skills conforme solicitado. Não substitui decisão final do fundador sobre nome e modelo de negócio, mas consolida o estado atual e as recomendações baseadas nas principais skills de brand, produto, marketing e monetização disponíveis.*

---

## 8. Propostas de nome (output do workflow `brand-naming` — aplicado ao contexto deste relatório)

> Modo: Geração. Contexto: leitor de áudio de documentos (markdown/tech), acesso auditivo, notificação lock-screen, engine única Rust, não-invasivo, público técnico/accessibility.

### Estratégia breve
- **Brand:** Leitor de áudio de documentos técnicos que respeita a atenção do usuário.
- **Posicionamento:** Técnico, confiável, simples; sem invasão (sem teleprompter, sem body text no lock-screen — conforme spec).
- **Direções:** (1) Descritiva técnica, (2) Híbrida voz+doc, (3) Abstrata poética.
- **Evitar:** nomes genéricos (`DocReader`), nomes de entretenimento (`PlayDoc`), nomes com som de "música" se o foco é fala/voz.

### Direções e nomes

**Descritiva:** `Leitora` (quem lê; forte pt-BR), `DocSona` (doc+som), `LeituraAud` (abreviação técnica), `AudioDoc` (direto, risco genérico).

**Híbrida:** `VozLeitura` (claro, longo), `DocVoz` (literal, positivo), `Audileitura` (palavra única, profissional), `LeituraViva` (evocativo, risco de confusão).

**Abstrata:** `Sona` (som/voz; minimalista, precisa de contexto), `Murmúrio` (leitura baixa, poético), `VentoLecto` (poético, difícil search).

### Avaliação rápida

| Nome | Clareza | Memória | Som | Visual | Risco | Nota |
|---|---|---|---|---|---|---|
| Leitora | 9 | 7 | 8 | 8 | Médio | Forte pt-BR; verificar domínio |
| DocSona | 7 | 8 | 7 | 9 | Baixo | Doc+som; curto; boa para logo |
| Audileitura | 7 | 7 | 8 | 8 | Baixo | Palavra única; soa profissional |
| Sona | 5 | 9 | 9 | 9 | Alto | Precisa de contexto; requer marketing |

### Top 5 (recomendados, ordenados)

1. **Leitora** — melhor equilíbrio clareza/memória para pt-BR; reflete propósito sem ser genérico.
2. **DocSona** — criativo; combina documento + som; visualmente forte.
3. **Audileitura** — técnico; bom para app store; pode ser marca enquanto `mdreader` fica como workspace técnico.
4. **LeituraAud** — direto; curto; bom se quiser manter `mdreader` apenas no código.
5. **Sona** — apenas se houver budget para construir contexto (landing, blog, comunidade); caso contrário, risco alto.

> **Nota do workflow `brand-naming`:** escolha por clareza + disponibilidade + som, não por gosto. **Não registre nenhum antes de confirmar GitHub, domínio, Play Store / App Store, e (se Brasil) INPI.**

---

## Atualização: disponibilidade + estratégia de uso (confirmado pelo usuário)

**Teste rápido de disponibilidade (2026-10-06):**
- `Sona` (GitHub repo): 404 -> provavelmente disponivel (mas nome de campeã de LoL — verificar conflito de marca real, nao só repo vazio).
- `Ressona`: 404 -> provavelmente disponivel.
- `Leitora`: 200 -> OCUPADO (repo existente).
- `DocSona`: 404 -> provavelmente disponivel.
- Domínios `.org`: `ressona.org` -> 301 (redireciona, pode ser registrado); `leitora.org` -> nao resolves; outros nao acessiveis.

**Estrategia confirmada pelo usuario:**
- `Sona` -> usada apenas como **nome do engine / motor interno** (ex.: "Engine Sona" dentro do workspace técnico `mdreader`). Assim guarda a referencia que voce gostou, sem arriscar confusao de marca na divulgacao.
- `Ressona` -> escolhida como **marca de divulgacao** (nome do produto aberto, landing page, F-Droid, comunidade). Justificativa: melhor sonoridade para audio/voz (ressonancia), risco medio (precisa de contexto no landing), disponivel em repos (404).
- `Leitora` -> descartada para marca devido a repo ocupado no GitHub.
- `DocSona` -> alternativa se `Ressona` nao funcionar; disponivel em repo.

**Proximo passo pratico:** se confirmar `Ressona`, crie o repo `ressona` (ou `ressona/leitura`) no GitHub, registre o dominio `.org` (verificar via registro de dominio), prepare metadados F-Droid, e mantenha `mdreader` como workspace tecnico interno.

---

## Atualização: escolha de candidato B — Sinalização (2026-10-06, confirmado)

Candidato escolhido: **B — Sinalização (CTB / DIN 1451 / Semáforo)**.
Motivo: mais próximo do SC-029 (ponto pulsante amarelo + TTL 3s) sem quebrar Android MaterialStyle / RemoteViews.
Paleta B confirmada: #0A0A0A fundo, #FFFFFF 19.8:1, #C9CDD2 12.39:1, #FFD400 amarelo exclusivo aviso 3s (13.83:1), #FF4D63 vermelho exclusivo falha (6.12:1 corrigido), #3D8BE8 azul exclusivo info (5.71:1 corrigido), #5D5D5D trilho 3.01:1 corrigido.
Regra fechada: amarelo só 3s, vermelho só falha, verde NÃO existe (repouso = ausência de cor).
Marca: ressona.dev (RDAP 404 = livre); Engine: Sona (interno, código); Workspace: mdreader (técnico); Distribuição: F-Droid.
SC-029: declarado FAIL parcial (marquee impossível em Notification/RemoteViews); solução = earcon descendente 3 notas (Rust 22050 Hz) + reason estática curto (≤40 chars pt-BR) + TTL 3s + ponto amarelo pulsante (in-app, alpha 1.2s).
Próximos passos técnicos: tokens CSS (--md-night, --md-paint, --md-warning, --md-prohibit, --md-info, --md-track); componente NotificationSurface (MediaStyle); StatusView in-app (grade 2-campo); AndroidManifest.xml (minSdk 29, mediaPlayback, POST_NOTIFICATIONS); testes obrigatórios Pixel/Samsung/Xiaomi (matrix 10/10).
