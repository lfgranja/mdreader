# Idea Research: FR-024 Síntese Mínima Viável — Adaptador sherpa-onnx + Piper pt-BR

- **Slug**: fr-024-synthesis-adapter
- **Created**: 2026-10-06
- **Evidence confidence (overall)**: medium (forte evidência interna de arquitetura; falta de dados de desempenho real, sinal externo de usuário e testes de voz em ambiente de carro)

## Users & Demand

- Os casos de uso P1 (US1-US3) são documentados explicitamente no `spec.md` (linhas 94-172): continuidade com tela apagada, interrução de navegação (duck ~20%), pausa ao puxar fone. — [source: `specs/001-android-media-shell/spec.md`, citado] (confidence: high)
- Não existe métrica de uso, ticket de suporte ou entrevista de usuário no repositório que quantifique a demanda por "leitura em voz alta de Markdown no trânsito"; os requisitos emergem das clarificações de sessão (2026-10-01 a 2026-10-05) do próprio spec. — [source: `spec.md` sessões de clarificação; nenhum arquivo de analytics/metrics encontrado no repo] (confidence: medium) — ASSUMPTION: a demanda é real porque o princípio VIII (Driving-first) e as 10/10 trials de lock-screen presuppõem um contexto de condução, mas não há prova observada.
- Distinção importante: é uma *want* declarada pelo autor do projeto (via especificação), não um *observed behavior* de usuários externos.

## Prior Art

- **Interno — mecanismo já provado:** os stand-ins (`standin_a`, `standin_b`) implementam o trait de `engine-core` e permitem verificar `SC-051` (failover de motor) sem áudio real. — [source: `crates/engine-core/src/standin_a.rs`, `standin_b.rs`, `lib.rs`] (confidence: high)
- **Interno — shell completo sem síntese:** `crates/player/src/lib.rs`, `transport.rs`, `focus.rs`, `resume.rs` estão construídos; o `player` aceita mono f32 @ 22050 Hz conforme contrato. — [source: `crates/player/`, `spec.md` linha 27] (confidence: high)
- **Extern — motor escolhido já existe:** `sherpa-onnx` (GitHub `k2-fsa/sherpa-onnx`) é um projeto ativamente mantido com suporte explícito a Android / embedded; publica APK `arm64-v8a`. — [source: GitHub `k2-fsa/sherpa-onnx`, título e descrição confirmados via `curl` 2026-10-06] (confidence: high)
- **Extern — vozes Piper pt-BR já disponíveis:** `rhasspy/piper` (GitHub `rhasspy/piper`) oferece modelos locais; a constituição identifica especificamente `faber`, `cadu`, `jeff` (CC0) e exclui `edresson` (CC-BY-4.0). — [source: `constitution.md` §Engine Registry linhas 529-537, 226-230; GitHub `rhasspy/piper` confirmado via `curl`] (confidence: high)
- **Extern — TTS como categoria:** Wikipedia `Speech synthesis` confirma que síntese envolve segmentação, SSML e alinhamento; mas o `Piper` não expõe `SentenceBoundary` / `WordBoundary` (apenas `msedge-tts` no registry da constituição). — [source: Wikipedia `Speech_synthesis` via curl 2026-10-06; `constitution.md` §Engine Registry] (confidence: medium)
- **Interno — parser ainda mínimo:** `crates/doc-structure/src/lib.rs` contém apenas uma linha de comentário ("built in US4 by T061c"); ou seja, a segmentação necessária para o contrato de sentença ainda não está implementada. — [source: `crates/doc-structure/src/lib.rs`] (confidence: high)

## Market & Context

- **Alternativas que usuários dependem hoje:** Android `TextToSpeech` (built-in) oferece vozes pt-BR, mas sem controle de bookmark por fingerprint (`blake3`), sem resume por seção (`FR-036`), sem rate ladder fechado (7 degraus), sem resposta de foco de áudio classificada (`FR-013`/`FR-013a`/`FR-014`), e sem notificação `MediaStyle` espelhada. — [source: ASSUMPTION — não há análise comparativa formal no repo, apenas princípios; Android TTS é conhecimento geral, mas não citado no spec] (confidence: medium)
- **Custo de não construir:** o shell (`001`) permanece uma prova de mecanismo (`SC-051` passa, `SC-044` ainda não). Sem síntese, nenhum caso de uso P1 pode ser demonstrado a um usuário real. — [source: `plan.md` linha 18, `spec.md` linha 67] (confidence: high)
- **Tendência de mercado:** aplicativos de leitura em voz alta para condução existem (ex.: versões de audiobook, "Listening to Wikipedia"), mas praticamente nenhum combina Markdown nativo com controle de lock-screen e resume por conteúdo (`blake3`). — [source: ASSUMPTION — não há pesquisa de concorrência no repo; inferido da especificidade da constituição] (confidence: low)

## Data & Constraints

- **Motor / runtime:** `ort` 2.0.0-rc.13 (exceção pré-gravada de estabilidade) com `NNAPI`; `sherpa-onnx` publica APK `arm64-v8a`; ambos são estáveis e permitidos pelo `deny.toml`/`constitution.md`. — [source: `constitution.md` linhas 224, 226; `Cargo.lock` pode ser verificado para versões exatas] (confidence: high)
- **Taxa / formato declarado:** 22050 Hz, mono f32, fixo; o engine entrega já resamplado; a conversão do dispositivo é feita pelo `player` (Rust), nunca pelo engine. — [source: `spec.md` linha 75, `plan.md` linha 67] (confidence: high)
- **Vozes disponíveis:** `faber` / `cadu` / `jeff` (Piper, CC0); `edresson` (CC-BY-4.0, excluído por atribuição); `Kokoro-82M` pt-BR deferred (sherpa-onnx só entrega zh+en). — [source: `constitution.md` §Engine Registry linhas 529-537] (confidence: high)
- **Restrições de plataforma (altas):**
  - Android 13+ deriva controles da notificação de `PlaybackState` (apenas 5 slots compactos); `Stop` pode ser só expandido (R3). — [source: `DESIGN/risks.md` R3; `spec.md` FR-018]
  - OEM relayout (Samsung One UI 7, Xiaomi HyperOS) pode colapsar layout; cor não é confiável; forma + posição é o que importa (R2). — [source: `DESIGN/risks.md` R2]
  - `Live Updates` (Android 16) proíbem `customContentView`; `MediaStyle` padrão é obrigatório (R4). — [source: `DESIGN/risks.md` R4]
  - `setWillPauseWhenDucked(true)` é deliberadamente desviado do padrão Android (ADR-001); duck ~20% em perda transitória. — [source: `.specify/memory/constitution.md` v1.3.0 Sync Impact Report, linha 14; `spec.md` linha 40]
- **Limites de áudio / buffer:** pre-roll ≤60 s (`FR-015a`); replay após kill ≤30 s (`FR-015b`); resume de posição com rewind acumulativo (5% do tempo pausado, cap 30 s). — [source: `spec.md` linhas 29, 44, 45]
- **Requisitos de verificação:** `SC-009` (≤3 s play-to-audio), `SC-051` (failover com stand-ins), `SC-044` (dois motores reais — deferred). — [source: `spec.md` linhas 24, 67, 93-94]

## Evidence Against the Idea

- **Latência não verificada:** `SC-009` exige ≤3 s do Press até áudio; os stand-ins têm custo zero de síntese; `sherpa-onnx` via `ort` + `NNAPI` pode exceder 3 s em ARM64 mid-range dependendo do tamanho do primeiro chunk. Nenhum benchmark foi executado no repo. — [source: `spec.md` linha 24 (clarificação), `plan.md` linha 59; ASSUMPTION sobre latência] (confidence: medium)
- **Qualidade de voz não validada para ambiente de carro:** Piper pt-BR (`faber`/`cadu`/`jeff`) é voz neural local, mas não há dado de inteligibilidade sob ruído de cabin/car, nem teste cego (`SC-008` menciona trial blind-test, mas para sinalização, não para voz). — [source: `DESIGN/risks.md` R1 (marquee impossível); `spec.md` US1-US3 não mencionam qualidade de voz; ASSUMPTION] (confidence: medium)
- **Segmentação pode ser insuficiente:** `doc-structure` é mínimo; `Piper` não oferece `SentenceBoundary`; o contrato `FR-024` exige alinhamento de posição a nodos/sentenças (`FR-036`, `FR-023a`). Se a segmentação básica não produz fronteiras confiáveis, o resume por posição (`redb` + `blake3`) pode quebrar ao trocar de voz/engine. — [source: `crates/doc-structure/src/lib.rs`; `constitution.md` §Engine Registry (Piper sem boundary events); `spec.md` linha 37] (confidence: high para lacuna, medium para impacto)
- **Restrições de notificação independentes:** mesmo que a síntese funcione perfeitamente, OEM relayout (R2), limitação de 5 slots (R3) e proibição de Live Updates (R4) podem fazer a notificação falhar em exibir controles, o que invalida o valor percebido pelo motorista. — [source: `DESIGN/risks.md` R2-R4; ASSUMPTION que isso é independente, mas relevante] (confidence: high)
- **Risco de regressão no `player`:** o `engine-core` trait pode sofrer mudanças quando o adaptador real é integrado (ex.: inclusão de capacidade `SSML` ou `timepoints`); `SC-051` atual passa com stand-ins, mas a integração real pode quebrar `SC-027` (transport), `SC-051` ou `SC-054`. — [source: `spec.md` linha 89 (SC-051); `plan.md` linha 232 (stand-ins são test doubles)] (confidence: medium)
- **Dependência de consentimento / fallback:** se o usuário não consentir `sherpa-onnx` (offline) e exigir cloud (`Google TTS` / `Azure` / `Edge`), o adaptador mínimo offline não resolve o caso do usuário. A constituição exige consentimento por provider e ordem de fallback por `engine_tier`; o mínimo viável assume offline como default, mas não prova que todos os casos de uso estão cobertos. — [source: `constitution.md` §Engine Registry; `spec.md` linha 68; `constitution.md` linha 522] (confidence: medium)

## Gaps & Open Questions

- [NEEDS CLARIFICATION: Qual a latência medida de `sherpa-onnx` + `ort` (`NNAPI`) no dispositivo de referência (`mid-range arm64`, `6 GB RAM`) para o primeiro chunk de 22050 Hz mono? `SC-009` (≤3 s) pode ser atingido sem pré-carregamento?
- [NEEDS CLARIFICATION: `Piper` expõe eventos de limite de sentença (`SentenceBoundary`) ou apenas entrega áudio contínuo? Se apenas contínuo, o contrato de alinhamento (`FR-036`, `FR-023a`) requer uma camada de segmentação externa — o `doc-structure` atual é suficiente?
- [NEEDS CLARIFICATION: Como o `byte budget` (5.000 bytes por chunk, `constitution.md` linha 353) interage com o tamanho dos chunks de texto Markdown? Uma seção de documento pode exceder 5.000 bytes, exigindo split antes da síntese.
- [NEEDS CLARIFICATION: A ordem de testes após integração real: `SC-051` (stand-ins) deve ser preservado enquanto `SC-044` é verificado? Há risco de que a mudança de `standin_a` para adaptador real exija refatoração no `player`?
- [NEEDS CLARIFICATION: Se o adaptador não implementa `FR-035` (falha nomeada) no primeiro ciclo, o que acontece quando `sherpa-onnx` entrega formato diferente de 22050 Hz mono? O `player` já declara `FR-035`, mas o adaptador precisa respeitar o contrato.

## Sources

- `specs/001-android-media-shell/intake.md` (slugs, ideia original, unknowns) — cited, internal
- `specs/001-android-media-shell/spec.md` (linhas 24, 27, 29, 40, 45, 67, 75, 93-94, 94-172, 289-305) — cited, internal
- `specs/001-android-media-shell/plan.md` (linhas 17-18, 59, 232, 237) — cited, internal
- `.specify/memory/constitution.md` (v1.3.0, Sync Impact Reports; Engine Registry §529-537, 224, 226-230, 343, 353, 522) — cited, internal
- `crates/engine-core/src/lib.rs`, `standin_a.rs`, `standin_b.rs`, `focus_offer.rs`, `format.rs` — cited, internal
- `crates/player/src/lib.rs`, `transport.rs`, `focus.rs`, `resume.rs`, `backend.rs` — cited, internal
- `crates/doc-structure/src/lib.rs` — cited, internal
- `DESIGN/risks.md` (R1-R8) — cited, internal
- GitHub `k2-fsa/sherpa-onnx` — cited, external (fetch via curl 2026-10-06; host: github.com, policy: allowlisted)
- GitHub `rhasspy/piper` — cited, external (fetch via curl 2026-10-06; host: github.com, policy: allowlisted)
- Wikipedia `Speech synthesis` (`https://en.wikipedia.org/wiki/Text-to-speech`) — cited, external (fetch via curl 2026-10-06; host: en.wikipedia.org, policy: allowlisted — nota: a fetch retornou conteúdo HTML extenso, título confirmado)
- `developer.android.com/reference/android/media/AudioManager` — cited, external (fetch via curl 2026-10-06; host: developer.android.com, policy: not on safe-list; fetch executado por ser referência técnica primária do Android — anotado como fetch técnico, não como fonte de decisão de produto)
