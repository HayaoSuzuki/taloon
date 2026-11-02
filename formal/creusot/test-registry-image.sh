#!/usr/bin/env bash
# One independent scenario per invocation; no real registry/network/Docker.
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
test_dir=$(mktemp -d "${TMPDIR:-/tmp}/taloon-registry-test.XXXXXX")
trap 'rm -rf "$test_dir"' EXIT
run_case() (
    # Arrange
    name=$1 mode=$2 failure=$3 expected=$4
    export CASE_DIR="$test_dir/$name" FAIL_AT="$failure"
    mkdir -p "$CASE_DIR/bin"
    printf 'synthetic pinned builder\n' > "$CASE_DIR/builder.sh"
    builder_hash=$(shasum -a 256 "$CASE_DIR/builder.sh"); builder_hash=${builder_hash%% *}
    export BUILDER_HASH="$builder_hash"
    image="ghcr.io/hayaosuzuki/taloon-creusot@sha256:$(printf '%064d' 1)"
    source_commit=1111111111111111111111111111111111111111
    case "$mode" in
        bootstrap) image=bootstrap; source_commit=- ;;
        tag) image=ghcr.io/hayaosuzuki/taloon-creusot:latest ;;
        wrong_repo) image="${image/taloon-creusot/other}" ;;
        wrong_hash) builder_hash=$(printf '%064d' 2) ;;
        bad_source) source_commit=main ;;
    esac
    printf '%s %s %s\n' "$image" "$source_commit" "$builder_hash" > "$CASE_DIR/image.pin"
    if [[ "$mode" == missing ]]; then rm "$CASE_DIR/image.pin"; fi
    if [[ "$mode" == empty ]]; then : > "$CASE_DIR/image.pin"; fi
    if [[ "$mode" == unterminated ]]; then printf '%s %s %s' "$image" "$source_commit" "$builder_hash" > "$CASE_DIR/image.pin"; fi
    if [[ "$mode" == extra_field ]]; then printf '%s %s %s extra\n' "$image" "$source_commit" "$builder_hash" > "$CASE_DIR/image.pin"; fi
    if [[ "$mode" == multiline ]]; then printf 'extra record\n' >> "$CASE_DIR/image.pin"; fi
    cat > "$CASE_DIR/bin/gh" <<'GH'
#!/usr/bin/env bash
printf 'gh %s\n' "$*" >> "$CASE_DIR/events"
if [[ "$FAIL_AT" == provenance ]]; then exit 17; fi
GH
    cat > "$CASE_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail
printf 'docker %s\n' "$*" >> "$CASE_DIR/events"
case "$1" in
    pull) if [[ "$FAIL_AT" == pull ]]; then exit 18; fi ;;
    image) if [[ "$FAIL_AT" == label ]]; then printf 'wrong\n'; else printf '%s\n' "$BUILDER_HASH"; fi ;;
    tag) if [[ "$FAIL_AT" == tag ]]; then exit 19; fi ;;
    *) exit 99 ;;
esac
DOCKER
    chmod +x "$CASE_DIR/bin/gh" "$CASE_DIR/bin/docker"
    # Act
    status=0
    set -- "$CASE_DIR/image.pin" "$CASE_DIR/builder.sh"
    if [[ "$failure" == mode ]]; then set -- "$@" --mode; fi
    PATH="$CASE_DIR/bin:$PATH" bash "$script_dir/registry-image.sh" "$@" > "$CASE_DIR/output" 2>&1 || status=$?
    # Assert
    [[ "$status" == "$expected" ]] || { cat "$CASE_DIR/output" >&2; echo "$name: got $status" >&2; exit 1; }
    if [[ "$failure" == mode ]]; then
        [[ ! -e "$CASE_DIR/events" ]] || exit 1
        if [[ "$mode" == bootstrap ]]; then expected_mode=bootstrap; else expected_mode=registry; fi
        [[ $(cat "$CASE_DIR/output") == "$expected_mode" ]] || exit 1
    elif [[ "$expected" == 0 ]]; then
        grep -qx "gh attestation verify oci://$image --repo HayaoSuzuki/taloon --signer-workflow HayaoSuzuki/taloon/.github/workflows/creusot-image.yml --source-ref refs/heads/main --source-digest $source_commit --signer-digest $source_commit --deny-self-hosted-runners" "$CASE_DIR/events" || exit 1
        grep -qx "docker pull --platform linux/amd64 $image" "$CASE_DIR/events" || exit 1
        grep -qx "docker tag $image taloon-creusot:local" "$CASE_DIR/events" || exit 1
    elif [[ "$expected" == 2 || "$expected" == 3 ]]; then
        [[ ! -f "$CASE_DIR/events" ]] || exit 1
    elif [[ "$failure" == provenance ]]; then
        ! grep -q '^docker ' "$CASE_DIR/events" || exit 1
    else
        ! grep -q '^docker tag ' "$CASE_DIR/events" || [[ "$failure" == tag ]] || exit 1
    fi
    printf 'PASS %s\n' "$name"
)
run_case pinned_image locked none 0
run_case explicit_bootstrap bootstrap none 2
run_case mutable_tag tag none 2
run_case foreign_repository wrong_repo none 2
run_case changed_builder wrong_hash none 2
run_case mutable_source bad_source none 2
run_case extra_lines multiline none 2
run_case rejected_provenance locked provenance 17
run_case failed_pull locked pull 18
run_case wrong_builder_label locked label 1
run_case failed_tag locked tag 19
run_case missing_pin missing none 2
run_case empty_pin empty none 2
run_case unterminated_pin unterminated none 2
run_case extra_field extra_field none 2
run_case registry_probe locked mode 0
run_case bootstrap_probe bootstrap mode 0
