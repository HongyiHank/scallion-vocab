#!/usr/bin/env bash
#
# Shared container runner setup. Source from other scripts.
# After sourcing: $RUNNER, $container_location, $PROJECT_ROOT are ready.
# Usage:  source "$SCRIPT_DIR/lib/common.sh"

_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$_LIB_DIR/../env.txt"

: "${platform:=podman}"
: "${container_location:=localhost/voc-builder}"

if [ -n "${storage_dir:-}" ]; then
    RUNNER=( "$platform" "--root" "$storage_dir" )
else
    RUNNER=( "$platform" )
fi
