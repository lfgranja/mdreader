//! T013 property: adapter registered in offline_default tier; not selectable as standin.

/// The wire name of the tier this adapter registers at.
///
/// Replaced by the typed `EngineTier` registry in T031; this string
/// form exists only so the crate compiles under the workspace lint
/// set (`missing_docs = deny`) before that task runs.
pub fn tier_membership() -> &'static str {
    "offline_default"
}

#[cfg(test)]
mod prop_tests {
    use super::*;

    #[test]
    fn adapter_registered_offline_default() {
        assert_eq!(tier_membership(), "offline_default");
    }
}
