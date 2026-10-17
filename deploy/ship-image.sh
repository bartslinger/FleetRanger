#!/bin/sh
# Builds the backend image for x86_64 from the current commit and loads it
# on a server over SSH, without a container registry.
#
# Usage: deploy/ship-image.sh <ssh-destination>
# Prints the image tag, which is the short commit hash.
set -eu

if [ $# -ne 1 ]; then
    echo "usage: $0 <ssh-destination>" >&2
    exit 2
fi
destination=$1

repo=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
cd "$repo"

# The tag names a commit, so the image must contain exactly that commit.
if [ -n "$(git status --porcelain)" ]; then
    echo "refusing to build: the working tree has uncommitted changes" >&2
    exit 1
fi
tag=$(git rev-parse --short=12 HEAD)
image="fleetranger-backend:$tag"

docker buildx build --platform linux/amd64 --tag "$image" --load backend >&2
docker save "$image" | gzip | ssh "$destination" 'gunzip | docker load' >&2

echo "$tag"
