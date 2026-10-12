//! T061b — US4 document-heading resolution property tests
//! (Phase 4 test wave, BLOCKING gate T056).
//!
//! Invariants under test (tasks.md T061b, verbatim): "proptest
//! invariants: `headings_before(pos)` is non-increasing in
//! `pos` and returns a contiguous prefix; the nearest boundary
//! at or before `pos` is `≤ pos` and is maximal among
//! boundaries; no boundary → `0.0`; re-anchor never lands
//! between two boundaries when a boundary exists; identical
//! (headings, pos) → identical re-anchor point; FR-028d
//! re-resolve triggers on boundary-cross/resume/skip/re-anchor
//! and re-checks the fingerprint, with no polling or watcher
//! between events; FAILS before implementation".
//!
//! The task's own note on why the headings are a runtime
//! input and not a fixture: "This was previously a fixture
//! file under `tests/fixtures/`, which cannot work: T061 is a
//! production module (`crates/player/src/surface.rs`) and a
//! production module cannot depend on a test fixture, so the
//! ordered-headings provider is a runtime capability in this
//! feature — FR-024 defers the parser — and is therefore
//! tested like one. The real parser replaces it later without
//! changing T051/T061b/T061 assertions".
//!
//! Spec anchors:
//! - FR-028d (spec.md): "The section title on the lock-screen
//!   surface MUST be re-resolved from the document currently
//!   being read on every playback state change capable of
//!   altering which section the reader is in: crossing a
//!   section boundary during playback, resuming a session, a
//!   skip in either direction, and re-anchoring after an
//!   engine or voice change. On every resume the stored
//!   content fingerprint MUST additionally be checked, so a
//!   document edited on disk re-resolves against the current
//!   headings rather than the remembered ones. The document
//!   MUST NOT be polled for heading changes while no such
//!   event has occurred, and no file watcher is kept: between
//!   two such events a stale title is tolerated and declared
//!   rather than silently refreshed."
//! - FR-038 (spec.md): "A stored position MUST remain valid
//!   when the voice or engine changes, and MUST be re-anchored
//!   to the nearest sentence or node boundary in the new
//!   engine's timeline before reading resumes. ... If no
//!   boundary is available at or before the stored position,
//!   the position MUST fall back to the start of the document
//!   rather than to an arbitrary offset."
//! - SC-047 (spec.md): the section title is correct immediately
//!   after a boundary crossing, a resume, a skip in either
//!   direction, and a re-anchor, and never names a section the
//!   position has already left or a section it has not yet
//!   entered (FR-028d).
//!
//! FAILS before `crates/doc-structure/src/lib.rs` (T061c)
//! implements the capability: `doc_structure::Document` and
//! `doc_structure::Heading` are unresolved, so this test
//! target does not build. That build failure is the required
//! pre-implementation red state (T056).
//!
//! Expected API contract for T061c (the canonical surface this
//! file is written against):
//! - `doc_structure::Heading { pub position_secs: f64, pub
//!   title: String }` — one ordered heading; derives
//!   `Debug, Clone, PartialEq, Eq, serde::Serialize,
//!   serde::Deserialize`.
//! - `doc_structure::Document::new(headings: Vec<Heading>) ->
//!   Self` — sorts ascending by `position_secs` (stable), so
//!   the invariants hold for any input order, including
//!   unsorted lists and duplicate positions.
//! - `Document::headings_before(&self, pos: f64) -> &[Heading]`
//!   — the contiguous prefix of the ordered headings at or
//!   before `pos`; its length is non-decreasing in `pos`.
//! - `Document::nearest_boundary_at_or_before(&self, pos: f64)
//!   -> Option<&Heading>` — the greatest heading position ≤
//!   `pos`, or `None` when no heading is at or before `pos`.
//! - `Document::reanchor(&self, pos: f64) -> f64` — the
//!   nearest boundary at or before `pos` (FR-038), else
//!   exactly `0.0`.
//!
//! Property legs:
//! - H1 prefix contiguity: `headings_before(pos)` is the first
//!   `k` ordered headings for some `k` — no gaps, no
//!   reordering — and every returned heading has
//!   `position_secs ≤ pos`.
//! - H2 monotone growth: for `pos1 < pos2`,
//!   `headings_before(pos1).len() <= headings_before(pos2).len()`
//!   — the prefix never loses a heading as the position
//!   advances (the task's "non-increasing in `pos`" wording,
//!   asserted in its mathematically precise form: the boundary
//!   set only grows, and the earlier prefix is a prefix of the
//!   later one).
//! - H3 nearest boundary: `nearest_boundary_at_or_before(pos)`
//!   is ≤ `pos` and maximal — no heading falls in
//!   `(nearest.position_secs, pos]` — and is `None` exactly
//!   when every heading position is `> pos`.
//! - H4 re-anchor fallback (FR-038): `reanchor(pos)` is the
//!   nearest boundary's position when a boundary exists at or
//!   before `pos`, else exactly `0.0` — never a value strictly
//!   between two boundaries.
//! - H5 determinism: identical `(headings, pos)` — called twice
//!   and constructed twice — yields identical resolution.
//! - H6 boundary-cross re-resolution trigger (FR-028d): the
//!   resolved section changes ONLY on the closed event set
//!   {`BoundaryCross`, `Resume`, `Skip`, `Reanchor`}; between
//!   events it
//!   is unchanged and the resolver is never called (no polling,
//!   no watcher); on each event it is recomputed from the
//!   current position, carrying the content fingerprint — a
//!   mismatch invalidates the section (`None`), and it stays
//!   `None` until a re-anchor.
//!
//! Exactness note: every position is generated on the
//! quarter-second grid (`k × 0.25`), an exact binary-fraction
//! grid, so every `==`/`<=`/`>` comparison below is bit-exact
//! and every oracle is computed with the same expressions the
//! contract states; a tolerance would admit an off-boundary
//! re-anchor as conforming (FR-038 demands the *nearest*
//! boundary, and exactly `0.0` when none exists). The grid is
//! built with `f64::from(i32)` — a lossless `From` impl, not
//! an `as` cast — so the workspace's denied
//! `cast_precision_loss` is not triggered.
//!
//! Format note: every `prop_assert*!` message uses explicit
//! positional arguments (`{}`, `arg`), never implicit named
//! captures (`{var}`): proptest 1.11 builds the message
//! through `concat!`, and rustc rejects implicit captures in
//! macro-expanded format strings.

use proptest::prelude::*;

use doc_structure::{Document, Heading};

/// Heading positions on the signed quarter-second grid
/// `k × 0.25` ∈ [−3600, 3600] — exact binary fractions, so
/// every comparison against them is bit-exact. The grid is
/// built with `f64::from(i32)` (a lossless `From` impl, not
/// an `as` cast) to stay inside the workspace's denied
/// `cast_precision_loss`.
fn position_strategy() -> impl Strategy<Value = f64> {
    (-14_400i32..=14_400).prop_map(|quarters| f64::from(quarters) * 0.25)
}

/// A heading title: a short lowercase identifier — titles are
/// never document body text (the identifier-pattern rule the
/// heading model shares with the diagnostic record).
fn title_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9 -]{0,31}"
}

/// One heading: an arbitrary position and title. Positions are
/// deliberately unordered and may repeat — the constructor
/// normalizes any input order (H1).
fn heading_strategy() -> impl Strategy<Value = Heading> {
    (position_strategy(), title_strategy()).prop_map(|(position_secs, title)| Heading {
        position_secs,
        title,
    })
}

/// An arbitrary heading list — including unsorted input and
/// duplicate positions, which the constructor sorts (stable)
/// ascending by `position_secs`.
fn headings_strategy() -> impl Strategy<Value = Vec<Heading>> {
    proptest::collection::vec(heading_strategy(), 0..=32)
}

/// Every member of the closed re-resolution event set (H6).
fn event_strategy() -> impl Strategy<Value = ReanchorEvent> {
    prop_oneof![
        Just(ReanchorEvent::BoundaryCross),
        Just(ReanchorEvent::Resume),
        Just(ReanchorEvent::Skip),
        Just(ReanchorEvent::Reanchor),
    ]
}

/// An opaque content fingerprint — the `[u8; 32]` blake3-256
/// form of FR-036/FR-028d — generated as exactly 32 arbitrary
/// bytes.
fn fingerprint_strategy() -> impl Strategy<Value = [u8; 32]> {
    proptest::array::uniform(0u8..=255)
}

/// The constructor's normalization, computed independently of
/// the crate: a stable ascending sort by `position_secs` —
/// the constructor's documented behavior — so the properties
/// are checked against an expected value, never against a
/// re-implementation of the crate's own code path.
fn ordered(headings: &[Heading]) -> Vec<Heading> {
    let mut ordered = headings.to_vec();
    ordered.sort_by(|left, right| left.position_secs.total_cmp(&right.position_secs));
    ordered
}

/// The contiguous prefix of the ordered headings at or before
/// `pos` — the expected value of `headings_before`.
fn expected_prefix(headings: &[Heading], pos: f64) -> Vec<Heading> {
    ordered(headings)
        .into_iter()
        .take_while(|heading| heading.position_secs <= pos)
        .collect()
}

/// The greatest heading at or before `pos`, owned — the
/// expected value of `nearest_boundary_at_or_before`.
fn expected_nearest(headings: &[Heading], pos: f64) -> Option<Heading> {
    let prefix = expected_prefix(headings, pos);
    prefix.into_iter().last()
}

/// The section the document resolves to at `pos`: the title of
/// the nearest boundary at or before `pos`, or `None` when no
/// boundary is at or before `pos`.
fn section_at(document: &Document, pos: f64) -> Option<String> {
    document
        .nearest_boundary_at_or_before(pos)
        .map(|heading| heading.title.clone())
}

/// The closed re-resolution event set (FR-028d): exactly these
/// four events trigger a re-resolution of the current section —
/// no `Other`, no polling, no watcher between events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReanchorEvent {
    /// The position crossed a heading boundary during playback.
    BoundaryCross,
    /// A paused session resumed.
    Resume,
    /// A skip moved the position in either direction.
    Skip,
    /// An explicit re-anchor after an engine or voice change.
    Reanchor,
}

/// Every member of the closed event set, mapped to its own `u8`
/// discriminant. The match is exhaustive with **no `_ =>`
/// arm**: it compiles only while the enum has exactly the four
/// members it names — a fifth member (an `Other`/`Unknown`
/// catch-all) would fail to compile right here, and that
/// compile failure is the assertion (Principle VII;
/// `mechanism-gate.md` CHK031).
fn event_discriminant(event: ReanchorEvent) -> u8 {
    match event {
        ReanchorEvent::BoundaryCross => 0,
        ReanchorEvent::Resume => 1,
        ReanchorEvent::Skip => 2,
        ReanchorEvent::Reanchor => 3,
    }
}

/// The closed re-resolution event set, enumerated once. Exactly
/// four members — the exhaustive match above refuses a fifth.
const REANCHOR_EVENTS: [ReanchorEvent; 4] = [
    ReanchorEvent::BoundaryCross,
    ReanchorEvent::Resume,
    ReanchorEvent::Skip,
    ReanchorEvent::Reanchor,
];

/// Count of distinct values in `values` — the closed-set size
/// check for the `u8` discriminants of every member.
fn distinct_count(values: &[u8]) -> usize {
    let mut count = 0;
    for (index, value) in values.iter().enumerate() {
        if !values[..index].contains(value) {
            count += 1;
        }
    }
    count
}

/// The test's model of the FR-028d re-resolver: it records the
/// last resolved section and re-resolves ONLY on the closed
/// event set. `resolution_calls` counts actual resolutions, so
/// the no-polling property can assert the resolver is never
/// called between events.
#[derive(Clone, Debug)]
struct SectionResolver {
    /// The document whose headings are resolved.
    document: Document,
    /// The content fingerprint re-checked on every re-resolution
    /// (FR-028d).
    fingerprint: [u8; 32],
    /// The last resolved section title, or `None` when no
    /// boundary is at or before the position or the fingerprint
    /// mismatched.
    resolved_section: Option<String>,
    /// How many times the resolver actually ran — the
    /// no-polling counter.
    resolution_calls: u32,
    /// Whether a fingerprint mismatch has invalidated the
    /// section; the invalidation holds until a re-anchor
    /// (FR-028d: a changed fingerprint yields no section until
    /// re-anchored).
    invalidated: bool,
}

impl SectionResolver {
    /// A resolver over `document`, holding `fingerprint` as the
    /// current content fingerprint, with nothing resolved yet.
    fn new(document: Document, fingerprint: [u8; 32]) -> Self {
        Self {
            document,
            fingerprint,
            resolved_section: None,
            resolution_calls: 0,
            invalidated: false,
        }
    }

    /// Re-resolve at `pos`, carrying `fingerprint` (FR-028d):
    /// the re-resolution carries the content fingerprint, and a
    /// mismatch invalidates the section (`None`); a match
    /// resolves the section from the current position.
    fn resolve(&mut self, pos: f64, fingerprint: [u8; 32]) {
        self.resolution_calls += 1;
        if fingerprint != self.fingerprint {
            self.invalidated = true;
            self.resolved_section = None;
        } else if self.invalidated {
            // Still invalid: only a re-anchor restores the
            // section (FR-028d).
            self.resolved_section = None;
        } else {
            self.resolved_section = section_at(&self.document, pos);
        }
    }

    /// An event: re-resolve ONLY on the closed event set. The
    /// exhaustive match — no `_ =>` arm — is the closed-set
    /// assertion itself (Principle VII).
    fn on_event(&mut self, event: ReanchorEvent, pos: f64, fingerprint: [u8; 32]) {
        match event {
            ReanchorEvent::BoundaryCross | ReanchorEvent::Resume | ReanchorEvent::Skip => {
                self.resolve(pos, fingerprint);
            }
            ReanchorEvent::Reanchor => {
                // A re-anchor is the recovery path: it clears
                // the invalidation before resolving (FR-028d).
                self.invalidated = false;
                self.resolve(pos, fingerprint);
            }
        }
    }
}

/// A step of the re-resolution simulation: either advancing
/// the position without an event (between events — nothing
/// may be re-resolved, so the position itself is
/// irrelevant to the resolver) or one of the four
/// triggering events, carrying the position and the
/// fingerprint presented at it.
#[derive(Clone, Debug)]
enum Step {
    /// Advance the position without an event.
    Advance,
    /// A triggering event.
    Trigger {
        /// The event that occurred.
        event: ReanchorEvent,
        /// The position at the event.
        pos: f64,
        /// The fingerprint presented at the event.
        fingerprint: [u8; 32],
    },
}

/// One step of the simulation: an advance, a triggering event
/// carrying the matching fingerprint, or a triggering event
/// carrying a changed fingerprint (a document edited on disk —
/// exactly one byte flipped, so it always mismatches).
fn step_strategy(fingerprint: [u8; 32]) -> impl Strategy<Value = Step> {
    prop_oneof![
        Just(Step::Advance),
        (event_strategy(), position_strategy()).prop_map(move |(event, pos)| Step::Trigger {
            event,
            pos,
            fingerprint,
        },),
        (event_strategy(), position_strategy(), 0..32u8).prop_map(move |(event, pos, byte)| {
            let mut changed = fingerprint;
            changed[usize::from(byte)] ^= 0xFF;
            Step::Trigger {
                event,
                pos,
                fingerprint: changed,
            }
        },),
    ]
}

/// A full simulation: a heading list, its content fingerprint,
/// and a step sequence over them.
fn simulation_strategy() -> impl Strategy<Value = (Vec<Heading>, [u8; 32], Vec<Step>)> {
    (headings_strategy(), fingerprint_strategy()).prop_flat_map(|(headings, fingerprint)| {
        (
            Just(headings),
            Just(fingerprint),
            proptest::collection::vec(step_strategy(fingerprint), 0..=24),
        )
    })
}

/// The re-resolution event set is exactly the four closed
/// members — no `Other`: every member maps to its own `u8`
/// discriminant, the count of distinct discriminants is 4, and
/// the exhaustive match in `event_discriminant` (no `_ =>`
/// arm) compiles only while the enum has exactly these members
/// (Principle VII; FR-028d's closed trigger set).
#[test]
fn the_re_resolution_event_set_is_exactly_four_closed_members() {
    assert_eq!(REANCHOR_EVENTS.len(), 4);
    assert_eq!(
        [
            event_discriminant(ReanchorEvent::BoundaryCross),
            event_discriminant(ReanchorEvent::Resume),
            event_discriminant(ReanchorEvent::Skip),
            event_discriminant(ReanchorEvent::Reanchor),
        ],
        [0, 1, 2, 3]
    );
    assert_eq!(
        distinct_count(&[
            event_discriminant(REANCHOR_EVENTS[0]),
            event_discriminant(REANCHOR_EVENTS[1]),
            event_discriminant(REANCHOR_EVENTS[2]),
            event_discriminant(REANCHOR_EVENTS[3]),
        ]),
        4,
        "the re-resolution event set is not a closed 4-member set"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// H1 — prefix contiguity: `headings_before(pos)` returns
    /// the contiguous prefix of the ordered headings at or
    /// before `pos` — exactly the first `k` headings for some
    /// `k` (no gaps, no reordering), every one with
    /// `position_secs ≤ pos`.
    #[test]
    fn headings_before_returns_the_contiguous_prefix_at_or_before_pos(
        headings in headings_strategy(),
        pos in position_strategy(),
    ) {
        let document = Document::new(headings.clone());
        let before = document.headings_before(pos);
        let expected = expected_prefix(&headings, pos);

        // Exactly the first k headings of the ordered list — a
        // prefix, never a subsequence with gaps or reordering.
        prop_assert_eq!(
            before,
            expected.as_slice(),
            "headings_before({}) is not the contiguous ordered prefix at or before it",
            pos
        );
        // Every returned heading is at or before the position.
        prop_assert!(
            before.iter().all(|heading| heading.position_secs <= pos),
            "headings_before({}) returned a heading after the position",
            pos
        );
    }

    /// H2 — monotone growth: for `pos1 < pos2`,
    /// `headings_before(pos1).len() <= headings_before(pos2).len()`
    /// — the prefix never loses a heading as the position
    /// advances.
    #[test]
    fn the_prefix_only_grows_as_the_position_advances(
        headings in headings_strategy(),
        earlier in position_strategy(),
        later in position_strategy(),
    ) {
        let document = Document::new(headings);
        let (earlier_pos, later_pos) = if earlier <= later {
            (earlier, later)
        } else {
            (later, earlier)
        };

        let early = document.headings_before(earlier_pos);
        let late = document.headings_before(later_pos);

        // The prefix never loses a heading as the position
        // advances.
        prop_assert!(
            early.len() <= late.len(),
            "the prefix shrank from {} headings at or before {} to {} at or before {}",
            early.len(),
            earlier_pos,
            late.len(),
            later_pos
        );
        // The earlier prefix is a prefix of the later one — no
        // heading is dropped or reordered between them.
        prop_assert_eq!(
            early,
            &late[..early.len()],
            "the prefix at or before {} is not a prefix of the prefix at or before {}",
            earlier_pos,
            later_pos
        );
    }

    /// H3 — nearest boundary: `nearest_boundary_at_or_before(pos)`
    /// is `≤ pos` and maximal — no heading has a position in
    /// `(nearest.position_secs, pos]` — and it is `None`
    /// exactly when every heading position is `> pos`.
    #[test]
    fn the_nearest_boundary_is_the_greatest_at_or_before_pos(
        headings in headings_strategy(),
        pos in position_strategy(),
    ) {
        let document = Document::new(headings.clone());
        let ordered_headings = ordered(&headings);
        let expected = expected_nearest(&headings, pos);

        prop_assert_eq!(
            document.nearest_boundary_at_or_before(pos),
            expected.as_ref(),
            "nearest_boundary_at_or_before({}) is not the greatest heading at or before it",
            pos
        );

        match document.nearest_boundary_at_or_before(pos) {
            Some(nearest) => {
                // The nearest boundary is at or before the
                // position.
                prop_assert!(
                    nearest.position_secs <= pos,
                    "the nearest boundary {} is after the position {}",
                    nearest.position_secs,
                    pos
                );
                // It is maximal: no heading falls strictly
                // between it and the position.
                for heading in &ordered_headings {
                    if heading.position_secs > nearest.position_secs {
                        prop_assert!(
                            heading.position_secs > pos,
                            "a heading at {} lies between the nearest boundary and the position {}",
                            heading.position_secs,
                            pos
                        );
                    }
                }
            }
            None => {
                // None exactly when every heading position is
                // after the position (vacuously true for an
                // empty document).
                prop_assert!(
                    ordered_headings
                        .iter()
                        .all(|heading| heading.position_secs > pos),
                    "nearest_boundary_at_or_before({}) is None although a heading is at or before it",
                    pos
                );
            }
        }
    }

    /// H4 — re-anchor fallback (FR-038): `reanchor(pos)` is the
    /// nearest boundary's position when a boundary exists at or
    /// before `pos`, else exactly `0.0` — never a value
    /// strictly between two boundaries, never a value in
    /// `(boundary, pos]`.
    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "the property compares the re-anchor point against the nearest boundary, the document start 0.0, and every heading position with ==; every position is an exact quarter-second binary fraction (k × 0.25), so the equality is bit-exact and a tolerance would admit a re-anchor landing between two boundaries as conforming, defeating H4 and FR-038"
    )]
    fn reanchor_returns_the_greatest_boundary_or_exactly_zero(
        headings in headings_strategy(),
        pos in position_strategy(),
    ) {
        let document = Document::new(headings.clone());
        let ordered_headings = ordered(&headings);
        let anchor = document.reanchor(pos);

        match document.nearest_boundary_at_or_before(pos) {
            Some(nearest) => {
                // A boundary exists at or before the position:
                // the re-anchor point is exactly that boundary.
                prop_assert_eq!(
                    anchor,
                    nearest.position_secs,
                    "reanchor({}) is not the nearest boundary at or before it",
                    pos
                );
            }
            None => {
                // No boundary is at or before the position: the
                // re-anchor point is exactly the document start.
                prop_assert_eq!(
                    anchor,
                    0.0,
                    "reanchor({}) is not exactly 0.0 when no boundary is at or before it",
                    pos
                );
            }
        }

        // The re-anchor point never lands strictly between two
        // boundaries: it is either exactly the document start or
        // exactly a boundary of the document.
        let at_a_boundary = ordered_headings
            .iter()
            .any(|heading| heading.position_secs == anchor);
        prop_assert!(
            anchor == 0.0 || at_a_boundary,
            "reanchor({}) landed at {}, strictly between two boundaries",
            pos,
            anchor
        );
    }

    /// H5 — determinism: resolution is a pure function of
    /// `(headings, pos)` — identical inputs, called twice and
    /// constructed twice, yield identical re-anchor points,
    /// nearest boundaries, and prefixes.
    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "the property asserts equality of re-anchor points, boundaries, and prefixes, whose positions are f64 values compared with ==; every position is an exact quarter-second binary fraction (k × 0.25), so the equality is bit-exact and a tolerance would admit a nondeterministic resolution as conforming, defeating H5"
    )]
    fn resolution_is_a_pure_function_of_headings_and_pos(
        headings in headings_strategy(),
        pos in position_strategy(),
    ) {
        let first = Document::new(headings.clone());
        let second = Document::new(headings);

        // Called twice on the same document.
        prop_assert_eq!(
            first.reanchor(pos),
            first.reanchor(pos),
            "reanchor({}) is not idempotent on one document",
            pos
        );
        // Equal inputs, constructed twice.
        prop_assert_eq!(
            first.reanchor(pos),
            second.reanchor(pos),
            "reanchor({}) differs between two documents built from equal headings",
            pos
        );
        // The nearest boundary and the prefix are equally pure.
        prop_assert_eq!(
            first.nearest_boundary_at_or_before(pos),
            second.nearest_boundary_at_or_before(pos),
            "nearest_boundary_at_or_before({}) differs between two documents built from equal headings",
            pos
        );
        prop_assert_eq!(
            first.headings_before(pos),
            second.headings_before(pos),
            "headings_before({}) differs between two documents built from equal headings",
            pos
        );
    }

    /// H6 — boundary-cross re-resolution trigger (FR-028d): the
    /// resolved section changes ONLY on the closed event set
    /// {`BoundaryCross`, `Resume`, `Skip`, `Reanchor`}. Between
    /// events the section is unchanged and the resolver is never called
    /// (no polling, no watcher); on each event the section is
    /// recomputed from the current position, carrying the content
    /// fingerprint — a mismatch invalidates the section (`None`),
    /// and it stays `None` until a re-anchor.
    #[test]
    fn re_resolution_happens_only_on_the_closed_event_set(
        (headings, fingerprint, steps) in simulation_strategy(),
    ) {
        let document = Document::new(headings);
        let mut resolver = SectionResolver::new(document.clone(), fingerprint);

        // Before any event nothing is resolved: the section is
        // absent and the resolver has never run.
        prop_assert!(resolver.resolved_section.is_none());
        prop_assert_eq!(resolver.resolution_calls, 0);

        // The reference model of FR-028d: what the resolver
        // must hold after each step.
        let mut expected_section: Option<String> = None;
        let mut expected_calls: u32 = 0;
        let mut invalidated = false;

        for (index, step) in steps.iter().enumerate() {
            match step {
                // Between events: nothing is re-resolved — the
                // resolver is never called, however far the
                // position moves (no polling, no watcher,
                // FR-028d).
                Step::Advance => {}
                Step::Trigger {
                    event,
                    pos,
                    fingerprint: presented,
                } => {
                    resolver.on_event(*event, *pos, *presented);
                    // The event re-resolves exactly once,
                    // carrying the presented fingerprint: a
                    // mismatch invalidates the section, and only
                    // a re-anchor clears the invalidation.
                    if *presented != fingerprint {
                        invalidated = true;
                    } else if *event == ReanchorEvent::Reanchor {
                        invalidated = false;
                    }
                    if invalidated {
                        expected_section = None;
                    } else {
                        expected_section = section_at(&document, *pos);
                    }
                    expected_calls += 1;
                }
            }
            // After every step the resolver matches the model:
            // between events the section and the call count are
            // unchanged (the resolver is never called between
            // events), and on an event the section is recomputed
            // from the current position.
            prop_assert_eq!(
                &resolver.resolved_section,
                &expected_section,
                "the resolved section diverged from the FR-028d model after step {} of {} steps",
                index,
                steps.len()
            );
            prop_assert_eq!(
                resolver.resolution_calls,
                expected_calls,
                "the resolver ran {} times after step {} of {} steps — it must run exactly once per event and never between events",
                resolver.resolution_calls,
                index,
                steps.len()
            );
        }
    }
}
