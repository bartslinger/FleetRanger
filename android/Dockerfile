# Builds the remote controller app. Google's Android build tools only run on
# x86_64 Linux, so this image is amd64; on an arm64 machine it runs under QEMU
# (see build.sh). Everything is pinned: the base image by digest, downloads by
# checksum, SDK packages by version.
FROM eclipse-temurin:21-jdk-noble@sha256:107215954359f1ab0be058a247d7c64ed6e69c71c1682d471d38ef751ec8e964

RUN apt-get update \
    && apt-get install -y --no-install-recommends unzip \
    && rm -rf /var/lib/apt/lists/*

ENV ANDROID_HOME=/opt/android-sdk
# Command-line tools 23.0 (checksum: sha1 as Google publishes it, plus sha256).
ARG CMDLINE_TOOLS=commandlinetools-linux-16111833_latest.zip
ARG CMDLINE_TOOLS_SHA256=0877a1d048fe4a24efe2eff536ca4223f7adeb58648bb81909d33c446918cfa8
RUN curl -fsSLo /tmp/tools.zip https://dl.google.com/android/repository/$CMDLINE_TOOLS \
    && echo "$CMDLINE_TOOLS_SHA256  /tmp/tools.zip" | sha256sum -c - \
    && mkdir -p $ANDROID_HOME/cmdline-tools \
    && unzip -q /tmp/tools.zip -d $ANDROID_HOME/cmdline-tools \
    && mv $ANDROID_HOME/cmdline-tools/cmdline-tools $ANDROID_HOME/cmdline-tools/latest \
    && rm /tmp/tools.zip

# Keep in step with compileSdk and buildToolsVersion in app/build.gradle.kts.
RUN yes | $ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager --licenses > /dev/null \
    && $ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager --install \
        "platforms;android-37.0" "build-tools;37.0.0" "platform-tools" > /dev/null

# Gradle comes from the wrapper (gradle/wrapper), checked against its sha256.
ENV GRADLE_USER_HOME=/gradle
WORKDIR /repo/android
