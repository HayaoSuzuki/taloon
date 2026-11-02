#!/usr/bin/env bash
# Reuse the reviewed public tool recipe without copying its Python installer.
set -euo pipefail
recipe_revision=7d473e08ec5ebb871e5df541369f247c20ea6c5a
recipe_archive_sha256=ea4d2ade34657026551c36411b64bb4e8cb3591e9a28919e448a8e456d72d1f3
build_dir=$(mktemp -d "${TMPDIR:-/tmp}/taloon-creusot-build.XXXXXX")
trap 'rm -rf "$build_dir"' EXIT

gh api "repos/HayaoSuzuki/hoimin/tarball/$recipe_revision" > "$build_dir/source.tar.gz"
printf '%s  %s\n' "$recipe_archive_sha256" "$build_dir/source.tar.gz" | shasum -a 256 --check --status
mkdir "$build_dir/source"
tar -xzf "$build_dir/source.tar.gz" --strip-components=1 -C "$build_dir/source"
cd "$build_dir/source"
case "${1:-amd64}" in
    amd64)
        build_args=(--platform linux/amd64)
        ;;
    arm64)
        build_args=(--platform linux/arm64
            --build-arg CREUSOT_BASE=ubuntu@sha256:18b33ff3829707f31ada654a963b9ee35f2e6f0241b964bd17a22a9b330cb63d
            --build-arg TOOLCHAIN_MANIFEST=toolchain-arm64.json)
        ;;
    *) echo 'usage: build-image.sh [amd64|arm64]' >&2; exit 2 ;;
esac
# Retry the unchanged build once for transient upstream acquisition failures.
# Keep Docker's layer cache and one exact context; fetch/checksum failures above
# remain fatal. CI's existing outer deadline bounds both attempts together.
tar -cf "$build_dir/context.tar" formal/creusot tools/creusot_environment.py
for attempt in 1 2; do
    if docker build "${build_args[@]}" -f formal/creusot/Dockerfile -t taloon-creusot:local - < "$build_dir/context.tar"; then
        exit 0
    else
        status=$?
    fi
    if [[ "$attempt" == 2 ]]; then
        exit "$status"
    fi
    echo 'Creusot image build failed; retrying the same pinned context once in 10 seconds.' >&2
    sleep 10
done
