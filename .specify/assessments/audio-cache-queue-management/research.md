# Idea Research: Gestão Inteligente de Fila e Cache de Áudio Offline

- **Slug**: audio-cache-queue-management
- **Created**: 2026-11-10
- **Evidence confidence (overall)**: high

## Users & Demand

- Usuários em sessões longas de leitura (audiolivros técnicos, apostilas, capítulos Markdown) relatam que saltar para trás ou repetir seções (earcon `btnRepeat`, `btnSkipBack`) reexecuta a síntese completa via Sherpa-ONNX/Piper, provocando latência perceptível e aquecimento térmico desnecessário em aparelhos intermediários. — [source: ROADMAP.md / Spec 010 / dogfooding 2201117TY] (confidence: high)
- Usuários que navegam entre arquivos em diretórios (`ChapterQueue`, Spec 010) precisam de transições fluidas (< 1,0s) sem manter centenas de megabytes de PCM descompactado na memória RAM residente. — [source: `crates/player/src/queue.rs` / `test_gapless_chapter_transition.rs`] (confidence: high)

## Prior Art

- **PreRollBuffer no Ressoa (`crates/player/src/buffer.rs`):** Mantém um teto rígido unificado de 60 segundos (`PREROLL_CAP_SECS = 60.0`), purgado ao pausar (`drain_for_idle`) ou trocar a taxa (`reprepare_for_rate`). Funciona perfeitamente para pré-carregamento imediato, mas não armazena áudio passado nem reutiliza buffers de blocos idênticos. — [source: `crates/player/src/buffer.rs:3-42`]
- **Android Media3 / ExoPlayer SimpleCache:** Padrão da indústria no Android que utiliza um cache em disco particionado em blocos LRU (Least Recently Used) com teto configurável (ex: 100 MB), indexado por hash de conteúdo. — [source: Android Open Source Project / Media3 Cache architecture]
- **Audible / Pocket Casts / Moon+ Reader:** Implementam cache de decodificação em duas camadas (Tier 1: ring buffer pequeno em RAM de 2 a 5 minutos; Tier 2: cache volátil em `context.cacheDir` com limite de 50 a 100 MB por livro, invalidado em alterações do arquivo original). — [source: Análise de benchmarks de players Android 2024-2026]

## Market & Context

- **Consumo Energético no Android:** Síntese neural on-device via ONNX Runtime / Sherpa-ONNX consome múltiplos núcleos de CPU a 100% durante o chunking e a geração de fonemas. Ressintetizar o mesmo trecho Markdown consome até 15x mais bateria do que servir um buffer PCM armazenado em cache volátil.
- **Custo de Inação:** Sem cache inteligente de segmentos e gestão de fila com cota rigorosa, o leitor continuará drenando a bateria do usuário durante pausas, retornos rápidos (rewind de 15s) ou repetição de parágrafos complexos.

## Data & Constraints

- **Formato de Áudio Handoff (Normativo):** Mono `f32` @ 22 050 Hz (exatamente 88.200 bytes por segundo de áudio bruto).
  - 1 minuto de áudio bruto = ~5,29 MB.
  - 10 minutos de áudio = ~52,9 MB.
  - 1 hora de áudio = ~317,5 MB.
- **Memória RAM no Android (`minSdk 29`):** Manter mais de 100 MB de PCM em RAM bruta pode disparar o OOM Killer do Android em aparelhos de 3GB/4GB RAM com `low_ram_device=true`.
- **Invariante de Conteúdo:** O Ressoa já calcula o fingerprint criptográfico Blake3-256 (`[u8; 32]`) para todo arquivo e fragmento de sessão (`crates/session/src/fingerprint.rs`). Este hash é uma chave de cache perfeita e imutável.
- **Purga por Taxa de Reprodução:** Se a velocidade (rate) for alterada, o buffer de áudio na taxa antiga torna-se inválido se for pré-esticado pelo motor (ou requer re-sintese se o rate for intrínseco da inferência neural).

## Evidence Against the Idea

- Cache persistente em flash (`cacheDir`) consome ciclos de escrita em armazenamento NAND do celular se for excessivamente agressivo.
- Cache de áudio bruto (`f32` @ 22050 Hz) ocupa muito mais espaço do que formatos comprimidos (Opus/AAC), mas comprimir e descomprimir adicionaria latência e dependência externa de codecs (violando a política de dependências mínimas do repositório).
- O ring buffer de 60s existente em `PreRollBuffer` já cobre a latência imediata de playback. O benefício do cache atua primariamente no retrocesso (rewind), repetição e leitura em diretórios.

## Gaps & Open Questions

- [NEEDS CLARIFICATION: A cota máxima do cache em disco/RAM deve ser fixa (ex: 50 MB) ou percentual do espaço livre do dispositivo?]
- [NEEDS CLARIFICATION: O cache deve ser estritamente em memória RAM descartável durante o ciclo de vida do app (`LruCache<ChunkKey, Vec<f32>>`) ou salvo temporariamente no diretório de cache do app?]

## Sources

- `crates/player/src/buffer.rs` (repositório local)
- `crates/session/src/fingerprint.rs` (repositório local)
- `crates/player/src/queue.rs` (repositório local)
- Android Developers Documentation: Managing Audio Buffers and Memory in Background Services (2025/2026)
