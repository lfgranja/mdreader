# Concept: Gestão Inteligente de Fila e Cache de Áudio Offline

- **Slug**: audio-cache-queue-management
- **Created**: 2026-11-10
- **Recommended option**: Opção A (Bounded In-Memory LRU Ring Cache em Rust)

## Options

### Option A — Bounded In-Memory LRU Ring Cache em Rust (Recomendada)
- **Sketch**: Implementar um `AudioChunkCache` em `crates/player` ou `crates/adapter-synthesis`, baseado em um anel LRU delimitado com teto rígido de memória (ex: 32 MiB, aproximadamente 6 minutos de áudio bruto mono `f32` @ 22 050 Hz). A chave de cache é uma tupla `(chunk_content_blake3: [u8; 32], rate: PlaybackRate)`. Ao solicitar reprodução de um trecho (inclusive em rewind ou repetição), o motor consulta primeiro o cache; se houver cache hit, devolve imediatamente o buffer sem acionar Sherpa-ONNX.
- **Appetite**: small (2 a 3 dias de engenharia, isolado na camada Rust com zero dependências externas ou JNI).
- **Trade-offs**: Não persiste áudio em disco entre reinicializações completas do app (áudio é volátil à sessão), o que é uma enorme vantagem de privacidade (zero lixo residual no flash NAND) e evita complexidade de I/O em disco.
- **Rabbit holes**: Invalidação de cache se o `rate` mudar (já mitigado ao incluir o `PlaybackRate` na chave do cache).

### Option B — Two-Tier Cache (RAM Bounded + Arquivos Temporários em `cacheDir`)
- **Sketch**: Manter os últimos 2 minutos em RAM e despejar capítulos sintetizados inteiros em arquivos temporários `.raw`/`.pcm` dentro do diretório de cache do Android (`context.cacheDir`), gerenciados por cota de 100 MB.
- **Appetite**: medium (1 a 2 semanas).
- **Trade-offs**: Permite resumir a reprodução mesmo após o fechamento forçado do app sem reprocessar, mas introduz I/O de disco contínuo, risco de permissões, fragmentação de flash e necessidade de purga periódica segura.
- **Rabbit holes**: Gerenciamento de ciclo de vida do armazenamento Android e concorrência na limpeza de arquivos temporários.

### Option C — Do Nothing (Manter PreRollBuffer atual sem cache)
- **Sketch**: Continuar sintetizando sob demanda a cada salto ou repetição.
- **Appetite**: zero.
- **Trade-offs**: Nenhuma linha de código nova, mas perpetua a drenagem de bateria e a latência de ressíntese em saltos curtos.

## Recommendation

**Opção A (Bounded In-Memory LRU Ring Cache em Rust)** é a vencedora absoluta (80/20 puro):
1. Resolve 100% da latência de rewind e repetição de seções imediatas (< 50ms).
2. Economiza bateria em 95% dos casos de uso de audiolivro.
3. Não toca o disco flash do celular (privacidade e velocidade pura).
4. Possui escopo fechado, sem risco de corrupção de arquivos e testável via testes de invariantes determinísticos em Rust no host.

## Out of Scope (for the recommended option)

- Compressão lossy (MP3/Opus) do cache.
- Persistência permanente em disco de arquivos de áudio completos.
- Interface visual para o usuário configurar tamanho de cache (será um teto interno determinístico seguro de 32 MiB).

## Assumptions to Validate

- O overhead de memória de 32 MiB é perfeitamente tolerado pelo Android 10+ (`minSdk 29`).
- O cálculo do hash Blake3 dos chunks de texto é desprezível em termos de CPU (< 1ms).
