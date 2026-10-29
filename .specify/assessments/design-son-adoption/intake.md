# Idea Intake: Adotar o DESIGN-SONA para o app

- **Slug**: design-son-adoption
- **Created**: 2026-10-22
- **Source**: pasted text — "Adotar o DESIGN SYSTEM definido em @DESIGN-SONA/ para o app." (codebase pointer: repositório local, diretório `DESIGN-SONA/`)
- **Type**: new-capability

## Idea (as captured)

> "Adotar o DESIGN SYSTEM definido em @DESIGN-SONA/ para o app."

A ideia é aplicar o design system `DESIGN-SONA/` (v2.0.0, homologado 2026-10-16; marca v2.1.0 2026-10-16; UI multi-modo v2.2.0 2026-10-17) ao aplicativo `mdreader` (técnico) / `Ressoa` (marca pública), incluindo paleta de tokens `--sona-*`, sistema de dupla assinatura (Concha Harmônica "R" + Selo do Ressonador Canônico), gramática sonora Sona (earcons Adagio/Andante/Allegro/Sforzando/Ostinato/Crescendo/Aria/Unison), e composição multi-modo UI/UX (Lente Sinestésica como player default, Cockpit Eyes-Free, Terminal Editorial Suíço, Matriz de Telemetria).

## Restated

Propor aplicar o design system DESIGN-SONA ao app mdreader/Ressoa: usar seus tokens, marca dupla, earcons e superfície multi-modo nos players, notificação, StatusView in-app e ativos de marca, mantendo as regras existentes (não body text na notificação, verde proibido no player, dark-only, eyes-free).

## Origin & Context

- **Raised by**: [NEEDS CLARIFICATION: proponent / role not stated in input]
- **Trigger**: [NEEDS CLARIFICATION: what prompted this now — design-system readiness (v2.0/v2.1 homologados), prior prototype state, release ask — not stated in input]

## First-Glance Unknowns

- [NEEDS CLARIFICATION: Quais superfícies do DESIGN-SONA estão em escopo para adoção — notificação MediaStyle, StatusView in-app, header do player, gramática de earcons, ativos de marca (ícone/selo), ou todas?]
- [NEEDS CLARIFICATION: Adoção significa implementar a UI Svelte (Lente Sinestésica etc.) ou apenas aplicar tokens/marca às superfícies já existentes?]
- [NEEDS CLARIFICATION: Qual a relação com as features já decididas — `ressoa-mobile-app` (assessment GO, Option A: sideload single-voice, maintainer-only) e `004-ressoa-dogfood-build` (tasks T001–T060 concluídos, aguardando T051/T052 on-device)? Esta ideia é uma nova feature ou uma emenda a specs existentes?]
- [NEEDS CLARIFICATION: Light mode não está especificado (apenas dark `#0B0F1A`) — dark-only é aceitável para esta adoção?]
- [NEEDS CLARIFICATION: Como isso se relaciona com `fr-024-synthesis-adapter` (GO, adaptador mínimo sherpa-onnx + Piper) ainda não implementado — o design system depende de áudio real para fazer sentido?]
