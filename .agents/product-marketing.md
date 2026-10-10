---
name: product-marketing
version: 1.0.0
date: 2026-10-06
project: mdreader (workspace Rust + Android media-shell / spec 001-android-media-shell)
status: greenfield / dogfooding — projeto pessoal, 0 faturamento, sem distribuição comercial ainda
---

# Product Marketing Context — mdreader

> Criado conforme workflow da skill `product-marketing` (instalada via `npx skills add coreyhaines31/marketingskills@product-marketing`).  
> Este arquivo é o contexto compartilhado para todas as outras skills de marketing (launch, pricing, paywalls, brand-naming).

---

## 1. Product Overview

**Uma linha:** Leitor de áudio de documentos (markdown/tech) com engine de voz exclusiva em Rust, notificação lock-screen não-invasiva e fingerprint de conteúdo (`blake3`).

**O que faz:**
- Reproduz documentos como áudio com posição salva (bookmark) baseada no conteúdo, não no nome do arquivo.
- Mostra no lock-screen apenas o título do documento + título da seção atual + progresso (sem body text, sem tempo decorrido, sem taxa, sem voz — conforme FR-028 / spec).
- Usa Rust como única autoridade para playback/posição/transport; Kotlin no Android apenas espelha (Service, FocusMirror, RoutingMirror, MediaMirror, Residency) — zero playback logic local (SC-035).
- Mantém registro diagnostic JSONL privado (30 dias / 10 MB, maior-primeiro-trim) para desenvolvimento, nunca transmitido.

**Tipo:** Aplicativo open source (não SaaS multi-tenant por enquanto).  
**Modelo de negócio:** $0 agora; futuro freemium apenas se houver pedido real de usuário (não planejado ainda — projeto pessoal / dogfooding).

---

## 2. Target Audience (ICP)

- **Primário:** Leitores de conteúdo técnico/longos que consomem enquanto dirigem, trabalham ou precisam de acesso auditivo (desenvolvedores, estudantes de engenharia, leitores de documentação).
- **Secundário:** Pessoas com baixa visão ou preferência auditiva; motoristas; consumidores de conteúdo em contextos de baixa atenção.
- **Decisores / Usuários:** O usuário é também o fundador; não há time de compras por enquanto.
- **Jobs to be done:** (1) "Quero ler docs/teses/artigos sem perder a posição." (2) "Quero saber onde estou sem abrir o app." (3) "Quero que o áudio respeite minha atenção (não invade lock-screen com texto)."

---

## 3. Problems & Pain Points

- Leitores de docs longos perdem posição quando o arquivo é movido/renomeado (solução: fingerprint `blake3`).
- Notificações de áudio de texto frequentemente mostram body text no lock-screen (invasão de privacidade / distração) — este produto rejeita isso explicitamente.
- Apps de áudio de texto frequentemente computam posição em Kotlin/Java e não têm engine única de referência — este produto mantém Rust como autoridade.
- Falta de registro diagnóstico privado para investigar falhas de áudio/foco/routing — este produto escreve JSONL, nunca transmite.

---

## 4. Differentiation (vs concorrentes indiretos/diretos)

- **Engine única:** Rust (`crates/player`, `crates/session`) — não Kotlin; não silenciosa resample.
- **Fingerprint de conteúdo:** `blake3` — posição sobrevive a renome/movimento.
- **Notificação lock-screen:** Título + seção atual + progresso, **sem body text, sem tempo decorrido, sem taxa, sem voz** — respeita privacidade e atenção.
- **Zero playback logic em Kotlin:** Service apenas espelha; SC-035 enforça.
- **Focus / Routing mirrors:** Classificado (Ganho / Perda / Permanente / Temporário), com pausa bounded de 3 s; nunca auto-resume sem escolha do usuario.
- **Rate ladder fechada:** `[0.75, 1.0, 1.25, 1.50, 1.75, 2.0, 4.0]`; nenhum `Other`/`Unknown`.
- **Diagnostic record:** Append-only, 30 dias / 10 MB, não transmitido, sem texto de documento.

---

## 5. Brand / Nome

**Workspace / repo técnico:** `mdreader` (mantido; não renomeado — confirma opção do usuário).

**Marcas de divulgação (geradas via workflow `brand-naming`, avaliadas):**
- **Leitora** — clareza alta, forte pt-BR, risco médio (verificar domínio/app).
- **DocSona** — criatividade, híbrida doc+som, risco baixo.
- **Audileitura** — palavra única profissional, bom para app store.
- **Sona** — excelente memória/som; **risco ALTO** porque é campeã de LoL (associação ao jogo); só recomendada se o público-alvo for dev/gamer e a marca for intencionalmente ligada a esse universo.
- **Ressona** — melhor sonoridade para áudio/voz (ressonância); clareza média; precisa de contexto (landing explicativa).
- **Sona Reader** — combo direto; pode ser visto como "revista/leitura da Sona"; risco médio-alto de confusão.

**Decisão pendente:** o usuário gostou de `Sona`/`Ressona`; recomenda-se testar disponibilidade (GitHub, domínio `.org`/`.dev`, F-Droid/app store) antes de registrar. **Não registrar ainda** — projeto ainda em greenfield.

---

## 6. Competitive Landscape

- **Diretos (leitores de áudio de docs):** pouco consolidado; apps genéricos de "text-to-speech" existem, mas poucos com engine única Rust, fingerprint de conteúdo e notificação lock-screen respeitosa.
- **Secundários (screen readers / acessibilidade):** NVDA, TalkBack, VoiceOver — não competem diretamente, são complementares; este produto pode ser visto como "leitura de conteúdo técnico" em vez de "acessibilidade de sistema".
- **Indiretos (áudio de conteúdo):** podcasts, audiolivros (Audible) — diferente porque este é leitura de documento original, não narração produzida.

---

## 7. Positioning & Tone

- **Categoria:** Leitor de áudio de documentos técnicos (não podcast, não audiolivro).
- **Posicionamento:** Técnico e confiável; não invasivo; respeita a atenção do leitor.
- **Tom:** Direto, sem hype; para quem precisa de acesso auditivo a conteúdo longo.
- **Visual (direcionado pela `brand-identity`):** Minimalista; paleta monocromática + acento frio (azul-acinzentado / verde-água); tipografia clara; logotipo simples (símbolo de onda/nota + documento).
- **Evitar:** cores quentes (entretenimento); logotipos complexos; teleprompter; body text no lock-screen.

---

## 8. Business Model & Monetization

- **Hoje:** $0, código aberto, sem distribuição comercial, sem faturamento.
- **Distribuição:** F-Droid (confirmado pelo usuário) — não Play Store. Vantagens: sem taxa de 30% Google, alinhado ao open source, distribuição via repositório F-Droid (metadados, build, manifesto, descrição, screenshots necessários).
- **Futuro (apenas se pedido real):** Freemium com 2–3 tiers (ex.: voz premium, bookmark sincronizado, exportação de record, API). Nenhum preço definido agora — `pricing` recomenda "bet para aprender" apenas após feedback real.
- **Doações / Sponsorship:** Aceitável para nicho técnico; não priorizado.

---

## 9. Launch Strategy (não agora — apenas quando gate 9 passar)

- **Status:** Não lançar. Spec `001-android-media-shell` ainda em implementação; build Android (`cargo tauri android build --target aarch64`) ainda não garantido.
- **Quando lançar:** Quando houver pelo menos uma demonstração de áudio real com notificação lock-screen funcionando em dispositivo.
- **Canais (framework ORB da `launch`):**
  - **Owned:** GitHub repo + README + GitHub Discussions + landing simples (GitHub Pages). Lista de e-mail leve para atualizações (não marketing agressivo).
  - **Rented:** 1–2 plataformas onde devs/leitores de docs são ativos (provavelmente GitHub / Reddit / Twitter/X). Não depender só delas.
  - **Borrowed:** Guest post técnico explicando por que Rust é autoridade única para playback; participação em comunidades de acessibilidade; podcasts técnicos (se aplicável).
- **Momento:** Lançamentos incrementais (v0.1.0 → v0.1.1 com focus mirror → v0.2 com record). Não big bang.

---

## 10. Paywall / Upgrade (se futuro freemium)

- **Ponto de upgrade:** Apenas após primeira leitura concluída com sucesso ("aha moment" — usuário já viu notificação, posição salva, áudio funcionando, focus respeitado).
- **Componentes:** Headline focado no benefício (ex.: "Salve bookmarks entre dispositivos"), demonstração de valor, comparação simples, preço claro, respeito ao "não" (não bloquear leitura gratuita).
- **Regra:** Não criar paywall antes de ter usuário real que peça.

---

## 11. Notes / Decisões confirmadas pelo usuário

- `mdreader` permanece como nome técnico do workspace/repo — confirmado.
- Nenhuma distribuição Play Store — confirmado; F-Store/F-Droid assumido (aguardando confirmação do usuário se é F-Droid).
- `.md` necessários: este arquivo criado; outros (`.agents/brand-context.md`, `product-marketing.md` atualizado) podem ser adicionados conforme necessidade.
- Projecto pessoal / dogfooding; 0 faturamento; monetização não é prioridade — confirmado.
- Nome preferido explorado: `Sona`, `Ressona`, `Sona Reader`; avaliação feita; decisão de registro pendente.

---

*Versão: 1.0.0 | Criado: 2026-10-06 | Baseado em: specs/001-android-media-shell, AGENTS.md, SKILL.md de product-marketing / pricing / launch / paywalls / brand-naming / brand-identity*
