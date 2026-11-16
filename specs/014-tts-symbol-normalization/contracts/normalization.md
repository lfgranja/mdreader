# Contract: Normalization Boundary

- Input: `&str` (segment.content), `NormalizationMode`.
- Output: `String` (spoken text).
- Applied in `synth.rs::synthesize` before `generate_with_config`; original segment.content unchanged.
- Fluid transforms:
  - Fenced code blocks delimited by lines starting with ` ``` ` or `~~~` are
    protected verbatim (including fence lines); fence state is tracked by line
    scan and interior lines are not normalized.
  - Thematic-break lines (`---`/`***`/`___`, whitespace-insensitive) become a
    pause (`"."`).
  - Pipe-table delimiter rows become a pause (`"."`). Pipe-table data rows keep
    cells separated by a spoken comma (`", "`): each data cell is normalized with
    the inline pipeline (link/image label reduction, protected zones for inline
    code, bare URLs, inline `$...$` math, and simple asterisk multiplication/
    intraword forms, raw HTML tag stripping, strikethrough/emphasis marker
    stripping, orphan-`*` removal, then protected-zone restoration). Structural
    line transforms (heading/blockquote/list-bullet/thematic-break handling) do
    not apply inside table cells.
  - Non-table, non-fenced lines: reduce links/images to visible label/alt text,
    protect inline verbatim zones, strip raw HTML tags, strip blockquote markers,
    strip heading markers, strip list-bullet markers, strip strikethrough and
    emphasis markers, remove orphan `*`, then restore protected zones.
- Literal: identity.
- Failure: never fail; always returns a String (may be empty → synth returns InitFailed for empty).