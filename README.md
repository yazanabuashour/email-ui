# email-ui

Render shared email presentation offline. This standalone Rust library and CLI
own fixed HTML components and plain text, not delivery, configuration,
credentials, application policy, or domain formatting. Mailgate remains a
separate transport; no Mailgate code or history is needed to build this package.

```sh
cargo build --locked --release
./target/release/email-ui render < examples/render-request.json
```

`email-ui render` reads `email-ui/render-request/v1` JSON from stdin and writes
`email-ui/render-result/v1` JSON containing `text` and `html`. It never sends,
fetches URLs, loads configuration, or persists bodies. Keep output out of logs.
See [the document contract](docs/render.md) for fields, validation, plain-text
rules, and the Rust `email_ui::render` API.

## Build and check

Install the pinned toolchain in `rust-toolchain.toml`, Python 3.11+, and ShellCheck.
Then run:

```sh
./scripts/check.sh
```

The gate checks shell syntax, ShellCheck, formatting, strict Clippy lints, tests,
Rust documentation, release build, executable version, offline render parity,
and the input-size receipt. No frontend, database, credentials, or network
service is needed. Cargo may need registry access to obtain dependencies on the
first build; rendering itself is offline. `Cargo.lock` pins the build.

Tests preserve SiftWire's frozen rich-evening HTML byte-for-byte, URL validation,
escaping, plain text, redacted diagnostics, library/CLI parity, and the inherited
1 MiB stdin tripwire. [The receipt](receipts/render-input-size.json) retains
original measurements and records the renamed standalone request sizes. This
transport-compatibility tripwire is not an ordinary content budget; there are
no new row-count or field-length limits. [The extraction parity receipt](receipts/standalone-parity.json)
records actual old/new offline release renders; the check script verifies both
HTML and plain-text hashes without needing Mailgate.

## Install the executable yourself

Installation belongs to this application, not a dotfiles installer. After
review and checks, run these commands from this repository when you explicitly
intend to replace the installed executable:

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
atomic. Put `$HOME/.local/bin` on your `PATH`. Rust consumers must rebuild their
own application; installing this executable does not update linked libraries.

## Consume and distribute

A sibling checkout can use `email-ui = { path = "../email-ui" }` during local
integration. A deployable cross-repository dependency must pin the reviewed
public Git revision and refresh the consumer lockfile. Do not ship a
developer-specific absolute path. Public repository creation, publication, and
pushing require explicit owner approval; none is performed by this project or
its checks. Registry publication is disabled with `publish = false` until an
approved distribution decision changes it.

Callers own subjects and delivery requests. Persist rendered bodies before the
first send and reuse those exact bytes for retries and reconciliation. A
renderer upgrade must not rewrite already-persisted send requests.
