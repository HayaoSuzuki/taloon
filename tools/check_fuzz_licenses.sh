#!/usr/bin/env bash
# Extend the shipping policy only for the two development-only fuzz crates.
set -euo pipefail
root=$(CDPATH='' cd -- "$(dirname "$0")/.." && pwd -P)
policy=$(mktemp "${TMPDIR:-/tmp}/taloon-fuzz-deny.XXXXXX")
trap 'rm -f "$policy"' EXIT
cat "$root/deny.toml" > "$policy"
cat >> "$policy" <<'TOML'

[[licenses.exceptions]]
name = "taloon-fuzz"
version = "=0.0.0"
allow = ["Elastic-2.0"]

# libFuzzer's bundled upstream code declares NCSA in addition to MIT/Apache.
[[licenses.exceptions]]
name = "libfuzzer-sys"
version = "=0.4.13"
allow = ["NCSA"]
TOML
"${CARGO_DENY:-cargo-deny}" --manifest-path "$root/fuzz/Cargo.toml" \
    --workspace --all-features --locked --offline --config "$policy" \
    check --deny warnings licenses sources
