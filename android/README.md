# Remote controller app

The FleetRanger app for DJI Enterprise remote controllers: Kotlin and Jetpack
Compose, with our own Carbon components (light theme only). See "Clients" in
`docs/architecture.md`.

## Building

`./build.sh assembleDebug` runs Gradle in a pinned Docker image (see
`Dockerfile`) and writes `app/build/outputs/apk/debug/app-debug.apk`. The
image is x86_64: on an arm64 machine, Docker needs QEMU's binfmt support
(`apt install qemu-user-binfmt` on Ubuntu), and builds are slow.

## Releases

Only tagged commits become releases. A release is tagged `android-vX.Y.Z`;
its version name is `X.Y.Z` and its version code `X × 1,000,000 + Y × 1,000 + Z`,
which Android requires to grow with every update. Other builds are `dev`.

```sh
git tag android-v0.1.0
FLEETRANGER_KEYSTORE=~/fleetranger-release/fleetranger-release.p12 \
FLEETRANGER_KEYSTORE_PASSWORD="$(cat ~/fleetranger-release/password.txt)" \
./release.sh
```

`release.sh` refuses uncommitted changes and untagged commits, builds with R8,
signs with the release key and writes `release/FleetRanger-X.Y.Z.apk` with its
`output-metadata.json` as `release/FleetRanger-X.Y.Z.json`. The install's
administrator puts both on the server: the APK as `FLEETRANGER_ANDROID_APK`
(e.g. `/data/android/FleetRanger.apk`) and the JSON next to it with the same
name (`FleetRanger.json`). Controllers see the new version within a day, or
at once under Settings › App › Check for updates.

### The release key

Releases are signed with one key, which is never in this repository. Every
update must be signed with it: with another key, Android refuses the update,
and controllers would have to uninstall the app (losing its settings) to
install it again. It is kept in Bitwarden as **FleetRanger Android signing
key**:

- the item's **password** is the keystore's password (the same for the store
  and the key);
- its **note** is the keystore file `fleetranger-release.p12` in base64
  (PKCS12, an EC P-256 key with alias `fleetranger`).

To build a release on a new machine, restore it first:

```sh
mkdir -p ~/fleetranger-release && chmod 700 ~/fleetranger-release
base64 -d > ~/fleetranger-release/fleetranger-release.p12   # paste the note, then Ctrl-D
# paste the password; read -s keeps it off the screen and out of the history
read -rs password && printf '%s' "$password" > ~/fleetranger-release/password.txt && unset password
chmod 600 ~/fleetranger-release/*
```

## Installing on a controller

Enable USB debugging on the controller, connect it, and run:

```sh
adb install -r app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n org.fleetranger.rc/.MainActivity
```

To have the controller open FleetRanger when it starts, choose it under
Settings › Apps & notifications › Auto start application selection.

IBM Plex Sans is included under the SIL Open Font License; see
`LICENSE-IBM-Plex.txt`.
