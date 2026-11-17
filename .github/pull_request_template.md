## Summary
<!-- What changed and why. Link spec/task/issue when applicable. -->

## Scope / Blast Radius
- [ ] Low (docs/tests/localized)
- [ ] Medium (single crate/surface)
- [ ] High (playback authority, session/store, JNI/Android shell, contracts, supply chain, CI)

## Deterministic Verification (required gate evidence)
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo nextest run --workspace` (or CI `--profile ci`)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo doc --workspace --no-deps` (if public API/docs affected)
- [ ] `cargo deny check advisories licenses bans sources` + `cargo audit` (if deps/lockfile affected)
- [ ] `bash scripts/verify.sh` (gates 1-8; report pass/fail summary)
- [ ] Android/gate 9 build or device evidence (if Android/JNI/UI/shell affected)

## Contracts / Constitution
- [ ] No Rust playback-authority logic moved to Kotlin/JNI (mirror/transport only).
- [ ] Closed sets preserved (no `_ =>` catch-all in `crates/*/src`; no `Other/Unknown` additions).
- [ ] No denied crates introduced; lockfile/supply-chain policy respected.
- [ ] Wire format/diagnostics changes include contract/tests where required.

## Risk / Rollback
<!-- Failure modes, mitigations, rollback plan. -->

## Evidence
<!-- Paste key command output summaries, logs, screenshots, or device validation notes. -->
