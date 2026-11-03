//! T196 (TDD RED): natural-sort invariants for chapter queues.

use player::queue::{sort_natural, ChapterItem, ChapterStatus};
use proptest::prelude::*;

fn item(name: String) -> ChapterItem {
    ChapterItem {
        relative_path: name.clone(),
        file_uri: name.clone(),
        filename: name,
        byte_size: 1,
        content_fingerprint: [0u8; 32],
        playback_status: ChapterStatus::Pending,
    }
}

fn names(items: &[ChapterItem]) -> Vec<String> {
    items.iter().map(|i| i.filename.clone()).collect()
}

fn is_index(n: &str) -> bool {
    let l = n.to_ascii_lowercase();
    l == "readme.md" || l == "index.md"
}

proptest! {
    #[test]
    fn sort_is_idempotent_and_permutation(v in proptest::collection::vec("[a-zA-Z0-9-]{1,8}\\.md", 0..40usize)) {
        let mut a: Vec<ChapterItem> = v.iter().cloned().map(item).collect();
        sort_natural(&mut a);
        let mut b = a.clone();
        sort_natural(&mut b);
        prop_assert_eq!(&a, &b);
        let mut x = names(&a);
        let mut y = v;
        x.sort();
        y.sort();
        prop_assert_eq!(x, y);
    }

    #[test]
    fn readme_and_index_come_first(v in proptest::collection::vec("[a-z0-9]{1,6}\\.md", 0..20usize), pos in 0usize..20) {
        let mut v = v;
        let at = pos.min(v.len());
        v.insert(at, "README.md".to_string());
        let mut a: Vec<ChapterItem> = v.into_iter().map(item).collect();
        sort_natural(&mut a);
        let n = names(&a);
        let first_non = n.iter().position(|s| !is_index(s)).unwrap_or(n.len());
        prop_assert!(n.iter().skip(first_non).all(|s| !is_index(s)));
        prop_assert!(n.first().is_some_and(|s| is_index(s)));
    }

    #[test]
    fn numeric_prefix_order_and_case_insensitive(a in 1u32..500, b in 1u32..500, up in any::<bool>()) {
        prop_assume!(a != b);
        let (lo, hi) = (a.min(b), a.max(b));
        let ext = if up { "MD" } else { "md" };
        let mut v = vec![
            item(format!("{hi}-x.{ext}")),
            item(format!("{lo:03}-x.{ext}")),
        ];
        sort_natural(&mut v);
        prop_assert_eq!(v[0].filename.clone(), format!("{lo:03}-x.{ext}"));
        let mut c = vec![item("b.md".to_string()), item("A.md".to_string())];
        sort_natural(&mut c);
        prop_assert_eq!(c[0].filename.as_str(), "A.md");
    }
}
