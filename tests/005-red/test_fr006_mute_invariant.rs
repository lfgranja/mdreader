// RED — FR-006 mute/dignity invariant (earcon plays regardless of mute state)
// Contract: contracts/surface.md L69-70 + contracts/earcon.md (mute/DND covered)
use proptest::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MuteState { Muted, Unmuted, DND }

fn earcon_plays(state: MuteState) -> bool {
    // RED: production missing — invariant not yet implemented
    false
}

proptest! {
    #[test]
    fn prop_fr006_earcon_plays_despite_mute(_state in prop::sample::select(vec![MuteState::Muted, MuteState::Unmuted, MuteState::DND])) {
        let s = _state;
        prop_assert!(earcon_plays(s), "FR-006: earcon must play in {:?} (mute/dignity)", s);
    }
}

#[test]
fn fr006_deterministic_mute_invariant() {
    // Pre: audio path available; Post: sound plays / silent path holds regardless
    for s in [MuteState::Muted, MuteState::Unmuted, MuteState::DND] {
        assert!(earcon_plays(s), "FR-006 invariant failed for state {:?}: earcon blocked by mute", s);
    }
}
