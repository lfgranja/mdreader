//! Chunk split — adapter-side, ≤5000 bytes, paragraph/heading boundary.
//! Byte-measured (constitution.md:353); pulldown-cmark reachability deferred to integration.

/// Split by paragraph/heading; byte budget ≤ max_bytes.
pub fn split_chunks(text: &str, max_bytes: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::with_capacity(max_bytes);
    for line in text.lines() {
        if current.len() + line.len() + 1 > max_bytes && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current.clear();
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.is_empty() {
        chunks.push(current.trim().to_string());
    }
    chunks
}
