//! Spec 018 voice catalog contract tests (host-checkable).

use std::str::FromStr;

use adapter_synthesis::voice::{self, VoiceId};

#[test]
fn catalog_is_closed_default_first_and_deterministic() {
    assert_eq!(VoiceId::DEFAULT, VoiceId::PtBrFaberMedium);
    assert_eq!(
        VoiceId::CATALOG,
        [VoiceId::PtBrFaberMedium, VoiceId::PtBrCaduMedium]
    );
    let ids: Vec<&str> = VoiceId::iter().copied().map(VoiceId::id).collect();
    assert_eq!(ids, vec!["pt_BR-faber-medium", "pt_BR-cadu-medium"]);
}

#[test]
fn cycle_wraps_deterministically_both_directions() {
    let mut next = VoiceId::DEFAULT;
    let mut seen_next = Vec::new();
    for _ in 0..VoiceId::CATALOG.len() * 2 {
        next = next.next();
        seen_next.push(next.id());
    }
    assert_eq!(
        seen_next,
        vec![
            "pt_BR-cadu-medium",
            "pt_BR-faber-medium",
            "pt_BR-cadu-medium",
            "pt_BR-faber-medium",
        ]
    );

    let mut prev = VoiceId::DEFAULT;
    let mut seen_prev = Vec::new();
    for _ in 0..VoiceId::CATALOG.len() * 2 {
        prev = prev.prev();
        seen_prev.push(prev.id());
    }
    assert_eq!(
        seen_prev,
        vec![
            "pt_BR-cadu-medium",
            "pt_BR-faber-medium",
            "pt_BR-cadu-medium",
            "pt_BR-faber-medium",
        ]
    );
}

#[test]
fn parse_accepts_known_and_rejects_unknown() {
    assert_eq!(
        VoiceId::parse_id("pt_BR-faber-medium"),
        Ok(VoiceId::PtBrFaberMedium)
    );
    assert_eq!(
        VoiceId::parse_id("pt_BR-cadu-medium"),
        Ok(VoiceId::PtBrCaduMedium)
    );
    assert_eq!(
        VoiceId::from_str("pt_BR-faber-medium"),
        Ok(VoiceId::PtBrFaberMedium)
    );
    for bad in [
        "pt_BR-jeff-medium",
        "pt_BR-edresson-low",
        "unknown",
        "",
        "PT_BR-FABER-MEDIUM",
        "pt_BR-faber-medium ",
    ] {
        assert!(
            VoiceId::parse_id(bad).is_err(),
            "expected rejection for {bad:?}"
        );
        assert!(
            VoiceId::from_str(bad).is_err(),
            "expected rejection for {bad:?}"
        );
    }
}

#[test]
fn each_catalog_voice_has_resolvable_present_bundle() {
    // Uses compile-time assets via voices_root()/voice_dir in dev/test.
    // Ensure root points at bundled assets for this test process.
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/voices");
    assert!(
        root.is_dir(),
        "bundled voices root missing: {}",
        root.display()
    );
    adapter_synthesis::synth::set_voices_dir(root.clone());
    let shared = root.join("shared").join("espeak-ng-data");
    assert!(shared.is_dir(), "shared espeak-ng-data missing");
    for voice in VoiceId::iter().copied() {
        let spec = voice::spec(voice);
        let dir = adapter_synthesis::synth::voice_dir(voice);
        assert!(
            dir.join(spec.onnx).is_file(),
            "missing onnx for {}",
            spec.voice_id.id()
        );
        assert!(
            dir.join(spec.tokens).is_file(),
            "missing tokens for {}",
            spec.voice_id.id()
        );
        assert!(
            dir.join(spec.provenance).is_file(),
            "missing provenance for {}",
            spec.voice_id.id()
        );
        let espeak = adapter_synthesis::synth::espeak_data_dir(voice);
        assert!(
            espeak.is_dir(),
            "missing espeak data for {} at {}",
            spec.voice_id.id(),
            espeak.display()
        );
        assert_eq!(spec.voice_id, voice);
    }
}

#[test]
fn inference_defaults_are_present_and_sane() {
    let faber = voice::inference_defaults(VoiceId::PtBrFaberMedium);
    let cadu = voice::inference_defaults(VoiceId::PtBrCaduMedium);
    for d in [faber, cadu] {
        assert!(d.noise_scale > 0.0);
        assert!(d.noise_scale_w > 0.0);
        assert!(d.length_scale > 0.0);
    }
    assert_ne!(faber.noise_scale, cadu.noise_scale);
    assert_ne!(faber.noise_scale_w, cadu.noise_scale_w);
}
