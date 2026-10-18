# Remote controller app

The FleetRanger app for DJI Enterprise remote controllers: Kotlin and Jetpack
Compose, with our own Carbon components (light theme only). See "Clients" in
`docs/architecture.md`.

## Building

`./build.sh assembleDebug` runs Gradle in a pinned Docker image (see
`Dockerfile`) and writes `app/build/outputs/apk/debug/app-debug.apk`. The
image is x86_64: on an arm64 machine, Docker needs QEMU's binfmt support
(`apt install qemu-user-binfmt` on Ubuntu), and builds are slow.

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
