//! Invariant and unit tests for `AudioChunkCache` (Spec 017).

use engine_core::closed::PlaybackRate;
use player::cache::{AudioChunkCache, ChunkCacheKey};

#[test]
fn test_cache_insertion_and_retrieval() {
    let mut cache = AudioChunkCache::with_capacity_bytes(1024);
    let key = ChunkCacheKey::new([1u8; 32], PlaybackRate::R1_0);
    let samples = vec![0.1_f32, 0.2_f32, 0.3_f32, 0.4_f32];

    cache.insert(key, samples.clone());
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.current_bytes(), 16); // 4 samples * 4 bytes

    let retrieved = cache.get(&key);
    assert_eq!(retrieved, Some(samples));
}

#[test]
fn test_cache_rate_differentiation() {
    let mut cache = AudioChunkCache::with_capacity_bytes(1024);
    let key1 = ChunkCacheKey::new([1u8; 32], PlaybackRate::R1_0);
    let key2 = ChunkCacheKey::new([1u8; 32], PlaybackRate::R1_25);

    let samples1 = vec![0.1_f32; 8];
    let samples2 = vec![0.2_f32; 8];

    cache.insert(key1, samples1.clone());
    cache.insert(key2, samples2.clone());

    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(&key1), Some(samples1));
    assert_eq!(cache.get(&key2), Some(samples2));
}

#[test]
fn test_strict_lru_eviction_under_capacity_limit() {
    // Capacity for exactly 2 entries of 16 bytes each = 32 bytes
    let mut cache = AudioChunkCache::with_capacity_bytes(32);

    let k1 = ChunkCacheKey::new([1u8; 32], PlaybackRate::R1_0);
    let k2 = ChunkCacheKey::new([2u8; 32], PlaybackRate::R1_0);
    let k3 = ChunkCacheKey::new([3u8; 32], PlaybackRate::R1_0);

    let s1 = vec![0.1_f32; 4]; // 16 bytes
    let s2 = vec![0.2_f32; 4]; // 16 bytes
    let s3 = vec![0.3_f32; 4]; // 16 bytes

    cache.insert(k1, s1);
    cache.insert(k2, s2);
    assert_eq!(cache.current_bytes(), 32);

    // Touch k1 to make k2 the least recently used
    let _ = cache.get(&k1);

    // Insert k3 -> must evict k2 (LRU)
    cache.insert(k3, s3.clone());

    assert_eq!(cache.current_bytes(), 32);
    assert!(
        cache.get(&k1).is_some(),
        "k1 was recently accessed, must remain"
    );
    assert!(cache.get(&k2).is_none(), "k2 was LRU, must be evicted");
    assert_eq!(cache.get(&k3), Some(s3));
}

#[test]
fn test_evict_rate() {
    let mut cache = AudioChunkCache::with_capacity_bytes(1024);
    let k1 = ChunkCacheKey::new([1u8; 32], PlaybackRate::R1_0);
    let k2 = ChunkCacheKey::new([2u8; 32], PlaybackRate::R1_5);

    cache.insert(k1, vec![0.1_f32; 4]);
    cache.insert(k2, vec![0.2_f32; 4]);
    assert_eq!(cache.len(), 2);

    cache.evict_rate(PlaybackRate::R1_0);
    assert_eq!(cache.len(), 1);
    assert!(cache.get(&k1).is_none());
    assert!(cache.get(&k2).is_some());
}
