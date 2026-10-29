# 005-adopt-design-sona — Verification-Only Attestation Schedule

Status: SCHEDULED END-ONLY. No verification executed yet.
Only this task (`verify-plan`) claims completion of scheduling; NO subagent / agent / device-trial / host-script claims that any attestation item has passed.
Plan anchor: `local://PLAN.md` (slug 005-adopt-design-sona-plan); spec dir: `specs/005-adopt-design-sona/`.
Verification is end-only (post all RED / implement / scaffold / edit / commit steps). Nothing in this file is a claim of pass.

---

## 1. Rule of attestation (invariant for every line below)

- ONLY this file / this task (`verify-plan`) claims that the item is SCHEDULED.
- The ATTENDING ATTOR (host script / device trial / contract diff) is the ONLY source that may ever claim PASS/fail for the item.
- The agent / subagent / implementer / scaffold-gen / red-tests / main NEVER claim pass for any attestation item — never at scheduling time, never at trial time, never after commit.
- Deferred device-only legs are owned by `device-build`; the host agent does NOT claim them; they stay deferred until the device-build owner opens them.
- Atomic-commit hook audit is a process check, not a behavior claim.

---

## 2. End-only attestation items (scheduled — NOT executed)

Format per item: `ID | name | type (host / device / contract / process) | source of attestation | scheduled position | deferred owner (if any) | claim rule`.

### 2.1 T028–T031 — Host verification gates (deterministic, exit 0/non-0)

These are the 4 named host gates from the spec / quickstart / plan; each maps to a deterministic host script / tool over the audited file set (`status_view.html`, `status_view.rs`, `MediaMirror.kt` constants, `DESIGN-SONA/` audited files, header asset per contracts/status-surface.md).

| ID | Gate | Maps to SC / C-H / source | Type | Attestation source | Scheduled position | Claim rule |
|---|---|---|---|---|---|---|
| T028 | Host census (10 hex, out-of-role = fail) | SC-001 item (i) C-H1; data-model.md §SonaToken | host | `tests/005-verify/host_census.sh` (or equivalent tool) over audited files; exits 0 iff 10 members and no out-of-role paint | End, after all implement / RED / scaffold / edit / atomic-commit | Only host script claims pass; agent never claims |
| T029 | Contrast / role exclusivity (WCAG 2.1 AA; only `--sona-*` + signals) | SC-001 item (i) C-H4; C-H2; data-model.md §ContrastPair | host | Same host tool + contrast solver (parametrized by `#0B0F1A` / `#141B2D`); exit 0 iff all pairs ≥4.5:1 text / ≥3:1 non-text and role-exclusive | End | Only host tool claims pass |
| T030 | Surface inventory (aria roles + hidden `status`; no numeric rate painted; inventory closed) | SC-001 item (ii) C-H5; item (iv) aria inventory; FR-012/028 value-blind | host | Host inventory over audited file; exit 0 iff enumerable elements match closed inventory (no extra visible/hidden/dead; `aria-label` non-empty per interactive; `role="progressbar"`; live region hidden for value only; dots `aria-hidden`) | End | Only host tool claims pass |
| T031 | Latency oracle (≤100 ms first-consumed-sample; isolated + burst) | SC-005 C-H9; FR-010; data-model.md latency section | host | Host oracle over `backend.rs` / `player` binary (first consumed sample ≤100 ms); exit 0 iff bound holds in isolated + rapid-sequence (2–3 heterogeneous in 100–150 ms) | End | Only host oracle claims pass |

Verification order within T028–T031: T028 → T029 → T030 → T031 (census → contrast → inventory → latency; inventory depends on census being closed; latency is independent but runs after inventory closes the audited set).

### 2.2 SC-001 — Host half (C-H1 … C-H6) — deterministic only

From `specs/005-adopt-design-sona/plan.md` lines 94, 156; `data-model.md` §SC-001 mapping.

| Sub-item | Check | Source / file | Type | Attestation source | Scheduled position | Deferred / claim rule |
|---|---|---|---|---|---|---|
| C-H1 | Census: 10 hex members (8 `--sona-*` + `--signal-caution` `#FFD400` + `--signal-failure` `#FF4D63`); no new token; out-of-role = fail | `DESIGN-SONA/` audited + touched surface files | host | Host census script; exit 0/non-0 | End (with T028) | Only host script claims; device leg deferred (see §3) |
| C-H2 | Role exclusivity: only `--sona-*` + 2 signal pins paint; no body text / rate value / voice identity / green on player | Same audited set | host | Same tool (contrast + role filter) | End (with T029) | Only host script |
| C-H3 | Accent-only constant check (`MediaMirror.kt` `0xFFF2F0E6`; single flat accent over track; no gradient) | `MediaMirror.kt`; `contracts/earcon.md` routing | host | Constant-check script / grep + hex audit | End | Only host script |
| C-H4 | Contrast solver (WCAG 2.1 AA) over `#0B0F1A` (void) and `#141B2D` (stage); text ≥4.5, non-text ≥3 | Same | host | Contrast calculator script; exit 0/non-0 | End (with T029) | Only host script |
| C-H5 | Aria inventory (enumerated interactive controls carry `aria-label`; `role="progressbar"`; hidden live-region for value only; dot strip `aria-hidden`; no numeric painted) | `status_view.html`; `status_view.rs` | host | Inventory script over audited HTML/RS; exit 0/non-0 | End (with T030) | Only host script |
| C-H6 | Movement census (freeze under `prefers-reduced-motion`; only continuous motion = sliding pill; 3 s yellow pulse inside TTL; failure static red; banner one-shot ≤200 ms; zero other loops; census counts `animation`/`transform`) | `status_view.html`; `contracts/surface.md` payload | host | Movement census (count declarations + media-query presence) | End | Only host script; freeze leg of SC-007 is device-only (deferred) |

Device leg of SC-001 (visual trial of surfaces at both progress states, adb screenshot, checklist against same census) is **DEFERRED** — owner = `device-build` / gate 9 (see §2.6, §3). Host half is claimed only by the host tool above.

### 2.3 SC-006 — Host half (C-H1 + C-H2/C-H3 + C-H4 + C-H5 + A2 dual-state)

From `plan.md` line 99; `spec.md` §SC-006.

| Sub-item | Check | Source | Type | Attestation source | Scheduled position | Claim rule |
|---|---|---|---|---|---|---|
| C-H1 (re-run) | Census (same 10 hex, no new token; stamp set in-app: header `R` / stage card / gold active dot; no mark on notification / lock-screen) | Audited files + in-app header asset | host | Host census (re-executed after restyle) | End (after SC-001 C-H1) | Only host script; never agent |
| C-H2 / C-H3 | Token / stamp placement (accent `#F2F0E6` only; `--sona-stage` `#141B2D` raised surface only; stamps enumerated; no green; exclusion of 4 excluded screens: Cockpit Eyes-Free, Console Braun & Rams, Sumário Editorial, Matriz Telemetria) | `status_view.html`; design-dir | host | Token-placement + exclusion-check script | End | Only host script |
| C-H4 | Contrast (same calculator; header/card/notification pairs) | Same | host | Contrast solver | End | Only host script |
| C-H5 | Aria inventory (same rules; live region value announced; dots hidden; progress `role="progressbar"`; reason banner `role="status"`; freeze keeps announcements inalterable) | `status_view.html`; `contracts/surface.md` | host | Inventory + semantics script | End | Only host script |
| A2 dual-state | Served identity: `status_view.html` = actually-rendered surface (`dist/index.html` wiring is prerequisite; without it device legs of SC-001/SC-006 cannot pass; `spec.md` Q at line 97) | Build / bundle; `spec.md` | host / process | Build-check + bundle-identity diff (`status_view.html` vs served `dist/index.html`) | End (with gate 9) | Only build/check claims; agent never claims |

Device visual trial of SC-006 (Lente reduzida, stamps visible, reduced-motion freeze, exclusion screens unreachable, 10/10 visual) is **DEFERRED** — owner = `device-build`.

### 2.4 SC-007 — TalkBack device trial (10/10) + freeze leg

From `plan.md` line 100; `data-model.md`; `quickstart.md` §3.7; `spec.md` line 121 (freeze trigger = system toggle + `animator_duration_scale == 0` shim).

| Sub-item | Check | Source / device config | Type | Attestation source | Scheduled position | Deferred / claim rule |
|---|---|---|---|---|---|---|
| SC-007 talkback | 10/10 announcements match labels (labels, ladder step «velocidade <rung>» via hidden live region, progress state, 3 s reason + failure states); rate numeric NEVER painted; dots `aria-hidden`; `status_view.html` rendered | Device: Redmi 2201117TY API 33, TalkBack ON, parked-car media-speaker, trial sheet per `quickstart.md` §3.7 | device | **Device trial sheet** (human observer / second-operator only for pilot buzz-check) — 10/10 recorded; NOT a host script | End, after host half of SC-001/SC-006 + after device-build opens device leg | **DEFERRED — owner = `device-build`**; agent / host never claim; trial sheet is sole attestation source |
| SC-007 freeze | `prefers-reduced-motion: reduce`; pill freezes VISIBLE at track position (not gone); announcements unchanged 10/10; trigger = system accessibility toggle + `animator_duration_scale == 0` shim | Same device + adb / settings | device | Trial sheet freeze-leg entry; same source as above | End (same session) | Same deferred / device-build owner |

No host claim for SC-007 — it is device-only by design (Principle V; quickstart §3.7; data-model.md attribution).

### 2.5 SC-002 — Blind 7 earcon trial (device only; sound-only; haptics inert)

From `plan.md` line 95; `data-model.md` §SC-002; `contracts/earcon.md` (grammar + routing); `spec.md` line 121 (haptics inert; `haptic=None` at `lib.rs:6047`; `MediaMirror.kt:540-583` never fires); quickstart §3.3.

| Sub-item | Check | Source / protocol | Type | Attestation source | Scheduled position | Deferred / claim rule |
|---|---|---|---|---|---|---|
| SC-002 protocol | 7 types (play/pausa/salto-frente/salto-trás/taxa/aviso/falha); ≥15 trials/type; randomized constant-stimulus; no in-session feedback; 5–10% traps; 5–10 min familiarization; rapid-sequence sub-test (2–3 heterogeneous in 100–150 ms); confusion matrix + direction-of-skip sub-score; ≥24 h replication | `contracts/earcon.md`; `quickstart.md` §3.3 | device | **Trial sheet** (matrix + per-attempt list + replication count); media-speaker parked-car channel; second person only for pilot buzz-check | End (after SC-007 or independent, device session) | **DEFERRED — owner = `device-build`**; agent / host never claim; sound-only attribution per decision D3 (2026-10-24) |

Host does NOT claim SC-002; latency oracle (T031 / SC-005 C-H9) is the host-side analog, not this trial.

### 2.6 SC-003 — Notification inspection (10/10) — deferred device-only

From `plan.md`; `quickstart.md` §3.1; `contracts/surface.md` payload (title / section / progress only; never body / rate / voice / mark; warning static reason line + Ostinato 3 s TTL; failure persistent).

| Sub-item | Check | Source / device | Type | Attestation source | Scheduled position | Deferred / claim rule |
|---|---|---|---|---|---|---|
| SC-003 | 10/10 inspections: lock-screen / notification renders ONLY title (original language proper noun) + current section + progress; accent = marfim `#F2F0E6` only; no body / taxa / voice / marca; warning = static reason line + 3 s pulse; failure = static red `#FF4D63`; no green `#5FD0A0`; indeterminate progress visible; zero excluded-screen leakage | Device trial (same reference device) + trial sheet | device | **Trial sheet** 10/10; adb screenshot per inspection | End (after SC-002 / independent device session) | **DEFERRED — owner = `device-build`**; host never claims |

Mirror of SC-003 posture: device-only; deferred.

### 2.7 Gate 9 — `cargo tauri android build --target aarch64`

From `specs/005-adopt-design-sona/plan.md` line 21; `quickstart.md` line 36; 004 posture (artifact must exist).

| ID | Check | Source | Type | Attestation source | Scheduled position | Claim rule |
|---|---|---|---|---|---|---|
| G9 | Android aarch64 build succeeds; artifact exists (`target/aarch64-linux-android/release/` or Tauri bundle); no manifest / Gradle edit required (carried by 004); wiring prerequisite `status_view.html` → served bundle verified before build | `.cargo`; `Cargo.toml`; `android-shell/`; build log | process / host | `cargo tauri android build --target aarch64` exit 0 + artifact presence check; build log artifact saved to `tests/005-verify/gate9_build.log` | End (with A2 dual-state / before device legs open) | Only build / artifact-check claims pass; agent never claims; deferred device trial depends on artifact |

If G9 fails: device legs of SC-001 / SC-006 / SC-003 / SC-007 / SC-002 stay deferred; no silent pass.

---

## 3. Deferred device-only legs (owned by `device-build`; NOT claimed by this agent / host)

These are the per-plan / per-spec deferred legs (Principle V; D2 2026-10-24; quickstart §3; plan lines 90, 94, 99, 100). They stay open until `device-build` owner schedules / executes / attests. The agent / verify-plan / red-tests / scaffold-gen / main NEVER claim them.

| Trial / leg | Defer reason | Owner | Source when open | Attestation source (when open) | Not claimed by |
|---|---|---|---|---|---|
| SC-001 device visual (census at both progress states + exclusion screens unreachable) | Wiring prerequisite (`status_view.html` served) + device-only visual | `device-build` / G9 | `quickstart.md` §3.2; trial sheet template | Trial sheet + adb screenshot; checklist vs same C-H1..C-H6 | Any agent / host script |
| SC-006 device visual (Lente reduzida stamps + freeze + exclusion) | Same prerequisite + reduced-motion toggle | `device-build` | `quickstart.md` §3.6 | Trial sheet + freeze-leg entry | Any agent |
| SC-002 blind 7 (7 types, 10/10, rapid-sequence, confusion matrix) | Sound-only; haptics inert; human observer required | `device-build` | `quickstart.md` §3.3; `data-model.md` §SC-002; `contracts/earcon.md` | Trial sheet (matrix + replication); media-speaker; rigid mount + buzz-check admin only for pilot | Any agent / host |
| SC-003 notification purity (10/10 lock-screen inspections) | Device-only; notification surface rendered by Android mirror | `device-build` | `quickstart.md` §3.1; `contracts/surface.md` | Trial sheet 10/10; adb per inspection | Any agent / host |
| SC-007 TalkBack (10/10 + freeze leg) | Device-only; TalkBack + reduced-motion toggle + `animator_duration_scale` shim | `device-build` | `quickstart.md` §3.7; `spec.md` line 121 | Trial sheet 10/10 + freeze entry | Any agent / host |

Until these open, the only claims are: (a) host half of SC-001 / SC-006 / T028–T031, (b) G9 build artifact, (c) contract alignment (§4), (d) atomic-commit audit (§5) — and ONLY by their respective sources.

---

## 4. Contract alignment checks (diff-based; not execution claims)

From instruction: `contracts/earcon-mapping.md` vs `DESIGN-SONA.md` rate rows; `contracts/surface-audit.md` vs `status_view.html` served identity.

Note: in the workspace the contracts are `contracts/earcon.md` (rate / grammar / routing / haptic / CLI) and `contracts/surface.md` (lock-screen / in-app / notification / payload / DESIGN-SONA tokens). The design-dir file is `DESIGN-SONA/DESIGN-SONA.md`. The served-identity reference is `status_view.html` (required to be the rendered surface, per `spec.md` line 97 / Q). The instruction's filenames (`earcon-mapping.md`, `surface-audit.md`) are the alignment targets — the matching source files are the ones above; the check compares them.

| ID | Check | Source A | Source B | Type | Attestation source | Scheduled position | Claim rule |
|---|---|---|---|---|---|---|---|
| A-earcon | Rate rows match: `contracts/earcon.md` rate-step row (`Unison 880 Hz / 60 ms`; `≤500 ms` all; 300–2500 Hz range; play/pause/skip/warning/failure shapes) aligns with `DESIGN-SONA.md` / design-dir rate/stage description; `PITCH_TOLERANCE_CENTS=50`; `DUCK_GAIN=0.20±0.02`; no new audio assets; `rate` CLI `player` binary matches PCM descriptor | `contracts/earcon.md` (§Rate step, Routing, Named constants) | `DESIGN-SONA.md` (tokens / rate / stage) | contract / diff | `tests/005-verify/align_earcon.md` — line-by-line mapping with differences listed (none expected; any drift = fail) | End (with contract audits) | Only the diff / mapping doc claims alignment; agent never claims |
| A-surface | Served identity: `contracts/surface.md` payload / fields (title, section_title, progress kind, transport, reason/reason_ttl_ms, dismissal) matches actually-served `status_view.html` (field presence, no document prose / rate numeric / voice identity / body text; progress `role="progressbar"`; reason line ≤40 chars / ≤8-word pt-BR; accent only `#F2F0E6` / `#A9B8CE` / `#55688F`; no green `#5FD0A0`; single-column preserved) | `contracts/surface.md` (§Payload, Lock-screen, In-app, Resident-paused) | Served `status_view.html` / `dist/index.html` (post G9 wire) | contract / identity | `tests/005-verify/align_surface.md` — payload / display field comparison; identity diff (`status_view.html` == served identity or deviation recorded) | End (with A2 / G9) | Only identity-diff doc claims; agent never claims |

Both alignment outputs are artifacts (`tests/005-verify/align_*.md`), not claims of behavior. Any drift = recorded deviation = not a pass by agent; only the diff doc states the result.

---

## 5. Atomic-commit hook audit (process — not a behavior claim)

From instruction: "Atomic commit audit after every edit (deterministic hook verification)"; plan / contract governance.

| ID | Check | Source / hook | Type | Attestation source | Scheduled position | Claim rule |
|---|---|---|---|---|---|---|
| AC-01 | Commit hook runs on every edit (pre-commit / post-edit hook verifies atomicity: one logical change = one commit; no partial / broken commits; message references spec / contract / task) | `.git/hooks/`; repo config (`.specify`; `.agents`) | process | `tests/005-verify/ac_audit.log` — hook execution log per commit in this session; exit 0/non-0 per hook definition | End (after all edits / commits in this spec session) | Only hook / audit log claims conformance; agent never claims the hook passed — if hook fails, it is recorded, not covered |
| AC-02 | This schedule file + commit are atomic (this file is added in exactly one conventional commit; message includes task + spec slug; no extra unstaged / partial changes carried) | `git status`; `git log --oneline -1`; commit message | process | `git log -1 --pretty=format:"%H %s"` output captured in `tests/005-verify/ac_commit.md`; `git diff --cached --stat` at commit time shows only intended files | End (immediately after file creation + commit) | Only the commit / log claims atomicity; agent never claims |

The audit is of the hook and this commit only — not of SC-001 / SC-002 / G9 / device legs.

---

## 6. Verification-only attestation statement (required — appears in file + commit message)

> VERIFICATION-ONLY SCHEDULE — NOT EXECUTION. Every item above (T028–T031, SC-001 C-H1..C-H6, SC-006 host half, SC-007/SC-002/SC-003 device, G9, A-earcon, A-surface, AC-01/AC-02) is SCHEDULED at END ONLY. No item is claimed PASS by this agent / subagent / implementer / scaffold-gen / red-tests / main. ONLY the named attestation source for each item (host script / trial sheet / build log / diff doc / hook log / commit log) may ever claim PASS or record failure. Deferred device-only legs (SC-001 device / SC-006 device / SC-002 / SC-003 / SC-007) are owned by `device-build`; they stay deferred until that owner opens them; this agent NEVER claims them, silently or otherwise. Atomic-commit audit applies to this file and this commit only.

---

## 7. Schedule position summary (end-order)

1. All RED / scaffold / implement / edit steps complete (preceding phases).
2. Atomic conventional commit of this file (AC-02; immediately after creation).
3. G9 build (`cargo tauri android build --target aarch64`) → artifact + build log.
4. Contract alignment (A-earcon, A-surface) → diff artifacts.
5. Host verification gates T028 → T029 → T030 → T031 (host scripts; exit 0/non-0 recorded; only scripts claim).
6. SC-001 host half C-H1..C-H6 (host tool; only tool claims).
7. SC-006 host half (C-H1 + C-H2/C-H3 + C-H4 + C-H5 + A2) (host tool + identity diff; only source claims).
8. AC-01 hook audit (per edit / this commit; log records only).
9. Device-only legs — ONLY when / if `device-build` opens them: SC-002 → SC-003 → SC-007 (trial sheets; only trial sheets claim; never agent).
10. Final attestation closure: this file + AC-01 log + AC-02 commit + all artifacts in `tests/005-verify/` are the complete record. Nothing else claims pass.

No verification is run now. This file is the schedule.
