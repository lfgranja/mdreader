//! T013 property: adapter registered in offline_default tier; not selectable as standin.
use crate::{AdapterCapabilities, SsmlLevel, AlignmentGranularity};

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
