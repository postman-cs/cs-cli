#!/bin/bash

# CS-Transcript-CLI Installer (Rust Edition)
# One-command setup for non-technical users

set -e  # Exit on any error

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

# Only macOS is supported
if [ "$OS" != "Darwin" ]; then
    echo "[ERROR] Only macOS is currently supported"
    exit 1
fi

case "$ARCH" in
    x86_64)
        ARCH_NAME="x86_64"
        ;;
    arm64|aarch64)
        ARCH_NAME="aarch64"
        ;;
    *)
        echo "[ERROR] Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Determine PKG name based on architecture
if [ "$ARCH" = "arm64" ]; then
    PKG_ARCH="arm64"
else
    PKG_ARCH="x86_64"
fi

# Get the latest version from GitHub releases
echo "[INFO] Fetching latest version..."
LATEST_VERSION=$(curl -s https://api.github.com/repos/postman-cs/cs-cli/releases/latest | grep -o '"tag_name": *"[^"]*"' | grep -o 'v[^"]*')

if [ -z "$LATEST_VERSION" ]; then
    echo "[ERROR] Could not determine latest version"
    exit 1
fi

# Remove 'v' prefix from version
VERSION=${LATEST_VERSION#v}

PKG_NAME="cs-cli-${PKG_ARCH}-${VERSION}.pkg"
PKG_URL="https://github.com/postman-cs/cs-cli/releases/download/${LATEST_VERSION}/${PKG_NAME}"

echo "[DOWNLOAD] Downloading CS-CLI installer..."
TEMP_PKG="/tmp/${PKG_NAME}"

if command -v curl &> /dev/null; then
    if ! curl -L -f -o "$TEMP_PKG" "$PKG_URL"; then
        echo ""
        echo "[ERROR] Failed to download CS-CLI installer"
        echo "This might mean:"
        echo "  1. No internet connection"
        echo "  2. GitHub is temporarily unavailable"
        echo "  3. The installer for your system (${PKG_ARCH}) isn't available yet"
        echo ""
        echo "Please try again later or contact support"
        exit 1
    fi
else
    echo "[ERROR] curl is not available"
    exit 1
fi

echo "[INSTALL] Installing CS-CLI..."
# Install the PKG (no admin privileges required)
installer -pkg "$TEMP_PKG" -target /

# Clean up
rm -f "$TEMP_PKG"

echo "[SETUP] CS-CLI installed successfully!"

# The PKG installer handles PATH configuration
export PATH="$HOME/.local/bin:$PATH"

echo ""
echo "[SUCCESS] Installation complete!"
echo ""
echo "To get started right now, type:"
echo "  source ~/.zshrc && cs-cli"
echo ""
echo "Or simply restart Terminal and then type 'cs-cli'"
echo ""
echo "The tool will ask you 3 simple questions:"
echo "  1. What customer are you looking for?"
echo "  2. How far back should I look?"
echo "  3. What would you like to analyze?"
echo ""
echo "That's it! No complex commands needed."
echo ""
echo "Files will appear on your Desktop in a folder like 'ct_customername'"
echo ""