#!/usr/bin/env bash
# A pin is data, never shell code. --mode classifies it independently of errors.
set -euo pipefail
invalid() { echo 'Invalid Creusot image pin; review the pin and pinned builder together.' >&2; exit 2; }
[[ $# == 2 || ( $# == 3 && "$3" == --mode ) ]] || invalid
pin=$1 builder=$2
[[ -f "$pin" && -f "$builder" ]] || invalid
{
    IFS= read -r record || invalid
    extra=''
    if IFS= read -r extra || [[ -n "$extra" ]]; then invalid; fi
} < "$pin"
read -r image source_commit pinned_builder_hash extra <<< "$record"
[[ -z "${extra:-}" && "$pinned_builder_hash" =~ ^[0-9a-f]{64}$ ]] || invalid
builder_hash=$(shasum -a 256 "$builder"); builder_hash=${builder_hash%% *}
[[ "$builder_hash" == "$pinned_builder_hash" ]] || invalid
if [[ "$image" == bootstrap ]]; then
    [[ "$source_commit" == - ]] || invalid
    if [[ ${3:-} == --mode ]]; then echo bootstrap; exit 0; fi
    echo 'Bootstrap pin has no registry image.' >&2
    exit 2
fi
[[ "$image" =~ ^ghcr\.io/hayaosuzuki/taloon-creusot@sha256:[0-9a-f]{64}$ && "$source_commit" =~ ^[0-9a-f]{40}$ ]] || invalid
if [[ ${3:-} == --mode ]]; then echo registry; exit 0; fi
# Authenticate to GHCR before calling this helper. Verification failure is fatal;
# no cache/local build may replace the explicitly selected environment.
gh attestation verify "oci://$image" --repo HayaoSuzuki/taloon \
    --signer-workflow HayaoSuzuki/taloon/.github/workflows/creusot-image.yml \
    --source-ref refs/heads/main --source-digest "$source_commit" \
    --signer-digest "$source_commit" --deny-self-hosted-runners
docker pull --platform linux/amd64 "$image"
observed_builder=$(docker image inspect --format '{{index .Config.Labels "io.taloon.creusot.builder-sha256"}}' "$image")
[[ "$observed_builder" == "$builder_hash" ]] || { echo 'Creusot image builder identity differs from the pin.' >&2; exit 1; }
docker tag "$image" taloon-creusot:local
