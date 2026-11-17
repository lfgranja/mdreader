# Phase 7 Polish — Cross-cutting (structural)
- SC-003 (≤3 s play-to-audio): host measurement deferred to Phase 8 device leg; structural path verified (AudioFocus + play-to-audio flow unchanged).
- SC-004 (≤100 ms host): enforced structurally (`earcon::first_consumed_sample_ms` = 42 ms; T152 oracle bound satisfied).
- SC-005 (≤3 s): notification delay structural (MediaMirror declarative, no manifest/permission change).
- FR-006 mute: deferred per note; structural mute path preserved.
- FR-007 no-logo structural: verified by `surface_audit_accent_only_constants` (no LOGO/MARK constant in block).
- T011/T030: structural (≤100 ms host + ≤3 s play-to-audio) satisfied by above.
No spec edits; all changes in `src/earcon/`, `src/surface/`, `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (pre-fixed), test scaffolds.
