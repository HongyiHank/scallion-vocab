FROM docker.io/rust:latest

# Override with --build-arg JDK_VERSION=xx; must match jdk_version in scripts/env.txt.
ARG JDK_VERSION=21
RUN set -ex; \
    apt-get update; \
    if [ "${JDK_VERSION}" = "latest" ]; then \
        JDK_VER=$(apt-cache search 'openjdk' | grep -oP '^openjdk-\K[0-9]+(?=-jdk\s)' | sort -V | tail -1); \
        echo "Resolved JDK: latest → ${JDK_VER}"; \
    else \
        JDK_VER="${JDK_VERSION}"; \
    fi; \
    apt-get install -y --no-install-recommends \
        gnupg ca-certificates curl unzip \
        "openjdk-${JDK_VER}-jdk" \
        libgtk-3-dev libsoup-3.0-dev libwebkit2gtk-4.1-dev \
        && rm -rf /var/lib/apt/lists/*

# Rust Android target
RUN rustup target add aarch64-linux-android

# Dioxus CLI
RUN cargo install dioxus-cli --version ">=0.7" --locked

ENV ANDROID_SDK_ROOT=/opt/android-sdk
ENV ANDROID_HOME=/opt/android-sdk
ENV PATH="${ANDROID_HOME}/cmdline-tools/latest/bin:${PATH}"

# Download cmdline-tools (provides sdkmanager)
RUN mkdir -p /opt/android-sdk && \
    curl -fsSL \
      https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip \
      -o /tmp/cmdline-tools.zip && \
    unzip -q /tmp/cmdline-tools.zip -d /tmp/cmdline-tools-extracted/ && \
    mkdir -p /opt/android-sdk/cmdline-tools && \
    mv /tmp/cmdline-tools-extracted/cmdline-tools /opt/android-sdk/cmdline-tools/latest && \
    rm -rf /tmp/cmdline-tools.zip /tmp/cmdline-tools-extracted

# Override with --build-arg ANDROID_PLATFORM=xx; must match android_platform in scripts/env.txt.
ARG ANDROID_PLATFORM=34

RUN yes | sdkmanager --licenses

# Auto-resolve "latest" platform; match only integer versions (skip -ext4, -ext14, etc.)
RUN set -ex; \
    if [ "${ANDROID_PLATFORM}" = "latest" ]; then \
        PLATFORM=$(sdkmanager --list 2>/dev/null \
          | grep -oP 'platforms;android-\K[0-9]+(?=\s)' \
          | sort -V | tail -1); \
        echo "Resolved Android platform: latest → ${PLATFORM}"; \
    else \
        PLATFORM="${ANDROID_PLATFORM}"; \
    fi; \
    sdkmanager --verbose "platforms;android-${PLATFORM}"

# Install latest build-tools (no preview/beta/rc)
RUN BUILD_TOOLS_VER=$(sdkmanager --list 2>/dev/null \
      | grep 'build-tools;[0-9]' \
      | grep -oP 'build-tools;\K[0-9]+\.[0-9]+\.[0-9]+' \
      | sort -V | tail -1) && \
    echo "Installing build-tools;${BUILD_TOOLS_VER}" && \
    sdkmanager "build-tools;${BUILD_TOOLS_VER}"

# Install latest NDK
RUN NDK_VER=$(sdkmanager --list 2>/dev/null \
      | grep 'ndk;[0-9]' \
      | grep -viE 'preview|beta|rc|alpha|canary' \
      | grep -oP 'ndk;\K[0-9]+\.[0-9]+\.[0-9]+' \
      | sort -V | tail -1) && \
    echo "Installing ndk;${NDK_VER}" && \
    sdkmanager "ndk;${NDK_VER}" && \
    echo "${NDK_VER}" > /opt/android-sdk/ndk-version.txt

# ANDROID_NDK_HOME needed for dx build — resolve at build time
RUN echo "export ANDROID_NDK_HOME=/opt/android-sdk/ndk/$(cat /opt/android-sdk/ndk-version.txt)" \
    > /etc/profile.d/android-ndk.sh && \
    echo "ANDROID_NDK_HOME=/opt/android-sdk/ndk/$(cat /opt/android-sdk/ndk-version.txt)" \
    >> /etc/environment

RUN mkdir -p /root/.android && keytool -genkey -v \
    -keystore /root/.android/debug.keystore \
    -storepass android -alias androiddebugkey \
    -keypass android -keyalg RSA -keysize 2048 \
    -validity 10000 \
    -dname "CN=Android Debug,O=Android,C=US"
