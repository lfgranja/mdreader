# Contract: Playback Surfaces (lock-screen + single in-app screen)

FR-002 + FR-009 + FR-011 + FR-012 + FR-013 + FR-014 + FR-019. DESIGN-SONA appearance. Rust is the single authority; Kotlin `MediaSession`/notification is a mirror (SC-035).

## Lock-screen surface (MediaStyle notification)

- Fields: **at most** document title + current section title + progress indication. Never: document prose, elapsed/remaining time, playback rate, voice identity.
- Controls: play, pause, skip-forward/back. Rate-step reachable without unlock via lock-screen custom action (no value display; in-app stepper is full control). Skip = next/previous section boundary, fallback ±10 s when zero sections. **No Stop control on lock-screen** — `transport: playing|paused|stopped` is STATE, distinct from the Stop CONTROL which exists in-app only.
- Progress: bar = content-offset fraction + label `"Section {k} of {N} — {p}% (content-offset)"` (frozen template); determinate only when section count + total units known at session start, else indeterminate whole session (no flip, no backward jump); zero sections → indeterminate + no title.
- Interruption reason: ≤8-word line + distinct earcon, `reason_ttl_ms=3000` then cleared; undeliverable failures (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`) persistent (`reason_ttl_ms=null`). No spoken announcement.
- Dismissal: refused while playing; pause is the route to dismissal. Obeys platform lock-screen privacy (no workaround).
- After kill/power-loss: wait paused + explicit resume control on this surface (no visual dialog).

## Single minimal in-app screen (Svelte 5)

- Fields: document title, current section title, progress indication, speed control, transport (play, pause, skip-forward/back) + dedicated **Stop** (clears stored position; the Stop CONTROL, distinct from `transport: ...stopped` STATE). Never document prose.
- Speed: single bullet — 7 closed rungs `0.75x–4.0x` (default `1.0x`), one step per action, pitch-preserving, persisted app-wide. Rate-step affordance without value display permitted (D1 fix); rate numeric value MUST NOT appear anywhere (stepper moves position only — never rate value/prose/times/voice).
- Resume: deliberate-pause reopen → 3 s visual dialog (continue vs start-over; silence = continue from stored). Stop/document-end/content-change → no offer, start at `0.0`.
- Read-only mirror of the same `SurfaceSnapshot` the lock-screen renders (`status_view.rs` extended with section + progress + stepper + Stop; owns no state, renders only `surface.rs`/`failure.rs` strings).

## DESIGN-SONA palette tokens (single source of truth, resolves T044)

- Title `#F2F0E6`, section `#A9B8CE`, progress white / `#55688F`.
- Yellow `#FFD400` warning-only, red `#FF4D63` failure-only, green `#5FD0A0` prohibited.
- No marquee / body text / rate-value / voice identity on either surface.

## Payload (Rust → Kotlin mirror, `Channel`)

```json
{
  "title": "string (metadata, never first-sentence)",
  "section_title": "string | null (most-recent-passed heading; null before first / zero sections)",
  "progress": { "kind": "determinate", "consumed_units": 0, "total_units": 0, "k": 0, "n": 0 }
    | { "kind": "indeterminate" },
  "transport": "playing | paused | stopped",
  "reason": "≤8-word line | null",
  "reason_ttl_ms": 3000 | null,
  "dismissal": "non_dismissable_while_playing | dismissable"
}
```

Exhaustive match on `progress.kind`, `transport`, `dismissal` (gate 7: no `_ =>`). `reason_ttl_ms` (payload overload fix, F8) is distinct from `dismissal` (notification behavior).
