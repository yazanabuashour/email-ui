#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

pinned_version="$(python3 -c 'import tomllib; print(tomllib.load(open("rust-toolchain.toml", "rb"))["toolchain"]["channel"])')"
actual_version="$(rustc --version | awk '{ print $2 }')"
if [[ "$actual_version" != "$pinned_version" ]]; then
  printf 'rustc %s does not match pinned Rust %s\n' "$actual_version" "$pinned_version" >&2
  exit 1
fi
bash -n scripts/check.sh
shellcheck scripts/check.sh
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --locked --doc
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps
cargo build --locked --release
target/release/email-ui --version
python3 scripts/check-offline.py
