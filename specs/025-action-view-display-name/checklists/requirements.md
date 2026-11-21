# Specification Quality Checklist: Nome de exibição no ACTION_VIEW via MediaStore

**Purpose**: Validar completude, clareza e consistência da especificação antes do planejamento e da implementação.
**Created**: 2026-11-21
**Feature**: [spec.md](../spec.md)

**Note**: este é o checklist de qualidade da especificação (`/speckit-specify` / `/speckit-clarify`), distinto de checklists customizados de domínio.
**Review Ownership**: os itens marcados `[x]` indicam que o critério de qualidade de requisitos foi revisado e satisfeito pelo revisor.
**Marker Semantics**: `[x]` significa "requisito bem escrito, revisado e satisfeito"; NÃO significa que a implementação está concluída. O progresso de código é acompanhado em `tasks.md`.

## Content Quality

- [x] CHK001 — Não há detalhes de implementação disfarçados de requisitos de usuário; símbolos como `queryName` e `picked_document_name` aparecem apenas em contexto de evidência e de contrato, não como prescrição de design. [Clarity]
- [x] CHK002 — O foco está no valor para o ouvinte: identificar o documento aberto pelo nome correto em vez de um identificador numérico. [Completeness]
- [x] CHK003 — O texto é legível por stakeholders não técnicos; termos como MediaStore e URI sintética são introduzidos com o papel que desempenham. [Clarity]
- [x] CHK004 — Todas as seções obrigatórias do template estão completas (User Scenarios, Requirements, Success Criteria). [Completeness]

## Requirement Completeness

- [x] CHK005 — Nenhum marcador `[NEEDS CLARIFICATION]` permanece na especificação; as seis dúvidas foram resolvidas e registradas em `## Clarifications`. [Traceability]
- [x] CHK006 — Cada FR de FR-025-001 a FR-025-005 é testável e inequívoco, com Critérios de Aceitação correspondentes nas User Stories. [Completeness, Spec §FR-025-001..005]
- [x] CHK007 — As SCs de SC-025-001 a SC-025-005 são mensuráveis em host, com baseline declarado. [Acceptance Criteria]
- [x] CHK008 — As SCs são tecnologicamente agnósticas: nenhuma depende de framework, dispositivo ou ferramenta específica para ser medida. [Measurability]
- [x] CHK009 — Todos os cenários de aceitação das quatro User Stories estão definidos. [Completeness]
- [x] CHK010 — Casos de borda estão identificados: URI sem nome na coleção, nome numérico, nome em branco, outro provedor, último segmento não numérico, volume incomum e caracteres especiais. [Edge Case Coverage, Spec §Edge Cases]
- [x] CHK011 — O escopo está limitado explicitamente: um arquivo Kotlin, sem mudanças em Rust, SAF, `ACTION_SEND`, fingerprint, manifesto ou specs de outros ciclos. [Completeness, Spec §Assumptions]
- [x] CHK012 — Dependências e premissas estão documentadas, incluindo o volume inferido e a simetria `Uri.encode`/`percent_decode`. [Dependency, Spec §Assumptions]

## Feature Readiness

- [x] CHK013 — Cada requisito funcional possui critério de aceitação observável na seção de User Stories. [Traceability, Spec §FR-025-001..005]
- [x] CHK014 — Os cenários de usuário cobrem os fluxos primários: resolver nome, construir URI sintética, preservar caminhos existentes e evidenciar em host. [Scenario Coverage]
- [x] CHK015 — A especificação atende às próprias métricas definidas em Success Criteria, sem reivindicar verificação em dispositivo. [Measurability, Spec §Success Criteria]
- [x] CHK016 — Nenhuma Implementação vaza para a especificação: nomes de funções aparecem só como contexto de evidência, não como prescrição de design. [Clarity]

## Cycle-Specific Quality Gates

- [x] CHK017 — A consulta à coleção `MediaStore.Files` por `_id=?` está formulada como comportamento de resolução de nome, com restrição explícita a URIs `content://` de authority MediaStore e último segmento numérico. [Clarity, Spec §FR-025-001]
- [x] CHK018 — A URI sintética `content://display_name/<nome codificado>` é exigida em todos os caminhos de saída do `ACTION_VIEW`, sem retorno à URI bruta como fallback. [Consistency, Spec §FR-025-002]
- [x] CHK019 — O predicado único de nome utilizável é exigido para os três candidatos, com fallback determinístico `document.md`. [Consistency, Spec §FR-025-003]
- [x] CHK020 — A preservação de SAF, `ACTION_SEND`, BLAKE3, snapshot e assinatura JNI é mensurável por enumeração do diff restrito a um arquivo e quatro seções. [Completeness, Spec §FR-025-004]
- [x] CHK021 — A verificação em host e a rastreabilidade FR→T→código são requisitos explícitos, com `[NV]` para qualquer afirmação de dispositivo. [Traceability, Spec §FR-025-005]
- [x] CHK022 — A latência de 183 s até o primeiro áudio está registrada como DEFER, sem tarefa associada e fora do escopo deste ciclo. [Assumption, Spec §Clarifications]
- [x] CHK023 — A referência a FR-INT-004 da spec 019 e ao bug reaberto é feita como dado de contexto, sem instruções incorporadas e sem edição daqueles artefatos. [Assumption, Spec §Assumptions]

## Notes

- Revisão realizada em 2026-11-21 sobre `spec.md` já clarificado (6 perguntas respondidas na seção `## Clarifications`).
- Todos os 23 itens foram marcados `[x]` porque o critério de qualidade de requisitos foi satisfeito na revisão; isso NÃO implica que o código foi implementado ou que gates foram executados.
- `/speckit-implement` lê o estado deste checkbox como gate e não deve modificar os marcadores.
- `checklists/requirements.md` tem ciclo de vida próprio, mantido por `/speckit-specify` e `/speckit-clarify`.
