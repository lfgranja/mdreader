# Tasks: 017-audio-cache-queue-management

- **Feature**: 017-audio-cache-queue-management
- **Plan**: `./plan.md`
- **Spec**: `./spec.md`
- **Total Tasks**: 6
- **Status**: Completed (Converged)

## Task List

- [x] **T001**: Criar `crates/player/src/cache.rs` definindo `ChunkCacheKey` e a estrutura `AudioChunkCache` com capacidade delimitada em bytes.
- [x] **T002**: Implementar lógica de evicção estrita LRU (`insert`, `get`, `evict_to_fit`) garantindo que `current_bytes <= capacity_bytes` sempre.
- [x] **T003**: Exportar `cache` em `crates/player/src/lib.rs` mantendo a pureza de tipos e isolamento do crate.
- [x] **T004**: Criar testes unitários e de propriedades em `crates/player/tests/prop_audio_cache_invariants.rs` validando ordenação LRU, teto de memória e diferenciação de taxas de reprodução (`PlaybackRate`).
- [x] **T005**: Integrar `AudioChunkCache` com API pública e tipagem fechada para consulta e inserção de buffers PCM no motor Rust.
- [x] **T006**: Rodar `./scripts/verify.sh` e atualizar `scripts/expected-tests.txt` com os novos testes, garantindo todos os Gates 1–8 verdes.
