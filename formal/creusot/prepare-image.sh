#!/usr/bin/env bash
# CI-only amd64 image reuse. The image CMD checks the pinned installation.
set -euo pipefail
if [[ ( $# != 2 && $# != 3 ) || -z "$1" || ( "$2" != true && "$2" != false ) ]]; then
    echo 'usage: prepare-image.sh IMAGE_ARCHIVE true|false [IMAGE_PIN]' >&2
    exit 2
fi
script_dir=$(cd "$(dirname "$0")" && pwd)
image_archive=$1 cache_hit=$2
registry=false
if [[ $# == 3 ]]; then
    mode=$(bash "$script_dir/registry-image.sh" "$3" "$script_dir/build-image.sh" --mode)
    case "$mode" in
        registry)
            # Registry errors retain their status; none may select bootstrap.
            bash "$script_dir/registry-image.sh" "$3" "$script_dir/build-image.sh"
            registry=true
            ;;
        bootstrap) ;;
        *) echo 'Unexpected environment pin mode.' >&2; exit 2 ;;
    esac
fi
if [[ "$registry" == true ]]; then
    : # The verified digest was pulled and tagged; do not replace it with a cache.
elif [[ "$cache_hit" == true ]]; then
    docker load --input "$image_archive"
else
    "$script_dir/build-image.sh"
fi
if [[ $(docker image inspect --format '{{.Architecture}}' taloon-creusot:local) != amd64 ]]; then
    echo 'Creusot CI image must target amd64.' >&2
    exit 1
fi
# Validation applies to cache hits too. Never save a failed installation.
docker run --rm --platform linux/amd64 taloon-creusot:local
if [[ "$registry" == false && "$cache_hit" == false ]]; then
    docker save --output "$image_archive" taloon-creusot:local
fi
