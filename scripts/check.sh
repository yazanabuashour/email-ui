#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# MSRV resolution requires Python 3.11+ (stdlib tomllib).
if [[ "${1:-}" == --print-msrv ]]; then
  python3 - <<'PY'
import re
import tomllib

with open("Cargo.toml", "rb") as manifest:
    data = tomllib.load(manifest)
package = data.get("package", data.get("workspace", {}).get("package", {}))
version = package.get("rust-version")
if version == {"workspace": True}:
    version = data.get("workspace", {}).get("package", {}).get("rust-version")
if not isinstance(version, str) or not re.fullmatch(r"1\.(0|[1-9][0-9]*)(\.(0|[1-9][0-9]*))?", version):
    raise SystemExit("Cargo.toml must declare a release rust-version in package or workspace.package")
print(version if version.count(".") == 2 else version + ".0")
PY
  exit
fi

case "${1:-}" in
  msrv)
    msrv="$("$repo_root/scripts/check.sh" --print-msrv)"
    rustup run "$msrv" rustc --version
    exec cargo "+$msrv" check --locked --offline --all-targets --all-features
    ;;
  '') ;;
  *) printf 'usage: %s [msrv|--print-msrv]\n' "$0" >&2; exit 2 ;;
esac

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
