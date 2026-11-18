# Quickstart: 014 TTS Symbol Normalization

## Verification
```sh
cargo fmt --all -- --check
cargo nextest run -p adapter-synthesis
cargo clippy -p adapter-synthesis -- -D warnings
```

## Device smoke (optional)
Synthesize a markdown doc with `*`, `**`, `***`, `#`, bullets, code, URLs, math; listen for reduced "asterisco" repetition. Record in converge.md.