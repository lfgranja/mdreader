//! The document fingerprint (T059): the blake3-256
//! content hash (FR-036 / FR-036a) and the FR-038
//! re-anchor.
//!
//! The fingerprint is a pure function of the
//! document's CONTENT — never of its path — so a
//! moved or renamed document resolves to the same
//! bookmark key and therefore the same stored
//! position (FR-036), while any edited byte yields
//! a different key, making the stored position
//! unreachable: a changed document is a different
//! document, offered nothing and starting from its
//! first spoken position (FR-036a).
//!
//! The re-anchor owns the FR-038 snap for a
//! cross-voice / cross-engine resume: the nearest
//! boundary at or before the stored position in
//! the new engine's timeline, else exactly `0.0` —
//! never an arbitrary offset. Boundary RESOLUTION
//! is delegated to doc-structure (T061c, the
//! ordered-headings provider); this side owns the
//! snap, the `0.0` fallback, and the report.

use blake3::Hasher;

/// Fingerprint: blake3-256 of document bytes (FR-036 / FR-038).
/// Same bytes → same 32-byte hash regardless of path; 1-bit change → different hash.
#[must_use]
pub fn fingerprint(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    // `update` returns `&mut Hasher` for chaining; the
    // chained form is not used here (unused_results deny).
    let _ = hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// The FR-038 re-anchor report: where reading
/// resumes in the new engine's timeline, and the
/// boundary that was reported for it.
///
/// `Eq` is precluded by the `f64` field; every
/// position lives on the quarter-second grid, an
/// exact binary-fraction grid, so `PartialEq` is
/// exact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reanchor {
    /// The re-anchor point: the greatest boundary at
    /// or before the stored position, or `0.0` when
    /// no boundary precedes it (the document start).
    pub nearest: f64,
    /// The boundary the re-anchor landed on, or
    /// `None` when the `0.0` fallback fired — the
    /// resume never lands on an arbitrary offset
    /// (FR-038).
    pub reported: Option<f64>,
}

/// The FR-038 re-anchor of `stored_offset_secs`
/// against the new engine's `boundaries`: the
/// greatest boundary at or before the stored
/// position, or exactly `0.0` when none precedes
/// it (FR-038).
///
/// `reported` carries the boundary the re-anchor
/// landed on, or `None` when the fallback fired. A
/// boundary exactly at the stored position is at or
/// before it (the lookup is inclusive). The scan
/// covers the whole list, so the result holds for
/// any input order — the caller supplies the
/// ordered list (doc-structure, T061c).
#[must_use]
pub fn reanchor(stored_offset_secs: f64, boundaries: &[f64]) -> Reanchor {
    // The greatest boundary at or before the stored
    // position.
    let mut greatest: Option<f64> = None;
    for &boundary in boundaries {
        if boundary <= stored_offset_secs && greatest.is_none_or(|previous| boundary > previous) {
            greatest = Some(boundary);
        }
    }
    greatest.map_or(
        Reanchor {
            nearest: 0.0,
            reported: None,
        },
        |reported| Reanchor {
            nearest: reported,
            reported: Some(reported),
        },
    )
}
