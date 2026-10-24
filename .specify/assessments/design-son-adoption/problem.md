# Problem Definition: Adotar o DESIGN-SONA para o app

- **Slug**: design-son-adoption
- **Created**: 2026-10-22
- **Inputs used**: intake.md? yes | research.md? yes | user input only no

## Problem Statement

O design system DESIGN-SONA/ existe apenas como especificação homologada (v2.0.0, marca v2.1.0, UI multi-modo v2.2.0) e documentos de referência, sem qualquer superfície implementada no aplicativo mdreader/Ressoa. O usuário único evidenciado — o mantenedor que dirige e escuta documentos longos com o celular trancado — não tem uma superfície visual e sonora coerente que aplique os tokens, earcons e a dupla assinatura especificados; o design system permanece um sistema de papel enquanto a casca de mídia (001) entrega apenas mecanismo verificado em host, sem identidade visual nem confirmação auditiva aplicada.

## Affected Users & Stakeholders

- **Users**: o mantenedor como leitor-driver dogfooder — o único usuário evidenciado no repositório; ouve documentos longos com tela apagada e precisa de uma superfície glanceable que respeite a atenção (sem body text, sem taxa, sem voz na notificação) e confirme controles por ouvido (earcons). [source: `.specify/assessments/ressoa-mobile-app/research.md` Users & Demand; `specs/001-android-media-shell/spec.md` US1–US3] (confidence: high) (cited)
- **Users**: leitores de longo conteúdo técnico (desenvolvedores, estudantes de engenharia) em situação de condução ou impossibilitados de olhar a tela — afetados da mesma forma, sem evidência de uso observado. [source: `.specify/assessments/ressoa-mobile-app/research.md` Users & Demand] (confidence: medium) (assumption)
- **Users**: pessoas com baixa visão ou preferência por áudio — afetadas onde operação eyes-free e confirmação não-visual importam. [source: `.specify/assessments/ressoa-mobile-app/research.md` Users & Demand] (confidence: medium) (assumption)
- **Users**: [NEEDS CLARIFICATION: primeiro usuário não-autor — nenhum usuário externo evidenciado; identidade e necessidades desconhecidas]
- **Stakeholders**: o mantenedor (usuário único, decididor, financiador) — define escopo, timing e compromissos de distribuição. [source: `.specify/assessments/ressoa-mobile-app/research.md` Users & Demand] (confidence: high) (cited)
- **Stakeholders**: [NEEDS CLARIFICATION: gatekeepers de distribuição e ônus de suporte — revisores do F-Droid / futuros leitores — interesse e poder de decisão desconhecidos até que "real" seja definido]

## Goals

- Uma superfície do aplicativo (notificação, player, StatusView in-app) que aplique os tokens de paleta do DESIGN-SONA (`--sona-*`), a dupla assinatura (Concha Harmônica "R" e Ressonador Canônico) e a gramática de earcons Sona de forma coerente com a especificação homologada.
- O usuário consiga operar os controles de condução (play, pause, skip, taxa) sem olhar a tela, com confirmação por earcon e indicação visual por cor+forma+posição, nunca por cor sozinha.
- A notificação MediaStyle continue mostrando apenas título do documento, seção atual e progresso — nunca body text, taxa, voz ou logotipo/marca — conforme FR-028/SC-030.
- O design system não introduza nenhuma cor fora do conjunto fechado aprovado (verde proibido no semáforo do player; amarelo exclusivo para aviso 3s; vermelho exclusivo para falha permanente), respeitando o Princípio VII da constituição.
- O app continue funcionando com ou sem som (mutismo como dignidade): earcon é redundância, não barreira de acesso.

## Non-Goals

- Implementar a UI Svelte completa (Lente Sinestésica, Cockpit Eyes-Free, Terminal Editorial Suíço, Matriz de Telemetria) se isso expande o escopo além do aprovado para o dogfood build (notificação-first, single-voice, nada publicado). [source: `.specify/assessments/ressoa-mobile-app/concept.md` §Out of Scope; `decision.md` go apenas para Option A]
- Qualquer distribuição pública (F-Droid listing, Play Store, landing, changelog, canal de suporte). [source: `.specify/assessments/ressoa-mobile-app/concept.md` §Out of Scope]
- Engine failover, segunda voz, nuvem, credenciais de provedor, fallback unattended. [source: `.specify/assessments/fr-024-synthesis-adapter/concept.md` §Out of Scope; `constitution.md` Princípio XI]
- Light mode (não especificado no DESIGN-SONA; exigiria re-verificação completa de contrastes). [source: `DESIGN-SONA/tokens.md` §5 Variantes; `DESIGN/risks.md` R7]
- Android Auto, integração com head-unit de carro, monetização, paywalls, plataformas não-Android. [source: `.specify/assessments/ressoa-mobile-app/problem.md` Non-Goals]
- Tela de diagnóstico in-app ou export do registro diagnóstico pelo usuário. [source: `.specify/assessments/ressoa-mobile-app/problem.md` Non-Goals]

## Success Metrics

- Superfície do aplicativo (notificação e StatusView in-app) renderizada com os tokens do DESIGN-SONA (`--sona-void`, `--sona-voice`, `--sona-etwahl`, `--sona-hush`, `--sona-harp`, `--sona-celeste`) sem vazamento de cor fora do conjunto fechado — verificado por inspeção de código e por trial de pureza visual (baseline: zero superfícies aplicam tokens DESIGN-SONA hoje; a ausência de qualquer token aplicado é o estado atual). (qualitative)
- Earcons Sona (Adagio, Andante, Allegro, Sforzando, Ostinato, Unison) sintetizados em Rust a 22050 Hz mono f32 e confirmados distintivos em blind-test sem olhar a tela (SC-008) — 10 de 10 por tipo de confirmação. (measurable; baseline: earcons do B existem, gramática Sona não implementada)
- Notificação MediaStyle exibe apenas título, seção e progresso; nenhum body text, taxa, voz ou marca — 10 de 10 inspeções no dispositivo de referência. (measurable; baseline: host-leg verde para pureza de superfície, device-leg DEFERRED)
- App operável com earcon desativado (cor+forma+texto+posição continuam funcionais) — verificado por trial mudo. (measurable; baseline: não testado)

## Cost of Inaction

Se isso nunca for construído, o DESIGN-SONA permanece um sistema de papel (v2.0/v2.1/v2.2 especificados, zero código no app); o mdreader/Ressoa continua sem identidade visual aplicada nem confirmação auditiva Sona; o mantenedor-driver continua usando uma casca de mídia sem superfície visual coerente; e o investimento em tokens, paleta, dupla assinatura e gramática sonora não retorna valor para nenhum usuário. [source: `.specify/assessments/fr-024-synthesis-adapter/problem.md` Cost of Inaction (espelhado para UI); `.specify/assessments/ressoa-mobile-app/research.md` Cost of Inaction] (confidence: high) (cited)

## Open Questions

- [NEEDS CLARIFICATION: Quais superfícies do DESIGN-SONA estão em escopo para esta adoção — apenas tokens/marca em superfícies existentes, ou implementação da UI Svelte completa?]
- [NEEDS CLARIFICATION: A adoção é uma feature nova ou uma emenda a specs existentes (`001-android-media-shell`, `004-ressoa-dogfood-build`, `fr-024-synthesis-adapter`)?]
- [NEEDS CLARIFICATION: Dark-only é aceitável, ou light mode é requerido? Light mode não está especificado e exigiria re-verificação completa.]
- [NEEDS CLARIFICATION: O design system depende do áudio real (`fr-024-synthesis-adapter`) para fazer sentido, ou pode ser adotado antes com stand-ins?]
- [NEEDS CLARIFICATION: Quem é o proponente e qual o trigger desta adoção agora?]
- [NEEDS CLARIFICATION: Como a adoção se relaciona com T051/T052 on-device pendentes da feature 004?]
