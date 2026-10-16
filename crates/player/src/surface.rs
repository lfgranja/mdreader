//! The notification-surface content resolver (T061; FR-028 series, FR-029).
//!
//! The lock-screen and notification surface carries
//! exactly title + section + progress — never document
//! body text (neither the sentence being read nor any
//! paragraph) and never elapsed/remaining time, rate,
//! or voice (FR-028 series). The surface is a pure
//! view of the session's authoritative state: the Rust
//! module is the single source of truth for position
//! and transport (FR-002 / FR-003 / SC-035), and this
//! resolver never estimates anything the session did
//! not declare.
//!
//! - **Section** (FR-028b / FR-028c / FR-028d): the
//!   most-recent-passed heading — the greatest heading
//!   `position_secs` at or before the resolve position
//!   — else absent; never forward-looking. Every
//!   [`SurfaceResolver::resolve`] call re-resolves, so
//!   the boundary-cross / resume / skip / re-anchor
//!   triggers re-run the same pure function and can
//!   never flicker.
//! - **Determinacy** (FR-028a / FR-013): fixed at
//!   session start and never switching mid-session — a
//!   session that knew its totals resolves
//!   [`Progress::Determinate`] at every position, and a
//!   session that did not resolves
//!   [`Progress::Indeterminate`] for the whole session.
//! - **The parsed-content unit** (FR-013 /
//!   `data-model.md`): one unit is ONE segment the
//!   adapter emits — the sentence-level chunk of
//!   `adapter_synthesis::chunk::split_document` (the
//!   closed `BoundaryKind` set has no `Sentence` member,
//!   so the segment is the finest granularity the split
//!   exposes). `total_units` is therefore the SEGMENT
//!   COUNT of the split — a content fact, fixed at
//!   session start — and `consumed_units` is the index of
//!   the unit currently being spoken, projected from the
//!   session's authoritative position
//!   ([`SurfaceResolver::consumed_units_at`]). The
//!   projection is a position → content mapping: the
//!   document's PCM is the per-segment buffers
//!   concatenated in segment order, so seconds-per-unit
//!   is a PROJECTION BASIS, never a total. Audio-duration
//!   estimates never feed the fraction
//!   `consumed_units / total_units` (FR-013).
//! - **Title** (FR-029): the document metadata title
//!   when present, else the filename as the stable
//!   placeholder — never the first spoken sentence,
//!   never a heading title.
//! - **Dismissal** (FR-028e):
//!   [`Dismissal::NonDismissable`] while playing; a
//!   transient pause auto-dismisses at
//!   [`TRANSIENT_DISMISS_MS`]; a resident pause becomes
//!   [`Dismissal::Dismissable`].
//!
//! Headings are a runtime input — the T061b headings
//! provider supplies the ordered list, so a production
//! module never depends on a test fixture — and the
//! resolver is a pure function of its inputs: identical
//! `(headings, position_secs)` resolve to an identical
//! snapshot. The sets here are closed by construction
//! (Principle VII / FR-044): the two [`Progress`]
//! variants and the three [`Dismissal`] members are the
//! complete sets, so a new member is a coordinated
//! edit across this module, the shell-IPC contract,
//! and the Kotlin mirror, with a semantic-version
//! bump — never a catch-all arm.
//!
//! The module also holds [`MirrorState`] — the
//! shell-IPC mirror view the Kotlin `MediaSession`
//! consumes (contracts/shell-ipc.md): the session's
//! authoritative transport and position plus the
//! session-start progress, serializing to exactly the
//! contract's snake_case wire keys.

use engine_core::closed::TransportState;

/// The transient auto-dismiss window (FR-028e):
/// exactly 3 s.
///
/// A transient surface — one carrying a stated reason
/// (a failure reason under FR-037 or a focus-loss
/// pause reason under FR-041) — is dismissed this long
/// after the pause. The resident surface is never
/// auto-dismissed; it merely becomes dismissable, so
/// its paused state and resume control survive until
/// the reader chooses to swipe it away, stops the
/// resident component, or the document ends (FR-032f).
pub const TRANSIENT_DISMISS_MS: u64 = 3000;

/// One ordered document heading (FR-028b) — the
/// runtime input the T061b headings provider supplies
/// to [`SurfaceResolver::resolve`].
#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    /// The heading's spoken position, in seconds. The
    /// provider supplies the list ordered ascending by
    /// this position, one heading per position.
    pub position_secs: f64,
    /// The heading's title — the section title the
    /// surface shows while the position is at or after
    /// this heading and before the next one.
    pub title: String,
}

/// The progress indication (FR-013 / FR-028a) — and
/// nothing else: never elapsed or remaining time.
///
/// The variant is fixed at session start and never
/// switches mid-session: a session whose totals (the
/// section count AND the parsed-content unit total)
/// were known at start is
/// [`Progress::Determinate`] at every position, and a
/// session whose totals were unknown is
/// [`Progress::Indeterminate`] for the whole session —
/// the total is never estimated.
///
/// ## The unit (FR-013 / `data-model.md`)
///
/// One unit is ONE sentence-level segment
/// `adapter_synthesis::chunk::split_document` emits
/// (the closed `BoundaryKind` set has no `Sentence`
/// member, so the segment is the finest granularity the
/// split exposes). `total_units` is therefore the
/// segment count of the split — a content fact fixed
/// at session start — and `consumed_units` is the
/// index of the unit currently being spoken. The
/// caller projects the session's authoritative
/// position onto that offset through
/// [`SurfaceResolver::consumed_units_at`]; no
/// audio-duration estimate ever feeds the fraction
/// `consumed_units / total_units` (FR-013).
///
/// The wire form is the externally-tagged variant
/// name (contracts/shell-ipc.md case-mapping table):
/// `{"determinate": {"consumed_units": …,
/// "total_units": …, "k": …, "n": …}}`
/// and `"indeterminate"` — the indeterminate
/// variant carries no unit fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Progress {
    /// The session-start totals are known: the progress
    /// carries the consumed and total parsed-content
    /// units, the current section `k` (1-based), and
    /// the total sections `n`. Fixed at session start —
    /// never estimated, never flipped mid-session.
    /// Freeze-then-drop on re-parse.
    Determinate {
        /// Parsed-content units consumed — the index of
        /// the sentence-level unit currently being
        /// spoken (content-offset, never audio-duration
        /// or time-based).
        consumed_units: u64,
        /// Total parsed-content units fixed at session
        /// start: the segment count of the chunk split
        /// (content-offset, never an estimate, never
        /// time-based).
        total_units: u64,
        /// Current section index `k` (1-based) from the
        /// heading list at the resolve position.
        k: u32,
        /// Total sections `n` from the heading list
        /// (0 when no headings / zero sections).
        n: u32,
    },
    /// The session-start totals were unknown: the
    /// progress carries no units — the whole session
    /// stays indeterminate, never an estimate
    /// (FR-028a / FR-013). Zero sections resolve to
    /// indeterminate with a null section title.
    Indeterminate,
}

/// The frozen progress label (contracts/surface.md).
///
/// The template `"Section {k} of {N} — {p}% (content-offset)"`
/// is an ENGLISH IDENTIFIER: the machine-readable surfaces
/// (CLI `--format json`, the IPC payload) emit it verbatim;
/// the rendered user-visible pt-BR translation is a separate
/// rendering step that MUST NOT change this identifier
/// (contracts/surface.md "CLI surface").
///
/// `{p}` is the integer percent `(consumed × 100) / total`,
/// truncated toward zero with no decimals — the contract
/// freezes the template but not the rounding, so the
/// truncated-integer form is the chosen and pinned one.
///
/// Returns `None` for [`Progress::Indeterminate`] — no
/// determinable percent, no label (zero sections ⇒
/// indeterminate + null title, whole session, no flip:
/// contracts/surface.md / FR-028a).
#[must_use]
pub fn progress_label(progress: &Progress) -> Option<String> {
    match progress {
        Progress::Determinate {
            consumed_units,
            total_units,
            k,
            n,
        } => {
            let percent = if *total_units == 0 {
                0
            } else {
                (consumed_units * 100) / total_units
            };
            Some(format!("Section {k} of {n} — {percent}% (content-offset)"))
        }
        Progress::Indeterminate => None,
    }
}

/// The dismissal policy of the notification surface
/// (FR-028e) — how the surface may go away.
///
/// The policy splits on the transport and the surface
/// kind (see [`SurfaceResolver::dismissal`]): a
/// playing document is never dismissable, a transient
/// pause auto-dismisses at [`TRANSIENT_DISMISS_MS`],
/// and a resident pause becomes dismissable without
/// being dismissed automatically, so its resume
/// control survives.
#[derive(Debug, Clone, PartialEq)]
pub enum Dismissal {
    /// The surface cannot be dismissed — the ongoing
    /// state while audio plays (FR-028e: pausing is
    /// the only route to dismissal while a document
    /// plays).
    NonDismissable,
    /// The surface dismisses itself after the stated
    /// window — the transient-reason surface (a
    /// failure reason under FR-037 or a focus-loss
    /// pause reason under FR-041), dismissed
    /// [`TRANSIENT_DISMISS_MS`] after the pause.
    AutoDismiss {
        /// The window after which the surface dismisses
        /// itself, in milliseconds.
        after_ms: u64,
    },
    /// The surface may be dismissed by the reader but
    /// is never dismissed automatically — the resident
    /// surface after a pause, whose paused state and
    /// resume control remain available until the
    /// reader swipes it away, stops the resident
    /// component, or the document ends (FR-032f).
    Dismissable,
}

/// One resolved view of the notification surface
/// (FR-028 series): exactly title + section +
/// progress — never document body text,
/// elapsed/remaining time, rate, or voice.
///
/// The wire form is exactly the snake_case keys
/// `title`, `section_title`, and `progress`, with
/// `section_title` absent (not `null`) when no
/// heading has been passed yet (FR-028c).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SurfaceSnapshot {
    /// The surface title (FR-029): the document
    /// metadata title, or the filename as the stable
    /// placeholder.
    pub title: String,
    /// The most-recent-passed section title (FR-028b /
    /// FR-028c / FR-028d), or `None` before the first
    /// heading and for a heading-less document — never
    /// a forward-looking heading, never document body
    /// text. Skipped when `None` (the absent key —
    /// not a `null` — is the contract).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_title: Option<String>,
    /// The progress indication (FR-028a).
    pub progress: Progress,
}

/// The shell-IPC mirror state (contracts/shell-ipc.md)
/// — the view of the session the Kotlin `MediaSession`
/// mirror consumes: the session's authoritative
/// transport and position, and the progress
/// indication fixed at session start.
///
/// The mirror is derived, never a second clock
/// (SC-035 / FR-003): it carries the position the
/// Rust module reports and nothing the module does
/// not. The wire form is exactly the snake_case keys
/// `transport`, `position_secs`, and `progress`, with
/// the progress externally tagged
/// (`{"determinate": {…}}` / `"indeterminate"`).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct MirrorState {
    /// The session's authoritative transport state
    /// (FR-020), mirrored — never computed here.
    pub transport: TransportState,
    /// The session's authoritative position, in
    /// seconds (FR-002 / FR-003), mirrored — never
    /// computed here.
    pub position_secs: f64,
    /// The progress indication (FR-028a), determinate
    /// iff the duration was known at session start.
    pub progress: Progress,
}

/// The notification-surface content resolver (T061) —
/// the pure function from the session's declared
/// inputs to the surface view (FR-028 series /
/// FR-029).
///
/// The resolver holds only what is fixed at session
/// start: the title (metadata, else the filename
/// placeholder -- FR-029), the parsed-content unit total
/// with its projection basis (FR-013), and the totals
/// knowledge (FR-028a). Everything else is a runtime
/// input: [`SurfaceResolver::resolve`] takes the
/// position, the parsed-content offset the caller
/// projects through [`SurfaceResolver::consumed_units_at`],
/// and the ordered heading list per call, so the
/// re-resolution on boundary-cross, resume, skip, and
/// re-anchor (FR-028d) re-runs the same pure
/// function and can never flicker.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceResolver {
    /// The surface title, resolved once at construction
    /// (FR-029): the metadata title when present, else
    /// the filename as the stable placeholder.
    title: String,
    /// The total parsed-content units of the document
    /// (FR-013): the segment count the adapter's chunk
    /// split emitted, fixed at construction -- a content
    /// fact, never an estimate, never re-derived
    /// mid-session (freeze-then-drop).
    total_units: u64,
    /// The projection basis: the spoken seconds per
    /// parsed-content unit at session start. This is
    /// NEVER a progress total -- it maps the session's
    /// authoritative position onto the parsed-content
    /// offset (the PCM is the per-segment buffers
    /// concatenated in segment order, so unit `index`
    /// starts at `index * unit_secs`). A document whose
    /// totals were not known at start carries `0.0`,
    /// which projects every position to `0`.
    unit_secs: f64,
    /// Whether the document's session-start totals (the
    /// section count AND the parsed-content unit total)
    /// were known at session start (FR-028a / FR-013) --
    /// fixed for the whole session, never switching,
    /// never estimated.
    duration_known_at_start: bool,
}

impl SurfaceResolver {
    /// Create a resolver for one session (FR-028 /
    /// FR-029 / FR-013).
    ///
    /// `metadata_title` is the document's metadata
    /// title — the surface title when present (FR-029);
    /// `filename` is the stable placeholder title when
    /// it is absent (FR-029: the file name,
    /// independent of whether the document has
    /// headings — never the first spoken sentence,
    /// never a heading title);
    /// `total_units` is the parsed-content unit total
    /// fixed at session start — the segment count the
    /// adapter's chunk split emitted (FR-013: one unit
    /// is one sentence-level segment), the denominator
    /// of the `consumed_units / total_units` fraction,
    /// never an estimate;
    /// `total_secs` is the spoken length the projection
    /// basis derives from (`total_secs / total_units`
    /// seconds per unit); and
    /// `duration_known_at_start` fixes the progress
    /// determinacy for the whole session (FR-028a /
    /// FR-013: determinate only when BOTH the section
    /// count and the unit total were known at start,
    /// never switches mid-session, never estimated).
    #[must_use]
    pub fn new(
        metadata_title: Option<String>,
        filename: String,
        total_units: u64,
        total_secs: f64,
        duration_known_at_start: bool,
    ) -> Self {
        let unit_secs = if total_units == 0 || !(total_secs > 0.0) {
            0.0
        } else {
            total_secs / total_units as f64
        };
        Self {
            title: metadata_title.unwrap_or(filename),
            total_units,
            unit_secs,
            duration_known_at_start,
        }
    }

    /// The parsed-content offset the session position
    /// projects to (FR-013): the index of the unit
    /// currently being spoken - the count of units the
    /// position has already entered.
    ///
    /// The document's PCM is the per-segment buffers
    /// concatenated in segment order, so the unit
    /// sounding at `position_secs` starts at
    /// `index * unit_secs` and the offset is
    /// `position_secs / unit_secs` truncated toward
    /// zero, clamped to the session-start total. This
    /// is the position-to-content projection, NOT an
    /// audio-duration estimate of the progress: the
    /// fraction's denominator is the content total, and
    /// `unit_secs` only maps seconds onto it.
    ///
    /// The mapping is monotone non-decreasing in the
    /// position, so no re-parse, resume, or later
    /// publish can move the progress backward
    /// (FR-013). A position at or before the start, or
    /// a session whose totals were not known at start
    /// (`unit_secs == 0.0`), projects to `0`.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "the projection divides the authoritative position by the fixed seconds-per-unit basis and clamps the result against the session-start total. The f64-to-u64 cast is Rust's saturating cast (NaN/negative to 0, out-of-range to u64::MAX) and the u64-to-f64 comparison is a bound check against a value that cannot exceed the session's segment count - a fixture-scale document cannot approach 2^53 units, so neither conversion can lose the compared value."
    )]
    pub fn consumed_units_at(&self, position_secs: f64) -> u64 {
        if !(self.unit_secs > 0.0) || !(position_secs > 0.0) {
            // No projection basis (the totals were unknown
            // at start) or a position at/before the start:
            // nothing has been consumed.
            return 0;
        }
        let units = (position_secs / self.unit_secs).floor();
        if units >= self.total_units as f64 {
            // The position is at or past the document's
            // end: every unit is consumed, never beyond
            // the frozen total (no over-run, no backward
            // jump).
            return self.total_units;
        }
        units as u64
    }

    /// Resolve the surface at a position (FR-028
    /// series).
    ///
    /// Pure in its inputs: identical
    /// `(headings, position_secs, consumed_units)`
    /// resolve to an identical snapshot (FR-028d), so
    /// the re-resolution on boundary-cross, resume,
    /// skip, and re-anchor can never flicker. The title
    /// is the session-start title (FR-029); the section
    /// is the most-recent-passed heading (FR-028c); the
    /// progress is determinate or indeterminate as fixed
    /// at session start (FR-028a), the determinate total
    /// being the session-start parsed-content unit count
    /// (FR-013).
    ///
    /// `consumed_units` is the CURRENT parsed-content
    /// offset, projected by the caller through
    /// [`SurfaceResolver::consumed_units_at`]: the
    /// resolver never derives it from `position_secs`
    /// (that derivation is the time-based defect FR-013
    /// forbids) and never estimates the total. The
    /// section half is unchanged: `position_secs` stays
    /// the section provider's input (FR-028b).
    #[must_use]
    pub fn resolve(
        &self,
        position_secs: f64,
        consumed_units: u64,
        headings: &[Heading],
    ) -> SurfaceSnapshot {
        let k = headings
            .iter()
            .rposition(|h| h.position_secs <= position_secs)
            .map(|i| (i + 1) as u32)
            .unwrap_or(0);
        let n = headings.len() as u32;
        SurfaceSnapshot {
            title: self.title.clone(),
            section_title: most_recent_passed_section(headings, position_secs),
            progress: if self.duration_known_at_start {
                Progress::Determinate {
                    consumed_units,
                    total_units: self.total_units,
                    k,
                    n,
                }
            } else {
                Progress::Indeterminate
            },
        }
    }

    /// The dismissal policy for the surface (FR-028e).
    ///
    /// The truth table over the closed transport set:
    /// [`TransportState::Playing`] is
    /// [`Dismissal::NonDismissable`] for every
    /// transiency input — pausing is the only route to
    /// dismissal while audio plays; a live non-playing
    /// state ([`TransportState::Paused`] or
    /// [`TransportState::Interrupted`]) splits on the
    /// surface kind — a transient surface carrying a
    /// stated reason (a failure reason under FR-037 or
    /// a focus-loss pause reason under FR-041) is
    /// [`Dismissal::AutoDismiss`] at
    /// [`TRANSIENT_DISMISS_MS`], while the resident
    /// surface becomes [`Dismissal::Dismissable`] —
    /// never auto-dismissed, so its resume control
    /// survives; a torn-down state
    /// ([`TransportState::Stopped`] or
    /// [`TransportState::Completed`]) has nothing left
    /// to keep on the surface (the FR-032f teardown
    /// dismisses the notification), so any residual
    /// surface is [`Dismissal::Dismissable`].
    ///
    /// `transient` declares the surface kind, not the
    /// transport: the caller knows whether the pause
    /// carries a stated transient reason, and only the
    /// playing state ignores it.
    #[must_use]
    #[expect(
        clippy::unused_self,
        reason = "the FR-028e policy is a pure function of the transport and the surface's transiency — the resolver holds no state the policy reads; the method lives on the resolver so every surface query routes through one type (Principle II / FR-042)"
    )]
    pub fn dismissal(&self, transport: TransportState, transient: bool) -> Dismissal {
        match transport {
            TransportState::Playing => Dismissal::NonDismissable,
            TransportState::Paused | TransportState::Interrupted => {
                if transient {
                    Dismissal::AutoDismiss {
                        after_ms: TRANSIENT_DISMISS_MS,
                    }
                } else {
                    Dismissal::Dismissable
                }
            }
            TransportState::Stopped | TransportState::Completed => Dismissal::Dismissable,
        }
    }
}

/// The most-recent-passed section title (FR-028b /
/// FR-028c / FR-028d): the title of the heading with
/// the greatest `position_secs` at or before the
/// resolve position — never a heading beyond the
/// position (never forward-looking), never document
/// body text.
///
/// The heading list is ordered ascending by
/// `position_secs` (the T061b headings provider
/// contract), so the last heading at or before the
/// position is the most-recent-passed one; an empty
/// list or a position before the first heading
/// resolves to `None`.
fn most_recent_passed_section(headings: &[Heading], position_secs: f64) -> Option<String> {
    headings
        .iter()
        .rev()
        .find(|heading| heading.position_secs <= position_secs)
        .map(|heading| heading.title.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The frozen label for a determinate progress at the
    /// contract's chosen rounding: `(consumed × 100) / total`
    /// truncated toward zero, no decimals.
    #[test]
    fn determinate_label_is_the_frozen_template() {
        let progress = Progress::Determinate {
            consumed_units: 30,
            total_units: 120,
            k: 2,
            n: 7,
        };
        assert_eq!(
            progress_label(&progress).as_deref(),
            Some("Section 2 of 7 — 25% (content-offset)")
        );
    }

    /// The truncating rounding: 1/3 ⇒ 33, never 33.33 and never
    /// a rounded 34 (the contract freezes the template; the
    /// truncated-integer percent is the pinned form).
    #[test]
    fn percent_truncates_toward_zero() {
        let progress = Progress::Determinate {
            consumed_units: 1,
            total_units: 3,
            k: 1,
            n: 1,
        };
        assert_eq!(
            progress_label(&progress).as_deref(),
            Some("Section 1 of 1 — 33% (content-offset)")
        );
    }

    /// Zero total (zero sections with a known-at-start session):
    /// the percent is defined as 0, never a division panic and
    /// never NaN.
    #[test]
    fn zero_total_labels_zero_percent() {
        let progress = Progress::Determinate {
            consumed_units: 0,
            total_units: 0,
            k: 0,
            n: 0,
        };
        assert_eq!(
            progress_label(&progress).as_deref(),
            Some("Section 0 of 0 — 0% (content-offset)")
        );
    }

    /// Indeterminate carries no label — no determinable percent
    /// (contracts/surface.md / FR-028a).
    #[test]
    fn indeterminate_labels_nothing() {
        assert_eq!(progress_label(&Progress::Indeterminate), None);
    }
}
