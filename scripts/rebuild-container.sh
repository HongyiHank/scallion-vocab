#!/usr/bin/env bash
#
# Rebuild the voc-builder container image.
# Usage:  bash scripts/rebuild-container.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

source "$SCRIPT_DIR/lib/common.sh"
: "${jdk_version:=21}"

: "${android_platform:=latest}"

echo "=== Rebuilding container ==="
echo "  Runtime:          ${RUNNER[0]}"
echo "  Storage:          ${storage_dir:-default}"
echo "  Image:            ${container_location}"
echo "  Context:          ${PROJECT_ROOT}"
echo "  JDK:              ${jdk_version}"
echo "  Android platform: ${android_platform}"
echo ""

"${RUNNER[@]}" build \
    --build-arg "JDK_VERSION=${jdk_version}" \
    --build-arg "ANDROID_PLATFORM=${android_platform}" \
    -t "$container_location" \
    "$PROJECT_ROOT"

echo ""
echo "=== Done: ${container_location} ==="
echo "  JDK:              ${jdk_version}"
echo "  Android platform: ${android_platform}"
