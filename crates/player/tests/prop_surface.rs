//! T051 — US4 property: notification-surface +
//! determinacy invariants (FR-028 series / FR-029).
//!
//! Invariants under test (tasks.md T051, verbatim):
//! "surface = title + section + progress + controls
//! ONLY — never body/elapsed/remaining/rate/voice;
//! section = most-recent-passed else absent (never
//! forward-looking, never body); determinacy fixed at
//! session start, never switches mid-session, never
//! estimated; title from metadata else
//! filename-stable-placeholder, never first sentence;
//! non-dismissable while playing, transient 3 s
//! auto-dismiss vs resident becomes-dismissable;
//! headings supplied via T061b stub; FAIL before
//! implementation".
//!
//! Spec anchors: FR-028 (the lock-screen and
//! notification surfaces carry the document title, the
//! most-recent-passed section title, and the progress
//! indication — and nothing else: no document body
//! text, neither the sentence being read nor any
//! paragraph), FR-028a (progress is determinate only
//! when the document's total spoken length is actually
//! known at session start, indeterminate otherwise —
//! never estimated, never switching mid-session),
//! FR-028b/FR-028c (the section title is the most
//! recent preceding heading — none before the first
//! heading or for a heading-less document), FR-028d
//! (the section is re-resolved on
//! boundary-cross/resume/skip/re-anchor), FR-028e
//! (non-dismissable while playing; a transient pause
//! auto-dismisses at 3 s; a resident pause becomes
//! dismissable), and FR-029 (the title comes from
//! document metadata, else a stable filename-derived
//! placeholder — never the first sentence).
//!
//! Expected API contract for T061 (the production
//! code this test gates — it does not exist yet, which
//! is the required red state), exactly as declared for
//! T057/T061:
//! - `player::surface::Heading { position_secs: f64,
//!   title: String }` — one ordered heading, supplied
//!   at runtime by the T061b headings provider (a
//!   production module cannot depend on a test
//!   fixture, so headings are a runtime input).
//! - `player::surface::Progress::{Determinate {
//!   consumed_units, total_units, k, n },
//!   Indeterminate}` — the progress indication.
//! - `player::surface::Dismissal::{NonDismissable,
//!   AutoDismiss { after_ms }, Dismissable}` — the
//!   dismissal policy.
//! - `player::surface::SurfaceSnapshot { title:
//!   String, section_title: Option<String>, progress:
//!   Progress }` — serializing to exactly the keys
//!   `title`, `section_title` (absent when `None`),
//!   and `progress`.
//! - `player::surface::SurfaceResolver::new(
//!   metadata_title: Option<String>, filename: String,
//!   total_units: u64, total_secs: f64,
//!   duration_known_at_start: bool)`,
//!   `consumed_units_at(&self, position_secs: f64) -> u64`,
//!   `resolve(&self, position_secs: f64,
//!   consumed_units: u64, headings: &[Heading]) ->
//!   SurfaceSnapshot`, and `dismissal(&self, transport:
//!   TransportState, transient: bool) -> Dismissal`.
//! - `player::surface::TRANSIENT_DISMISS_MS: u64 =
//!   3000`.
//!
//! Declared oracles (the contract T061 implements
//! against):
//! - Section (FR-028c/FR-028d): the title of the
//!   heading with the greatest `position_secs` at or
//!   before the resolve position — the
//!   most-recent-passed heading — else `None`. Never a
//!   heading beyond the position (never
//!   forward-looking), never document body text.
//! - Determinacy (FR-028a/FR-028c):
//!   `duration_known_at_start` fixes the variant for
//!   the whole session. The determinate total is the
//!   marked document extent — the greatest heading
//!   position the caller-supplied heading list marks
//!   (the resolver's only duration input), `0.0` for
//!   an empty list — never an estimate.
//! - Title (FR-029): the metadata title when present,
//!   else a stable placeholder derived from the
//!   filename — never the first sentence, never a
//!   heading title.
//! - Dismissal (FR-028e): `NonDismissable` while
//!   `Playing` (for every transiency input);
//!   `AutoDismiss { after_ms: TRANSIENT_DISMISS_MS }`
//!   for a transient pause; `Dismissable` for a
//!   resident pause.
//!
//! Shape contract: all four types derive `Debug`,
//! `Clone`, and `PartialEq`; `SurfaceSnapshot` and
//! `Progress` additionally derive `Serialize`, with
//! `section_title` skipped when `None` (the absent
//! key — not a `null` — is the contract) and the
//! `Indeterminate` variant carrying no duration field.
//!
//! Exactness note: every position and heading is
//! generated on the quarter-second grid (`k × 0.25`),
//! an exact binary-fraction grid, so position
//! comparisons are bit-exact.
//!
//! Format note: every `prop_assert*!` message uses
//! explicit positional arguments (`{}`, `arg`), never
//! implicit named captures (`{var}`) — proptest
//! builds the message through `concat!`, and rustc
//! rejects implicit captures in macro-expanded format
//! strings.
//!
//! FAILS before `crates/player/src/surface.rs` (T061)
//! exists: the `player::surface` module is
//! unresolved, so this test target does not build.
//! That build failure is the required pre-implementation
//! red state.

use proptest::prelude::*;

use engine_core::closed::TransportState;
use player::surface::{Dismissal, Heading, Progress, SurfaceResolver, TRANSIENT_DISMISS_MS};

/// Heading titles — a closed vocabulary deliberately
/// disjoint from every filename (and from every
/// metadata title), so a filename-derived placeholder
/// can never collide with a heading title and the
/// never-a-heading-title assertion stays meaningful.
fn heading_title_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("alpha"),
        Just("bravo"),
        Just("charlie"),
        Just("delta"),
        Just("echo"),
        Just("foxtrot"),
    ]
    .prop_map(|title| title.to_string())
}

/// Filenames — a closed vocabulary disjoint from every
/// heading title and every metadata title (see
/// [`heading_title_strategy`]).
fn filename_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("recording-0001.wav"),
        Just("document-0002.mp3"),
        Just("broadcast-0003.ogg"),
        Just("lecture-0004.m4a"),
    ]
    .prop_map(|filename| filename.to_string())
}

/// Document metadata titles — `None` (undeclared) or
/// the one declared title, disjoint from every heading
/// title and filename.
fn metadata_title_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        Just(Some("Declared Metadata Title".to_string())),
    ]
}

/// Ordered heading lists — ascending `position_secs`,
/// one heading per position, possibly empty — the
/// shape the T061b headings provider supplies at
/// runtime.
fn headings_strategy() -> impl Strategy<Value = Vec<Heading>> {
    proptest::collection::vec((0u64..=14_400, heading_title_strategy()), 0..=12).prop_map(
        |mut entries| {
            entries.sort_by_key(|(quarters, _)| *quarters);
            entries.dedup_by_key(|(quarters, _)| *quarters);
            entries
                .into_iter()
                .map(|(quarters, title)| Heading {
                    position_secs: quarters as f64 * 0.25,
                    title,
                })
                .collect()
        },
    )
}

/// Resolve positions on the exact quarter-second grid
/// `k × 0.25` ∈ [0, 3600].
fn position_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// The keys the notification surface MUST NEVER carry
/// (the FR-028 series: no body text, no elapsed or
/// remaining time, no rate, no voice).
const FORBIDDEN_SURFACE_KEYS: [&str; 5] = ["body", "elapsed", "remaining", "rate", "voice"];

/// The most-recent-passed section (FR-028c/FR-028d):
/// the title of the heading with the greatest
/// `position_secs` at or before `position_secs`, else
/// `None` — never a heading beyond the position (never
/// forward-looking), never document body.
#[expect(
    clippy::float_cmp,
    reason = "heading positions and resolve positions are exact binary fractions on the quarter-second grid; <= is the at-or-before test and a tolerance would admit a forward-looking section"
)]
fn most_recent_passed_section(headings: &[Heading], position_secs: f64) -> Option<String> {
    headings
        .iter()
        .rev()
        .find(|heading| heading.position_secs <= position_secs)
        .map(|heading| heading.title.clone())
}

/// The session-start parsed-content unit total oracle
/// (FR-013): the unit count the resolver was constructed
/// with, fixed for the whole session — never the greatest
/// heading position the caller-supplied list marks, and
/// never an estimate.
const fn total_units_oracle() -> u64 {
    24
}

/// The projection basis oracle: the spoken seconds per
/// parsed-content unit. The unit total times this basis
/// is the document's spoken length.
const fn unit_secs_oracle() -> f64 {
    5.0
}

/// The consumed parsed-content offset the oracle projects
/// the position onto (FR-013): the whole units the
/// position has entered, clamped to the session-start
/// total.
fn consumed_units_oracle(position_secs: f64) -> u64 {
    let units = (position_secs / unit_secs_oracle()).floor();
    let total = total_units_oracle() as f64;
    if units >= total {
        return total_units_oracle();
    }
    units as u64
}

/// Build a resolver over the oracle totals — the one
/// construction every property drives, so the totals are
/// fixed at session start exactly once.
fn resolver_for(
    metadata_title: Option<String>,
    filename: String,
    duration_known_at_start: bool,
) -> SurfaceResolver {
    SurfaceResolver::new(
        metadata_title,
        filename,
        total_units_oracle(),
        total_units_oracle() as f64 * unit_secs_oracle(),
        duration_known_at_start,
    )
}

/// The serialized key set of a snapshot: the JSON
/// object's keys, sorted — empty when the value did
/// not serialize to an object (a failure the callers'
/// key-set assertion reports).
fn serialized_keys(value: &serde_json::Value) -> Vec<&str> {
    let mut keys: Vec<&str> = value
        .as_object()
        .into_iter()
        .flat_map(serde_json::Map::keys)
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn the_surface_carries_exactly_title_section_and_progress(
        (metadata_title, filename, duration_known_at_start, headings, position) in (
            metadata_title_strategy(),
            filename_strategy(),
            prop_oneof![Just(true), Just(false)],
            headings_strategy(),
            position_strategy(),
        ),
    ) {
        // S1 (FR-028 series): the surface is title +
        // section + progress ONLY. Whatever the inputs,
        // the serialized snapshot carries exactly the
        // declared keys — never
        // body/elapsed/remaining/rate/voice.
        let resolver = resolver_for(metadata_title, filename, duration_known_at_start);
        let snapshot = resolver.resolve(position, resolver.consumed_units_at(position), &headings);
        let value = match serde_json::to_value(&snapshot) {
            Ok(value) => value,
            Err(_) => serde_json::Value::Null,
        };
        let keys = serialized_keys(&value);
        let mut expected_keys = vec!["progress", "title"];
        if snapshot.section_title.is_some() {
            expected_keys.push("section_title");
        }
        expected_keys.sort_unstable();
        prop_assert_eq!(
            &keys,
            &expected_keys,
            "the surface did not carry exactly title + section + progress (FR-028 series)"
        );
        for forbidden in FORBIDDEN_SURFACE_KEYS {
            prop_assert!(
                !keys.contains(&forbidden),
                "the surface carried the forbidden {} key (FR-028 series: never body/elapsed/remaining/rate/voice)",
                forbidden
            );
        }
        // The Indeterminate variant asserts no total —
        // an indeterminate bar carries no duration
        // field (FR-028a: never estimated).
        if matches!(snapshot.progress, Progress::Indeterminate) {
            let progress_carries_a_duration = value
                .get("progress")
                .and_then(|progress| progress.get("determinate"))
                .is_some();
            prop_assert!(
                !progress_carries_a_duration,
                "the indeterminate progress carried a determinate duration field (FR-028a)"
            );
        }
    }

    #[test]
    fn the_section_is_the_most_recent_passed_heading_else_absent(
        (headings, position) in (headings_strategy(), position_strategy()),
    ) {
        // S2 (FR-028c/FR-028d): the section is the
        // most-recent-passed heading — the greatest
        // heading position at or before the resolve
        // position — else absent. Never forward-looking
        // (never the next heading), never document
        // body.
        let resolver =
            resolver_for(None, "recording-0001.wav".to_string(), true);
        let snapshot = resolver.resolve(position, resolver.consumed_units_at(position), &headings);
        let expected = most_recent_passed_section(&headings, position);
        prop_assert_eq!(
            snapshot.section_title.as_deref(),
            expected.as_deref(),
            "the section at {} did not resolve to the most-recent-passed heading (FR-028c/FR-028d)",
            position
        );
        if let Some(section) = &snapshot.section_title {
            let passed_a_heading_with_that_title = headings
                .iter()
                .any(|heading| heading.position_secs <= position && heading.title == *section);
            prop_assert!(
                passed_a_heading_with_that_title,
                "the section {:?} at {} is not a heading at or before the position — a forward-looking or body-text section (FR-028c)",
                section,
                position
            );
        }
    }

    #[test]
    fn determinacy_is_fixed_at_session_start_and_never_switches(
        (duration_known_at_start, metadata_title, filename, headings, positions) in (
            prop_oneof![Just(true), Just(false)],
            metadata_title_strategy(),
            filename_strategy(),
            headings_strategy(),
            proptest::collection::vec(position_strategy(), 1..=16),
        ),
    ) {
        // S3 (FR-028a/FR-028c): determinacy is fixed at
        // session start and never switches mid-session,
        // and the total is never estimated — a session
        // that knew its total is determinate at every
        // position; a session that did not is
        // indeterminate at every position.
        let resolver = resolver_for(metadata_title, filename, duration_known_at_start);
        let mut determinate_count = 0usize;
        let mut indeterminate_count = 0usize;
        for position in &positions {
            let snapshot =
                resolver.resolve(*position, resolver.consumed_units_at(*position), &headings);
            match snapshot.progress {
                Progress::Determinate {
                    consumed_units,
                    total_units,
                    ..
                } => {
                    prop_assert_eq!(
                        consumed_units,
                        consumed_units_oracle(*position),
                        "the determinate progress at {} did not carry the projected parsed-content offset as the consumed content units (FR-013)",
                        position
                    );
                    prop_assert_eq!(
                        total_units,
                        total_units_oracle(),
                        "the determinate progress at {} did not carry the session-start unit total as content units (FR-013: the segment count, never the heading extent)",
                        position
                    );
                    determinate_count += 1;
                }
                Progress::Indeterminate => {
                    indeterminate_count += 1;
                }
            }
        }
        if duration_known_at_start {
            prop_assert_eq!(
                indeterminate_count,
                0,
                "a session that knew its total at start went indeterminate — determinacy switched mid-session (FR-028a/FR-028c)"
            );
            prop_assert_eq!(
                determinate_count,
                positions.len(),
                "a session that knew its total at start was not determinate at every position (FR-028a/FR-028c)"
            );
        } else {
            prop_assert_eq!(
                determinate_count,
                0,
                "a session with an unknown total went determinate — the total was estimated (FR-028a/FR-028c)"
            );
            prop_assert_eq!(
                indeterminate_count,
                positions.len(),
                "a session with an unknown total was not indeterminate at every position (FR-028a/FR-028c)"
            );
        }
    }

    #[test]
    fn the_title_comes_from_metadata_else_a_stable_filename_placeholder(
        (metadata_title, filename, headings, position) in (
            metadata_title_strategy(),
            filename_strategy(),
            headings_strategy(),
            position_strategy(),
        ),
    ) {
        // S4 (FR-029): the metadata title wins when
        // present; else a stable filename-derived
        // placeholder — never the first sentence, never
        // any heading title.
        let resolver = resolver_for(metadata_title.clone(), filename.clone(), true);
        let snapshot = resolver.resolve(position, resolver.consumed_units_at(position), &headings);
        match &metadata_title {
            Some(declared) => {
                prop_assert_eq!(
                    snapshot.title.as_str(),
                    declared.as_str(),
                    "the metadata title did not win over the placeholder (FR-029)"
                );
            }
            None => {
                let again =
                    resolver.resolve(position, resolver.consumed_units_at(position), &headings);
                prop_assert_eq!(
                    snapshot.title.as_str(),
                    again.title.as_str(),
                    "the filename-derived placeholder was not stable across calls for the same filename (FR-029)"
                );
                prop_assert!(
                    !snapshot.title.is_empty(),
                    "the filename-derived placeholder was empty (FR-029)"
                );
                let first_heading_title =
                    headings.first().map(|heading| heading.title.as_str());
                prop_assert!(
                    first_heading_title.is_none_or(|first| first != snapshot.title.as_str()),
                    "the title was the first heading's title — the first-sentence defect (FR-029)"
                );
                let any_heading_title = headings
                    .iter()
                    .any(|heading| heading.title == snapshot.title);
                prop_assert!(
                    !any_heading_title,
                    "the title was a heading title — the placeholder must be filename-derived (FR-029)"
                );
            }
        }
    }

    #[test]
    fn the_dismissal_policy_follows_the_transport_and_the_transiency(
        (transport, transient) in (
            prop_oneof![Just(TransportState::Playing), Just(TransportState::Paused)],
            prop_oneof![Just(true), Just(false)],
        ),
    ) {
        // S5 (FR-028e): non-dismissable while playing
        // (for every transiency input); a transient
        // pause auto-dismisses at exactly the 3 s
        // window; a resident pause becomes dismissable.
        // The strategy covers the policy's named
        // domain — `Playing` and `Paused`, the two
        // states the dismissal contract names.
        let resolver = resolver_for(None, "recording-0001.wav".to_string(), true);
        let dismissal = resolver.dismissal(transport, transient);
        let expected = if transport == TransportState::Playing {
            Dismissal::NonDismissable
        } else if transient {
            Dismissal::AutoDismiss {
                after_ms: TRANSIENT_DISMISS_MS,
            }
        } else {
            Dismissal::Dismissable
        };
        prop_assert_eq!(
            &dismissal,
            &expected,
            "the dismissal for {:?} (transient: {}) did not resolve to the FR-028e policy",
            transport,
            transient
        );
    }

    #[test]
    fn re_resolution_is_deterministic_for_identical_inputs(
        (metadata_title, filename, duration_known_at_start, headings, position) in (
            metadata_title_strategy(),
            filename_strategy(),
            prop_oneof![Just(true), Just(false)],
            headings_strategy(),
            position_strategy(),
        ),
    ) {
        // S6 (FR-028d): the resolver is a pure function
        // of its inputs — identical (headings, position)
        // resolve to an identical snapshot, so the
        // re-resolution on boundary-cross, resume,
        // skip, and re-anchor can never flicker.
        let resolver = resolver_for(metadata_title, filename, duration_known_at_start);
        let first = resolver.resolve(position, resolver.consumed_units_at(position), &headings);
        let second = resolver.resolve(position, resolver.consumed_units_at(position), &headings);
        prop_assert_eq!(
            &first,
            &second,
            "identical (headings, position) inputs resolved to different snapshots (FR-028d)"
        );
    }
}

/// The transient auto-dismiss window is exactly 3 s
/// (FR-028e).
#[test]
fn the_transient_dismiss_window_is_exactly_three_seconds() {
    assert_eq!(
        TRANSIENT_DISMISS_MS, 3000,
        "the transient auto-dismiss window drifted from 3 s (FR-028e)"
    );
}

/// The dismissal truth table (FR-028e), pinned:
/// `Playing` is non-dismissable for every transiency
/// input; a transient pause auto-dismisses at exactly
/// the declared window; a resident pause becomes
/// dismissable.
#[test]
fn the_dismissal_truth_table_is_pinned() {
    let resolver = resolver_for(None, "recording-0001.wav".to_string(), true);
    assert_eq!(
        resolver.dismissal(TransportState::Playing, false),
        Dismissal::NonDismissable,
        "a playing session is non-dismissable (FR-028e)"
    );
    assert_eq!(
        resolver.dismissal(TransportState::Playing, true),
        Dismissal::NonDismissable,
        "a playing session is non-dismissable whatever the transiency (FR-028e)"
    );
    assert_eq!(
        resolver.dismissal(TransportState::Paused, true),
        Dismissal::AutoDismiss {
            after_ms: TRANSIENT_DISMISS_MS,
        },
        "a transient pause auto-dismisses at the declared window (FR-028e)"
    );
    assert_eq!(
        resolver.dismissal(TransportState::Paused, false),
        Dismissal::Dismissable,
        "a resident pause becomes dismissable (FR-028e)"
    );
}

/// The section is absent before the first heading and
/// for a heading-less document, and present at the
/// boundary itself (FR-028c: the most-recent-passed
/// heading, never forward-looking).
#[test]
fn the_section_is_absent_before_the_first_heading() {
    let resolver = resolver_for(None, "recording-0001.wav".to_string(), true);
    let headings = vec![
        Heading {
            position_secs: 10.0,
            title: "alpha".to_string(),
        },
        Heading {
            position_secs: 20.0,
            title: "bravo".to_string(),
        },
    ];
    let before = resolver.resolve(5.0, resolver.consumed_units_at(5.0), &headings);
    assert_eq!(
        before.section_title, None,
        "the section must be absent before the first heading (FR-028c)"
    );
    let headingless = resolver.resolve(5.0, resolver.consumed_units_at(5.0), &[]);
    assert_eq!(
        headingless.section_title, None,
        "the section must be absent for a heading-less document (FR-028c)"
    );
    let at_the_boundary = resolver.resolve(10.0, resolver.consumed_units_at(10.0), &headings);
    assert_eq!(
        at_the_boundary.section_title,
        Some("alpha".to_string()),
        "the section at a heading boundary is that heading — a passed heading counts (FR-028c)"
    );
    let between = resolver.resolve(15.0, resolver.consumed_units_at(15.0), &headings);
    assert_eq!(
        between.section_title,
        Some("alpha".to_string()),
        "the section between headings is the most-recent-passed heading (FR-028c)"
    );
}

/// The metadata title wins over the filename
/// placeholder, and the placeholder is stable for an
/// identical filename (FR-029).
#[test]
fn the_metadata_title_wins_and_the_placeholder_is_stable() {
    let declared = resolver_for(
        Some("Declared Title".to_string()),
        "recording-0001.wav".to_string(),
        true,
    );
    let snapshot = declared.resolve(15.0, declared.consumed_units_at(15.0), &[]);
    assert_eq!(
        snapshot.title, "Declared Title",
        "the metadata title must win (FR-029)"
    );
    let first = resolver_for(None, "document-0002.mp3".to_string(), true);
    let second = resolver_for(None, "document-0002.mp3".to_string(), true);
    assert_eq!(
        first
            .resolve(15.0, first.consumed_units_at(15.0), &[])
            .title,
        second
            .resolve(15.0, second.consumed_units_at(15.0), &[])
            .title,
        "the filename-derived placeholder must be stable for an identical filename (FR-029)"
    );
}

/// The serialized surface shape (S1): exactly `title`
/// + `progress` when the section is absent, `title` +
/// `section_title` + `progress` when present — never
/// body/elapsed/remaining/rate/voice.
#[test]
fn the_serialized_surface_shape_carries_exactly_the_declared_keys() {
    let resolver = resolver_for(
        Some("Declared Title".to_string()),
        "recording-0001.wav".to_string(),
        true,
    );
    let headings = vec![Heading {
        position_secs: 10.0,
        title: "alpha".to_string(),
    }];
    let with_section = resolver.resolve(15.0, resolver.consumed_units_at(15.0), &headings);
    let value = match serde_json::to_value(&with_section) {
        Ok(value) => value,
        Err(_) => serde_json::Value::Null,
    };
    assert_eq!(
        serialized_keys(&value),
        vec!["progress", "section_title", "title"],
        "a snapshot with a section must serialize to exactly title + section_title + progress (FR-028 series)"
    );
    let without_section = resolver.resolve(5.0, resolver.consumed_units_at(5.0), &headings);
    let value = match serde_json::to_value(&without_section) {
        Ok(value) => value,
        Err(_) => serde_json::Value::Null,
    };
    assert_eq!(
        serialized_keys(&value),
        vec!["progress", "title"],
        "a snapshot without a section must serialize to exactly title + progress, with the absent section key omitted (FR-028 series)"
    );
}
