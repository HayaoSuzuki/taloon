#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
test_dir=$(mktemp -d "${TMPDIR:-/tmp}/taloon-publish-test.XXXXXX")
trap 'rm -rf "$test_dir"' EXIT
run_case() (
    # Arrange
    name=$1 scenario=$2 expected=$3
    export CASE_DIR="$test_dir/$name" SCENARIO="$scenario"
    export GITHUB_REPOSITORY=HayaoSuzuki/taloon GITHUB_REF=refs/heads/main GITHUB_SHA=1111111111111111111111111111111111111111
    mkdir -p "$CASE_DIR/bin"
    cp "$script_dir/publish-image.sh" "$CASE_DIR/publish-image.sh"
    printf 'synthetic builder\n' > "$CASE_DIR/build-image.sh"
    builder_hash=$(shasum -a 256 "$CASE_DIR/build-image.sh"); export BUILDER_HASH=${builder_hash%% *}
    if [[ "$scenario" == branch ]]; then GITHUB_REF=refs/heads/feature; fi
    if [[ "$scenario" == repository ]]; then GITHUB_REPOSITORY=other/project; fi
    if [[ "$scenario" == source ]]; then GITHUB_SHA=main; fi
    cat > "$CASE_DIR/prepare-image.sh" <<'BUILD'
#!/usr/bin/env bash
printf 'build\n' >> "$CASE_DIR/events"
if [[ "$SCENARIO" == build ]]; then exit 17; fi
BUILD
    cat > "$CASE_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$CASE_DIR/events"
case "$1" in
    image)
        if [[ "$4" == *builder-sha256* ]]; then
            if [[ "$SCENARIO" == label ]]; then echo wrong; else echo "$BUILDER_HASH"; fi
        elif [[ "$SCENARIO" != missing_digest ]]; then
            printf 'ghcr.io/hayaosuzuki/taloon-creusot@sha256:%064d\n' 1
        fi ;;
    tag) ;;
    push) if [[ "$SCENARIO" == push ]]; then exit 18; fi ;;
    *) exit 99 ;;
esac
DOCKER
    chmod +x "$CASE_DIR/prepare-image.sh" "$CASE_DIR/bin/docker"
    # Act
    status=0
    PATH="$CASE_DIR/bin:$PATH" bash "$CASE_DIR/publish-image.sh" "$CASE_DIR/image.pin" > "$CASE_DIR/output" 2>&1 || status=$?
    # Assert
    [[ "$status" == "$expected" ]] || { cat "$CASE_DIR/output" >&2; echo "$name: got $status" >&2; exit 1; }
    if [[ "$expected" == 0 ]]; then
        printf 'ghcr.io/hayaosuzuki/taloon-creusot@sha256:%064d %s %s\n' 1 "$GITHUB_SHA" "$BUILDER_HASH" > "$CASE_DIR/expected.pin"
        cmp "$CASE_DIR/expected.pin" "$CASE_DIR/image.pin" || exit 1
        grep -qx "push ghcr.io/hayaosuzuki/taloon-creusot:source-$GITHUB_SHA" "$CASE_DIR/events" || exit 1
    else
        [[ ! -e "$CASE_DIR/image.pin" ]] || exit 1
        if [[ "$expected" == 2 ]]; then [[ ! -f "$CASE_DIR/events" ]] || exit 1; fi
    fi
    printf 'PASS %s\n' "$name"
)
run_case source_receipt good 0
run_case untrusted_branch branch 2
run_case foreign_repository repository 2
run_case mutable_source source 2
run_case failed_build build 17
run_case stale_builder label 1
run_case failed_push push 18
run_case absent_digest missing_digest 1
