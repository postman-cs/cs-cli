#!/bin/bash

# CS-Transcript-CLI Installer (Rust Edition)
# One-command setup for non-technical users

set -e  # Exit on any error

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

# Map to GitHub release naming convention
case "$OS" in
    Darwin)
        PLATFORM="darwin"
        ;;
    Linux)
        PLATFORM="linux"
        ;;
    *)
        echo "[ERROR] Unsupported operating system: $OS"
        exit 1
        ;;
esac

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

# For macOS, download and install the PKG
if [ "$OS" = "Darwin" ]; then
    # Determine PKG name based on architecture
    if [ "$ARCH" = "arm64" ]; then
        PKG_ARCH="arm64"
    else
        PKG_ARCH="x86_64"
    fi

    PKG_NAME="cs-cli-${PKG_ARCH}-1.0.0.pkg"
    PKG_URL="https://github.com/postman-cs/cs-cli/releases/latest/download/${PKG_NAME}"

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
    # Install the PKG (this will prompt for password)
    if ! installer -pkg "$TEMP_PKG" -target CurrentUserHomeDirectory 2>/dev/null; then
        # If user home install fails, try with sudo
        echo "Installing CS-CLI (may require your password)..."
        sudo installer -pkg "$TEMP_PKG" -target /
    fi

    # Clean up
    rm -f "$TEMP_PKG"

    echo "[SETUP] CS-CLI installed successfully!"

    # The PKG installer already sets up PATH, but ensure it's in current session
    export PATH="$HOME/.local/bin:$PATH"

    PATH_ADDED=false
else
    # For non-macOS, download binary directly (future support)
    echo "[ERROR] Only macOS is currently supported"
    exit 1
fi

# Add to PATH if not already there
PATH_ADDED=false
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bash_profile"
    export PATH="$HOME/.local/bin:$PATH"
    PATH_ADDED=true
    echo "[CONFIG] Added cs-cli to your PATH"
fi

echo ""
echo "[SUCCESS] Installation complete!"
echo ""
echo "CS-CLI has been installed and is ready to use!"
echo ""
echo "You're ready to find customer insights!"
echo ""

# Try to source the shell config to make cs-cli immediately available
if [ "$PATH_ADDED" = true ]; then
    echo "To get started right now, type:"
    echo "  source ~/.zshrc && cs-cli"
    echo ""
    echo "Or simply restart Terminal and then type 'cs-cli'"
else
    echo "To get started, type:"
    echo "  cs-cli"
fi
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