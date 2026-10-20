# Contract: Playback Surfaces (lock-screen + resident notification + single in-app screen)

FR-002 + FR-009 + FR-011 + FR-012 + FR-013 + FR-014 + FR-019, plus 001 FR-009a/FR-028e/FR-032 for the resident-paused notification surface. DESIGN-SONA appearance. Rust is the single authority; Kotlin `MediaSession`/notification is a mirror (SC-035).

## Lock-screen surface (MediaStyle notification — the media playback surface)

- Fields: **at most** document title (original language — proper noun, exempt from pt-BR per FR-011) + current section title + progress indication. Never: document prose, elapsed/remaining time, playback rate, voice identity.
- Controls: play, pause, skip-forward/back. Rate-step reachable without unlock via lock-screen custom action (no value display; in-app stepper is full control). Skip = next/previous section boundary, fallback ±10 s when zero sections. **No Stop CONTROL on the media lock-screen surface** — `transport: playing|paused|stopped` is STATE (Interrupted/Completed are EndKind, never transport values), distinct from the Stop CONTROL (the transport Stop, which clears the stored position) which exists in-app only. The always-resident service's paused notification is a DIFFERENT surface and is outside this clause: its resident-stop ends residency and PRESERVES the stored position (§Resident-paused notification surface).
- Progress: bar = content-offset fraction + label `"Section {k} of {N} — {p}% (content-offset)"` (frozen template; English identifier — CLI `--format json` per T030b/T040b outputs this identifier string directly; the rendered user-visible pt-BR translation, e.g. `"Seção {k} de {N} — {p}% (deslocamento de conteúdo)"`, is a separate rendering step); determinate only when section count + total units known at session start, else indeterminate whole session (no flip, no backward jump); zero sections → indeterminate + no title.
- Interruption reason: ≤8-word pt-BR line (soft target) + ≤40-char hard limit (Android content-text limit; if the two conflict, ≤40 chars wins) + distinct earcon, `reason_ttl_ms=3000` then cleared; undeliverable failures (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`) persistent (`reason_ttl_ms=null`). No spoken announcement. Mirror visual = giant transport glyph (≥48dp, high-contrast DESIGN-SONA tokens below, never body text) + progress + warning/failure banner asserted by T026 snapshot; haptic distinct per contracts/earcon.md rides outside STREAM_MUSIC so sounds-off/DND path holds.
- Fixed reason lines (closed; `FocusDelayed`/`Deferred` shows no line — zero-audio hold):
| Reason | Lock-screen line (pt-BR) |
|---|---|
| `FocusDenied` | `Foco de áudio negado — pausado` |
| `TransientPaused` (pause outcome) | `Pausado por interrupção — retome` |
| `PermanentLoss` | `Áudio perdido — pausado` |
| `DeviceRemoved` | `Fones removidos — pausado` |
| `VoiceUnavailable` | `Voz indisponível — leitura não inicia` |
| `FormatUndeliverable` | `Formato do documento não pode ser lido` |
| `DecodeFailed` | `Falha ao decodificar — leitura parada` |
- Dismissal: refused while playing; pause is the route to dismissal. Obeys platform lock-screen privacy (no workaround).
- After kill/power-loss: wait paused + explicit resume control on this surface (no visual dialog).

## Single minimal in-app screen (`src-tauri/src/status_view.html` + `src-tauri/src/status_view.rs`)

- Fields: document title, current section title, progress indication, speed control, transport (play, pause, skip-forward/back) + dedicated **Stop** (clears stored position; the Stop CONTROL, distinct from `transport: ...stopped` STATE). Never document prose.
- Speed: single bullet — 7 closed rungs `0.75x–4.0x` (default `1.0x`), one step per action, pitch-preserving, persisted app-wide. Rate-step affordance without value display permitted (D1 fix); rate numeric value MUST NOT appear anywhere (value-blind on both surfaces; stepper moves position only and orients by position + earcon/haptic — never rate value/prose/times/voice).
- Resume: deliberate-pause reopen → 3 s visual dialog (continue vs start-over; silence = continue from stored). Stop/document-end/content-change → no offer, start at `0.0`.
- Read-only mirror of the same `SurfaceSnapshot` the lock-screen renders (`status_view.rs` extended with section + progress + stepper + Stop; owns no state, renders only `surface.rs`/`failure.rs` strings).

## CLI surface (Principle II) — `player progress` mirrors tasks.md T030b flag shape (`--consumed/--total/--k/--n | --indeterminate`, `--format json`); tasks.md is normative for flags.

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

Display-vs-payload: purity applies to display fields only — display projection keys ⊆ {title, section_title, progress}. The full mirror payload additionally carries transport ∈ {playing,paused,stopped}, reason/reason_ttl_ms, dismissal per this contract; these are state/behavior, not display leaks. Prohibited anywhere: document prose, elapsed/remaining time, numeric rate value, voice identity.

Exhaustive match on `progress.kind`, `transport`, `dismissal` (gate 7: no `_ =>`). `reason_ttl_ms` (payload overload fix, F8) is distinct from `dismissal` (notification behavior).

## Resident-paused notification surface (always-resident service)

The always-resident service's paused notification — the surface that persists between sessions while a resumable position exists (`specs/001-android-media-shell/spec.md` FR-009a) — is a DISTINCT surface from the media lock-screen surface above, and this section, not the clause above, governs it. It occupies the same notification id (one id is one surface), so exactly one of the two is on the lock screen at a time: while the session is resident-paused the resident notification REPLACES the media one, and while audio plays the media one replaces the resident one. It is rendered by the resident lifecycle, not by the §Payload mirror, which governs the media surface only.
- Fields: document title + the paused-state word. Never: section title, progress indication, elapsed/remaining time, playback rate, voice identity, document prose — residency without active playback is not a position within the document (001 FR-009a).
- Controls (closed set): resume (continue at the stored position), resident-stop (end residency, PRESERVE the stored position), and the swipe-away dismissal (ends residency, preserves the position). Never: play/pause transport, skip-forward/back, rate-step, or the Stop CONTROL.
- The resident-stop is NOT the Stop CONTROL: the Stop CONTROL is the transport Stop that CLEARS the stored position (001 FR-032; the in-app Stop of FR-019), and it exists in-app only. The resident-stop never clears the stored position — it ends residency and leaves the document resumable — so FR-010 is untouched: a resident-stopped document is still offered for resume, a Stop CONTROL document is not.
- Label: the resident-stop's pt-BR label names the verb ("Parar", T130) and never states an effect — the in-app Stop control announces its effect ("Parar e limpar posição"); the two controls are told apart by their EFFECT (preserve vs clear) — a shared verb never makes them the same control.
- Rationale (T131 adjudication): FR-002 forbids the Stop CONTROL on the lock screen because that control is destructive — it discards the stored position — and a one-tap destructive action does not belong on a glance surface. The resident-stop is not destructive, so the prohibition's purpose is not defeated by it, and the surface it lives on is not the media lock-screen surface. The resident-stop exists because the resident notification MAY persist indefinitely after a pause (001 FR-009a): it is the reader's labelled, no-unlock way out of that persistence, and it is the route 001 FR-028e names as "stops the resident component"; the swipe-away is a second route (001 FR-009a), not a substitute for a labelled one, and the resident notification occupies the foreground-service id, so the platform's foreground handling makes the swipe-away an unreliable sole exit. Removing the resident-stop would contradict 001 FR-009a/FR-028e and the `resident_stop` intent in 001's `contracts/shell-ipc.md`, and would leave the reader unable to end a resident session without unlocking — a driving-safety regression under constitution Principle VIII. Resolution: scope adjudication — this surface is outside FR-002's lock-screen-surface scope, the contract names it explicitly, and the implementation is unchanged.

Adjudication record: T131 (Phase 16 Addendum 3, feature 004-ressoa-dogfood-build) — scope adjudication; contract amended, no code changed. The paired change required in `specs/004-ressoa-dogfood-build/spec.md` FR-002 (scoping the prohibition to the media playback surface and naming this surface) is recorded in the T131 adjudication report and MUST be applied by the spec owner in the same coordination cycle (004 spec.md §Normative Contract References: "When a contract and this spec appear to diverge, the contract is authoritative; updates must be coordinated").
