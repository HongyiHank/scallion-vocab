#!/usr/bin/env bash
#
# Post-dx-build patching: replace Dioxus-generated icon WebP with correctly-centered scallion images.
# Usage:  bash scripts/lib/patch-android.sh <release|debug>

set -euo pipefail

PROFILE="${1:-release}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

SRC_RES="$PROJECT_ROOT/gui/android/app/src/main/res"
SRC_KOTLIN="$PROJECT_ROOT/gui/android/app/src/main/kotlin"
TARGET_RES="$PROJECT_ROOT/gui/target/dx/scallion-vocab/${PROFILE}/android/app/app/src/main/res"
TARGET_KOTLIN="$PROJECT_ROOT/gui/target/dx/scallion-vocab/${PROFILE}/android/app/app/src/main/kotlin"
TARGET_MANIFEST="$PROJECT_ROOT/gui/target/dx/scallion-vocab/${PROFILE}/android/app/app/src/main/AndroidManifest.xml"
TARGET_BUILD_GRADLE="$PROJECT_ROOT/gui/target/dx/scallion-vocab/${PROFILE}/android/app/app/build.gradle.kts"

if [ ! -d "$TARGET_RES" ]; then
    echo "ERROR: Generated project not found at $TARGET_RES"
    echo "Run 'dx build' first."
    exit 1
fi

echo "=== Patching Android project ==="

# Remove generated splash files (Dioxus defaults)
echo "  → Removing splash files"
rm -vf "$TARGET_RES/drawable/splash_screen.xml" 2>/dev/null || true
rm -vf "$TARGET_RES/values/themes.xml" 2>/dev/null || true

echo "  → mipmap-anydpi-v26/ic_launcher.xml"
cp -v "$SRC_RES/mipmap-anydpi-v26/ic_launcher.xml" \
      "$TARGET_RES/mipmap-anydpi-v26/ic_launcher.xml"

# Copy density-specific WebP icons
for density in mdpi hdpi xhdpi xxhdpi xxxhdpi; do
    for file in ic_launcher_foreground.webp ic_launcher.webp; do
        src="$SRC_RES/mipmap-${density}/${file}"
        dst="$TARGET_RES/mipmap-${density}/${file}"
        if [ -f "$src" ]; then
            cp -v "$src" "$dst"
        fi
    done
done

# Dioxus overwrites values resources with defaults — restore ours
echo "  → values/styles.xml"
cp -v "$SRC_RES/values/styles.xml" "$TARGET_RES/values/styles.xml"
echo "  → values/colors.xml"
cp -v "$SRC_RES/values/colors.xml" "$TARGET_RES/values/colors.xml"
echo "  → values/themes.xml"
cp -v "$SRC_RES/values/themes.xml" "$TARGET_RES/values/themes.xml"
echo "  → values/strings.xml"
cp -v "$SRC_RES/values/strings.xml" "$TARGET_RES/values/strings.xml"

# Copy white background drawable
echo "  → drawable/ic_launcher_background.xml"
cp -v "$SRC_RES/drawable/ic_launcher_background.xml" \
      "$TARGET_RES/drawable/ic_launcher_background.xml"

# Remove old vector drawable from generated project if present
OLD_VECTOR="$TARGET_RES/drawable-v24/ic_launcher_foreground.xml"
if [ -f "$OLD_VECTOR" ]; then
    echo "  → Removing old vector drawable"
    rm -v "$OLD_VECTOR"
fi

# Patch Kotlin source files
echo "  → kotlin/dev/dioxus/main/WryActivity.kt"
cp -v "$SRC_KOTLIN/dev/dioxus/main/WryActivity.kt" \
      "$TARGET_KOTLIN/dev/dioxus/main/WryActivity.kt"
echo "  → kotlin/dev/dioxus/main/MainActivity.kt"
cp -v "$SRC_KOTLIN/dev/dioxus/main/MainActivity.kt" \
      "$TARGET_KOTLIN/dev/dioxus/main/MainActivity.kt"

# Patch RustWebChromeClient — fix missing imports & ensure onCreateWindow exists
CHROME_CLIENT="$TARGET_KOTLIN/dev/dioxus/main/RustWebChromeClient.kt"
if [ -f "$CHROME_CLIENT" ]; then
    echo "  → kotlin/dev/dioxus/main/RustWebChromeClient.kt (imports + onCreateWindow)"
    python3 "$SCRIPT_DIR/patch-chrome-client.py" "$CHROME_CLIENT"
fi

echo "=== Patching complete ==="
