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
for mutation in duplicate consumption precedence exit warning strength association content unpinned sidecar aggregate name elapsed counter; do
    cp "$root"/crates/taloon-decisions/src/*.rs crates/taloon-decisions/src/
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
        strength)
            file=crates/taloon-decisions/src/evidence.rs
            sed 's/(Self::IndexAssociated { .. }, EvidenceRequirement::ArchiveVerified) => false,/(Self::IndexAssociated { .. }, EvidenceRequirement::ArchiveVerified) => true,/' "$file" > "$file.tmp"
            obligation=evidence/impl_ProofBinding/satisfies.coma
            goal=vc_satisfies
            ;;
        association)
            file=crates/taloon-decisions/src/evidence.rs
            sed 's/same(source, expected_source) \&\& same(request, expected_request)/same(source, expected_source) || same(request, expected_request)/' "$file" > "$file.tmp"
            obligation=evidence/impl_ProofBinding/matches.coma
            goal=vc_matches
            ;;
        content)
            file=crates/taloon-decisions/src/evidence.rs
            sed '/pub fn content_matches/,/^}/s/Some(id) => same(id, observed),/Some(_) => true,/' "$file" > "$file.tmp"
            obligation=evidence/content_matches.coma
            goal=vc_content_matches
            ;;
        unpinned)
            file=crates/taloon-decisions/src/evidence.rs
            sed 's/None => same(original, expected),/None => true,/' "$file" > "$file.tmp"
            obligation=evidence/request_matches.coma
            goal=vc_request_matches
            ;;
        sidecar)
            file=crates/taloon-decisions/src/evidence.rs
            sed '/pub fn sidecar_matches/,/^}/s/None => false,/None => true,/' "$file" > "$file.tmp"
            obligation=evidence/sidecar_matches.coma
            goal=vc_sidecar_matches
            ;;
        aggregate)
            file=crates/taloon-decisions/src/status.rs
            sed 's/while index < packages.len() {/while index + 1 < packages.len() {/' "$file" > "$file.tmp"
            obligation=status/aggregate.coma
            goal=vc_aggregate_P
            ;;
        name)
            file=crates/taloon-decisions/src/name.rs
            sed 's/b + 32 }/b + 31 }/' "$file" > "$file.tmp"
            obligation=name/normalize.coma
            goal=vc_normalize
            ;;
        elapsed|counter)
            file=crates/taloon-decisions/src/accounting.rs
            if [[ "$mutation" == elapsed ]]; then
                addition='        self.elapsed_us = self.elapsed_us.saturating_add(other.elapsed_us);'
            else
                addition='        self.network_requests = 0;'
            fi
            sed "/pub fn merge(\&mut self, other: \&Self) {/a\\$addition" "$file" > "$file.tmp"
            obligation=accounting/impl_AcquisitionSummary/merge.coma
            goal="vc_merge'0"
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
