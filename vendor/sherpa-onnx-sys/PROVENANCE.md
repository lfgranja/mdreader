# Provenance — vendored `sherpa-onnx-sys` 1.13.8 (zip-9 build fix)

This directory is a vendored copy of the crates.io release
`sherpa-onnx-sys` 1.13.8, carrying one build-dependency change.

- Origin: crates.io — `sherpa-onnx-sys-1.13.8.crate`
- sha256 of the `.crate` archive:
  `14bbeabb73f73f1c1f4a278af0ee9ae90a19f9727482027047fb9422fa14f8fe`
- License: Apache-2.0 (see `LICENSE`)

## Change from the published crate

Exactly one line, in `Cargo.toml`:

```diff
 [build-dependencies.zip]
-version = "2"
+version = "9"
```

Rationale: three RUSTSEC advisories against `zip` 2.x
(RUSTSEC-2026-0367 / RUSTSEC-2026-0368 / RUSTSEC-2026-0369) are fixed
only in `zip >= 9`. The crate's `build.rs` is source-compatible with
zip 9 — `ZipArchive::new` + `extract` are unchanged and the default
feature set still includes the decompressors (verified: the unmodified
build script compiles against zip 9.0.4).

Upstream trail:

- Issue: https://github.com/k2-fsa/sherpa-onnx/issues/4058
- PR: https://github.com/k2-fsa/sherpa-onnx/pull/4059 (fork
  `lfgranja`, branch `fix/rust-zip9-advisories`, commit `5821f23`)

Packaging-only artifacts removed from the published archive (no source
impact): `.cargo_vcs_info.json`, `Cargo.toml.orig`, `Cargo.lock`.

Build-time note: `zip` 9 requires Rust 1.88, so building this
workspace requires rustc >= 1.88 while this vendored copy is in the
graph (build-script dependency only; nothing from `zip` is linked into
shipped artifacts).

Licence note: `zip` 9's default feature set includes `bzip2`,
resolving to `libbz2-rs-sys` under the `bzip2-1.0.6` licence. That
licence is admitted to the workspace allow-list (`deny.toml`,
constitution v1.8.0) — a permissive, no-copyleft licence with
notice-and-attribution conditions only. The admission survives this
patch's exit: upstream PR #4059 bumps the version only (default
features kept), so the crate returns to the graph with the released
`sherpa-onnx`.

## Exit condition

When a released `sherpa-onnx` ships `zip >= 9` (PR #4059 merged and
published): bump `sherpa-onnx` in `crates/adapter-synthesis/Cargo.toml`
to that release, delete this directory and the `[patch.crates-io]`
stanza in the root `Cargo.toml`, and drop the corresponding
Exceptions Log entry in the constitution.
