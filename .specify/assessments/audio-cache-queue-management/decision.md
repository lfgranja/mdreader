# Decision: Gestão Inteligente de Fila e Cache de Áudio Offline

- **Slug**: audio-cache-queue-management
- **Decided**: 2026-11-10
- **Verdict**: go
- **Artifacts reviewed**: intake.md, research.md, problem.md, concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | A ressíntese contínua de trechos idênticos em rewind/repeat causa latência perceptível e drena a bateria do dispositivo. |
| Evidence strength | strong | Baseada no código existente de `PreRollBuffer`, `ChapterQueue` e medições empíricas de consumo da inferência Sherpa-ONNX. |
| Value vs. inaction | strong | Entrega resposta instantânea (< 50ms) em repetições e reduz consumo de CPU sem adicionar peso ao app. |
| Feasibility / appetite | strong | A Opção A (Bounded LRU Ring Cache em memória) tem apetite `small`, cabe 100% no core Rust e não requer novas dependências. |
| Strategic fit | strong | Alinha-se perfeitamente aos princípios de soberania local, privacidade (zero resíduos em disco) e performance fluida do Ressoa. |
| Risk posture | strong | Teto de memória rígido (32 MiB) impede vazamentos ou risco de OOM no Android. |

## Verdict & Rationale

**Veredito: GO.** A funcionalidade atende plenamente ao princípio 80/20: máximo ganho de responsividade e economia de bateria com mínimo esforço de engenharia focado no motor Rust.

## Handoff to `/speckit-specify`

- **Problem**: Eliminar a ressíntese desnecessária e latência em saltos para trás e repetições imediatas, economizando bateria e CPU.
- **Chosen approach**: Opção A — Bounded In-Memory LRU Ring Cache (`AudioChunkCache`) em Rust com cota estrita de 32 MiB, indexado por `(blake3_fingerprint, PlaybackRate)`.
- **In scope**: Estrutura `AudioChunkCache`, consulta e inserção na pipeline de síntese/player, invalidação automática por rate e purga LRU estrita.
- **Out of scope**: Persistência em disco, compressão de áudio, configurações manuais no frontend.
- **Success metrics**: Latência de retorno < 50ms para chunks em cache; teto de memória estritamente respeitado (≤ 32 MiB); testes de invariantes de cache 100% aprovados.
- **Carried-forward open questions**: Nenhuma impeditiva; cota padrão definida em 32 MiB (~6 minutos de áudio mono f32 @ 22 050 Hz).
