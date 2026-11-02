#!/usr/bin/env bash
# Called only by the trusted-main publisher after registry authentication.
set -euo pipefail
if [[ $# != 1 || -z "$1" || ${GITHUB_REPOSITORY:-} != HayaoSuzuki/taloon || ${GITHUB_REF:-} != refs/heads/main || ! ${GITHUB_SHA:-} =~ ^[0-9a-f]{40}$ ]]; then
    echo 'Image publication requires this repository on main at an exact source commit.' >&2
    exit 2
fi
script_dir=$(cd "$(dirname "$0")" && pwd)
output=$1
build_dir=$(mktemp -d "${TMPDIR:-/tmp}/taloon-image-publication.XXXXXX")
trap 'rm -rf "$build_dir"' EXIT
# Force a fresh pinned build, independent of the currently adopted image pin.
bash "$script_dir/prepare-image.sh" "$build_dir/image.tar" false
builder_hash=$(shasum -a 256 "$script_dir/build-image.sh"); builder_hash=${builder_hash%% *}
observed_builder=$(docker image inspect --format '{{index .Config.Labels "io.taloon.creusot.builder-sha256"}}' taloon-creusot:local)
[[ "$builder_hash" == "$observed_builder" ]] || { echo 'Built image has the wrong builder identity.' >&2; exit 1; }
image_tag="ghcr.io/hayaosuzuki/taloon-creusot:source-$GITHUB_SHA"
docker tag taloon-creusot:local "$image_tag"
docker push "$image_tag"
digests=$(docker image inspect --format '{{range .RepoDigests}}{{println .}}{{end}}' "$image_tag")
image=''
while IFS= read -r candidate; do
    if [[ "$candidate" =~ ^ghcr\.io/hayaosuzuki/taloon-creusot@sha256:[0-9a-f]{64}$ ]]; then
        [[ -z "$image" || "$image" == "$candidate" ]] || { echo 'Ambiguous pushed image digest.' >&2; exit 1; }
        image=$candidate
    fi
done <<< "$digests"
[[ -n "$image" ]] || { echo 'No digest for the pushed image.' >&2; exit 1; }
printf '%s %s %s\n' "$image" "$GITHUB_SHA" "$builder_hash" > "$output"
