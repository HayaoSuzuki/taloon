#!/usr/bin/env bash
# Compiling negative witnesses run only on a temporary copy of production code.
set -euo pipefail
root=$(CDPATH='' cd -- "$(dirname "$0")/../.." && pwd -P)
work=$(mktemp -d /tmp/taloon-contract-witness.XXXXXX)
trap 'rm -rf "$work"' EXIT
cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/why3find.json" "$work/"
cp -R "$root/crates" "$root/src" "$work/"
mkdir -p "$root/formal/creusot/reports/sensitivity"
cd "$work"
for mutation in duplicate consumption precedence exit warning; do
    cp "$root/crates/taloon-decisions/src/observation.rs" crates/taloon-decisions/src/observation.rs
    cp "$root/crates/taloon-decisions/src/status.rs" crates/taloon-decisions/src/status.rs
    case "$mutation" in
        duplicate)
            file=crates/taloon-decisions/src/observation.rs
            sed 's/\*self = Self::Duplicate;/let _ = self;/' "$file" > "$file.tmp"
            obligation=observation/impl_Observation_T/mark_duplicate.coma
            goal=vc_mark_duplicate_T
            ;;
        consumption)
            file=crates/taloon-decisions/src/observation.rs
            sed 's/match slot.take() {/match std::mem::replace(slot, Some(Self::Duplicate)) {/' "$file" > "$file.tmp"
            obligation=observation/impl_Observation_T/take.coma
            goal=vc_take_T
            ;;
        precedence)
            file=crates/taloon-decisions/src/status.rs
            sed 's/(Self::Incomplete, _) | (_, Self::Incomplete) => Self::Incomplete,/(Self::Incomplete, _) | (_, Self::Incomplete) => Self::Violation,/' "$file" > "$file.tmp"
            obligation=status/impl_Status/combine.coma
            goal=vc_combine
            ;;
        exit)
            file=crates/taloon-decisions/src/status.rs
            sed 's/Self::Incomplete => 2,/Self::Incomplete => 1,/' "$file" > "$file.tmp"
            obligation=status/impl_Status/exit_code.coma
            goal=vc_exit_code
            ;;
        warning)
            file=crates/taloon-decisions/src/status.rs
            sed 's/Self::Compliant => u8::from(deny_warnings \&\& has_warnings),/Self::Compliant => u8::from(deny_warnings || has_warnings),/' "$file" > "$file.tmp"
            obligation=status/impl_Status/exit_code.coma
            goal=vc_exit_code
            ;;
    esac
    cmp -s "$file" "$file.tmp" && { echo 'mutation did not change source' >&2; exit 1; }
    mv "$file.tmp" "$file"
    report="$root/formal/creusot/reports/sensitivity/$mutation"
    mkdir -p "$report"
    python3 /recipe/tools/creusot_environment.py check \
        --manifest /recipe/formal/creusot/toolchain.json --project-root "$work" > "$report/preflight.log"
    cargo clean -p taloon-decisions
    rm -rf verif/taloon_decisions_rlib
    timeout 300s cargo creusot -p taloon-decisions --only=coma -j 2 -- \
        --lib --locked --ignore-rust-version > "$report/generation.log" 2>&1
    if timeout 300s cargo creusot -p taloon-decisions --only=prove --no-cache -j 2 \
        --why3find-arg=--goals "verif/taloon_decisions_rlib/$obligation" > "$report/proof.log" 2>&1; then
        echo "mutation unexpectedly proved: $mutation" >&2
        exit 1
    else
        result=$?
        # A timeout or crashed process is not evidence of a counterexample.
        test "$result" -eq 1
    fi
    grep -E "Goal $goal:.*✘" "$report/proof.log" > /dev/null
    printf 'Rejected compiling mutation: %s\n' "$mutation"
done
