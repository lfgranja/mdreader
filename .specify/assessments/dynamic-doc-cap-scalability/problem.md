# Problem: dynamic-doc-cap-scalability

- **Slug**: `dynamic-doc-cap-scalability`
- **Date**: 2026-11-11

## 1. Problem Statement
The current 93,636-byte (~91.4 KiB) limit causes premature `FormatUndeliverable` refusals when users attempt to listen to substantial documents (e.g., long-form technical articles, thesis drafts, compiled Obsidian vault exports, or full-length public domain literature).

## 2. Impacted Users & Workflows
- **Power PKMS Users**: Users exporting large notes or compiling monthly notes in Obsidian/Logseq (> 100 KB) are abruptly blocked.
- **Audiobook / Literature Readers**: Classic books in markdown format exceed 91 KB within just 2 or 3 chapters if unified into a single document.
- **Developer Documentation**: Large READMEs, specifications (like SpecKit artifacts, ADR compendiums, or RFCs) frequently range from 150 KB to 500 KB.

## 3. Goals & Non-Goals
- **Goals**:
  - Raise the document and chapter byte budget to 1,048,576 bytes (1 MiB / 1,024 KiB).
  - Retain strict fail-closed boundary checks for files exceeding 1 MiB.
  - Maintain identical behavior between Kotlin shell and Rust playback core.
- **Non-Goals**:
  - Infinite or unbuffered streaming of gigabyte-sized files (which would introduce unbounded OOM risk).
  - Changing the closed `InterruptionReason::FormatUndeliverable` set.
