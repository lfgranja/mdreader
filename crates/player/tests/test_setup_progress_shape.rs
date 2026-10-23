//! T002f — BLOCKING TEST (red before T005).
//!
//! Asserts the `Progress` payload shape for feature
//! 004-ressoa-dogfood-build: the new `Determinate`
//! variant must carry `consumed_units`, `total_units`,
//! `k`, `n` (not the old `position_secs`/`duration_secs`);
//! `Indeterminate` carries nothing; the progress
//! determinacy is fixed at session start; zero sections
//! implies indeterminate; and `None`-section rules are
//! never included incorrectly in the payload.
//!
//! FAILS (red) before T005 updates the shape.

use player::surface::{Progress, SurfaceResolver};

/// Shape contract: `Determinate` must expose the new
/// unit-based fields, not the old time-based ones.
#[test]
fn progress_determinate_has_new_shape_not_old_time_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let resolver = SurfaceResolver::new(
        Some("Title".to_string()),
        "doc.wav".to_string(),
        120,   // total_units: the chunk split's segment count
        120.0, // total_secs: one second per parsed-content unit
        true,
    );
    let snapshot = resolver.resolve(10.0, resolver.consumed_units_at(10.0), &[]);
    match snapshot.progress {
        Progress::Determinate { .. } => {
            let value = serde_json::to_value(&snapshot.progress)?;
            let obj = value.get("determinate").ok_or("determinate key present")?;
            // The new shape requires these four keys.
            assert!(
                obj.get("consumed_units").is_some(),
                "Determinate missing consumed_units (new shape T002f)"
            );
            assert!(
                obj.get("total_units").is_some(),
                "Determinate missing total_units (new shape T002f)"
            );
            assert!(
                obj.get("k").is_some(),
                "Determinate missing k (new shape T002f)"
            );
            assert!(
                obj.get("n").is_some(),
                "Determinate missing n (new shape T002f)"
            );
            // The old time-based fields must NOT appear.
            assert!(
                obj.get("position_secs").is_none(),
                "old position_secs must not appear in new Determinate shape (T002f)"
            );
            assert!(
                obj.get("duration_secs").is_none(),
                "old duration_secs must not appear in new Determinate shape (T002f)"
            );
        }
        Progress::Indeterminate => {
            assert!(
                matches!(snapshot.progress, Progress::Determinate { .. }),
                "expected Determinate when duration_known_at_start=true (T002f)"
            );
        }
    }
    Ok(())
}

/// Fixed-at-session-start: determinacy never switches.
/// With zero headings and `fixed_at_session_start` false,
/// progress must stay `Indeterminate` for every call.
#[test]
fn fixed_at_session_start_is_false_for_zero_sections() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = SurfaceResolver::new(
        None,
        "doc.wav".to_string(),
        0,   // total_units: unknown at session start
        0.0, // total_secs: unknown at session start
        false,
    );
    let snapshot = resolver.resolve(0.0, resolver.consumed_units_at(0.0), &[]);
    assert!(
        matches!(snapshot.progress, Progress::Indeterminate),
        "zero sections + fixed_at_session_start=false must yield Indeterminate (T002f)"
    );
    // The payload must not include `None`-section rules incorrectly.
    assert!(
        snapshot.section_title.is_none(),
        "section_title must be None for zero headings, not a rule payload (T002f)"
    );
    // Assert no forbidden `None`-section payload key is injected.
    let serialized = serde_json::to_string(&snapshot)?;
    assert!(
        !serialized.contains("none_section_rule"),
        "payload must not include None-section rule incorrectly (T002f)"
    );
    Ok(())
}

/// Fixed-at-session-start true: progress stays determinate
/// regardless of headings count — the total is known.
/// The determinate payload carries the four new fields.
#[test]
fn fixed_at_session_start_true_keeps_determinate_shape() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = SurfaceResolver::new(
        Some("Title".to_string()),
        "doc.wav".to_string(),
        120,   // total_units: the chunk split's segment count
        120.0, // total_secs: one second per parsed-content unit
        true,  // fixed_at_session_start = true
    );
    // Even with zero headings, the session knows its total,
    // so progress must remain determinate with the new shape.
    let snapshot = resolver.resolve(0.0, resolver.consumed_units_at(0.0), &[]);
    assert!(
        matches!(snapshot.progress, Progress::Determinate { .. }),
        "fixed_at_session_start=true must keep Determinate even with zero sections (T002f)"
    );
    let value = serde_json::to_value(&snapshot.progress)?;
    let obj = value.get("determinate").ok_or("determinate")?;
    assert!(obj.get("consumed_units").is_some());
    assert!(obj.get("total_units").is_some());
    assert!(obj.get("k").is_some());
    assert!(obj.get("n").is_some());
    Ok(())
}

/// The unit total is the parsed-content unit count fixed at
/// session start (FR-013): the chunk split's segment count —
/// never the greatest heading position the list marks, and
/// never re-estimated mid-session (freeze-then-drop).
#[test]
fn the_total_is_the_session_start_parsed_content_units_not_the_heading_extent() {
    let resolver = SurfaceResolver::new(
        Some("Title".to_string()),
        "doc.wav".to_string(),
        72,   // total_units: the chunk split's segment count
        72.0, // total_secs: one second per parsed-content unit
        true,
    );
    // A heading list whose greatest position (30.0) differs
    // from the unit total: the total is the unit count.
    let headings = vec![
        player::surface::Heading {
            position_secs: 0.0,
            title: "alpha".to_string(),
        },
        player::surface::Heading {
            position_secs: 30.0,
            title: "bravo".to_string(),
        },
    ];
    let at_start = resolver.resolve(0.0, resolver.consumed_units_at(0.0), &headings);
    let later = resolver.resolve(45.0, resolver.consumed_units_at(45.0), &headings);
    for (label, snapshot) in [("start", &at_start), ("later", &later)] {
        match snapshot.progress {
            Progress::Determinate {
                total_units,
                consumed_units,
                ..
            } => {
                assert_eq!(
                    total_units, 72,
                    "{label}: the total must be the session-start unit count, not the heading extent (FR-013)"
                );
                assert_eq!(
                    consumed_units,
                    resolver.consumed_units_at(if label == "start" { 0.0 } else { 45.0 }),
                    "{label}: the consumed offset is the projected parsed-content offset"
                );
            }
            Progress::Indeterminate => {
                assert!(
                    matches!(snapshot.progress, Progress::Determinate { .. }),
                    "{label}: the totals are known at start"
                );
            }
        }
    }
}

/// The consumed units are the position's projection onto the
/// unit grid — never the position rendered as units and never
/// an audio-duration estimate (FR-013).
#[test]
fn consumed_units_are_the_position_projection_not_the_position() {
    let resolver = SurfaceResolver::new(
        Some("Title".to_string()),
        "doc.wav".to_string(),
        72,    // total_units
        360.0, // total_secs: five seconds per parsed-content unit
        true,
    );
    // 45 s into a 360 s document of 72 units: unit 9 of 72.
    assert_eq!(resolver.consumed_units_at(45.0), 9);
    assert_ne!(
        resolver.consumed_units_at(45.0),
        45,
        "the consumed units must not be the position rendered as units (FR-013)"
    );
    // Past the document's end: the frozen total, never beyond it.
    assert_eq!(resolver.consumed_units_at(10_000.0), 72);
    // At or before the start: nothing consumed.
    assert_eq!(resolver.consumed_units_at(0.0), 0);
    assert_eq!(resolver.consumed_units_at(-5.0), 0);
}
