# Idea Intake: Reading an Entire Directory

- **Slug**: directory-reading
- **Created**: 2026-10-01
- **Source**: pasted text + local corpus paths (`/home/luis/development/MDMV/docs/monetization/`, `/home/luis/development/MDMV/documentos/`)
- **Type**: new-capability

## Idea (as captured)

> Pesquise longa, completa, profunda, ampla e detalhada na internet de forma
> atualizada e utilize todas as suas ferramentas, capacidades, agentes, subagentes em
> background e habilidades, e avalie a viabilidade, possibilidade, desejabilidade,
> interesse do desenvolvimento de uma funcionalidade de leitura para diretórios
> inteiros, tanto estruturados - por exemplo
> /home/luis/development/MDMV/docs/monetization/ ou mesmo um conjunto não organizado -
> como /home/luis/development/MDMV/documentos/

No URLs were supplied, so no fetch was performed and the URL Trust Policy was not
engaged. The two paths are local, outside this repository, and were read directly.

## Restated

Allow mdreader to ingest an entire directory of Markdown files as one continuous
reading session, handling both a deliberately ordered, curated collection and an
ad-hoc unorganized pile.

## Origin & Context

- **Raised by**: project maintainer (sole maintainer, sole user)
- **Trigger**: two real corpora named as targets — a curated one and a messy one.
  Measured during intake:

| Corpus | Files | Words | Depth | Ordering convention | Speech @150wpm |
|---|---|---|---|---|---|
| `docs/monetization/` | 7 | 13,407 | flat | numeric prefix `00-`…`04-` plus a `README.md` index table | ~1.5 h |
| `documentos/` | 20 | 60,515 | 3 levels (`incentivos/`, `projetos/macro/`) | none — snake_case plus ISO date suffix | ~6.7 h |

- **Relation to feature 001 — decided at intake**: this is the *sibling "document
  feature"* that `001-android-media-shell` already anticipates, not an amendment to
  it. `spec.md:385-387` (**FR-024**) states that document parsing, segmentation,
  synthesis and voice selection are separate features that the media shell "MUST
  consume through a defined interface without implementing them", and `spec.md:683`
  assumes section titles arrive "as supplied by the document feature". The
  single-document constraint at `spec.md:686` is scoped to feature 001, not to the
  project. This idea therefore *defines* that interface rather than contradicting it.
- **Repository state**: the repository holds no implementation code. 55 tracked files,
  all Spec Kit scaffolding plus `spec.md`. Feature 001 is `Status: Draft` with no
  `plan.md` and no `tasks.md`. The corpus model will define an interface that 001 has
  not yet pinned down.
- **Scope of this assessment round**: intake only. The research the request asked for
  was run in the `research` stage and is recorded in `research.md`.

## First-Glance Unknowns

Carried into `research.md` as numbered gaps. Recorded here as the questions intake
could not answer without evidence.

- [NEEDS CLARIFICATION: Ordering semantics for a directory with no ordering
  convention. `documentos/` contains three superseded drafts of one document
  (`opencode-configuracao-ideal` at 4,711 / 2,949 / 1,562 words, dated 2026-09-22,
  -09-23, -09-25), so naive alphabetical traversal reads a document and then its own
  two abandoned revisions in that order.]
- [NEEDS CLARIFICATION: Android directory access — SAF `ACTION_OPEN_DOCUMENT_TREE`
  persistence versus `MANAGE_EXTERNAL_STORAGE`, and whether a granted tree survives
  reboot and folder relocation.]
- [NEEDS CLARIFICATION: Is a directory one continuous session, or a browsable library
  of documents that the user enters individually?]
- [NEEDS CLARIFICATION: `FR-036` keys resume position by document fingerprint — which
  fingerprint is the unit when the unit is a directory, and what happens to position
  when a file inside it is added, removed or edited?]
- [NEEDS CLARIFICATION: `FR-027`'s 3-second time-to-audible-audio budget when a corpus
  must first be enumerated, ordered and segmented.]
- [NEEDS CLARIFICATION: Is a `README.md` index table authoritative ordering input, or
  decorative documentation?]
- [NEEDS CLARIFICATION: Driving-time controls across a multi-hour corpus
  (Principle VIII) — what "next section" means when a section is a file, and whether
  file-level navigation is reachable from the lock screen.]
- [NEEDS CLARIFICATION: Whether the document feature's interface should expose a
  directory as a distinct first-class type or as a synthetic single document with
  concatenated chapters.]