# Idea Intake: Primeiro segmento útil para reduzir TTFA

- **Slug**: first-useful-segment
- **Created**: 2026-11-21
- **Source**: repositório `mdreader` (ponteiro de código) + relato de usuário registrado em `.specify/memory/STATE.md` e no bug `playback-not-starting-after-file-selection`
- **Type**: fix

## Idea (as captured)

> "Ciclo SDD 026: reduzir o tempo até o primeiro áudio em um início fresco de
> playback. Diagnóstico: `prepare_session_document` (`src-tauri/src/lib.rs:6706`)
> chama `split_document` com teto de 5000 bytes; um documento Hamlet de ~3,9 KB
> vira UM segmento; `synth.rs` devolve o PCM inteiro de uma vez;
> `stream_synthesis` só faz handover depois do primeiro segmento → 183 s de
> TTFA. Patche proposto: helper privado `split_first_segment_primer` com
> `PRIMER_BYTES=160`, dividindo o primeiro segmento no último whitespace antes
> de 160 bytes; `prepare_session_document` ganha parâmetro `primer: bool`,
> ligado no início com `start_offset_secs == 0.0` e desligado no estacionamento
> a frio; segment_index sequencial e metadados herdados; contrato mono f32
> 22050 intacto; testes no módulo inline `ttfa_013_tests` do target já pinado."

## Restated

Propor que, quando a reprodução começa do zero, o primeiro segmento enviado à
síntese seja pequeno o suficiente para que o primeiro áudio chegue antes da
síntese integral do documento. Retomadas com posição armazenada e documentos
apenas estacionados devem manter o comportamento atual.

## Origin & Context

- **Raised by**: usuário, a partir da reproducão do documento "Hamlet trecho.md"
  em dispositivo 2201117TY (183 s sem áudio, `play_to_audio_ms=183089`).
- **Trigger**: ciclo 025 corrigiu o título exibido e registrou a latência como
  DEFER; o diagnóstico estrutural de `ScoutChunking` localizou a causa na
  segmentação de 5000 bytes, não no resolvedor de nome.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: como dividir sem quebrar palavras quando não há whitespace antes de 160 bytes?]
- [NEEDS CLARIFICATION: a retomada com `start_offset_secs > 0` deve conservar exatamente a segmentação atual?]
- [NEEDS CLARIFICATION: como preservar `source_range`, `boundary_kind`, `resume_point` e a sequência de `segment_index` após dividir o primeiro segmento?]
- [NEEDS CLARIFICATION: onde alocar os testes sem criar novo target, dado o gate 2 de diff bidirecional?]
- [NEEDS CLARIFICATION: que evidência de TTFA é reivindicada, dado que a captura pareada em dispositivo não está disponível neste checkout?]
