//! Property-based and unit tests for [`AudioChunkCache`] invariants.

use engine_core::closed::PlaybackRate;
use player::cache::{AudioChunkCache, ChunkCacheKey};
use proptest::prelude::*;

#[test]
fn test_cache_insert_and_get() {
    let mut cache = AudioChunkCache::with_capacity_bytes(1024 * 1024); // 1 MiB
    let key = ChunkCacheKey::new([1u8; 32], PlaybackRate::R1_0);
    let samples = vec![0.0f32; 100];
    cache.insert(key, samples.clone());

    let cached = cache.get(&key);
    assert!(cached.is_some());
    if let Some(buf) = cached {
        assert_eq!(buf.as_slice(), samples.as_slice());
    }
}

#[test]
fn test_cache_rate_differentiation() {
    let mut cache = AudioChunkCache::with_capacity_bytes(1024 * 1024);
    let key_normal = ChunkCacheKey::new([2u8; 32], PlaybackRate::R1_0);
    let key_fast = ChunkCacheKey::new([2u8; 32], PlaybackRate::R1_5);

    cache.insert(key_normal, vec![0.1f32; 50]);

    assert!(cache.get(&key_normal).is_some());
    assert!(cache.get(&key_fast).is_none());
}

#[test]
fn test_cache_strict_lru_eviction() {
    // Capacity for exactly 2 entries of 100 f32 samples each (400 bytes each -> 800 bytes cap)
    let sample_bytes = 100 * std::mem::size_of::<f32>();
    let mut cache = AudioChunkCache::with_capacity_bytes(sample_bytes * 2);

    let k1 = ChunkCacheKey::new([1; 32], PlaybackRate::R1_0);
    let k2 = ChunkCacheKey::new([2; 32], PlaybackRate::R1_0);
    let k3 = ChunkCacheKey::new([3; 32], PlaybackRate::R1_0);

    cache.insert(k1, vec![0.1f32; 100]);
    cache.insert(k2, vec![0.2f32; 100]);

    assert_eq!(cache.current_bytes(), sample_bytes * 2);
    assert!(cache.get(&k1).is_some()); // Access k1 to make k2 LRU

    // Insert k3 -> k2 should be evicted because k1 was touched
    cache.insert(k3, vec![0.3f32; 100]);

    assert!(cache.get(&k1).is_some());
    assert!(cache.get(&k3).is_some());
    assert!(cache.get(&k2).is_none());
    assert!(cache.current_bytes() <= sample_bytes * 2);
}

proptest! {
    #[test]
    fn prop_cache_never_exceeds_capacity(
        cap_entries in 1usize..10,
        ops in prop::collection::vec((0u8..3, 1usize..50), 1..50)
    ) {
        let sample_bytes = 10 * std::mem::size_of::<f32>();
        let capacity = cap_entries * sample_bytes;
        let mut cache = AudioChunkCache::with_capacity_bytes(capacity);

        for (fp_byte, sample_count) in ops {
            let key = ChunkCacheKey::new([fp_byte; 32], PlaybackRate::R1_0);
            let samples = vec![0.5f32; sample_count];
            cache.insert(key, samples);
            prop_assert!(cache.current_bytes() <= capacity);
        }
    }
}
