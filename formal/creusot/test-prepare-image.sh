#!/usr/bin/env bash
# Real preparation control flow with local Docker/build doubles; no network.
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
test_dir=$(mktemp -d "${TMPDIR:-/tmp}/taloon-image-cache-test.XXXXXX")
test_dir=$(cd "$test_dir" && pwd)
trap 'rm -rf "$test_dir"' EXIT

run_case() (
    # Arrange
    name=$1 hit=$2 build_status=$3 load_status=$4 preflight_status=$5 architecture=$6 expected=$7
    export CASE_DIR="$test_dir/$name" BUILD_STATUS="$build_status" LOAD_STATUS="$load_status" PREFLIGHT_STATUS="$preflight_status" IMAGE_ARCH="$architecture" INSPECT_STATUS="${8:-0}" SAVE_STATUS="${9:-0}"
    mkdir -p "$CASE_DIR/bin"
    cp "$script_dir/prepare-image.sh" "$CASE_DIR/prepare-image.sh"
    printf 'cached image bytes\n' > "$CASE_DIR/image.tar"
    cat > "$CASE_DIR/build-image.sh" <<'BUILD'
#!/usr/bin/env bash
printf 'build\n' >> "$CASE_DIR/events"
exit "$BUILD_STATUS"
BUILD
    cat > "$CASE_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$CASE_DIR/events"
case "$1" in
    load) test "$2" = --input; cmp "$3" "$CASE_DIR/original.tar"; exit "$LOAD_STATUS" ;;
    image) if [[ "$INSPECT_STATUS" != 0 ]]; then exit "$INSPECT_STATUS"; fi; printf '%s\n' "$IMAGE_ARCH" ;;
    run) exit "$PREFLIGHT_STATUS" ;;
    save) test "$2" = --output; if [[ "$SAVE_STATUS" != 0 ]]; then exit "$SAVE_STATUS"; fi; printf 'new verified image\n' > "$3" ;;
    *) exit 99 ;;
esac
DOCKER
    cp "$CASE_DIR/image.tar" "$CASE_DIR/original.tar"
    chmod +x "$CASE_DIR/build-image.sh" "$CASE_DIR/bin/docker"
    # Act
    status=0
    PATH="$CASE_DIR/bin:$PATH" bash "$CASE_DIR/prepare-image.sh" "$CASE_DIR/image.tar" "$hit" > "$CASE_DIR/output" 2>&1 || status=$?
    # Assert
    [[ "$status" == "$expected" ]] || { cat "$CASE_DIR/output" >&2; echo "$name: unexpected status $status" >&2; exit 1; }
    if [[ "$hit" == true ]]; then
        ! grep -q '^build$' "$CASE_DIR/events" || exit 1
        ! grep -q '^save ' "$CASE_DIR/events" || exit 1
        cmp "$CASE_DIR/image.tar" "$CASE_DIR/original.tar" || exit 1
    elif [[ "$hit" == false ]]; then
        grep -q '^build$' "$CASE_DIR/events" || exit 1
        ! grep -q '^load ' "$CASE_DIR/events" || exit 1
    fi
    if [[ "$expected" == 0 ]]; then
        grep -qx 'run --rm --platform linux/amd64 taloon-creusot:local' "$CASE_DIR/events" || exit 1
        if [[ "$hit" == false ]]; then
            grep -q '^save ' "$CASE_DIR/events" || exit 1
            grep -qx 'new verified image' "$CASE_DIR/image.tar" || exit 1
        fi
    else
        if [[ -f "$CASE_DIR/events" && "$SAVE_STATUS" == 0 ]]; then ! grep -q '^save ' "$CASE_DIR/events" || exit 1; fi
    fi
    printf 'PASS %s\n' "$name"
)
run_case cache_hit true 0 0 0 amd64 0
run_case cold_build false 0 0 0 amd64 0
run_case build_failure false 17 0 0 amd64 17
run_case corrupt_archive true 0 18 0 amd64 18
run_case failed_warm_preflight true 0 0 19 amd64 19
run_case failed_cold_preflight false 0 0 19 amd64 19
run_case wrong_architecture true 0 0 0 arm64 1
run_case invalid_mode invalid 0 0 0 amd64 2
run_case inspect_failure true 0 0 0 amd64 1 18
run_case save_failure false 0 0 0 amd64 20 0 20

registry_case() (
    # Arrange: pin resolution is external here; the real resolver has separate tests.
    name=$1 resolver_status=$2 expected=$3
    export REGISTRY_CASE_DIR="$test_dir/$name" RESOLVER_STATUS="$resolver_status" REGISTRY_MODE="${4:-registry}"
    mkdir -p "$REGISTRY_CASE_DIR/bin"
    cp "$script_dir/prepare-image.sh" "$REGISTRY_CASE_DIR/prepare-image.sh"
    cat > "$REGISTRY_CASE_DIR/registry-image.sh" <<'RESOLVER'
#!/usr/bin/env bash
printf 'resolve %s\n' "$*" >> "$REGISTRY_CASE_DIR/events"
if [[ ${3:-} == --mode ]]; then echo "$REGISTRY_MODE"; exit 0; fi
exit "$RESOLVER_STATUS"
RESOLVER
    cat > "$REGISTRY_CASE_DIR/build-image.sh" <<'BUILDER'
#!/usr/bin/env bash
printf 'build\n' >> "$REGISTRY_CASE_DIR/events"
BUILDER
    cat > "$REGISTRY_CASE_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
printf 'docker %s\n' "$*" >> "$REGISTRY_CASE_DIR/events"
if [[ "$1" == image ]]; then echo amd64; fi
DOCKER
    chmod +x "$REGISTRY_CASE_DIR/build-image.sh" "$REGISTRY_CASE_DIR/bin/docker"
    # Act
    status=0
    PATH="$REGISTRY_CASE_DIR/bin:$PATH" bash "$REGISTRY_CASE_DIR/prepare-image.sh" "$REGISTRY_CASE_DIR/cache.tar" false "$REGISTRY_CASE_DIR/image.pin" > "$REGISTRY_CASE_DIR/output" 2>&1 || status=$?
    # Assert
    [[ "$status" == "$expected" ]] || { cat "$REGISTRY_CASE_DIR/output" >&2; echo "$name: got $status" >&2; exit 1; }
    grep -qx "resolve $REGISTRY_CASE_DIR/image.pin $REGISTRY_CASE_DIR/build-image.sh --mode" "$REGISTRY_CASE_DIR/events" || exit 1
    if [[ "$resolver_status" == 0 ]]; then
        ! grep -Eq '^(build|docker load|docker save)' "$REGISTRY_CASE_DIR/events" || exit 1
        grep -qx 'docker run --rm --platform linux/amd64 taloon-creusot:local' "$REGISTRY_CASE_DIR/events" || exit 1
    elif [[ "$REGISTRY_MODE" == bootstrap ]]; then
        grep -qx build "$REGISTRY_CASE_DIR/events" || exit 1
        grep -q '^docker save ' "$REGISTRY_CASE_DIR/events" || exit 1
    else
        ! grep -Eq '^(build|docker )' "$REGISTRY_CASE_DIR/events" || exit 1
    fi
    printf 'PASS %s\n' "$name"
)
registry_case pinned_registry 0 0
registry_case explicit_bootstrap_pin 3 0 bootstrap
registry_case invalid_registry_pin 2 2
registry_case rejected_registry_attestation 17 17
registry_case registry_status_three_is_failure 3 3
