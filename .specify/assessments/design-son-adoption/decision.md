# Decision: Adotar o DESIGN-SONA para o app

- **Slug**: design-son-adoption
- **Decided**: 2026-10-22
- **Verdict**: go
- **Artifacts reviewed**: intake.md? yes | research.md? yes | problem.md yes | concept.md yes

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | adequate | Real gap para o usuário único evidenciado (mantenedor-driver): DESIGN-SONA é spec homologada sem superfície no app. Validity holds at dogfood scope, not release scope — no external user evidenced. [problem.md Problem Statement; research.md Users & Demand] |
| Evidence strength | adequate | Strong internal (design system especificado e homologado, constitution gates, spec completa de 001/004) with explicitly-marked assumption gaps on demand/market and unmeasured on-device numbers. Medium overall, never over-claimed. [research.md throughout] |
| Value vs. inaction | adequate | Solving closes the gap between DESIGN-SONA como papel e DESIGN-SONA como código; inaction deixa o design system como spec não-implementada e o investimento em tokens/paleta/gramática sem retorno. [problem.md Cost of Inaction] |
| Feasibility / appetite | adequate | Option A credível dentro de appetite small (dias) sobre a casca já construída (001) e o design system especificado. Desconhecidos on-device são risco medido, não oculto. [concept.md Option A; research.md Evidence Against #4-#5] |
| Strategic fit | strong | Driving-first, offline-first, closed sets, dark-only, do-not-launch-yet, $0-now — tudo alinha com a constituição e o product-marketing record. [constitution.md Princípios VIII, IX, X, XI; research.md Market & Context] |
| Risk posture | adequate | Riscos principais identificados com mitigações documentadas (scope creep para Option B, relayout OEM, sobreposição T044, earcons sem áudio real). Residuais on-device são risco medido da Option A. [concept.md Trade-offs/Rabbit holes; research.md Evidence Against] |

## Verdict & Rationale

**Go — para o escopo da Option A apenas.** Problem validity e evidence strength ambos clear `adequate` (never `weak` or `unknown`), um recommended concept option existe, e o strategic fit é `strong`. O scorecard suporta especificar o artefato estreito que aplica tokens, dupla assinatura e earcons Sona ao usuário único evidenciado (mantenedor-driver), comholding toda distribuição. Um `go` para o release completo (Option B) over-claims evidência externa fina e é explicitamente não concedido.

## If go — Handoff to `/speckit.specify`

- **Problem**: O design system DESIGN-SONA/ existe apenas como especificação homologada (v2.0.0, marca v2.1.0, UI multi-modo v2.2.0) e documentos de referência, sem qualquer superfície implementada no aplicativo mdreader/Ressoa; o usuário único evidenciado (mantenedor-driver) não tem superfície visual e sonora coerente que aplique tokens, earcons e dupla assinatura.
- **Chosen approach**: Option A — Tokens, marca e earcons nas superfícies existentes (notificação-first): aplicar tokens `--sona-*`, dupla assinatura (Concha Harmônica "R" no cabeçalho in-app; Ressonador Canônico no mostrador/diagnóstico) e gramática de earcons Sona (Adagio, Andante, Allegro, Sforzando, Ostinato, Unison) sobre notificação MediaStyle, StatusView in-app e earcons sintetizados em Rust a 22050 Hz mono f32. Sem UI Svelte completa. Dark-only.
- **In scope / out of scope**: In scope: tokens de paleta, dupla assinatura, earcons Sona, notificação MediaStyle (título/seção/progresso apenas, sem body text/taxa/voz/marca), StatusView in-app grade 2-campo, operação com ou sem som. Out of scope: UI Svelte completa (Lente Sinestésica, Cockpit, Terminal Suíço, Matriz de Telemetria), distribuição pública, light mode, Android Auto, failover/nuvem, diagnóstico in-app, monetização, validação multi-OEM. [concept.md Out of Scope]
- **Success metrics**: Superfície renderizada com tokens DESIGN-SONA sem vazamento de cor fora do conjunto fechado (inspeção + trial de pureza visual); earcons Sona distintivos em blind-test 10/10 por tipo (SC-008); notificação exibe apenas título/seção/progresso 10/10 inspeções no dispositivo de referência; app operável com earcon desativado (trial mudo). [problem.md Success Metrics]
- **Carried-forward open questions**: Quais superfícies do DESIGN-SONA estão em escopo; feature nova ou emenda a specs existentes; dark-only vs light mode; depende de áudio real ou stand-ins; proponente e trigger da adoção; relação com T051/T052 on-device pendentes da feature 004. [problem.md Open Questions]
