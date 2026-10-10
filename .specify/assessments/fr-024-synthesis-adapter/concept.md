# Concept: FR-024 Adaptador Mínimo de Síntese Offline

- **Slug**: fr-024-synthesis-adapter
- **Created**: 2026-10-06
- **Recommended option**: Option A — Adaptador mínimo (sherpa-onnx + Piper pt-BR, uma voz, segmentação básica)

## Options

### Option A — Adaptador mínimo (recomendado)
- **Sketch**: Conectar o `engine-core` trait a um adaptador que chama `sherpa-onnx` (via `ort`/`NNAPI`) com uma voz Piper pt-BR (`faber`), entregando mono f32 @ 22050 Hz diretamente ao `player`. A segmentação usa `pulldown-cmark` + guarda pt-BR apenas; SSML básico se o motor aceitar; falha de formato é nomeada (`FR-035`) apenas se o mismatch for óbvio. Nenhum cloud, nenhuma segunda voz, nenhum export de log, nenhuma tela in-app de diagnóstico — apenas o som real que desbloqueia P1.
- **Appetite**: `small` a `medium` (1–3 semanas de integração + testes de hardware; não um mês)
- **Trade-offs**: **Ganha** verificação rápida de `SC-044`/`SC-009`; risco de regressão no `player` é baixo porque o trait já está definido; respeita todos os não-objetivos (não redesenha `001`, não adiciona cloud). **Sacrifica** segmentação perfeita (`SentenceBoundary`), segunda voz, byte-budget rigoroso, e refinamento de falha — mas esses são explicitamente aceitos pelo `intake.md` como não-essenciais para provar o mecanismo.
- **Rabbit holes**: (1) Latência real do `sherpa-onnx` + `ort` em ARM64 pode estourar `SC-009` (≤3 s) — precisa de benchmark no dispositivo de referência antes de considerar pronto; (2) Se `Piper` não expõe fronteiras de sentença, o resume por posição (`FR-036`) pode ser aproximado e não exato — aceitável para o mínimo, mas pode quebrar quando o usuário troca de voz/engine no futuro; (3) `doc-structure` mínimo pode não ser suficiente para segmentação confiável — pode precisar de 1–2 dias de extensão do parser, não de arquitetura nova.

### Option B — Adaptador completo (dois motores + erro + segmento)
- **Sketch**: Mesmo adaptador mínimo, mas adiciona segunda voz real (`cadu` ou `jeff`), implementa `FR-035` formalmente para qualquer mismatch de formato, adiciona verificação básica de byte-budget (5.000 bytes por chunk), e estende `doc-structure` para produzir fronteiras de sentença razoáveis. Ainda sem cloud, sem diagnostic viewer.
- **Appetite**: `medium` a `large` (3–6 semanas; o risco é que `doc-structure` + `SSML` + `FR-035` sejam maiores que parecem)
- **Trade-offs**: **Ganha** `SC-044` completo com dois motores reais (não só um), maior confiabilidade de resume, e preparação para `gate-8-full`. **Sacrifica** velocidade — o problema é que o shell já está construído e o usuário só precisa de som; adicionar perfeição antes de provar o som pode adiar `SC-009` indefinidamente.
- **Rabbit holes**: (1) `SentenceBoundary` pode não existir no `Piper` — se for necessário, a solução pode exigir `msedge-tts` como fallback (que viola o offline-default) ou uma camada de segmentação externa complexa; (2) Byte-budget de 5.000 bytes (`constitution.md` linha 353) pode exigir split por chunk que afeta alinhamento de posição; (3) `FR-035` exige que o adapter declare falha nomeada para qualquer formato diferente de mono f32 @ 22050 Hz — se o `sherpa-onnx` entregar algo próximo mas não exato (ex.: estéreo, taxa ligeiramente diferente), o adapter precisa rejeitar, mas a rejeição pode ser confusa sem testes de conformidade.

### Option C — Não construir agora / Deferir totalmente ao synthesis feature
- **Sketch**: Manter `standin_a`/`standin_b` como única fonte de áudio; aceitar que `SC-044` e `gate-8-full` permanecem `DEFERRED`; investir apenas no `player`, `session`, `diag` e `engine-core` (que já estão em progresso); esperar que um feature futuro (`FR-024` oficial) entregue o adaptador completo com todos os requisitos (SSML, segmentação, cloud, export de logs).
- **Appetite**: `zero` (não há trabalho de síntese)
- **Trade-offs**: **Ganha** zero risco de regressão no `player`; preserva todos os recursos de desenvolvimento para refinamento do shell. **Sacrifica** completamente a possibilidade de demonstrar P1 a um usuário real; mantém `gate-8-full` bloqueado; o produto nunca sai do estágio de mecanismo.
- **Rabbit holes**: Nenhum técnico, mas o risco estratégico é máximo — se o usuário precisa de um produto que funcione, diferir indefinidamente significa que todo o investimento em `player`, `session` e `diag` não retorna valor externo.

## Recommendation

**Recomendado: Option A (Adaptador mínimo).**

Justificativa baseada no problema:
- O objetivo é fazer o shell (`001`) produzir áudio real para que P1 (`US1-US3`) seja demonstrável (`problem.md` Goals / Success Metrics). Only Option A delivers real sound with the lowest risk to `SC-051` (stand-in mechanism) and the fastest path to `SC-009` / `SC-044`.
- Option B adiciona perfeição (segunda voz, `FR-035`, byte-budget) antes de provar que a primeira voz funciona — o que vai contra o princípio do 80/20 identificado na análise e explicitamente aceito no `intake.md` (não tentar SSML perfeito, cloud, ou byte-budget rigoroso agora).
- Option C aceita o custo da inação integral — o `problem.md` explicita que sem áudio o shell é apenas uma prova de mecanismo; portanto, não recomendar C a menos que o usuário explicitamente rejeite qualquer risco de integração.

A recomendação depende de validar uma suposição: que `sherpa-onnx` + `ort` pode entregar o primeiro chunk dentro do limite de ≤3 s (`SC-009`) no dispositivo de referência. Se o benchmark falhar, a recomendação pode ser ajustada (possivelmente adicionar pré-carregamento ou aceitar latência maior para o mínimo, não abortar).

## Out of Scope (for the recommended option — Option A)

- **Não é escopo**: redimensionamento do `player`, `session` ou `diag`; mudanças no `MediaSession` Kotlin; nova notificação `MediaStyle`; redesign de foco de áudio (`FR-013`/`FR-013a`/`FR-014`); adicionar `stop` como controle compacto adicional (já definido por `R3`).
- **Não é escopo**: cloud synthesis (`Google TTS`, `Azure`, `Edge`) como default ou fallback — o `offline_default` (`sherpa-onnx` + `Piper`) é suficiente para provar o mecanismo; cloud é enhancement (`constitution.md` linha 343).
- **Não é escopo**: tela in-app do registro de diagnóstico (`FR-039a`) — o JSONL já existe; lê-lo vem depois (`spec.md` linha 51).
- **Não é escopo**: picker / lista de documentos recentes — a abertura de arquivo já é função do Tauri; não bloqueia P1.
- **Não é escopo**: segmentação perfeita (`SentenceBoundary` / `WordBoundary`) — `Piper` não oferece; segmentação básica (`pulldown-cmark` + guarda pt-BR) é suficiente para o mínimo.
- **Não é escopo**: orçamento de bytes rigoroso a 5.000 bytes por chunk (`constitution.md` linha 353) — uma verificação básica de tamanho pode ser adicionada se simples, mas não é requisito de sucesso do mínimo.
- **Não é escopo**: segunda voz (`cadu`/`jeff`) — primeiro provar `faber`; segundo voice vem se `SC-044` exigir dois motores reais e o primeiro já passar.

## Assumptions to Validate

- **Assumption 1 (latência)**: `sherpa-onnx` + `ort` (`NNAPI`) pode entregar áudio emitível no dispositivo de referência (`mid-range arm64`, 4x Cortex-A78, 6 GB RAM) dentro do limite de `SC-009` (≤3 s do Play para áudio audível). **Validação**: benchmark manual no dispositivo de referência antes de considerar o adaptador pronto.
- **Assumption 2 (interface)**: o trait `engine-core` aceita a entrada do adaptador sem modificação (mono f32 @ 22050 Hz, falha nomeada se formato errar). **Validação**: integração mínima (`standin_a` → adaptador real) e verificação de `SC-051` não quebrado.
- **Assumption 3 (segmentação)**: `pulldown-cmark` + guarda pt-BR (`Dr.` etc.) produz fronteiras suficientes para resume básico (`FR-036` aproximado). **Validação**: teste de resume após trocar de documento / fechar app; se o posicionamento estiver >30 s de diferença, a segmentação precisa de ajuste.
- **Assumption 4 (voz)**: `Piper` pt-BR `faber` é inteligível o suficiente no ambiente de carro (ruído de cabin, fala simultânea com voz de navegação). **Validação**: trial cego (`SC-008`) com usuário em situação simulada de condução (não apenas laboratório).
- **Assumption 5 (plataforma)**: `MediaStyle` + `PlaybackState` derivado (Android 13+) não quebra a exibição dos controles do adapter apenas porque a síntese é real; a notificação depende apenas do `player`, não do motor. **Validação**: teste de notificação em Pixel, Samsung One UI, Xiaomi HyperOS com áudio real ativo.
