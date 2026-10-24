# Idea Research: Adotar o DESIGN-SONA para o app

- **Slug**: design-son-adoption
- **Created**: 2026-10-22
- **Evidence confidence (overall)**: medium

## Users & Demand

- O problema é descrito apenas como uma want declarada no input ("Adotar o DESIGN SYSTEM definido em @DESIGN-SONA/ para o app.") — não há métrica de uso, ticket de suporte, entrevista ou pedido de usuário externo no repositório que justifique a adoção agora. A demanda sinalizada é do mantenedor/autor do projeto, via intenção especificada, não comportamento observado. [source: intake.md Origin, marcado NEEDS CLARIFICATION; nenhum arquivo de analytics/metrics/support encontrado no repositório] (confidence: low) (assumption)
- O único usuário evidenciado do projeto é o mantenedor próprio (dogfooding pessoal, $0 revenue, distribuição ainda não definida), conforme `.agents/product-marketing.md` e o assessment de `ressoa-mobile-app`. [source: `.agents/product-marketing.md` v1.0.0 §§1–2; `.specify/assessments/ressoa-mobile-app/research.md` Users & Demand] (confidence: high) (cited)
- Os jobs-to-be-done do mantenedor driver-leitor (ler documentos longos com tela apagada, não perder posição, interrupções seguras sem olhar a tela) já estão cobertos pelos P1 (US1–US3) do spec `001-android-media-shell`, não dependem do design system para existir. [source: `specs/001-android-media-shell/spec.md` US1–US3 + clarificações 2026-10-01 a 05] (confidence: high) (cited)

## Prior Art

- Interno — o design system DESIGN-SONA/ já está **especificado e homologado** (v2.0.0 2026-10-16; marca v2.1.0; UI multi-modo v2.2.0 2026-10-17), mas não implementado como código no app: existem apenas `DESIGN-SONA/*.md`, `preview.html` e assets em `DESIGN-SONA/assets/`. [source: `DESIGN-SONA/DESIGN-SONA.md` §1 header; `DESIGN-SONA/brand.md` §6 histórico; leitura do diretório `DESIGN-SONA/observed`] (confidence: high) (cited)
- Interno — a feature `ressoa-mobile-app` (assessment GO, Option A) já decidiu o escopo do dogfood build: notificação-first, uma única voz offline, nada publicado, e DESIGN-SONA superfícies além do loop notificável estão explicitamente fora de escopo (concept.md §Out of Scope). [source: `.specify/assessments/ressoa-mobile-app/concept.md` §Out of Scope; `decision.md` Verdict & Rationale] (confidence: high) (cited)
- Interno — a feature `004-ressoa-dogfood-build` tem spec, plan e tasks T001–T060 concluídos, incluindo T044 (DESIGN-SONA token styling de MediaStyle + in-app grid) marcado [X], mas T051/T052 (on-device) ainda não executados. [source: `specs/004-ressoa-dogfood-build/tasks.md` Phase 6 T044, Phase 8 T051/T052] (confidence: high) (cited)
- Externo — a inspiração do DESIGN-SONA vem da personagem Sona de League of Legends (Universe:Sona, Sona/Audio, brand.riotgames.com), mas o design system declara explicitamente que não é cópia de jogo nem marca de entretenimento. [source: `DESIGN-SONA/DESIGN-SONA.md` §1 "Não é"; referências §Referências do design Sona] (confidence: medium) (cited)
- Não há estudo de concorrência formal no repositório comparando o mdreader com leitores de áudio existentes (TalkBack, VoiceOver, audiobooks, etc.) — apenas claims não testados no product-marketing. [source: `.agents/product-marketing.md` §4; `RELATORIO_BRAND_PRODUTO.md` §6, marcados ASSUMPTION] (confidence: low) (assumption)

## Market & Context

- A estratégia de lançamento do projeto é explicitamente "do not launch yet" — lançar somente após áudio real + demonstração de notificação no dispositivo. A adoção do DESIGN-SONA não muda essa estratégia por si só. [source: `.agents/product-marketing.md` §§8–9; `RELATORIO_BRAND_PRODUTO.md` §4] (confidence: high) (cited)
- Distribuição pretendida é F-Droid (não Play Store), sem taxa de 30%, alinhado a open source. [source: `.agents/product-marketing.md` §§8–9; `RELATORIO_BRAND_PRODUTO.md` §4] (confidence: high) (cited)
- Monetização é $0 agora, freemium apenas se houver demanda real posteriormente; nenhum paywall antes de um usuário pedir. [source: `.agents/product-marketing.md` §§8, 10] (confidence: high) (cited)
- O custo de não fazer nada: o shell (001) permanece um mecanismo verificado em host sem saída de áudio real para o usuário (SC-044 deferido, gate-8-full deferido); DESIGN-SONA permanece um sistema de papel; o mantenedor continua dogfooding apenas via CLI/host. [source: `specs/002-fr-024-synthesis-adapter/spec.md` US1 rationale; `specs/001-android-media-shell/plan.md` Summary] (confidence: high) (cited)
- Não há dimensionamento de mercado nem dados de tendência sobre leitores de áudio para condução — claims de mercado são inferência, não evidência do repositório. [source: ASSUMPTION] (confidence: low) (assumption)

## Data & Constraints

- Plataforma floor: Android 10 (API 29) mínimo até Android 14+ (restrições FGS-start); dispositivo de referência Redmi 2201117TY (Android 13/API 33) é normativo para bounds de latência. [source: `specs/001-android-media-shell/spec.md:39`; `plan.md` Technical Context/Reference device] (confidence: high) (cited)
- Áudio: engine entrega mono f32 a 22050 Hz declarado; mismatch é `EngineError` nomeado, nunca reamostragem silenciosa. [source: `spec.md:27,75`; `plan.md` Constraints; `constitution.md` Audio Pipeline Contracts] (confidence: high) (cited)
- Taxa: ladder fechado de 7 degraus [0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]; default 1.0x. [source: `spec.md` clarificações; `plan.md` Performance Goals] (confidence: high) (cited)
- Notificação: título + seção atual + progresso apenas — nunca body text, elapsed/remaining, taxa, voz; não-dismissable enquanto toca; pause é a rota para dismissão. [source: `spec.md` clarificações; `constitution.md` Notification rules] (confidence: high) (cited)
- Design-SONA: dark-only (`#0B0F1A`), light mode não especificado e exigiria re-verificação completa de contrastes. [source: `DESIGN-SONA/tokens.md` §5 Variantes; `DESIGN/risks.md` R7] (confidence: high) (cited)
- Regra de conjunto fechado (constituição §VII): qualquer nova cor exige edição coordenada, justificativa semiótica e bump de versão; verde proibido no semáforo do player; amarelo exclusivo para aviso 3s; vermelho exclusivo para falha permanente. [source: `constitution.md` §VII; `DESIGN-SONA/tokens.md` §2; `DESIGN/risks.md` R6] (confidence: high) (cited)
- Registro diagnóstico: JSONL append-only, app-private, 30 dias + 10 MB, never transmitted, no in-app surface, no document text. [source: `spec.md` FR-039; `plan.md` Storage/Constraints] (confidence: high) (cited)
- Consent/soberania: Edge endpoint é consent-gated fallback-last, nunca default, nunca via wrapper Python OpenAI-compatível. [source: `constitution.md` Princípio XI] (confidence: high) (cited)
- Supply chain: latest stable, crates.io-only, `--locked`, `cargo deny` + `cargo audit` fail-closed. [source: `constitution.md` Princípio IV; `scripts/verify.sh` gate 5] (confidence: high) (cited)

## Evidence Against the Idea

- A adoção do DESIGN-SONA como UI Svelte completa (Lente Sinestésica, Cockpit, Terminal Suíço, Matriz de Telemetria) amplia o escopo muito além do que o assessment `ressoa-mobile-app` aprovou (Option A: notificação-first, single-voice sideload, nada publicado). Isso reintroduz o risco de Option B ("shipping breadth nobody asked for") sem evidência de usuário externo. [source: `.specify/assessments/ressoa-mobile-app/concept.md` Option B trade-offs; `decision.md` go only for Option A scope] (confidence: high) (cited)
- O áudio real ainda não existe (feature `fr-024-synthesis-adapter` decidida mas não implementada; `SC-044` e `gate-8-full` deferidos). Um design system rico em áudio/earcons sem áudio real é superfície sem produto — o custo de inação do assessment de síntese aplica-se espelhado: investir em UI antes de som não retorna valor externo. [source: `.specify/assessments/fr-024-synthesis-adapter/problem.md` Cost of Inaction; `decision.md` Evidence Against] (confidence: high) (cited)
- Os 48 procedimentos on-device (T051/T052) da feature 004 ainda não foram executados — não há número medido de latência de primeiro áudio, inteligibilidade de voz no carro, ou comportamento OEM (Samsung One UI, Xiaomi HyperOS) que valide se a superfície DESIGN-SONA sobrevive em hardware real. [source: `specs/001-android-media-shell/VERIFICATION.md` §5 on-device trials DEFERRED; `specs/004-ressoa-dogfood-build/tasks.md` T051/T052 unchecked] (confidence: high) (cited)
- A separação marca vs. operação (verde proibido no player, amarelo só aviso, vermelho só falha) é uma restrição que a adoção deve respeitar estritamente; qualquer deslize viola o Princípio VII da constituição. [source: `constitution.md` §VII; `DESIGN/risks.md` R6; `DESIGN-SONA/tokens.md` §2] (confidence: high) (cited)
- Não há evidência de que algum usuário além do mantenedor queira este app — o "cost of doing nothing" não se torna um argumento a favor da adoção do design system sem um primeiro usuário não-autor. [source: `.specify/assessments/ressoa-mobile-app/research.md` Evidence Against #1] (confidence: medium) (cited)

## Gaps & Open Questions

- [NEEDS CLARIFICATION: Quais superfícies do DESIGN-SONA estão em escopo para esta adoção — apenas tokens/marca em superfícies existentes, ou implementação da UI Svelte completa (Lente Sinestésica, Cockpit, Terminal Suíço, Matriz de Telemetria)?]
- [NEEDS CLARIFICATION: A adoção é uma feature nova ou uma emenda a specs existentes (`001-android-media-shell`, `004-ressoa-dogfood-build`, `fr-024-synthesis-adapter`)?]
- [NEEDS CLARIFICATION: Dark-only é aceitável, ou light mode é requerido? Light mode não está especificado e exigiria re-verificação completa.]
- [NEEDS CLARIFICATION: O design system depende do áudio real (`fr-024-synthesis-adapter`) para fazer sentido, ou pode ser adotado antes com stand-ins?]
- [NEEDS CLARIFICATION: Quem é o proponente e qual o trigger desta adoção agora?]
- [NEEDS CLARIFICATION: Como a adoção se relaciona com T051/T052 on-device pendentes da feature 004?]

## Sources

- `.specify/assessments/design-son-adoption/intake.md` (slug, ideia original, unknowns) — internal, cited
- `DESIGN-SONA/DESIGN-SONA.md` (v2.0.0, paleta, dupla assinatura, multi-modo UI/UX, riscos Sona) — internal, cited
- `DESIGN-SONA/brand.md` (v2.1.0, Dupla Assinatura, matriz de superfícies) — internal, cited
- `DESIGN-SONA/components.md` (componentes: NotificationSurface, PlayerView, Cockpit, OutlineEditorialDrawer, TelemetryMatrixModal, earcon) — internal, cited
- `DESIGN-SONA/tokens.md` (camada primitive/semantic/component, conjunto fechado, variantes dark-only) — internal, cited
- `DESIGN-SONA/accessibility.md` (earcon gramática Sona, synesthesia, muda como dignidade, testes blind/colorblind) — internal, cited
- `DESIGN-SONA/risks.md` (R1–R12, incluindo R9 confusão LoL, R11 inviolabilidade lock-screen, R12 Material You monocromático) — internal, cited
- `.specify/assessments/ressoa-mobile-app/{intake,research,problem,concept,decision}.md` (verdict GO Option A, escopo não-publicado, DESIGN-SONA fora de escopo além do notificável) — internal, cited
- `.specify/assessments/fr-024-synthesis-adapter/{intake,research,problem,concept,decision}.md` (verdict GO Option A adaptador mínimo, áudio real não implementado) — internal, cited
- `specs/001-android-media-shell/{spec,plan,VERIFICATION}.md` (P1 US1–US3, notificação, áudio, gates, on-device deferred) — internal, cited
- `specs/004-ressoa-dogfood-build/{spec,plan,tasks}.md` (T044 DESIGN-SONA styling concluído, T051/T052 on-device pendentes) — internal, cited
- `.agents/product-marketing.md` v1.0.0 (usuário único, estratégia do-not-launch-yet, F-Droid, $0) — internal, cited
- `RELATORIO_BRAND_PRODUTO.md` (escolha marca Ressoa/ressona.dev, estratégia lançamento) — internal, cited
- `.specify/memory/constitution.md` v1.12.0 (Princípios IV, VII, X, XI; Rust Code Rigor; Verification Gates; Exceptions Log) — internal, cited
- `scripts/verify.sh` (gates 1–9, gate-8-full DEFERRED owner=synthesis) — internal, cited
- Não foram feitas fetches de URL externas para esta pesquisa (nenhuma URL no input; codebase pointer local). URL Trust Policy branch: N/A (no fetch).
