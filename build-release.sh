#!/usr/bin/env bash
set -e

BINARY_NAME="pe"
BIN_DIR="$HOME/bin"
DIST_DIR="dist"

TARGETS=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-unknown-linux-musl"
    "x86_64-pc-windows-gnu"
)

mkdir -p "$DIST_DIR"

echo "Building $BINARY_NAME for all targets..."
echo ""

for target in "${TARGETS[@]}"; do
    echo "  Building $target..."
    if rustup target add "$target" 2>/dev/null; then
        if cargo build --release --target "$target" 2>/dev/null; then
            src="target/$target/release/$BINARY_NAME"
            # Windows binary has .exe extension
            if [[ "$target" == *windows* ]]; then
                src="${src}.exe"
                cp "$src" "$DIST_DIR/${BINARY_NAME}-windows-x86_64.exe"
            elif [[ "$target" == "aarch64-apple-darwin" ]]; then
                cp "$src" "$DIST_DIR/${BINARY_NAME}-macos-arm64"
            elif [[ "$target" == "x86_64-apple-darwin" ]]; then
                cp "$src" "$DIST_DIR/${BINARY_NAME}-macos-x86_64"
            elif [[ "$target" == *linux* ]]; then
                cp "$src" "$DIST_DIR/${BINARY_NAME}-linux-x86_64"
            fi
            echo "    OK"
        else
            echo "    SKIPPED (cross-compiler not available for $target)"
        fi
    else
        echo "    SKIPPED (could not add target $target)"
    fi
done

echo ""
echo "Copying native binary to $BIN_DIR..."

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        if [[ "$ARCH" == "arm64" ]]; then
            NATIVE_BIN="$DIST_DIR/${BINARY_NAME}-macos-arm64"
        else
            NATIVE_BIN="$DIST_DIR/${BINARY_NAME}-macos-x86_64"
        fi
        ;;
    Linux)
        NATIVE_BIN="$DIST_DIR/${BINARY_NAME}-linux-x86_64"
        ;;
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
        NATIVE_BIN="$DIST_DIR/${BINARY_NAME}-windows-x86_64.exe"
        ;;
    *)
        echo "Unknown OS: $OS — skipping copy to ~/bin"
        exit 0
        ;;
esac

if [[ -f "$NATIVE_BIN" ]]; then
    mkdir -p "$BIN_DIR"
    cp "$NATIVE_BIN" "$BIN_DIR/$BINARY_NAME"
    chmod +x "$BIN_DIR/$BINARY_NAME"
    echo "Installed: $BIN_DIR/$BINARY_NAME"
else
    echo "Native binary not found at $NATIVE_BIN — skipping install"
fi
