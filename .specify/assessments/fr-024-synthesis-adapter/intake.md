# Idea Intake: FR-024 Síntese Mínima Viável — Adaptador sherpa-onnx + Piper pt-BR

- **Slug**: fr-024-synthesis-adapter
- **Created**: 2026-10-06
- **Source**: pasted text (turno anterior de análise 80/20 do aplicativo mdreader)
- **Type**: new-capability

## Idea (as captured)

> síntese mínima viável (FR-024) — adaptador sherpa-onnx + Piper pt-BR (faber/cadu/jeff) — antes de qualquer outra coisa.
> Justificação curta (com referências já citadas na resposta completa):
> spec.md linha 67: FR-024 é o único item que transforma o shell (001) em produto real.
> constitution.md linhas 529-537: motor offline já escolhido (sherpa-onnx, ort aarch64, NNAPI); vozes CC0 já identificadas.
> crates/engine-core/src/lib.rs: trait pronto, stand-ins (standin_a/standin_b) provam a interface.
> plan.md linha 18: SC-044 (dois motores reais) só passa depois da síntese.
> Sem áudio, P1 (US1-US3) não existe; picker, export de log ou TOC são inúteis.
> O que fazer: um adaptador que entrega mono f32 @ 22050 Hz; uma voz Piper; segmentação básica (pulldown-cmark + guarda pt-BR); falha nomeada (FR-035) se formato errar. Não tentar SSML perfeito, cloud, byte budget rigoroso ou export de diagnóstico agora.

## Restated

Propor a implementação do adaptador de síntese offline (`FR-024`) como a próxima capacidade do aplicativo `mdreader`, utilizando o motor `sherpa-onnx` com vozes Piper pt-BR (`faber`/`cadu`/`jeff`) para converter Markdown em áudio mono f32 @ 22050 Hz, antes de investir em recursos secundários como picker de documentos, export de diagnóstico ou controle de seção.

## Origin & Context

- **Raised by**: usuário (turno de avaliação 80/20 após pesquisa profunda no repositório)
- **Trigger**: análise do estado atual do projeto (`001-android-media-shell`) revelou que o shell está quase completo, mas a síntese real está deferida (`FR-024`, `SC-044`); sem áudio, os casos de uso P1 (condução com tela apagada, foco de áudio, pausa ao puxar fone) não são realizáveis.
- **Related artifacts**: `specs/001-android-media-shell/spec.md`, `plan.md`; `.specify/memory/constitution.md` (Engine Registry §529-537); `crates/engine-core/src/lib.rs`; `crates/player/src/lib.rs`

## First-Glance Unknowns

- [NEEDS CLARIFICATION: Qual é a latência real do `sherpa-onnx` + `ort` (`NNAPI`) em `aarch64-linux-android` no dispositivo de referência do projeto (mid-range arm64, 4x Cortex-A78, 6 GB RAM)? A meta de ≤3 s play-to-audio (`SC-009`) pode ser atingida com apenas uma voz Piper ou requer pré-carregamento?
- [NEEDS CLARIFICATION: A segmentação básica (`pulldown-cmark` + `unicode-segmentation`) é suficiente para o contrato de sentença/nó (`FR-024` / `SC-050`) ou o `doc-structure` (atualmente mínimo) precisa de `SentenceBoundary` / `WordBoundary` que o Piper não oferece?
- [NEEDS CLARIFICATION: Como o byte-budget inicial (5.000 bytes, `constitution.md` linha 353) deve ser aplicado quando ainda não há rede (offline default) — é uma verificação de tamanho por chunk antes da síntese, ou uma restrição de quota apenas para cloud?
- [NEEDS CLARIFICATION: O adaptador deve implementar o trait de `engine-core` com falha nomeada (`FR-035`) no primeiro ciclo, ou é aceitável entregar áudio primeiro e refinar falhas depois?
- [NEEDS CLARIFICATION: Qual a ordem exata dos testes: `SC-051` (dois stand-ins) deve continuar passando enquanto o adaptador real é integrado, ou há risco de regressão no `player`?
