# Feature Specification: 017-audio-cache-queue-management

- **Feature**: 017-audio-cache-queue-management
- **Title**: Bounded In-Memory Audio Chunk Cache & Queue Governance
- **Status**: Draft / Ready for Plan
- **Created**: 2026-11-10
- **Assessment**: `.specify/assessments/audio-cache-queue-management/decision.md` (Verdict: GO)
- **Target Release**: v0.4.0

## 1. Problem & Context

No Ressoa, a síntese de voz neural offline (Sherpa-ONNX + Piper) é computacionalmente intensiva. Em sessões comuns de leitura, os ouvintes frequentemente utilizam controles de **repetição de seção** (`btnRepeat`), **retrocesso de 15s/30s** (`btnSkipBack`) ou saltam entre capítulos contíguos de um diretório (`ChapterQueue`).

Atualmente, qualquer salto temporal para trás exige que o motor re-sintetize o trecho de texto via inferência neural do zero, consumindo múltiplos núcleos de CPU a 100%, gerando atrasos audíveis de até 1,5s e drenando a bateria do dispositivo móvel. Além disso, a alocação irrestrita de buffers temporários sem uma cota rígida de retenção pode causar pressão excessiva de memória em aparelhos Android com 3GB a 4GB de RAM.

## 2. Requirements & Capabilities

### 2.1 Functional Requirements (FR)

- **FR-001 (Bounded Cache Store):** O sistema deve manter um cache em memória (`AudioChunkCache`) delimitado por cota máxima de bytes (padrão estrito de 32 MiB, aproximadamente 6 minutos de áudio mono `f32` @ 22 050 Hz).
- **FR-002 (Deterministic Keying):** Cada entrada do cache deve ser indexada por uma chave composta imutável `ChunkCacheKey { content_fingerprint: [u8; 32], rate: PlaybackRate }`, onde `content_fingerprint` é o hash Blake3-256 do texto normalizado do chunk.
- **FR-003 (Cache-First Synthesis Query):** Antes de acionar o motor de inferência neural Sherpa-ONNX, o pipeline de síntese deve consultar o `AudioChunkCache`. Em caso de acerto (*cache hit*), o buffer PCM deve ser retornado imediatamente sem consumo de CPU neural.
- **FR-004 (Strict LRU Eviction):** Quando a inserção de um novo chunk sintetizado ultrapassar a cota de 32 MiB, os chunks menos recentemente acessados (LRU) devem ser descartados até que a cota seja restabelecida.
- **FR-005 (Immediate Invalidation on Rate Change):** Como a taxa de velocidade afeta a duração e a amostragem percebida, o cache diferencia estritamente chunks por `PlaybackRate`. Se o usuário alterar a taxa, apenas chunks compatíveis com a nova taxa são servidos.
- **FR-006 (Zero Residual Flash I/O):** O cache deve residir exclusivamente na memória volátil do processo Rust, garantindo zero escrita residual em armazenamento flash (privacidade absoluta e preservação do chip NAND).

### 2.2 Invariants & Quality Gates (SC)

- **SC-001 (Sub-50ms Rewind Latency):** Em qualquer salto para trás ou repetição cujo chunk já esteja no cache, o tempo entre o comando de reprodução e a entrega do buffer deve ser inferior a 50 milissegundos.
- **SC-002 (Strict Memory Cap):** O consumo de memória do `AudioChunkCache` nunca deve exceder `32 * 1024 * 1024` bytes sob nenhuma sequência de operações ou carga de estresse.
- **SC-003 (Deterministic Invariant Tests):** O comportamento LRU, o respeito à cota e a correspondência exata de fingerprints devem ser provados via testes de propriedades (`proptest`) e testes unitários no host, integrando-se aos Gates 1–8 do `./scripts/verify.sh`.

## 3. Architecture & Integration Seams

- **Crate Afetado:** `crates/player` (ou `crates/adapter-synthesis`). A estrutura `AudioChunkCache` será injetada como autoridade de cache do pipeline de áudio.
- **Handoff Shape:** Permanece estritamente o formato canônico: Mono `f32` @ 22 050 Hz.
- **Zero Android API em Rust:** A lógica é 100% Rust puro, testável no host Linux sem dependência de emulador ou dispositivo físico.
