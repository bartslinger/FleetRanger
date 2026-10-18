#!/bin/sh
# Runs Gradle in the pinned build image, e.g. `./build.sh assembleDebug`.
# The image is amd64: on an arm64 machine Docker runs it under QEMU, which
# needs binfmt support (on Ubuntu: apt install qemu-user-binfmt). Slow, but
# the same everywhere. Gradle's caches live in a Docker volume, and so does the
# home directory, which keeps the debug signing key: a new key on every build
# would stop debug builds from updating the installed app.
set -eu
cd "$(dirname "$0")"
image=fleetranger-android-build
docker build -q --platform linux/amd64 -t "$image" . > /dev/null
exec docker run --rm --platform linux/amd64 \
    --user "$(id -u):$(id -g)" -e HOME=/gradle/home \
    -v "$PWD":/repo/android \
    -v fleetranger-android-gradle:/gradle \
    "$image" ./gradlew --no-daemon "$@"
