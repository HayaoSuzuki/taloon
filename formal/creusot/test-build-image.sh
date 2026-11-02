#!/usr/bin/env bash
# Synthetic orchestration checks; no Docker daemon or network access required.
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
test_dir=$(mktemp -d "${TMPDIR:-/tmp}/taloon-creusot-test.XXXXXX")
trap 'rm -rf "$test_dir"' EXIT

run_case() (
    # Arrange: a minimal recipe archive, real checksum/tar, and external-tool doubles.
    name=$1 architecture=$2 failures=$3 expected_status=$4 expected_calls=$5 corrupt=$6
    export CASE_DIR="$test_dir/$name" FAILURES="$failures" FETCH_STATUS="${7:-0}"
    mkdir -p "$CASE_DIR/bin" "$CASE_DIR/recipe/formal/creusot" "$CASE_DIR/recipe/tools" "$CASE_DIR/tmp"
    printf 'synthetic Dockerfile\n' > "$CASE_DIR/recipe/formal/creusot/Dockerfile"
    printf 'synthetic installer\n' > "$CASE_DIR/recipe/tools/creusot_environment.py"
    tar -czf "$CASE_DIR/source.tar.gz" -C "$CASE_DIR" recipe
    checksum=$(shasum -a 256 "$CASE_DIR/source.tar.gz")
    checksum=${checksum%% *}
    sed "s/^recipe_archive_sha256=.*/recipe_archive_sha256=$checksum/" \
        "$script_dir/build-image.sh" > "$CASE_DIR/build-image.sh"
    if [[ "$corrupt" == yes ]]; then
        printf 'corruption' >> "$CASE_DIR/source.tar.gz"
    fi
    cat > "$CASE_DIR/bin/gh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$CASE_DIR/fetches"
if [[ "$FETCH_STATUS" != 0 ]]; then exit "$FETCH_STATUS"; fi
cat "$CASE_DIR/source.tar.gz"
EOF
    cat > "$CASE_DIR/bin/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
count=0
if [[ -f "$CASE_DIR/count" ]]; then read -r count < "$CASE_DIR/count"; fi
count=$((count + 1))
printf '%s\n' "$count" > "$CASE_DIR/count"
printf '%s\n' "$@" > "$CASE_DIR/args.$count"
cat > "$CASE_DIR/context.$count"
if ((count <= FAILURES)); then exit 17; fi
EOF
    cat > "$CASE_DIR/bin/sleep" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$CASE_DIR/sleeps"
EOF
    chmod +x "$CASE_DIR/bin/gh" "$CASE_DIR/bin/docker" "$CASE_DIR/bin/sleep"

    # Act: execute the real wrapper, changing only the synthetic archive checksum.
    status=0
    PATH="$CASE_DIR/bin:$PATH" TMPDIR="$CASE_DIR/tmp" \
        bash "$CASE_DIR/build-image.sh" "$architecture" > "$CASE_DIR/output" 2>&1 || status=$?

    # Assert: final status, bounded attempts, stable inputs, and cleanup.
    [[ "$status" == "$expected_status" ]] || {
        printf '%s: expected status %s, got %s\n' "$name" "$expected_status" "$status" >&2
        cat "$CASE_DIR/output" >&2
        exit 1
    }
    calls=0
    if [[ -f "$CASE_DIR/count" ]]; then read -r calls < "$CASE_DIR/count"; fi
    [[ "$calls" == "$expected_calls" ]] || exit 1
    [[ $(wc -l < "$CASE_DIR/fetches") -eq 1 ]] || exit 1
    [[ $(cat "$CASE_DIR/fetches") == 'api repos/HayaoSuzuki/hoimin/tarball/7d473e08ec5ebb871e5df541369f247c20ea6c5a' ]] || exit 1
    [[ -z $(ls -A "$CASE_DIR/tmp") ]] || exit 1
    if ((calls > 0)); then
        printf '%s\n' build --platform "linux/$architecture" > "$CASE_DIR/expected-args"
        if [[ "$architecture" == arm64 ]]; then
            printf '%s\n' --build-arg \
                CREUSOT_BASE=ubuntu@sha256:18b33ff3829707f31ada654a963b9ee35f2e6f0241b964bd17a22a9b330cb63d \
                --build-arg TOOLCHAIN_MANIFEST=toolchain-arm64.json >> "$CASE_DIR/expected-args"
        fi
        printf '%s\n' -f formal/creusot/Dockerfile -t taloon-creusot:local - >> "$CASE_DIR/expected-args"
        cmp "$CASE_DIR/expected-args" "$CASE_DIR/args.1" || exit 1
        tar -tf "$CASE_DIR/context.1" > "$CASE_DIR/members"
        rg -q '^formal/creusot/Dockerfile$' "$CASE_DIR/members" || exit 1
        rg -q '^tools/creusot_environment.py$' "$CASE_DIR/members" || exit 1
    fi
    if ((calls == 2)); then
        cmp "$CASE_DIR/context.1" "$CASE_DIR/context.2" || exit 1
        cmp "$CASE_DIR/args.1" "$CASE_DIR/args.2" || exit 1
        [[ $(cat "$CASE_DIR/sleeps") == 10 ]] || exit 1
    else
        [[ ! -f "$CASE_DIR/sleeps" ]] || exit 1
    fi
    printf 'PASS %s\n' "$name"
)

run_case amd64-first-success amd64 0 0 1 no
run_case amd64-retry-success amd64 1 0 2 no
run_case arm64-retry-success arm64 1 0 2 no
run_case persistent-failure amd64 9 17 2 no
run_case checksum-failure amd64 0 1 0 yes
run_case fetch-failure amd64 0 22 0 no 22
run_case invalid-architecture invalid 0 2 0 no
