# Idea Research: Reading an Entire Directory

- **Slug**: directory-reading
- **Created**: 2026-11-03
- **Evidence confidence (overall)**: high

## Users & Demand

- **Primary user & maintainer signal**: Direct request to ingest both curated collections with clear numbering (e.g. `00-`..`04-` + `README.md`) and uncurated, hierarchical document sets with drafts and subdirectories [source: `.specify/assessments/directory-reading/intake.md`] (confidence: high).
- **Knowledge Worker & Obsidian Community Demand**: Heavy observed demand across personal knowledge management (PKM) users for listening to Markdown vaults/folders like podcasts or audiobooks without manually opening each note [source: Obsidian Community Plugins "Voice" and "Local TTS Reader", `obsidian.md`] (confidence: high).
- **Stated Want vs. Observed Behavior**: While users state they want "read my whole folder", observed reader app behavior shows users get frustrated when obsolete drafts, temporary scratchpads, or ancillary notes are read sequentially without filtering [source: `.specify/assessments/directory-reading/intake.md`, gap on `opencode-configuracao-ideal`] (confidence: high).

## Prior Art

- **Obsidian "Voice" & "Local TTS Reader" Plugins**:
  - Treat notes within a vault/folder as chapters in an audiobook/playlist sidebar pane.
  - Require explicit note sequencing or manual playlist queueing to avoid traversing non-content or template markdown files [source: `obsidian.md` community directory].
- **Evie (eVoice Book Reader for Android)**:
  - Supports folder selection via Android Storage Access Framework (SAF) `ACTION_OPEN_DOCUMENT_TREE`.
  - Scans directories into a temporary "Reading List" rather than maintaining an ad-hoc continuous stream, requiring user confirmation of the scanned list to avoid loading hundreds of files at once [source: `evoicereader.com` documentation].
- **Voice Dream Reader (iOS/Android)**:
  - Explicitly does *not* auto-advance through loose documents inside a standard folder; playback halts when a file ends unless files are pre-merged into a multi-part container (DAISY/Zipped MP3) or manually queued in a reading list [source: `apple.com` app documentation & user forums].
- **Ressoa Core (Features 001–009)**:
  - Single-document model bounded by `MAX_DOC_BYTES = 93_636` bytes.
  - Rust is the sole playback authority; Content-addressed bookmarking via Blake3-256 fingerprint; Strict offline guarantee with no `INTERNET` permission [source: `.specify/memory/constitution.md`].

## Market & Context

- **Alternatives users rely on today**:
  - Manual merging: Users manually combine multiple Markdown files into one large document before listening.
  - Desktop-only scripts: Running Python/pandoc concatenation scripts to convert vaults into EPUBs/audio files.
  - Individual note clicking: Opening and playing notes one by one.
- **Cost of doing nothing**:
  - High friction for multi-chapter books, series of technical notes, or documentation guides stored as separate `.md` files. Ressoa remains strictly a single-file reader, forfeiting the high-growth PKM/Obsidian mobile audio use case.

## Data & Constraints

- **Android Storage Access Framework (`ACTION_OPEN_DOCUMENT_TREE`) IPC Overhead**:
  - Google official Android documentation explicitly warns that iterating through large directories via `ACTION_OPEN_DOCUMENT_TREE` causes severe performance degradation due to `ContentResolver` IPC cursor roundtrips.
  - Traversal must occur strictly on a background I/O coroutine/worker, never on the Main UI thread [source: `developer.android.com/training/data-storage/shared/documents-files`].
- **Persistable URI Permissions Limit**:
  - Android enforces a hard ceiling of 512 persistable URI grants per app (`takePersistableUriPermission`). Taking tree grants without releasing previous ones can lead to silent `SecurityException` [source: Android Platform API Guidelines].
- **Latency Budget (FR-027 / TTFA ≤ 3s)**:
  - Full recursive enumeration and Blake3 hashing of a 60,000-word multi-level corpus on slow flash storage can exceed the 3.0s time-to-audible-audio ceiling if done eagerly before playback starts.
  - Mitigation: Stream/chunk processing where chapter 1 begins synthesis while background worker indexes subsequent files [ASSUMPTION based on Ressoa synthesis pipeline].
- **Size Budget**:
  - Single files are bounded at 93,636 bytes in Ressoa memory model. A directory corpus (e.g. 60,515 words ≈ 350 KB) exceeds the single-document memory limit, requiring a synthetic playlist/book model rather than in-memory concatenation.

## Evidence Against the Idea

- **Draft & Garbage Pollution**: In uncurated directories (such as `documentos/`), alphabetical or filesystem ordering reads obsolete drafts, scratchpads, and superseded revisions back-to-back, severely hurting listening comprehension [source: `intake.md` real corpus measurements].
- **Complex Navigation & Driving Safety (Principle VIII)**: On lock-screen and media notifications, standard headset/car controls only expose Next/Previous track. Ambiguity arises between "skip 30s", "skip heading", or "skip entire file" without looking at the screen.
- **Android SAF Fragility**: SAF tree permissions can be invalidated if the user moves the folder externally or unmounts external storage, requiring robust error recovery mechanisms.

## Gaps & Open Questions

- [NEEDS CLARIFICATION: Should directory ingestion present a lightweight "Chapter/Track Confirmation" sheet allowing users to deselect drafts and reorder before starting playback, as done by Evie?]
- [NEEDS CLARIFICATION: What is the Blake3 fingerprint scheme for a directory: a composite hash of all file paths + hashes (invalidated on any file change), or a tree root identity where each file keeps its individual bookmark?]
- [NEEDS CLARIFICATION: Precedence of `README.md` index tables versus numeric filename prefixes (`01-`, `02-`) when determining playback order.]
- [NEEDS CLARIFICATION: How to handle deep subfolder hierarchies (flattened depth-first vs. breadth-first vs. folder-by-folder).]

## Sources

- https://developer.android.com/training/data-storage/shared/documents-files (host: developer.android.com, policy: confirmed-by-user)
- https://obsidian.md/plugins (host: obsidian.md, policy: allowlisted)
- https://evoicereader.com (host: evoicereader.com, policy: confirmed-by-user)
- .specify/assessments/directory-reading/intake.md (host: local-filesystem, policy: repo-internal)
- .specify/memory/constitution.md (host: local-filesystem, policy: repo-internal)
