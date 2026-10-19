#!/bin/sh
# Builds the backend and web images for x86_64 from the release tagged on the
# current commit (vX.Y.Z) and loads them on a server over SSH, without a
# container registry.
#
# Usage: deploy/ship-image.sh <ssh-destination>
# Prints the release, which is the images' tag and FLEETRANGER_VERSION.
set -eu

if [ $# -ne 1 ]; then
    echo "usage: $0 <ssh-destination>" >&2
    exit 2
fi
destination=$1

repo=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
cd "$repo"

# A release is exactly its tag: no uncommitted changes, and a tag to name it.
if [ -n "$(git status --porcelain)" ]; then
    echo "refusing to build: the working tree has uncommitted changes" >&2
    exit 1
fi
tag=$(git tag --points-at HEAD --list 'v[0-9]*' | head -n 1)
if [ -z "$tag" ]; then
    echo "this commit has no vX.Y.Z tag: tag the release first, e.g. git tag v0.1.0" >&2
    exit 1
fi

for part in backend web; do
    image="fleetranger-$part:$tag"
    docker buildx build --platform linux/amd64 --tag "$image" --load "$part" >&2
    docker save "$image" | gzip | ssh "$destination" 'gunzip | docker load' >&2
done

echo "$tag"
