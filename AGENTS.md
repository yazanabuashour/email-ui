# Work on email-ui

This package owns offline email presentation only: the `email_ui` Rust library
and `email-ui render` CLI. Keep transport, credentials, configuration, persistence,
and caller policy outside it. Do not add raw HTML or CSS input.

## Preserve the contract

Read [docs/render.md](docs/render.md) before changing the document API or output.
Keep HTML bytes, inline styles, escaping, plain-text rules, URL validation, and
unknown-field rejection stable unless the owner approves a contract change.
SiftWire's frozen HTML fixture is an independent regression oracle, not output
to regenerate merely to make a test pass. Existing persisted delivery requests
must never be rerendered during a renderer upgrade.

Errors must not echo document values or unknown field names. The CLI's inherited
1 MiB tripwire is measured in [the receipt](receipts/render-input-size.json).
Remeasure and update that receipt before changing its boundary; do not add
unmeasured field or row limits. The Rust library has no CLI byte ceiling.

## Validate and hand off

Use the exact toolchain and strict manifest lints. Run `./scripts/check.sh`
before handing off changes. It covers only this package: no frontend, database,
real sends, private state, or installed application changes. Report blocked
gates. Follow the global checkpoint-review procedure when review is approved;
do not substitute local smoke tests for review.

Preserve the starting state and unrelated changes. Report paths, gates, review
status, commit status, and remaining risks. Do not commit, publish, push, create
remotes, or install the executable without explicit owner authorization.
Installation belongs to the application and is documented in README.md; do not
add a dotfiles installer. Public distribution requires separate owner approval.
