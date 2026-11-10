# Tasks: 017-audio-cache-queue-management

- **Feature**: 017-audio-cache-queue-management
- **Plan**: `./plan.md`
- **Spec**: `./spec.md`
- **Total Tasks**: 6
- **Status**: Ready for Implementation

## Task List

- [ ] **T001**: Criar `crates/player/src/cache.rs` definindo `ChunkCacheKey` e a estrutura `AudioChunkCache` com capacidade delimitada em bytes.
- [ ] **T002**: Implementar lógica de evicção estrita LRU (`insert`, `get`, `evict_to_fit`) garantindo que `current_bytes <= capacity_bytes` sempre.
- [ ] **T003**: Exportar `cache` em `crates/player/src/lib.rs` mantendo a pureza de tipos e isolamento do crate.
- [ ] **T004**: Criar testes unitários e de propriedades em `crates/player/tests/test_audio_cache.rs` validando ordenação LRU, teto de memória e diferenciação de taxas de reprodução (`PlaybackRate`).
- [ ] **T005**: Integrar `AudioChunkCache` na sessão/pipeline de áudio de `player` para servir buffers cacheados em rewind/repeat antes da síntese.
- [ ] **T006**: Rodar `./scripts/verify.sh` e atualizar `scripts/expected-tests.txt` com os novos testes, garantindo todos os Gates 1–8 verdes.
