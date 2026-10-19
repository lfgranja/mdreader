//! T133 — BLOCKING TEST: Kotlin-facing mirror payload contract
//! (host half; the Rust→Kotlin publish bridge).
//!
//! Requirement: every transport/position/pause-reason event must
//! reach the Kotlin surface in ITS own wire form — the form
//! `MediaMirror.publishJson` parses — so the lock-screen surface
//! carries title + section + progress and nothing else (FR-003,
//! FR-013, FR-028 series, FR-028a, FR-028e, SC-035).
//!
//! The Kotlin parser is not host-runnable; this test is the host half
//! of that contract (the device half is the T051 trial set). It pins
//! EXACTLY the payload `src_tauri::publish::build_kotlin_mirror_json`
//! emits:
//!
//! ```json
//! {
//!   "transport": "playing" | "paused" | "stopped" | "completed"
//!              | "interrupted",
//!   "positionSecs": f64,
//!   "title": "...",
//!   "sectionTitle": "..." | null,
//!   "transientReason": "..." | null,
//!   "haptic": null | <int id>,
//!   "progress": {"determinate": {"consumedUnits": u, "totalUnits": t}}
//!             | {},
//!   "dismissal": {"nonDismissable": null}
//!              | {"autoDismiss": <ms>}
//!              | {"dismissable": null}
//! }
//! ```
//!
//! camelCase keys and the single-key `dismissal` object are the
//! Kotlin enum shapes; `progress.determinate` carries the
//! content-offset units (`consumed_units` / `total_units`) so the
//! platform bar renders the FR-013 fraction — never a time estimate.
//! `k`/`n` ride the in-app payload only.

use player::surface::{Dismissal, Progress, SurfaceSnapshot, TRANSIENT_DISMISS_MS};
use player::transport::TransportState;
use serde_json::{Map, Value};
use src_tauri::publish::build_kotlin_mirror_json;

#[expect(
    clippy::expect_used,
    reason = "test harness: the Kotlin mirror payload shape IS the contract under test, so a missing/ill-typed key must fail this test loudly rather than be swallowed"
)]
fn parse(json: &str) -> Value {
    serde_json::from_str(json).expect("kotlin mirror payload is valid JSON")
}

#[expect(
    clippy::expect_used,
    reason = "test harness: the Kotlin mirror payload shape IS the contract under test, so a missing/ill-typed key must fail this test loudly rather than be swallowed"
)]
fn object_of(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("payload is a JSON object")
}

fn determinate(consumed: u64, total: u64) -> SurfaceSnapshot {
    SurfaceSnapshot {
        title: String::from("Falando com estranhos"),
        section_title: Some(String::from("Capítulo 2 — A chegada")),
        progress: Progress::Determinate {
            consumed_units: consumed,
            total_units: total,
            k: 2,
            n: 5,
        },
    }
}

fn sorted_keys(object: &Map<String, Value>) -> Vec<&str> {
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    keys
}

/// The playing surface: the whole key set, the line position verbatim,
/// the content-offset units in the Kotlin determinate shape, and the
/// ongoing (non-dismissable) policy.
#[test]
#[expect(
    clippy::expect_used,
    reason = "test harness: the Kotlin mirror payload shape IS the contract under test, so a missing/ill-typed key must fail this test loudly rather than be swallowed"
)]
fn playing_publishes_units_and_an_ongoing_surface() {
    let surface = determinate(12, 60);
    let payload = parse(&build_kotlin_mirror_json(
        TransportState::Playing,
        123.5,
        &surface,
        None,
        None,
        &Dismissal::NonDismissable,
    ));
    let object = object_of(&payload);

    assert_eq!(
        sorted_keys(object),
        vec![
            "dismissal",
            "haptic",
            "positionSecs",
            "progress",
            "sectionTitle",
            "title",
            "transientReason",
            "transport",
        ],
        "the Kotlin key set is closed and camelCase"
    );
    assert_eq!(object.get("transport"), Some(&Value::from("playing")));
    assert_eq!(object.get("positionSecs"), Some(&Value::from(123.5)));
    assert_eq!(
        object.get("title"),
        Some(&Value::from("Falando com estranhos"))
    );
    assert_eq!(
        object.get("sectionTitle"),
        Some(&Value::from("Capítulo 2 — A chegada"))
    );
    assert_eq!(object.get("transientReason"), Some(&Value::Null));
    assert_eq!(object.get("haptic"), Some(&Value::Null));

    let progress = object.get("progress").expect("progress present");
    let determinate_obj = progress
        .as_object()
        .expect("progress is an object")
        .get("determinate")
        .expect("determinate form present");
    assert_eq!(
        sorted_keys(determinate_obj.as_object().expect("determinate object")),
        vec!["consumedUnits", "totalUnits"],
        "the Kotlin determinate form carries the two unit fields only"
    );
    assert_eq!(
        determinate_obj.get("consumedUnits"),
        Some(&Value::from(12_u64))
    );
    assert_eq!(
        determinate_obj.get("totalUnits"),
        Some(&Value::from(60_u64))
    );

    let dismissal = object
        .get("dismissal")
        .expect("dismissal present")
        .as_object()
        .expect("dismissal is an object");
    assert_eq!(sorted_keys(dismissal), vec!["nonDismissable"]);
    assert_eq!(dismissal.get("nonDismissable"), Some(&Value::Null));
}

/// T083 RED: the lock-screen contract requires the progress label to be
/// pt-BR even though the CLI's frozen identifier remains English.
#[test]
fn determinate_lock_screen_publishes_localized_progress_label() {
    let surface = determinate(12, 60);
    let payload = parse(&build_kotlin_mirror_json(
        TransportState::Playing,
        123.5,
        &surface,
        None,
        None,
        &Dismissal::NonDismissable,
    ));
    assert_eq!(
        payload.get("progressLabel"),
        Some(&Value::from("Seção 2 de 5 — 20% (content-offset)")),
        "T089 RED: determinante progress label is pt-BR identifier per surface.md FR-011/FR-013; includes (content-offset) not rendered prose"
    );
}

#[test]
fn indeterminate_lock_screen_omits_localized_progress_label() {
    let surface = SurfaceSnapshot {
        title: String::from("notas.md"),
        section_title: None,
        progress: Progress::Indeterminate,
    };
    let payload = parse(&build_kotlin_mirror_json(
        TransportState::Paused,
        0.0,
        &surface,
        None,
        None,
        &Dismissal::Dismissable,
    ));
    assert!(
        payload.get("progressLabel").is_none(),
        "indeterminate progress must not invent a localized label"
    );
}

/// T089 property: for every determinate (k, n, p) triple the Kotlin
/// payload must carry the frozen pt-BR identifier label with
/// (content-offset), and never for indeterminate.
#[test]
#[expect(
    clippy::expect_used,
    reason = "test harness: the Kotlin mirror payload shape IS the contract under test, so a missing/ill-typed key must fail this test loudly rather than be swallowed"
)]
fn prop_kotlin_mirror_progress_label_invariant() {
    // Property-style: sweep k in [1..4], n in [k+1..7], p = floor(100*k/n)
    for k in 1..=4 {
        for n in (k + 1)..=7 {
            let consumed = u64::from(100 * k / n) * 10; // arbitrary content-offset units
            let total = 100u64;
            let surface = SurfaceSnapshot {
                title: String::from("Prop doc"),
                section_title: Some(String::from("Section")),
                progress: Progress::Determinate {
                    consumed_units: consumed,
                    total_units: total,
                    k,
                    n,
                },
            };
            let payload = parse(&build_kotlin_mirror_json(
                TransportState::Playing,
                0.0,
                &surface,
                None,
                None,
                &Dismissal::NonDismissable,
            ));
            let label = payload
                .get("progressLabel")
                .expect("T089 RED: progressLabel present for determinate");
            let s = label.as_str().expect("label is string");
            assert!(
                s.contains("Seção")
                    && s.contains("de")
                    && s.contains("—")
                    && s.contains('%')
                    && s.contains("(content-offset)"),
                "T089 RED: determinate label format broken: {s} (expected pt-BR identifier with (content-offset))"
            );
        }
    }
    // Indeterminate exclusion property: for any surface, key absent
    for _ in 0..10 {
        let surface = SurfaceSnapshot {
            title: String::from("notes.md"),
            section_title: None,
            progress: Progress::Indeterminate,
        };
        let payload = parse(&build_kotlin_mirror_json(
            TransportState::Paused,
            0.0,
            &surface,
            None,
            None,
            &Dismissal::Dismissable,
        ));
        assert!(
            payload.get("progressLabel").is_none(),
            "T089 RED: indeterminate must exclude progressLabel"
        );
    }
}

/// A session whose totals were unknown at start stays indeterminate
/// for the whole session (FR-028a): the determinate key is absent —
/// the presence of the key IS the policy, so no zeros stand in.
#[test]
fn indeterminate_progress_carries_no_determinate_key() {
    let surface = SurfaceSnapshot {
        title: String::from("notas.md"),
        section_title: None,
        progress: Progress::Indeterminate,
    };
    let payload = parse(&build_kotlin_mirror_json(
        TransportState::Paused,
        0.0,
        &surface,
        None,
        None,
        &Dismissal::Dismissable,
    ));
    let object = object_of(&payload);

    assert_eq!(object.get("sectionTitle"), Some(&Value::Null));
    assert_eq!(
        object.get("progress"),
        Some(&Value::Object(Map::new())),
        "indeterminate is an empty progress object"
    );
}

/// A transient pause auto-dismisses at the shared window (FR-028e):
/// the Kotlin shape is the single-key `autoDismiss` object.
#[test]
#[expect(
    clippy::expect_used,
    reason = "test harness: the Kotlin mirror payload shape IS the contract under test, so a missing/ill-typed key must fail this test loudly rather than be swallowed"
)]
fn transient_pause_auto_dismisses_at_the_shared_window() {
    let surface = determinate(7, 60);
    let payload = parse(&build_kotlin_mirror_json(
        TransportState::Interrupted,
        42.0,
        &surface,
        Some("Silence demanded"),
        None,
        &Dismissal::AutoDismiss {
            after_ms: TRANSIENT_DISMISS_MS,
        },
    ));
    let object = object_of(&payload);

    assert_eq!(object.get("transport"), Some(&Value::from("interrupted")));
    assert_eq!(
        object.get("transientReason"),
        Some(&Value::from("Silence demanded"))
    );
    let dismissal = object
        .get("dismissal")
        .expect("dismissal present")
        .as_object()
        .expect("dismissal is an object");
    assert_eq!(sorted_keys(dismissal), vec!["autoDismiss"]);
    assert_eq!(dismissal.get("autoDismiss"), Some(&Value::from(3000)));
    assert_eq!(
        dismissal.get("autoDismiss").and_then(Value::as_u64),
        Some(TRANSIENT_DISMISS_MS),
        "the window is the shared constant, never a literal"
    );
}

/// A resident pause (no stated reason) carries the dismissable
/// shape: the resume control survives dismissal (FR-028e).
#[test]
#[expect(
    clippy::expect_used,
    reason = "test harness: the Kotlin mirror payload shape IS the contract under test, so a missing/ill-typed key must fail this test loudly rather than be swallowed"
)]
fn resident_pause_is_dismissable_without_auto_dismissal() {
    let surface = determinate(30, 60);
    let payload = parse(&build_kotlin_mirror_json(
        TransportState::Paused,
        900.0,
        &surface,
        None,
        None,
        &Dismissal::Dismissable,
    ));
    let dismissal = object_of(&payload)
        .get("dismissal")
        .expect("dismissal present")
        .as_object()
        .expect("dismissal is an object");
    assert_eq!(sorted_keys(dismissal), vec!["dismissable"]);
    assert_eq!(dismissal.get("dismissable"), Some(&Value::Null));
}

/// The haptic pattern id (FR-016) rides the same publish verbatim
/// when one was selected, and is null (never a default id) when the
/// trigger carried none.
#[test]
fn haptic_id_rides_verbatim_when_selected() {
    let surface = determinate(1, 60);
    let with_id = parse(&build_kotlin_mirror_json(
        TransportState::Playing,
        1.0,
        &surface,
        None,
        Some(3),
        &Dismissal::NonDismissable,
    ));
    assert_eq!(object_of(&with_id).get("haptic"), Some(&Value::from(3)));

    let without_id = parse(&build_kotlin_mirror_json(
        TransportState::Playing,
        1.0,
        &surface,
        None,
        None,
        &Dismissal::NonDismissable,
    ));
    assert_eq!(object_of(&without_id).get("haptic"), Some(&Value::Null));
}

/// The Kotlin form never leaks the in-app (`snake_case`) names or the
/// section counters: the vocabulary of the two wire forms stays
/// disjoint, so a field rename on one side cannot silently reach the
/// other.
#[test]
fn the_kotlin_form_never_carries_in_app_names() {
    let surface = determinate(5, 60);
    let json = build_kotlin_mirror_json(
        TransportState::Playing,
        10.0,
        &surface,
        Some("becoming noisy"),
        Some(2),
        &Dismissal::NonDismissable,
    );
    for leaked in [
        "position_secs",
        "section_title",
        "transient_reason",
        "consumed_units",
        "total_units",
        "\"k\"",
        "\"n\"",
    ] {
        assert!(
            !json.contains(leaked),
            "in-app name {leaked} leaked into the Kotlin form: {json}"
        );
    }
}
