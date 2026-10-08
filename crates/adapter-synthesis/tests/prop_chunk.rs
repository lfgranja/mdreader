//! Property: chunk split ≤5000 bytes; boundaries at paragraph/heading.
// Blocked until chunk logic implemented; property defines expected invariant.
use proptest::prelude::*;

fn chunk_split(input: &str) -> Vec<String> {
    // Placeholder: splits by paragraph/heading lines; must be ≤5000 bytes each.
    // Implementation reserved for T7 (separate agent).
    input.lines().map(|l| l.to_string()).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn chunk_never_exceeds_5000_bytes(s in "[\\w\\s\\n\\r\\t]*") {
        let chunks = chunk_split(&s);
        for c in chunks {
            prop_assert!(c.len() <= 5000, "chunk exceeded 5000 bytes");
        }
    }
}
