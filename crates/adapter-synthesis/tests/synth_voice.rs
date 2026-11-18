//! Spec 018 voice-aware synthesis tests.
//!
//! These tests exercise real offline synthesis with bundled CC0 pt-BR voices.
//! They are intentionally narrow (single short segment) to keep host runs bounded.

use adapter_synthesis::chunk::{BoundaryKind, DocumentSegment, SourceRange};
use adapter_synthesis::voice::VoiceId;

fn short_segment(text: &str) -> DocumentSegment {
    DocumentSegment {
        content: text.to_string(),
        source_range: SourceRange::new(1, 1, 0, u32::try_from(text.len()).unwrap_or(u32::MAX)),
        boundary_kind: BoundaryKind::Paragraph,
        segment_index: 0,
        resume_point: None,
    }
}

fn ensure_bundled_voices_root() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/voices");
    adapter_synthesis::synth::set_voices_dir(root);
}

#[test]
fn synthesize_with_each_catalog_voice_produces_pcm() -> Result<(), String> {
    ensure_bundled_voices_root();
    let segment = short_segment("Olá, teste de voz.");
    for voice in VoiceId::iter().copied() {
        let pcm = adapter_synthesis::synth::synthesize_with_voice(voice, &segment)
            .map_err(|err| format!("synthesis failed for {}: {err:?}", voice.id()))?;
        if pcm.is_empty() {
            return Err(format!("expected non-empty PCM for {}", voice.id()));
        }
        if !pcm.iter().any(|sample| sample.abs() > 0.0_f32) {
            return Err(format!(
                "expected audible/non-zero samples for {}",
                voice.id()
            ));
        }
    }
    Ok(())
}

#[test]
fn synthesize_uses_active_voice_and_restore() -> Result<(), String> {
    ensure_bundled_voices_root();
    let segment = short_segment("Teste.");
    let original = adapter_synthesis::synth::active_voice();
    let alt = if original == VoiceId::PtBrFaberMedium {
        VoiceId::PtBrCaduMedium
    } else {
        VoiceId::PtBrFaberMedium
    };
    let _previous = adapter_synthesis::synth::replace_active_voice(alt);
    let pcm_alt = adapter_synthesis::synth::synthesize(&segment)
        .map_err(|err| format!("active-voice synthesis should succeed: {err:?}"))?;
    if pcm_alt.is_empty() {
        return Err("active voice produced empty PCM".to_string());
    }
    let _restored = adapter_synthesis::synth::replace_active_voice(original);
    let pcm_default = adapter_synthesis::synth::synthesize(&segment)
        .map_err(|err| format!("default synthesis should succeed: {err:?}"))?;
    if pcm_default.is_empty() {
        return Err("default voice produced empty PCM".to_string());
    }
    Ok(())
}
