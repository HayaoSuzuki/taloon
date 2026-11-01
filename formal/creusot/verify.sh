#!/usr/bin/env bash
# Run inside the pinned tool image, from this checkout, without proof reuse.
set -euo pipefail
root=$(CDPATH='' cd -- "$(dirname "$0")/../.." && pwd -P)
cd "$root"
scope=verif/taloon_decisions_rlib
inventory=formal/creusot/obligations.tsv
reports=formal/creusot/reports
mkdir -p "$reports"
inputs() {
    sha256sum Cargo.toml Cargo.lock why3find.json "$inventory" formal/creusot/verify.sh \
        crates/taloon-decisions/Cargo.toml crates/taloon-decisions/src/*.rs
}
inputs > "$reports/inputs.sha256"
# Targets describe compilation semantics, not execution on the foreign OS.
for target in x86_64-unknown-linux-gnu aarch64-apple-darwin x86_64-pc-windows-msvc; do
    report="$reports/$target"
    mkdir -p "$report"
    python3 /recipe/tools/creusot_environment.py check \
        --manifest /recipe/formal/creusot/toolchain.json --project-root "$root" > "$report/preflight.log"
    cargo clean -p taloon-decisions --target "$target"
    rm -rf "$scope"
    timeout 300s cargo creusot -p taloon-decisions --only=coma -j 2 -- \
        --lib --locked --ignore-rust-version --target "$target" > "$report/generation.log" 2>&1
    # Fresh generation must produce exactly the reviewed, nonempty scope.
    awk '{print $1}' "$inventory" | LC_ALL=C sort > "$report/expected-files.txt"
    find "$scope" -type f -name '*.coma' | sed "s|^$scope/||" | LC_ALL=C sort > "$report/actual-files.txt"
    test -s "$report/actual-files.txt"
    diff -u "$report/expected-files.txt" "$report/actual-files.txt"
    inputs > "$report/inputs-after-generation.sha256"
    cmp "$reports/inputs.sha256" "$report/inputs-after-generation.sha256"
    python3 /recipe/tools/creusot_environment.py check \
        --manifest /recipe/formal/creusot/toolchain.json --project-root "$root" >> "$report/preflight.log"
    while read -r file goals; do
        test -s "$scope/$file"
        log="$report/$(echo "$file" | tr / _).log"
        timeout 300s cargo creusot -p taloon-decisions --only=prove --no-cache -j 2 \
            --why3find-arg=--goals "$scope/$file" > "$log" 2>&1
        for goal in $goals; do
            grep -F "Goal $goal: ✔" "$log" > /dev/null
        done
        grep -F "Proved ($scope/$file) ✔" "$log" > /dev/null
    done < "$inventory"
    inputs > "$report/inputs-after-proof.sha256"
    cmp "$reports/inputs.sha256" "$report/inputs-after-proof.sha256"
    rm -rf "$report/verif"
    cp -R "$scope" "$report/verif"
    printf 'Verified %s: %s files\n' "$target" "$(wc -l < "$report/actual-files.txt")"
done
