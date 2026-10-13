# email-ui

Render fixed email components offline with the `email_ui` Rust library or
`email-ui render` CLI. This package owns HTML and plain text, not transport,
credentials, configuration, persistence, application policy, or domain formatting.
It never sends email or fetches URLs.

## Build and render

Install the toolchain pinned in `rust-toolchain.toml`, then run:

```sh
cargo build --locked --release
./target/release/email-ui render < examples/render-request.json
```

The CLI reads `email-ui/render-request/v1` JSON from stdin and writes
`email-ui/render-result/v1` JSON containing `text` and `html`. Keep output out of
logs because it contains message bodies. See [the renderer contract](docs/render.md)
for fields, validation, plain-text rules, and library entry points.

## Change and check the renderer

Read [the repository rules](AGENTS.md) and [the renderer contract](docs/render.md)
before changing output or the document API. The source follows the rendering path:

- `src/main.rs` owns CLI input, the byte tripwire, and output diagnostics.
- `src/lib.rs` exposes the library entry points and validates URLs.
- `src/document.rs` defines the document types and JSON decoding rules.
- `src/html.rs` and `src/html/components.rs` render fixed HTML; `src/text.rs`
  renders plain text.

With the pinned toolchain, Python 3.11+, and ShellCheck installed, run:

```sh
./scripts/check.sh
```

The gate checks shell syntax, ShellCheck, formatting, strict Clippy lints, tests,
Rust documentation, release build, executable version, offline render parity,
and the input-size receipt. It touches only this package: no frontend, database,
private state, installed executable, or network service. Cargo may need registry
access to obtain locked dependencies on the first build; rendering is offline.
Generated Rust API documentation is in `target/doc/email_ui/index.html`.

Keep the independent regression oracles intact:

- `tests/fixtures/rich-evening.html` is frozen from SiftWire's original renderer.
  `tests/render.rs` compares it byte-for-byte except for the approved mobile-header
  CSS and classes. Do not regenerate it to make a test pass.
- [The extraction parity receipt](receipts/standalone-parity.json) records old/new
  offline release renders. `scripts/check-offline.py` checks the original HTML
  hash after removing that same header delta, and the unchanged plain-text hash.
- [The input-size receipt](receipts/render-input-size.json) records the inherited
  1 MiB stdin tripwire and synthetic request measurements. Remeasure before
  changing the boundary; it is not an ordinary content budget. The library has
  no CLI byte ceiling, and neither interface adds field-length or row-count limits.
- [The extraction receipt](receipts/extraction.json) retains source and fixture
  hashes from the original migration, not hashes of current source files.

## Integrate a caller

Use `email-ui = { path = "../email-ui" }` for local Rust integration, adjusting the
path relative to the caller's manifest. For a deployable cross-repository build,
pin the reviewed public Git revision and refresh the consumer lockfile. Do not
ship a developer-specific absolute path. Public repository creation, publication,
and pushing require explicit owner approval. Registry publication is disabled
with `publish = false`.

Callers own subjects and delivery requests. The document's `title` is an HTML
title, not a transport subject. Persist rendered bodies before the first send
and reuse those exact bytes for retries and reconciliation. A renderer upgrade
must never rewrite already-persisted send requests. Mailgate remains a separate
transport; building or checking this package needs no Mailgate checkout.

## Install the executable yourself

Installation belongs to this application, not a dotfiles installer. After
review and checks, run these commands from this repository only when you
explicitly intend to replace the installed executable:

```sh
cargo build --locked --release
(
  set -eu
  destination="$HOME/.local/bin"
  mkdir -p "$destination"
  temporary="$(mktemp "$destination/.email-ui.XXXXXXXX")"
  trap 'rm -f -- "$temporary"' EXIT
  install -m 0755 target/release/email-ui "$temporary"
  mv -fT -- "$temporary" "$destination/email-ui"
)
"$HOME/.local/bin/email-ui" --version
```

The temporary file and destination share a directory, so the final rename is
atomic. Put `$HOME/.local/bin` on your `PATH`. Installing the executable updates
callers of `email-ui render`; Rust consumers must rebuild their own application.
