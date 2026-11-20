# Controller app

Pilots install FleetRanger's app on their DJI remote controller from your server's download page, `/download`. The server offers the app you put in `data/android`.

Each app release is a GitHub release named `android-vX.Y.Z`, with two files: the app, `FleetRanger.apk`, and its version, `FleetRanger.json`. Put both in `data/android`, such as for android-v0.5.1:

```
cd data/android
curl -fLO https://github.com/bartslinger/FleetRanger/releases/download/android-v0.5.1/FleetRanger.apk
curl -fLO https://github.com/bartslinger/FleetRanger/releases/download/android-v0.5.1/FleetRanger.json
```

Your server's address is written into the app as it is downloaded, so pilots only type a pairing code. See [Getting started](../user/01-getting-started.md).

## Updates

For a new app release, replace both files. Controllers notice the new version when the app comes to the front, and the pilot confirms the update.
