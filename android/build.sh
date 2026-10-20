#!/bin/sh
# Runs Gradle in the pinned build image, e.g. `./build.sh assembleDebug`.
# The image is built for the machine's own architecture, so Java runs
# natively; on an arm64 machine only aapt2 (x86_64 only) runs under QEMU,
# which needs binfmt support (on Ubuntu: apt install qemu-user-binfmt).
# Gradle's caches live in a Docker volume, and so does
# ANDROID_USER_HOME, which keeps the debug signing key: a new key on every build
# would stop debug builds from updating the installed app.
#
# With FLEETRANGER_KEYSTORE set (see release.sh), the release key is mounted
# read-only and its password passed on.
set -eu
cd "$(dirname "$0")"
image=fleetranger-android-build
docker build -q -t "$image" . > /dev/null
if [ -n "${FLEETRANGER_KEYSTORE:-}" ]; then
    set -- -v "$(realpath "$FLEETRANGER_KEYSTORE")":/keystore.p12:ro \
        -e FLEETRANGER_KEYSTORE=/keystore.p12 -e FLEETRANGER_KEYSTORE_PASSWORD \
        "$image" ./gradlew --no-daemon "$@"
else
    set -- "$image" ./gradlew --no-daemon "$@"
fi
exec docker run --rm \
    --user "$(id -u):$(id -g)" -e HOME=/tmp -e ANDROID_USER_HOME=/gradle/android-user \
    -v "$PWD":/repo/android \
    -v fleetranger-android-gradle:/gradle \
    "$@"
