# Contract: Session, Tauri Bridge & Shell IPC — Voice Selection/Cycle (018-sona-inspired-voice)

- **Feature**: `specs/018-sona-inspired-voice`
- **Status**: Draft
- **Related normative contract**: `contracts/shell-ipc.md` (update in-place for any normative shell changes)
- **Primary surfaces**: `crates/session/src/store.rs`, `src-tauri/src/lib.rs` (+ commands/publish/inventory), player/shell verbs

## 1. Session Persistence (`crates/session`)

- **Key:** `last_voice: String` (value is a known `VoiceId`).
- **API:** `put_last_voice(&self, voice: VoiceId/impl AsRef<str-or-id>)` and `get_last_voice(&self) -> VoiceId` (or equivalent) consistent with existing settings methods.
- **Restore rule:** missing, unknown, or malformed stored value => return/select `pt_BR-faber-medium` (default). MUST NOT panic.

## 2. Bridge / Tauri Commands & Inventory

### Startup
- Resolve voices root (dev packaged assets / Android extracted catalog), initialize `adapter-synthesis` catalog root, seed normalization mode and selected voice from session store (default fallback), and publish initial catalog-aware inventory/state.

### Diagnose / voice inventory (additive, catalog-aware)
Replace single-voice hardcoded `pt_BR-faber-medium` assumptions with catalog-aware reporting:
- `active_voice`, `default_voice`, `voices`/catalog entries (or equivalent additive structure) including per-voice availability/presence/status fields as applicable.
- Preserve existing field meanings where retained; additive evolution preferred for compatibility with current consumers/tests.

### Commands (implemented additive)
- `get_voice` -> returns deterministic JSON `{active_voice, default_voice, voices:[{id,label,default,active,present}]}`.
- `set_voice` (argument: `voice: String`, catalog voice id) -> validates known catalog id via `VoiceId::parse_id`, persists `last_voice`, updates active selection, publishes state, returns `{active_voice,default_voice,persisted}`; on invalid id returns named refusal (`"voice selection refused: ..."`, no panic, no silent remap).
- Existing `set_normalization_mode` behavior unchanged.

All commands registered in `tauri::generate_handler![...]` and executed through the same Rust synthesis/session authority.

## 3. Cycle & Live Control Semantics

- **Verbs:** `CycleVoiceNext`, `CycleVoicePrev` remain part of the closed shell verb set.
- **Order:** fixed catalog declaration order; each invocation advances exactly one step (next or previous) with deterministic wrap across catalog bounds.
- **Application:**
  - Update active selection + persist + publish inventory/mirror state.
  - If playback/production is active, perform **safe restart-on-change**: invalidate current production generation, stop feeding stale buffers as needed, and restart synthesis/production from the **authoritative resume position** in the newly selected voice via the same Rust path.
  - MUST NOT splice or mix PCM from different voices in the active output stream.
- **Eligibility/refusal:** respect existing shell/session state rules (e.g. rejected in states where live control/cycle is not permitted such as Stopped/Completed, per `contracts/shell-ipc.md` and player contract tests) — update normative text/tests only to the extent required to define catalog-aware behavior without regressing unrelated semantics.

## 4. Mirror / Surface Impact

- Kotlin (`android-shell`) and frontend surfaces remain **mirrors only**: render/present published state, no synthesis/position/cycle authority logic added in Rust-forbidden layers.
- Any new published fields (active voice, catalog availability) are additive and must be ignored safely by consumers that do not use them, unless contract versioning requires coordinated updates.

## 5. Compatibility & Migration

- Old persisted settings lacking `last_voice` restore default voice.
- Existing diagnose consumers may see additional fields; removed/renamed hardcoded single-voice fields require coordinated updates to tests/contracts in the same change.
- `scripts/expected-tests.txt` and relevant `src-tauri`/player contract tests updated if observable behavior/targets change.