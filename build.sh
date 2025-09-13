#!/bin/bash

# CS-CLI Cross-Platform Build Script
# Builds and signs binaries for macOS and Windows

set -e

echo "================================"
echo "CS-CLI Cross-Platform Builder"
echo "================================"
echo ""

# Configuration
BINARY_NAME="cs-cli"
VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)

# macOS signing identity (update this with your cert name)
# Find your identity with: security find-identity -v -p codesigning
MAC_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"

# Windows signing (requires signtool.exe and certificate)
WINDOWS_CERT_PATH="./certificates/windows_cert.pfx"
WINDOWS_CERT_PASSWORD=""

# Build flags for optimized binaries
export RUSTFLAGS='--cfg reqwest_unstable -C target-cpu=native -C opt-level=3'

# Function to build for a target
build_target() {
    local TARGET=$1
    local OUTPUT_NAME=$2

    echo "Building for $TARGET..."

    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "Installing target $TARGET..."
        rustup target add "$TARGET"
    fi

    cargo build --release --target "$TARGET"

    if [ -f "target/$TARGET/release/$BINARY_NAME" ]; then
        cp "target/$TARGET/release/$BINARY_NAME" "./$OUTPUT_NAME"
        echo "✓ Built $OUTPUT_NAME"
    elif [ -f "target/$TARGET/release/$BINARY_NAME.exe" ]; then
        cp "target/$TARGET/release/$BINARY_NAME.exe" "./$OUTPUT_NAME"
        echo "✓ Built $OUTPUT_NAME"
    else
        echo "✗ Failed to build $OUTPUT_NAME"
        return 1
    fi
}

# Clean previous builds
echo "Cleaning previous builds..."
rm -f cs-cli-*
cargo clean

# Build for macOS Intel
echo ""
echo "Building macOS Intel (x86_64)..."
build_target "x86_64-apple-darwin" "cs-cli-macos-intel"

# Build for macOS Apple Silicon
echo ""
echo "Building macOS Apple Silicon (arm64)..."
build_target "aarch64-apple-darwin" "cs-cli-macos-arm64"

# Create universal macOS binary
echo ""
echo "Creating universal macOS binary..."
lipo -create -output "cs-cli-macos-universal" \
    "cs-cli-macos-intel" \
    "cs-cli-macos-arm64"
echo "✓ Created universal binary"

# Sign macOS binaries
echo ""
echo "Signing macOS binaries..."
if security find-identity -v -p codesigning | grep -q "$MAC_SIGNING_IDENTITY"; then
    # Sign with hardened runtime for notarization
    codesign --force --options runtime --timestamp \
        --sign "$MAC_SIGNING_IDENTITY" \
        "cs-cli-macos-universal"

    # Verify signature
    codesign --verify --verbose "cs-cli-macos-universal"
    echo "✓ Signed macOS universal binary"

    # Optional: Notarize the binary
    # echo "Notarizing binary..."
    # xcrun notarytool submit "cs-cli-macos-universal" \
    #     --apple-id "your-apple-id@example.com" \
    #     --team-id "TEAM_ID" \
    #     --password "app-specific-password" \
    #     --wait
else
    echo "⚠ Warning: Signing identity not found. Binary will not be signed."
    echo "  Users will see Gatekeeper warnings."
    echo "  Update MAC_SIGNING_IDENTITY in this script with your certificate."
fi

# Build for Windows x64
if command -v x86_64-pc-windows-gnu-gcc &> /dev/null; then
    echo ""
    echo "Building Windows x64..."
    build_target "x86_64-pc-windows-gnu" "cs-cli-windows-x64.exe"
elif rustup target list | grep -q "x86_64-pc-windows-msvc"; then
    echo ""
    echo "Building Windows x64 (MSVC)..."
    build_target "x86_64-pc-windows-msvc" "cs-cli-windows-x64.exe"
else
    echo ""
    echo "⚠ Skipping Windows build (cross-compilation not set up)"
    echo "  To build for Windows, install:"
    echo "  - mingw-w64 (brew install mingw-w64) OR"
    echo "  - Run this script on Windows with MSVC"
fi

# Sign Windows binary (if built and certificate available)
if [ -f "cs-cli-windows-x64.exe" ] && [ -f "$WINDOWS_CERT_PATH" ]; then
    echo ""
    echo "Signing Windows binary..."
    # This requires signtool.exe from Windows SDK
    # Usually run on Windows or via Wine
    if command -v signtool &> /dev/null; then
        signtool sign /f "$WINDOWS_CERT_PATH" /p "$WINDOWS_CERT_PASSWORD" \
            /t http://timestamp.digicert.com \
            /fd SHA256 \
            "cs-cli-windows-x64.exe"
        echo "✓ Signed Windows binary"
    else
        echo "⚠ signtool not found. Windows binary will not be signed."
    fi
fi

# Create release packages
echo ""
echo "Creating release packages..."

# macOS package
mkdir -p release/macos
cp cs-cli-macos-universal release/macos/cs-cli
cat > release/macos/README.txt << 'EOF'
CS-CLI for macOS
================

Just double-click 'cs-cli' to run!

If macOS says "can't be opened":
1. Right-click on 'cs-cli'
2. Select "Open"
3. Click "Open" in the dialog

This only needs to be done once.
EOF
cd release/macos && zip -r ../../cs-cli-macos-v${VERSION}.zip * && cd ../..
echo "✓ Created cs-cli-macos-v${VERSION}.zip"

# Windows package (if built)
if [ -f "cs-cli-windows-x64.exe" ]; then
    mkdir -p release/windows
    cp cs-cli-windows-x64.exe release/windows/cs-cli.exe
    cat > release/windows/README.txt << 'EOF'
CS-CLI for Windows
==================

Just double-click 'cs-cli.exe' to run!

The tool will open in PowerShell automatically.

If Windows Defender blocks it:
1. Click "More info"
2. Click "Run anyway"
EOF
    cd release/windows && zip -r ../../cs-cli-windows-v${VERSION}.zip * && cd ../..
    echo "✓ Created cs-cli-windows-v${VERSION}.zip"
fi

# Summary
echo ""
echo "================================"
echo "Build Complete!"
echo "================================"
echo ""
echo "Built binaries:"
ls -lh cs-cli-* 2>/dev/null | grep -v ".zip"
echo ""
echo "Release packages:"
ls -lh *.zip 2>/dev/null
echo ""
echo "Next steps:"
echo "1. Test the binaries on target platforms"
echo "2. Upload .zip files to GitHub Releases"
echo "3. Users can download and double-click to run!"