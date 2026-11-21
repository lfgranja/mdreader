# Concept: Primeiro segmento útil para reduzir TTFA

- **Slug**: first-useful-segment
- **Created**: 2026-11-21
- **Recommended option**: Option A — primário apenas no início fresco

## Options

### Option A — Primário apenas no início fresco
- **Sketch**: ao preparar um documento para playback que começa do zero, o
  primeiro segmento é separado em uma unidade curta (até 160 bytes, cortada no
  último whitespace), seguida pelo restante como próxima unidade. Retomadas e
  documentos estacionados continuam usando a segmentação atual; o ouvinte ouve
  áudio enquanto o restante é produzido em segundo plano.
- **Appetite**: small (ajuste localizado no preparador, sem novo subsistema).
- **Trade-offs**: ganha-se tempo até o primeiro áudio sem tocar no sintetizador
  nem no contrato de áudio; sacrifica-se a contagem original de unidades em
  inícios frescos e exige reindexação cuidadosa dos segmentos.
- **Rabbit holes**: tentar generalizar a regra para todos os segmentos;
  recalcular offsets de retomada; mexer no planejamento de progresso/seções.

### Option B — Orçamento global menor para todo documento
- **Sketch**: trocar o teto de 5000 bytes por um teto pequeno em toda a
  segmentação, de modo que todos os segmentos sejam curtos.
- **Appetite**: medium.
- **Trade-offs**: aumenta muito o número de unidades e muda o timeline, o
  progresso e a semântica de retomada de todos os documentos; benefício incerto
  além do primeiro segmento.
- **Rabbit holes**: regressão de `resume_point`, fingerpints de bookmark e
  contagem de seções; violação de contratos de specs de chunking já estáveis.

### Option C — Não construir; aguardar T232 / worker de pré-síntese
- **Sketch**: manter o diagnóstico como DEFER e investir em um novo worker de
  pré-síntese de capítulo.
- **Appetite**: large.
- **Trade-offs**: evita qualquer risco no caminho atual, mas deixa o problema
  relatado sem solução por mais um ciclo e exige prova em dispositivo.
- **Rabbit holes**: 200-350 LOC, gate 9, dependência de MIUI/dispositivo, escopo
  muito maior que o benefício imediato.

## Recommendation

**Option A**. É a única que ataca diretamente a causa medida em host — o
primeiro segmento de ~3,9 KB — usando o handover que 013 já implementou, com
diff pequeno e reversível. Satisfaz as métricas relativas de sucesso sem
introduzir meta absoluta de TTFA e sem reivindicar prova de dispositivo.
Option B muda demasiados contratos; Option C é o custo de inação que o problema
já identificou.

## Out of Scope (for the recommended option)

- Síntese intra-modelo, voz/modelo/asset, tuning de threads (T232/013 Option B).
- Mudança global do orçamento de chunking (`chunk.rs`, `MAX_CHUNK_BYTES`).
- Novo target de teste, `scripts/expected-tests.txt`, HTML/capabilities/workflow.
- Retroatividade em bookmarks, snapshots ou offsets já persistidos.
- Qualquer afirmação de TTFA em dispositivo nesta cadeia documental (`[NV]`).

## Assumptions to Validate

- Existe whitespace antes de 160 bytes no primeiro segmento dos documentos
  representativos; quando não existir, a regra de não dividir é aceitável.
- `time-to-first-audio` é dominado pelo tamanho da primeira unidade de síntese
  no caso observado; a parcela fixa de inicialização do modelo pode limitar o
  ganho (a confirmar em captura pareada, `[NV]`).
- `start_offset_secs == 0.0` identifica corretamente um início fresco e
  `> 0.0` uma retomada, inclusive no caminho parked.
- O target já pinado aceita mais testes no módulo inline `ttfa_013_tests` sem
  alterar a lista canônica de testes.
