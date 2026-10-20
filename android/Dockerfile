# Builds the remote controller app, natively on x86_64 and arm64 machines:
# Java (Gradle, Kotlin, R8), which does nearly all the work, runs on the
# machine's own architecture. Only aapt2, which Google publishes for x86_64
# Linux alone, needs x86_64; on arm64 it runs under QEMU (see build.sh), with
# the x86_64 glibc and libgcc_s it needs copied from the x86_64 variant of the
# same base image (on x86_64 those are the image's own). Everything is
# pinned: the base image by digest, downloads by checksum, SDK packages by
# version.
FROM --platform=linux/amd64 eclipse-temurin:21-jdk-noble@sha256:107215954359f1ab0be058a247d7c64ed6e69c71c1682d471d38ef751ec8e964 AS x86

FROM eclipse-temurin:21-jdk-noble@sha256:107215954359f1ab0be058a247d7c64ed6e69c71c1682d471d38ef751ec8e964

COPY --from=x86 /lib64/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2
COPY --from=x86 /usr/lib/x86_64-linux-gnu/libc.so.6 /usr/lib/x86_64-linux-gnu/libm.so.6 \
    /usr/lib/x86_64-linux-gnu/libgcc_s.so.1 /usr/lib/x86_64-linux-gnu/libdl.so.2 \
    /usr/lib/x86_64-linux-gnu/libpthread.so.0 /usr/lib/x86_64-linux-gnu/librt.so.1 \
    /usr/lib/x86_64-linux-gnu/

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
