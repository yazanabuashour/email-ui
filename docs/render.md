# Render an email without sending it

`email-ui render` reads one JSON document from stdin and writes one JSON result
to stdout. It has no configuration, credential, database, or network dependency.
There is no configuration option or environment-variable interface.

```sh
email-ui render < examples/render-request.json
```

Input:

```json
{
  "schema_version": "email-ui/render-request/v1",
  "document": {
    "title": "Example alert",
    "heading": "Example automation",
    "preheader": "A check needs attention.",
    "header": { "edition": "Alert", "date": "" },
    "footer": "Inspect before retrying.",
    "sections": [
      {
        "kind": "summary",
        "heading": "Check failed",
        "lines": ["The fictional service check did not complete."],
        "links": [{ "label": "Runbook", "url": "https://example.test/runbook" }]
      }
    ],
    "notice": null
  }
}
```

Output has exactly `schema_version` (`email-ui/render-result/v1`), `text`, and
`html`. The caller still supplies its own subject to `mailgate send`; the HTML
`title` is not a transport subject. Rendering does not send, persist, or prove
provider acceptance. Keep output out of logs because it contains message bodies.

## Build a presentation document

Every field below is required unless explicitly optional. Textual leaves are
strings, including already-formatted dates and schedule day numbers. Every
object rejects unknown fields. `notice` defaults to `null`; schedule `images`
and summary `links` default to empty arrays. No raw HTML or CSS input exists.

The document fields are `title`, `heading`, `preheader`, `header`, `footer`,
`sections`, and optional `notice`. `header` contains `edition` and `date`.
`notice`, when supplied, contains `label` and `text`.

Sections use the `kind` discriminator:

| Kind | Fields |
| --- | --- |
| `cards` | `heading`, `rows` of `{title, url, eyebrow}` |
| `stories` | `heading`, `rows` of `{title, url, eyebrow}` |
| `schedule` | `heading`, `groups` of `{heading, rows}` |
| `summary` | `heading`, `lines` (strings), optional `links` of `{label, url}` |

Schedule rows contain `weekday`, `day`, `month`, `title`, `url`, `metadata`,
and optional `images` of `{url}`. Images are decorative: the fixed component
renders the first two with empty alternative text, matching the original
SiftWire component. All supplied image URLs are validated, including undisplayed
images. This is a component layout, not an input-count limit.

Callers choose section/group order, omit unwanted sections and notices, and
format all counts, labels, dates, timezones, and metadata. The renderer neither
sorts nor infers domain rules. An explicitly supplied empty section remains
present. An empty summary heading omits its heading element; an empty footer
omits its row. Empty header metadata and preheader produce no invented text.

Nonempty links and all image URLs must be ordinary absolute HTTP(S) URLs with a
host, no credentials, whitespace, control characters, or backslashes. The
renderer never fetches them. Empty links render as spans; empty image URLs fail.
Validated URL strings retain their original bytes before HTML escaping. HTML
text and attributes use the original SiftWire escaping (`&`, `<`, `>`, `"`, `'`).

The CLI retains the 1 MiB stdin tripwire inherited from Mailgate's request
reader for transport compatibility, using its own bounded reader. It reads at
most 1,048,577 bytes to detect overflow; exactly 1,048,576 bytes is accepted.
This preserves the prior ceiling rather than introducing a new content budget.
The synthetic input receipt is
[`receipts/render-input-size.json`](../receipts/render-input-size.json). There
are no additional field-length or row-count limits. Invalid JSON, unknown fields
or variants, wrong types, unsupported versions, and unsafe URLs return nonzero
with `email-ui/error/v1` JSON on stderr and no body on stdout. Diagnostics do not
echo input values or field names.

## Plain-text rules

Text preserves caller order and embedded line breaks. Nonempty document heading,
preheader, edition, and date each get a trailing newline, in that order. Each
section starts with a blank line. Empty ordinary text leaves add no line.

- Cards and stories emit the section heading, then each row's eyebrow and link.
- Schedules emit the section and group headings, then each row's
  `weekday day month` line (including its separating spaces), link, and metadata.
- Summaries emit their heading, lines, and links in order.
- Nonempty URLs emit `label: URL` plus a newline; empty URLs emit only a nonempty
  label. Row titles supply the label for row links.
- A supplied notice emits a blank line followed by `label · text` and a newline.
  A nonempty footer emits a blank line followed by the footer and a newline.

The HTML document title and decorative images add no plain text. No wrapping,
trimming, sorting, or invented labels occur.

## Use the Rust library

The `email-ui` package exposes these task interfaces as `email_ui::render` and
`email_ui::render_json`:

```rust
pub fn render(document: &Document) -> Result<RenderResult, RenderError>;
pub fn render_json(input: &[u8]) -> Result<RenderResult, RenderError>;
```

`Document`, `Header`, `Section`, `Row`, `ScheduleGroup`, `ScheduleRow`, `Image`,
`Notice`, `Link`, `RenderRequest`, and `RenderResult` are public serde-derived
DTOs (data transfer objects). `Section` variants are `Cards`, `Stories`,
`Schedule`, and `Summary`. `RENDER_REQUEST_SCHEMA` and `RENDER_RESULT_SCHEMA`
identify the JSON protocols. Both entry points share URL validation and return
the same rendered bytes. `RenderError` contains only static diagnostic variants.
The library does not impose the CLI's byte tripwire on already-owned documents.

Rust consumers own their dependency and lockfile. During coordinated local work,
a path dependency can point to `../email-ui` (adjust relative to the caller's
manifest). For a deployable cross-repository build, pin the reviewed public
email-ui Git revision and refresh the consumer lockfile after that revision is
available. Do not deploy a developer-specific absolute path. Public distribution
requires explicit owner approval. Transport and delivery state remain separate
in Mailgate. Updating the installed email-ui binary updates Python/shell callers
of `email-ui render`; Rust consumers require their own rebuild.

## Preserve existing send identities

The `mailgate/send-request/v1` contract has not changed: callers provide exact
text and HTML. Persist the chosen bodies before the first send and reuse them
for every retry and reconciliation, rather than rendering them again. A renderer
upgrade must never rewrite a previously persisted request. SiftWire keeps its
existing plain-text generation and maps domain data to shared HTML components.

The shared component golden fixture was frozen from SiftWire's original renderer
before migration. email-ui checks it byte-for-byte and checks library/CLI
parity. SiftWire additionally owns fixtures for domain ordering, morning/evening,
timezone rollover, empty content, escaping, and multiple images.
