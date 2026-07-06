#!/usr/bin/env bash
#
# Full Android build pipeline (Podman container edition).
# Auto-wraps itself inside a Podman container; runs directly if already inside.
# Usage:  bash scripts/build-android.sh

set -euo pipefail

# Set VOC_BUILDER_DAEMON=1 to keep container alive (warm Gradle daemon)
if [ "${INSIDE_VOC_BUILDER:-}" != "1" ]; then
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

    source "$SCRIPT_DIR/lib/common.sh"
    : "${container_name:=voc-builder-daemon}"
    : "${keystore_file:=${HOME}/.android/debug.keystore}"
    : "${keystore_pass:=android}"
    : "${app_package_name:=com.scallion.vocab}"

    # Split path for bind mount (dirname) and container path (basename)
    keystore_dir="$(dirname "$keystore_file")"
    keystore_basename="$(basename "$keystore_file")"

    if [ -f "$keystore_file" ]; then
        key_alias=$(keytool -list -v -keystore "$keystore_file" -storepass "$keystore_pass" 2>/dev/null \
            | grep -oP '(?<=^(Alias name|別名名稱): ).*')
    fi
    : "${key_alias:=androiddebugkey}"
    keystore_default="$HOME/.android/debug.keystore"
    if [ "$keystore_file" = "$keystore_default" ] && [ ! -f "$keystore_file" ]; then
        keystore_mount=
    else
        mkdir -p "$(dirname "$keystore_file")"
        keystore_mount="$keystore_dir"
    fi

    # Ensure host cache directories exist (bind mount source must exist)
    mkdir -p "${HOME}/.gradle" "${HOME}/.cache/sccache" \
        "${HOME}/.cargo/registry" "${HOME}/.cargo/git" \
        "${PROJECT_ROOT}/gui/target"

    if [ "${VOC_BUILDER_DAEMON:-}" = "1" ]; then
        if "${RUNNER[@]}" container exists "$container_name"; then
            "${RUNNER[@]}" start "$container_name" >/dev/null 2>&1 || true
        else
            "${RUNNER[@]}" run -d --name "$container_name" \
                -v "${PROJECT_ROOT}:/workspace:rslave" \
                -v "${PROJECT_ROOT}/gui/target:/workspace/gui/target:rslave" \
                -v "${HOME}/.cargo/registry:/root/.cargo/registry:rslave" \
                -v "${HOME}/.cargo/git:/root/.cargo/git:rslave" \
                -v "${HOME}/.gradle:/root/.gradle:rslave" \
                -v "${HOME}/.cache/sccache:/root/.cache/sccache:rslave" \
                ${keystore_mount:+-v "${keystore_dir}:/root/.android:rslave"} \
                -e INSIDE_VOC_BUILDER=1 \
                -e KEYSTORE_FILE="${keystore_basename}" \
                -e KEYSTORE_PASS="${keystore_pass}" \
                -e KEY_ALIAS="${key_alias}" \
                -e APP_PACKAGE_NAME="${app_package_name}" \
                -w /workspace \
                "$container_location" \
                bash -c "while true; do sleep infinity; done"
        fi
        exec "${RUNNER[@]}" exec "$container_name" \
            bash /workspace/scripts/build-android.sh "$@"
    fi

    exec "${RUNNER[@]}" run --rm \
        -v "${PROJECT_ROOT}:/workspace:rslave" \
        -v "${PROJECT_ROOT}/gui/target:/workspace/gui/target:rslave" \
        -v "${HOME}/.cargo/registry:/root/.cargo/registry:rslave" \
        -v "${HOME}/.cargo/git:/root/.cargo/git:rslave" \
        -v "${HOME}/.gradle:/root/.gradle:rslave" \
        -v "${HOME}/.cache/sccache:/root/.cache/sccache:rslave" \
        ${keystore_mount:+-v "${keystore_dir}:/root/.android:rslave"} \
        -e INSIDE_VOC_BUILDER=1 \
        -e KEYSTORE_FILE="${keystore_basename}" \
        -e KEYSTORE_PASS="${keystore_pass}" \
        -e KEY_ALIAS="${key_alias}" \
        -e APP_PACKAGE_NAME="${app_package_name}" \
        -w /workspace \
        "$container_location" \
        bash /workspace/scripts/build-android.sh "$@"
fi

# Inside container — actual build logic below

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

ANDROID_HOME="${ANDROID_HOME:-/opt/android-sdk}"
ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$(ls -d /opt/android-sdk/ndk/* 2>/dev/null | head -1)}"
export ANDROID_HOME ANDROID_NDK_HOME
: "${KEYSTORE_FILE:=debug.keystore}"
: "${KEYSTORE_PASS:=android}"
: "${KEY_ALIAS:=androiddebugkey}"
KEYSTORE="$HOME/.android/${KEYSTORE_FILE}"
BUILD_TOOLS_DIR="${ANDROID_HOME}/build-tools"
PROFILE="release"
GRADLE_DIR="$PROJECT_ROOT/gui/target/dx/scallion-vocab/${PROFILE}/android/app"

echo "=== Step 0: Checking dependencies ==="
echo "  ANDROID_HOME:    ${ANDROID_HOME}"
echo "  ANDROID_NDK_HOME: ${ANDROID_NDK_HOME}"

# Find latest build-tools version
BT_VERSION=$(ls "${BUILD_TOOLS_DIR}" | sort -V | tail -1)
APKSIGNER="${BUILD_TOOLS_DIR}/${BT_VERSION}/apksigner"
ZIPALIGN="${BUILD_TOOLS_DIR}/${BT_VERSION}/zipalign"
echo "  build-tools:     ${BT_VERSION}"
echo "  apksigner:       ${APKSIGNER}"
echo "  keystore:        ${KEYSTORE} (alias: ${KEY_ALIAS})"

if [ ! -f "$APKSIGNER" ]; then
    echo "ERROR: apksigner not found at $APKSIGNER"
    exit 1
fi
if [ ! -d "$ANDROID_NDK_HOME" ]; then
    echo "ERROR: NDK not found at $ANDROID_NDK_HOME"
    exit 1
fi
if [ ! -f "$KEYSTORE" ]; then
    echo "ERROR: Debug keystore not found at $KEYSTORE"
    exit 1
fi
echo ""

echo "=== Step 1: Running dx build ==="
cd "$PROJECT_ROOT/gui"

# dx build's Gradle step may fail (resource linking after splash removal) — that's OK
set +e
dx build --platform android --target aarch64-linux-android --${PROFILE} 2>&1
DX_EXIT=$?
set -e

if [ $DX_EXIT -ne 0 ]; then
    echo "  (dx build Gradle step may have failed — expected after resource patching; continuing...)"
fi
cd "$PROJECT_ROOT"
echo ""

echo "=== Step 2: Patching generated Android resources ==="

bash "$SCRIPT_DIR/lib/patch-android.sh" "$PROFILE"

echo "=== Step 2b: Fixing build warnings ==="

GPATH="$GRADLE_DIR/gradle.properties"
sed -i '/android.defaults.buildfeatures.buildconfig/d' "$GPATH"

APP_BUILD_GRADLE="$GRADLE_DIR/app/build.gradle.kts"
if ! grep -q 'useLegacyPackaging' "$APP_BUILD_GRADLE"; then
    sed -i '/kotlinOptions {/i\    packaging {\n        jniLibs.useLegacyPackaging = true\n    }\n' "$APP_BUILD_GRADLE"
fi
# migrate deprecated kotlinOptions syntax
sed -i '/^import org.jetbrains.kotlin.gradle.dsl.JvmTarget/d' "$APP_BUILD_GRADLE"
sed -i '1i import org.jetbrains.kotlin.gradle.dsl.JvmTarget' "$APP_BUILD_GRADLE"
sed -i '/kotlinOptions {/,/}/d' "$APP_BUILD_GRADLE"
sed -i '/compileOptions {/i\    kotlin {\n        compilerOptions.jvmTarget.set(JvmTarget.JVM_17)\n    }\n' "$APP_BUILD_GRADLE"

echo "  → Warning fixes complete"
echo ""

echo "=== Step 2c: Applying package name ==="
APP_PACKAGE_NAME="${APP_PACKAGE_NAME:-com.scallion.vocab}"
echo "  app_package_name: ${APP_PACKAGE_NAME}"
sed -i "s/namespace=\"[^\"]*\"/namespace=\"${APP_PACKAGE_NAME}\"/" "$APP_BUILD_GRADLE"
sed -i "s/applicationId = \"[^\"]*\"/applicationId = \"${APP_PACKAGE_NAME}\"/" "$APP_BUILD_GRADLE"
# Update BuildConfig typealias in Kotlin source
find "$GRADLE_DIR/app/src/main/kotlin" -name "*.kt" -exec \
    sed -i "s/typealias BuildConfig = [a-zA-Z0-9._]*\.BuildConfig/typealias BuildConfig = ${APP_PACKAGE_NAME}.BuildConfig/" {} +
echo "  → Package name applied to build.gradle.kts and Kotlin sources"
echo ""

echo "=== Step 3: Rebuilding APK ==="
cd "$GRADLE_DIR"
./gradlew assembleRelease -x lintVitalAnalyzeRelease 2>&1
cd "$PROJECT_ROOT"
echo ""

echo "=== Step 4: Finding unsigned APK ==="

APK_OUTPUT_DIR="$GRADLE_DIR/app/build/outputs/apk/release"
UNSIGNED_APK=$(find "$APK_OUTPUT_DIR" -name "*.apk" 2>/dev/null | head -1)

if [ -z "$UNSIGNED_APK" ]; then
    echo "ERROR: No APK found in ${APK_OUTPUT_DIR}"
    exit 1
fi

echo "  Found: ${UNSIGNED_APK}"
echo ""

echo "=== Step 5: Signing APK ==="

ALIGNED_APK="${UNSIGNED_APK%.apk}-aligned.apk"
FINAL_APK="$APK_OUTPUT_DIR/app-release-signed.apk"

if [ -f "$ZIPALIGN" ]; then
    "$ZIPALIGN" -f -p 4 "$UNSIGNED_APK" "$ALIGNED_APK"
    SIGN_INPUT="$ALIGNED_APK"
else
    echo "  zipalign not found, skipping alignment"
    SIGN_INPUT="$UNSIGNED_APK"
fi

JDK_JAVA_OPTIONS="--enable-native-access=ALL-UNNAMED" \
"$APKSIGNER" sign \
    --ks "$KEYSTORE" \
    --ks-pass "pass:${KEYSTORE_PASS}" \
    --ks-key-alias "$KEY_ALIAS" \
    "$SIGN_INPUT"

if [ "$SIGN_INPUT" != "$UNSIGNED_APK" ]; then
    mv "$SIGN_INPUT" "$FINAL_APK"
else
    cp "$SIGN_INPUT" "$FINAL_APK"
fi

echo "  Signed APK: ${FINAL_APK}"
echo ""

echo "=== Step 6: Verifying APK signature ==="
if JDK_JAVA_OPTIONS="--enable-native-access=ALL-UNNAMED" \
   "$APKSIGNER" verify --verbose "$FINAL_APK" 2>&1; then
    echo ""
    echo "✓ APK signature verified!"
else
    echo ""
    echo "✗ APK signature verification FAILED!"
    exit 1
fi
echo ""

echo "=== Step 7: Copying to build/ ==="
mkdir -p "$PROJECT_ROOT/build"
cp -v "$FINAL_APK" "$PROJECT_ROOT/build/scallion-vocab.apk"
echo ""

echo "======================================"
echo "✓ BUILD COMPLETE!"
echo "  APK: build/scallion-vocab.apk"
echo "  Size: $(du -h "$PROJECT_ROOT/build/scallion-vocab.apk" | cut -f1)"
echo "======================================"
