#!/bin/sh
# Builds the release APK for the android-vX.Y.Z tag on the current commit,
# signed with the release key, as release/FleetRanger-X.Y.Z.apk with its
# output-metadata.json (the version, for the server). See README.md.
#
#   FLEETRANGER_KEYSTORE=~/fleetranger-release/fleetranger-release.p12 \
#   FLEETRANGER_KEYSTORE_PASSWORD="$(cat ~/fleetranger-release/password.txt)" \
#   ./release.sh
set -eu
cd "$(dirname "$0")"
if [ -n "$(git status --porcelain)" ]; then
    echo "Commit or stash the changes first: a release is exactly its tag." >&2
    exit 1
fi
tag=$(git tag --points-at HEAD --list 'android-v*' | head -n 1)
if [ -z "$tag" ]; then
    echo "This commit has no android-v… tag: tag the release first, e.g. git tag android-v0.1.0" >&2
    exit 1
fi
: "${FLEETRANGER_KEYSTORE:?set FLEETRANGER_KEYSTORE to the release keystore}"
: "${FLEETRANGER_KEYSTORE_PASSWORD:?set FLEETRANGER_KEYSTORE_PASSWORD}"
export FLEETRANGER_KEYSTORE FLEETRANGER_KEYSTORE_PASSWORD

version=${tag#android-v}
./build.sh -PreleaseVersion="$version" clean assembleRelease
mkdir -p release
cp app/build/outputs/apk/release/app-release.apk "release/FleetRanger-$version.apk"
cp app/build/outputs/apk/release/output-metadata.json "release/FleetRanger-$version.json"
echo "release/FleetRanger-$version.apk"
