# Specification Quality Checklist: Retomada de Diretório na Reabertura

**Purpose**: Validar completude, clareza e consistência da especificação antes do planejamento e da implementação.
**Created**: 2026-11-20
**Feature**: [spec.md](../spec.md)

**Note**: este é o checklist de qualidade da especificação (`/speckit-specify` / `/speckit-clarify`), distinto de checklists customizados de domínio.
**Review Ownership**: os itens marcados `[x]` indicam que o critério de qualidade de requisitos foi revisado e satisfeito pelo revisor.
**Marker Semantics**: `[x]` significa "requisito bem escrito, revisado e satisfeito"; NÃO significa que a implementação está concluída. O progresso de código é acompanhado em `tasks.md`.

## Content Quality

- [x] CHK001 — Não há detalhes de implementação (linguagens, frameworks, APIs) disfarçados de requisitos de usuário; `lib.rs`/`shell_jni.rs` aparecem apenas em contexto de evidência, não como exigência de comportamento. [Clarity]
- [x] CHK002 — O foco está no valor para o ouvinte: oferta visível, nome do arquivo e posição salva. [Completeness]
- [x] CHK003 — O texto é legível por stakeholders não técnicos; termos internos são introduzidos com o papel que desempenham. [Clarity]
- [x] CHK004 — Todas as seções obrigatórias do template estão completas (User Scenarios, Requirements, Success Criteria). [Completeness]

## Requirement Completeness

- [x] CHK005 — Nenhum marcador `[NEEDS CLARIFICATION]` permanece na especificação; as cinco dúvidas foram resolvidas e registradas em `## Clarifications`. [Traceability]
- [x] CHK006 — Cada FR de FR-024-001 a FR-024-006 é testável e inequívoco, com Critérios de Aceitação correspondentes nas User Stories. [Completeness, Spec §FR-024-001..006]
- [x] CHK007 — As SCs de SC-024-001 a SC-024-006 são mensuráveis em host, com baseline declarado. [Acceptance Criteria]
- [x] CHK008 — As SCs são tecnologicamente agnósticas: nenhuma depende de framework, linguagem ou ferramenta específica para ser medida. [Measurability]
- [x] CHK009 — Todos os cenários de aceitação das três User Stories estão definidos. [Completeness]
- [x] CHK010 — Casos de borda estão identificados: offset zero, ausência de bookmark, expiry sem foreground, reaberturas repetidas e divergência de bundles. [Edge Case Coverage, Spec §Edge Cases]
- [x] CHK011 — O escopo está limitado explicitamente: sem edição de `specs/010`, sem renumerar T232/T233, sem novo target e sem mudança de markup. [Completeness, Spec §Assumptions]
- [x] CHK012 — Dependências e premissas estão documentadas, incluindo o único ingresso de reabertura assumido e a seam de teste. [Dependency, Spec §Assumptions]

## Feature Readiness

- [x] CHK013 — Cada requisito funcional possui critério de aceitação observável na seção de User Stories. [Traceability, Spec §FR-024-001..006]
- [x] CHK014 — Os cenários de usuário cobrem os fluxos primários: publicar oferta, exibir contexto, medir comportamento. [Scenario Coverage]
- [x] CHK015 — A especificação atende às próprias métricas definidas em Success Criteria, sem reivindicar verificação em dispositivo. [Measurability, Spec §Success Criteria]
- [x] CHK016 — Nenhuma Implementação vaza para a especificação: nomes de funções aparecem só como contexto de evidência, não como prescrição de design. [Clarity]

## Cycle-Specific Quality Gates

- [x] CHK017 — A janela de 3.000 ms e o auto-continue em foreground são tratados como comportamento já especificado e verificáveis apenas em host, com `[NV]` explícito para device. [Ambiguity, Spec §SC-024-001, Spec §Clarifications]
- [x] CHK018 — A exigência de `route_offer` uma única vez está formulada como regra de comportamento (sem re-derivação em poll), não como detalhe de chamada de função. [Clarity, Spec §FR-024-003]
- [x] CHK019 — A origem única da posição salva está protegida por requisito explícito de derivação a partir do bookmark. [Consistency, Spec §FR-024-002]
- [x] CHK020 — A byte-identity das cinco superfícies HTML é mensurável por comparação de bytes e nomeia todas as superfícies. [Completeness, Spec §FR-024-006]
- [x] CHK021 — A estratégia de teste está ancorada no target já pinado, sem criar novo target nem alterar `scripts/expected-tests.txt`. [Consistency, Spec §FR-024-005]
- [x] CHK022 — A referência a FR-009 da spec 010 é feita como dado de contexto, sem instruções incorporadas e sem edição daquela spec. [Assumption, Spec §Assumptions]
- [x] CHK023 — Não há afirmação remota ou de dispositivo tratada como verificada neste ciclo; todas estão marcadas `[NV]`. [Traceability, Spec §Success Criteria]

## Notes

- Revisão realizada em 2026-11-20 sobre `spec.md` já clarificado (5 perguntas respondidas na seção `## Clarifications`).
- Todos os itens foram marcados `[x]` porque o critério de qualidade de requisitos foi satisfeito na revisão; isso NÃO implica que o código foi implementado.
- `/speckit-implement` lê o estado deste checkbox como gate e não deve modificar os marcadores.
- `checklists/requirements.md` tem ciclo de vida próprio, mantido por `/speckit-specify` e `/speckit-clarify`.
