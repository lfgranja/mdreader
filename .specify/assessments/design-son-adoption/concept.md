# Concept: Adotar o DESIGN-SONA para o app

- **Slug**: design-son-adoption
- **Created**: 2026-10-22
- **Recommended option**: Option A

## Options

### Option A — Tokens, marca e earcons nas superfícies existentes (notificação-first)
- **Sketch**: Aplicar os tokens de paleta do DESIGN-SONA (`--sona-*`), a dupla assinatura (Concha Harmônica "R" no cabeçalho in-app; Ressonador Canônico no mostrador/diagnóstico) e a gramática de earcons Sona (Adagio, Andante, Allegro, Sforzando, Ostinato, Unison) sobre as superfícies já existentes da casca de mídia: notificação MediaStyle, StatusView in-app (grade 2-campo) e earcons sintetizados em Rust a 22050 Hz mono f32. Não criar a UI Svelte completa (Lente Sinestésica, Cockpit Eyes-Free, Terminal Editorial Suíço, Matriz de Telemetria). Manter dark-only. O usuário vê apenas título, seção e progresso na notificação; ouve earcons distintos por ação; opera sem olhar a tela.
- **Appetite**: small (dias) — orçamento, não estimativa.
- **Trade-offs**: Ganha identidade visual e confirmação auditiva aplicadas ao usuário único evidenciado (mantenedor-driver) com baixo risco de escopo; respeita o escopo já aprovado para `ressoa-mobile-app` Option A (notificação-first, single-voice, nada publicado). Sacrifica a riqueza visual da UI multi-modo (microfita sinestésica, halo cockpit, sumário suíço, matriz de telemetria) e a distinção de marca em telas além da notificação. Risco: sobrepor-se ao já marcado [X] da T044 (token styling de MediaStyle + in-app grid) na feature 004 — precisa verificar o que realmente resta para fazer. Risco: earcons sem áudio real (stand-ins) podem soar diferente da síntese real.
- **Rabbit holes**: construir a UI Svelte completa aos poucos (desliza para Option B); adicionar light mode (não especificado, exigiria re-verificação completa); tentar validar em múltiplos OEM antes de ter um usuário não-autor.

### Option B — UI multi-modo completa (Lente Sinestésica, Cockpit, Terminal Suíço, Matriz de Telemetria)
- **Sketch**: Implementar toda a composição multi-modo especificada no DESIGN-SONA v2.2.0 como UI Svelte no app: player padrão com microfita sinestésica e Concha Harmônica, modo Cockpit Eyes-Free com halo 180dp e aviso temporal de pausa, drawer sumário suíço com ribbon-marker, modal de telemetria com selo do Ressonador e métricas do pipeline Rust.
- **Appetite**: large (meses) — orçamento, não estimativa.
- **Trade-offs**: Ganha a superfície visual e sonora completa do design system, pronta para demonstração e distribuição futura. Sacrifica foco e velocidade; expande o escopo muito além do aprovado para o dogfood build (notificação-first, nada publicado); reintroduz o risco de Option B ("shipping breadth nobody asked for") sem evidência de usuário externo; exige áudio real para fazer sentido (still deferred em fr-024).
- **Rabbit holes**: animações de envelope RMS, microfita 4px ondulante, Doppelrand, frosted glass, detecção de mobilidade/condução, detecção de gesto de pinça, matriz de Chladni, múltiplos OEMs para validação visual, light mode.

### Option C — Não adotar agora (manter DESIGN-SONA como spec de papel)
- **Sketch**: Não escrever código de adoção. Manter DESIGN-SONA/ como especificação homologada (v2.0/v2.1/v2.2) sem superfície no app. Revisitar a adoção apenas quando houver áudio real implementado (fr-024), on-device trials concluídos (T051/T052) e um primeiro usuário não-autor evidenciado.
- **Appetite**: zero (não há trabalho de adoção agora).
- **Trade-offs**: Ganha disciplina e evita compromisso prematuro (distribuição, suporte, exposição de marca). Sacrifica momentum: o mantenedor-driver continua sem identidade visual aplicada nem earcons Sona; o investimento em tokens, paleta, dupla assinatura e gramática sonora não retorna valor para nenhum usuário. O design system permanece um sistema de papel indefinidamente.
- **Rabbit holes**: nenhum técnico; o risco estratégico é a deferral indefinida — "not yet" se torna a resposta permanente sem um gatilho de re-entrada.

## Recommendation

**Option A.** É a única opção que serve aos objetivos do problema na ordem certa: aplica tokens, dupla assinatura e earcons Sona ao usuário único evidenciado (mantenedor-driver) com um appetite pequeno (dias), dentro do escopo já aprovado para `ressoa-mobile-app` (notificação-first, single-voice, nada publicado), e respeita todos os non-goals (sem UI Svelte completa, sem distribuição pública, sem light mode, sem Android Auto, sem diagnóstico in-app, sem monetização). Option B não limpa nenhum barra que Option A não limpe primeiro — compra largura e ônus de suporte sem demandante externo. Option C é a escolha segura como segunda opção, mas falha no teste de custo da inação: o design system continua sem código no app.

## Out of Scope (for the recommended option)

- UI Svelte completa (Lente Sinestésica, Cockpit Eyes-Free, Terminal Editorial Suíço, Matriz de Telemetria) — se adotada, fica para uma decisão posterior com escopo próprio.
- Qualquer distribuição pública (F-Droid listing, Play Store, landing, changelog, canal de suporte).
- Light mode — dark-only é o único modo especificado no DESIGN-SONA.
- Android Auto, integração com head-unit de carro, plataformas não-Android.
- Engine failover, segunda voz, nuvem, credenciais de provedor, fallback unattended.
- Tela de diagnóstico in-app ou export do registro diagnóstico pelo usuário.
- Monetização, paywalls, pricing tiers.
- Validação em múltiplos OEMs (Samsung One UI, Xiaomi HyperOS, Pixel) — deixada para a decisão de distribuição posterior.

## Assumptions to Validate

- Dark-only é aceitável para esta adoção (light mode não especificado e exigiria re-verificação completa). [research.md Data & Constraints]
- A adoção pode prosseguir com áudio stand-in (earcons em Rust a 22050 Hz mono f32 não dependem de síntese real para existir, apenas para fazer sentido completo). [research.md Evidence Against #2]
- T044 (token styling de MediaStyle + in-app grid na feature 004) já cobre parte do terreno — verificar o que realmente resta para evitar trabalho duplicado. [research.md Prior Art #3]
- O usuário único evidenciado (mantenedor-driver) justifica a adoção em escopo dogfood, sem necessidade de usuário não-autor nesta fase. [research.md Users & Demand #2]
- Earcons sintetizados em Rust a 22050 Hz mono f32 são distintivos em blind-test sem olhar a tela (SC-008) — métrica a ser validada, não assumida.
