# Implementation Plan: 017-audio-cache-queue-management

- **Feature**: 017-audio-cache-queue-management
- **Spec**: `./spec.md`
- **Status**: Draft / Ready for Tasks
- **Created**: 2026-11-10

## 1. Architectural Strategy

Para atingir a latência sub-50ms em repetições e retrocessos de áudio sem violar a política de zero dependências externas pesadas e memória delimitada:
1. Criar o módulo `crates/player/src/cache.rs` com a estrutura `AudioChunkCache`.
2. Implementar a política de desalocação LRU interna (usando estruturas da biblioteca padrão de Rust, como `std::collections::VecDeque` + `HashMap`, garantindo ausência de crates de terceiros desnecessários e tempo O(1) de acesso).
3. Integrar a consulta de cache no pipeline de reprodução/síntese antes da chamada de inferência do adaptador.
4. Adicionar testes de invariantes determinísticos em `crates/player/tests/prop_audio_cache_invariants.rs`.

## 2. Technical Contracts & Invariants

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkCacheKey {
    pub content_fingerprint: [u8; 32],
    pub rate: PlaybackRate,
}

pub struct AudioChunkCache {
    capacity_bytes: usize,
    current_bytes: usize,
    // LRU storage
}
```

## 3. Milestones & Slices

- **Slice 1 (Core Store & Invariants):** Implementar `AudioChunkCache`, cálculo de tamanho de buffer `f32`, inserção, recuperação e evicção LRU quando a capacidade limite (32 MiB) for atingida.
- **Slice 2 (Integration & Pipeline Handoff):** Conectar o `AudioChunkCache` ao fluxo de reprodução de chunks no `player`, verificando o cache antes de despachar a síntese.
- **Slice 3 (Verification & Gate Evidence):** Executar bateria de testes com `proptest`, verificar Gates 1–8 e certificar conformidade.
