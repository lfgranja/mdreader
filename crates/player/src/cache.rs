//! In-memory LRU audio chunk cache for offline synthesis governance.
//!
//! Provides [`AudioChunkCache`] and [`ChunkCacheKey`] to retain recently
//! synthesized audio buffers (mono `f32` @ 22 050 Hz) up to a bounded memory
//! capacity (default 32 MiB, ~6 minutes of raw audio), eliminating redundant
//! neural inference on rewinds, repeat commands, and sequential chapter navigation.

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::mem::size_of;

use engine_core::closed::PlaybackRate;

/// Default capacity for the in-memory audio chunk cache: 32 MiB.
///
/// At 22 050 Hz mono `f32` (88 200 bytes per second), 32 MiB stores
/// approximately 380 seconds (~6.3 minutes) of synthesized speech.
pub const DEFAULT_CACHE_CAPACITY_BYTES: usize = 32 * 1024 * 1024;

/// Cache lookup key composed of the content's Blake3 fingerprint and the playback rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCacheKey {
    /// Blake3-256 fingerprint of the normalized text content for the chunk.
    pub content_fingerprint: [u8; 32],
    /// Playback rate at which this chunk was prepared.
    pub rate: PlaybackRate,
}

impl Hash for ChunkCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content_fingerprint.hash(state);
        // PlaybackRate variants mapped to closed-set discriminant for Hash
        let rate_tag: u8 = match self.rate {
            PlaybackRate::R0_75 => 0,
            PlaybackRate::R1_0 => 1,
            PlaybackRate::R1_25 => 2,
            PlaybackRate::R1_5 => 3,
            PlaybackRate::R1_75 => 4,
            PlaybackRate::R2_0 => 5,
            PlaybackRate::R4_0 => 6,
        };
        rate_tag.hash(state);
    }
}

impl ChunkCacheKey {
    /// Constructs a new cache key.
    #[must_use]
    pub const fn new(content_fingerprint: [u8; 32], rate: PlaybackRate) -> Self {
        Self {
            content_fingerprint,
            rate,
        }
    }
}

/// Entry stored in the cache containing the PCM sample buffer.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Raw mono `f32` PCM samples.
    samples: Vec<f32>,
    /// Pre-calculated memory footprint of this entry in bytes.
    byte_size: usize,
}

/// Bounded, in-memory LRU cache for synthesized audio chunks.
///
/// Ensures strict memory governance: `current_bytes` never exceeds `capacity_bytes`.
#[derive(Debug)]
pub struct AudioChunkCache {
    /// Maximum allowed memory footprint in bytes.
    capacity_bytes: usize,
    /// Current total memory occupied by cached sample vectors in bytes.
    current_bytes: usize,
    /// Primary storage mapping cache keys to sample entries.
    entries: HashMap<ChunkCacheKey, CacheEntry>,
    /// Access history queue for strict LRU eviction (front = oldest/least recently used).
    access_order: VecDeque<ChunkCacheKey>,
}

impl AudioChunkCache {
    /// Creates a new cache with the specified byte capacity.
    #[must_use]
    pub fn with_capacity_bytes(capacity_bytes: usize) -> Self {
        Self {
            capacity_bytes,
            current_bytes: 0,
            entries: HashMap::new(),
            access_order: VecDeque::new(),
        }
    }

    /// Creates a new cache using the standard 32 MiB capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity_bytes(DEFAULT_CACHE_CAPACITY_BYTES)
    }

    /// Returns the maximum byte capacity of the cache.
    #[must_use]
    pub const fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }

    /// Returns the current memory occupied by cached PCM samples in bytes.
    #[must_use]
    pub const fn current_bytes(&self) -> usize {
        self.current_bytes
    }

    /// Returns the number of cached chunks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the cache contains no chunks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Retrieves a cloned reference to the cached PCM samples if present.
    ///
    /// Updates the access order to mark this key as most recently used.
    pub fn get(&mut self, key: &ChunkCacheKey) -> Option<Vec<f32>> {
        let samples = self.entries.get(key).map(|entry| entry.samples.clone());
        if samples.is_some() {
            self.touch(key);
        }
        samples
    }

    /// Inserts a synthesized audio buffer into the cache.
    ///
    /// If the item is larger than the entire capacity, it is safely rejected without error.
    /// Otherwise, older entries are evicted in strict LRU order until the new item fits.
    pub fn insert(&mut self, key: ChunkCacheKey, samples: Vec<f32>) {
        let entry_bytes = samples.len().saturating_mul(size_of::<f32>());

        if entry_bytes > self.capacity_bytes {
            return;
        }

        // Remove existing entry if key is already present to recalibrate size.
        if let Some(existing) = self.entries.remove(&key) {
            self.current_bytes = self.current_bytes.saturating_sub(existing.byte_size);
            self.remove_from_access_order(&key);
        }

        // Evict LRU entries until there is sufficient room for the new entry.
        while self.current_bytes.saturating_add(entry_bytes) > self.capacity_bytes {
            if let Some(lru_key) = self.access_order.pop_front() {
                if let Some(evicted) = self.entries.remove(&lru_key) {
                    self.current_bytes = self.current_bytes.saturating_sub(evicted.byte_size);
                }
            } else {
                break;
            }
        }

        let entry = CacheEntry {
            samples,
            byte_size: entry_bytes,
        };

        self.current_bytes = self.current_bytes.saturating_add(entry_bytes);
        let _ = self.entries.insert(key, entry);
        self.access_order.push_back(key);
    }

    /// Evicts all entries matching a specific playback rate (e.g. when rate changed).
    pub fn evict_rate(&mut self, rate: PlaybackRate) {
        let keys_to_remove: Vec<ChunkCacheKey> = self
            .entries
            .keys()
            .copied()
            .filter(|k| k.rate == rate)
            .collect();

        for key in keys_to_remove {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_bytes = self.current_bytes.saturating_sub(entry.byte_size);
            }
            self.remove_from_access_order(&key);
        }
    }

    /// Clears all entries from the cache.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.current_bytes = 0;
    }

    /// Updates the access order moving the key to the back (most recently used).
    fn touch(&mut self, key: &ChunkCacheKey) {
        self.remove_from_access_order(key);
        self.access_order.push_back(*key);
    }

    /// Removes all occurrences of a key from the access order queue.
    fn remove_from_access_order(&mut self, key: &ChunkCacheKey) {
        self.access_order.retain(|k| k != key);
    }
}

impl Default for AudioChunkCache {
    fn default() -> Self {
        Self::new()
    }
}
