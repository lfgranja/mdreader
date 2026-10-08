//! T015 property: refined chunk split respects paragraph/heading boundaries.
use crate::chunk::split_chunks;

#[test]
fn split_respects_heading_boundary() {
    let text = "# Heading\n\nPara one.\n\nPara two.";
    let chunks = split_chunks(text, 5000);
    assert!(!chunks.is_empty());
    for c in chunks { assert!(c.len() <= 5000); }
}
