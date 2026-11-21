# Specification Quality Checklist: Exaustão da produção não finaliza o documento

**Purpose**: Validar completude, clareza e consistência da especificação antes do planejamento e da implementação.
**Created**: 2026-11-21
**Feature**: [spec.md](../spec.md)

**Note**: este é o checklist de qualidade da especificação (`/speckit-specify` / `/speckit-clarify`), distinto de checklists customizados de domínio.
**Review Ownership**: os itens marcados `[x]` indicam que o critério de qualidade de requisitos foi revisado e satisfeito pelo revisor.
**Marker Semantics**: `[x]` significa "requisito bem escrito, revisado e satisfeito"; NÃO significa que a implementação, os gates ou a verificação em dispositivo estão concluídos.

## Content Quality

- [x] CHK001 — Não há detalhes de implementação disfarçados de requisitos de usuário; símbolos como `produce_bounded` e `finish_production_if_current` aparecem apenas como contexto de evidência e de contrato. [Clarity]
- [x] CHK002 — O foco está no valor para o ouvinte: a sessão conclui com `session_end` quando o documento termina, em vez de ficar presa em PLAYING. [Completeness]
- [x] CHK003 — O texto é legível por stakeholders não técnicos; exaustão da produção, gate de drenagem e geração são introduzidos com o papel que desempenham. [Clarity]
- [x] CHK004 — Todas as seções obrigatórias do template estão completas (User Scenarios, Requirements, Success Criteria). [Completeness]

## Requirement Completeness

- [x] CHK005 — Nenhum marcador `[NEEDS CLARIFICATION]` permanece na especificação; as sete dúvidas foram resolvidas em `## Clarifications`. [Traceability]
- [x] CHK006 — Cada FR de FR-027-001 a FR-027-006 é testável e inequívoco, com cenários de aceitação correspondentes nas User Stories. [Completeness, Spec §FR-027-001..006]
- [x] CHK007 — As SCs de SC-027-001 a SC-027-006 são mensuráveis em host, com baseline declarado quando aplicável (baseline de exaustão: 0% de finalização). [Acceptance Criteria]
- [x] CHK008 — As SCs são tecnologicamente agnósticas; nenhuma depende de framework, ferramenta ou dispositivo para ser formulada. [Measurability]
- [x] CHK009 — Todos os cenários de aceitação das quatro User Stories estão definidos. [Completeness]
- [x] CHK010 — Casos de borda estão identificados: exaustão no topo, `room == 0`, última unidade no mesmo tick, `pending` não vazio, geração divergente, `max_units == 0`, lock envenenado, `started == false`, documento vazio, diretório, reabertura e primário de 026. [Edge Case Coverage, Spec §Edge Cases]
- [x] CHK011 — O escopo está limitado explicitamente: uma checagem no produtor, uma chamada a helper existente, um teste no target pinado e a cadeia documental; nenhuma mudança em watch, contrato de áudio, config ou specs de outros ciclos. [Completeness, Spec §Assumptions]
- [x] CHK012 — Dependências e premissas estão documentadas, incluindo a generation-safety do helper, a idempotência do `take()`, a legitimidade de `started == false` e a exigência de reprodução limpa em device. [Dependency, Spec §Assumptions]

## Feature Readiness

- [x] CHK013 — Cada requisito funcional possui critério de aceitação observável na seção de User Stories. [Traceability, Spec §FR-027-001..006]
- [x] CHK014 — Os cenários de usuário cobrem os fluxos primários: exaustão finalizada, generation-safety, fronteiras preservadas e evidência em host. [Scenario Coverage]
- [x] CHK015 — A especificação atende às próprias métricas definidas em Success Criteria sem reivindicar verificação em dispositivo. [Measurability, Spec §Success Criteria]
- [x] CHK016 — Nenhuma implementação vaza para a especificação: nomes de funções aparecem só como contexto de evidência, não como prescrição de design. [Clarity]

## Cycle-Specific Quality Gates

- [x] CHK017 — FR-027-001 fixa a detecção de exaustão no topo do `while`, antes do `break` de backpressure e de qualquer retorno, preservando a checagem de `done` pós-commit. [Clarity, Spec §FR-027-001]
- [x] CHK018 — FR-027-002 exige guard solto e chamada a `finish_production_if_current`, com finalização restrita à geração atual e sem deadlock. [Consistency, Spec §FR-027-002]
- [x] CHK019 — FR-027-003 proíbe `signal_document_end`, `dispatch_document_end` e `ShellVerb::Complete` no produtor, mantendo o gate `drained && !production_running` como única decisão de fim. [Consistency, Spec §FR-027-003]
- [x] CHK020 — FR-027-004 proíbe enum novo e mudanças em `production_running`, handover, backpressure, primário de 026 e contrato PCM mono f32 22050 Hz. [Completeness, Spec §FR-027-004]
- [x] CHK021 — FR-027-005 mantém o teste no módulo inline `ttfa_013_tests` do target pinado, cobrindo exaustão com geração atual, geração divergente, `pending` não vazio e ausência de `complete`. [Traceability, Spec §FR-027-005]
- [x] CHK022 — FR-027-006 restringe o diff a `src-tauri/src/lib.rs`, preserva `expected-tests.txt`, specs/bugs alheios e delega gates ao orquestrador, com `[NV]` para dispositivo. [Consistency, Spec §FR-027-006]
- [x] CHK023 — SC-027-001..006 são mensuráveis em host, com checklist 23/23, nenhuma meta absoluta e `[NV]` explícito para `action=complete`/`session_end` em dispositivo. [Measurability, Spec §Success Criteria]

## Notes

- Revisão realizada em 2026-11-21 sobre `spec.md` já clarificado (sete perguntas respondidas na seção `## Clarifications`).
- Todos os 23 itens foram marcados `[x]` porque o critério de qualidade de requisitos foi satisfeito na revisão; isso NÃO implica que o código foi implementado, que gates foram executados ou que a conclusão foi observada em dispositivo.
- `/speckit-implement` lê o estado deste checkbox como gate e não deve modificar os marcadores.
- `checklists/requirements.md` tem ciclo de vida próprio, mantido por `/speckit-specify` e `/speckit-clarify`.
