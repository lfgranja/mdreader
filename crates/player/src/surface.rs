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
//! - **Determinacy** (FR-028a): fixed at session start
//!   and never switching mid-session — a session that
//!   knew its total resolves [`Progress::Determinate`]
//!   at every position (the total is the marked
//!   document extent, never an estimate), and a session
//!   that did not resolves [`Progress::Indeterminate`]
//!   for the whole session.
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

/// The progress indication (FR-028a) — and nothing
/// else: never elapsed or remaining time.
///
/// The variant is fixed at session start and never
/// switches mid-session: a session whose total spoken
/// length was known at start is
/// [`Progress::Determinate`] at every position, and a
/// session whose total was unknown is
/// [`Progress::Indeterminate`] for the whole session —
/// the total is never estimated.
///
/// The wire form is the externally-tagged variant
/// name (contracts/shell-ipc.md case-mapping table):
/// `{"determinate": {"position_secs": …,
/// "duration_secs": …}}` and `"indeterminate"` — the
/// indeterminate variant carries no duration field.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Progress {
    /// The total spoken length is known: the progress
    /// carries the session's position and the marked
    /// document extent (never an estimate).
    Determinate {
        /// The session's authoritative position, in
        /// seconds (FR-002 / FR-003).
        position_secs: f64,
        /// The marked document extent, in seconds — the
        /// greatest heading position the heading list
        /// marks (the resolver's only duration input).
        duration_secs: f64,
    },
    /// The total spoken length was unknown at session
    /// start: the progress carries no total — the
    /// whole session stays indeterminate, never an
    /// estimate (FR-028a).
    Indeterminate,
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
/// placeholder — FR-029) and the duration knowledge
/// (FR-028a). Everything else is a runtime input:
/// [`SurfaceResolver::resolve`] takes the position and
/// the ordered heading list per call, so the
/// re-resolution on boundary-cross, resume, skip, and
/// re-anchor (FR-028d) re-runs the same pure
/// function and can never flicker.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceResolver {
    /// The surface title, resolved once at construction
    /// (FR-029): the metadata title when present, else
    /// the filename as the stable placeholder.
    title: String,
    /// Whether the document's total spoken length was
    /// known at session start (FR-028a) — fixed for
    /// the whole session, never switching, never
    /// estimated.
    duration_known_at_start: bool,
}

impl SurfaceResolver {
    /// Create a resolver for one session (FR-028 /
    /// FR-029).
    ///
    /// `metadata_title` is the document's metadata
    /// title — the surface title when present (FR-029);
    /// `filename` is the stable placeholder title when
    /// it is absent (FR-029: the file name,
    /// independent of whether the document has
    /// headings — never the first spoken sentence,
    /// never a heading title);
    /// `duration_known_at_start` fixes the progress
    /// determinacy for the whole session (FR-028a:
    /// never switches mid-session, never estimated).
    #[must_use]
    pub fn new(
        metadata_title: Option<String>,
        filename: String,
        duration_known_at_start: bool,
    ) -> Self {
        Self {
            title: metadata_title.unwrap_or(filename),
            duration_known_at_start,
        }
    }

    /// Resolve the surface at a position (FR-028
    /// series).
    ///
    /// Pure in its inputs: identical
    /// `(headings, position_secs)` resolve to an
    /// identical snapshot (FR-028d), so the
    /// re-resolution on boundary-cross, resume, skip,
    /// and re-anchor can never flicker. The title is
    /// the session-start title (FR-029); the section
    /// is the most-recent-passed heading (FR-028c);
    /// the progress is determinate or indeterminate as
    /// fixed at session start (FR-028a), the
    /// determinate total being the marked document
    /// extent — never an estimate.
    #[must_use]
    pub fn resolve(&self, position_secs: f64, headings: &[Heading]) -> SurfaceSnapshot {
        SurfaceSnapshot {
            title: self.title.clone(),
            section_title: most_recent_passed_section(headings, position_secs),
            progress: if self.duration_known_at_start {
                Progress::Determinate {
                    position_secs,
                    duration_secs: marked_document_total(headings),
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

/// The marked document extent (FR-028a): the greatest
/// heading position the caller-supplied heading list
/// marks — the resolver's only duration input — or
/// `0.0` for an empty list. Never an estimate.
///
/// The list is ordered ascending by `position_secs`
/// (the T061b headings provider contract), so the
/// last heading marks the greatest position.
fn marked_document_total(headings: &[Heading]) -> f64 {
    match headings.last() {
        Some(heading) => heading.position_secs,
        None => 0.0,
    }
}
