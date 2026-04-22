#!/usr/bin/env bash
# Builds standalone PyInstaller binaries for all Python scripts that ctrl-human
# spawns at runtime. Run this once per machine/platform before `pnpm tauri build`.
#
# Output: src-tauri/binaries/{bridge,camera_server,extract_pose}-<target-triple>
#
# Requirements: Rust toolchain on PATH (for `rustc -vV`), pygmy venv in place.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARIES_DIR="$SCRIPT_DIR/../src-tauri/binaries"
BUILD_TMP="$SCRIPT_DIR/.pyinstaller_build"

PYTHON="$SCRIPT_DIR/pygmy/bin/python"
PIP="$SCRIPT_DIR/pygmy/bin/pip"

TARGET=$(rustc -vV | sed -n 's|host: ||p')
echo "Building sidecars for target: $TARGET"

mkdir -p "$BINARIES_DIR" "$BUILD_TMP"

"$PIP" install pyinstaller --quiet

build_binary() {
    local script="$1"
    local name="$2"
    shift 2
    local extra_args=("$@")

    echo "-> Building $name..."
    "$PYTHON" -m PyInstaller \
        --onefile \
        --name "$name" \
        --distpath "$BUILD_TMP" \
        --workpath "$BUILD_TMP/work" \
        --specpath "$BUILD_TMP/specs" \
        --noconfirm \
        "${extra_args[@]}" \
        "$SCRIPT_DIR/$script"

    mv "$BUILD_TMP/$name" "$BINARIES_DIR/$name-$TARGET"
    echo "   $BINARIES_DIR/$name-$TARGET"
}

# bridge.py uses only stdlib - fast build, tiny binary
build_binary bridge.py bridge

# camera_server.py uses cv2 + mediapipe (mediapipe imported lazily inside inference_loop)
build_binary camera_server.py camera_server \
    --collect-all mediapipe \
    --collect-all cv2

# extract_pose.py uses mediapipe
build_binary extract_pose.py extract_pose \
    --collect-all mediapipe

echo ""
echo "Done. Sidecars written to: $BINARIES_DIR/"
echo "You can now run: pnpm tauri build"
