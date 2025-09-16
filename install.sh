#!/bin/bash

# CS-Transcript-CLI Installer (Rust Edition)
# One-command setup for non-technical users
# Updated to download architecture-specific binaries

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
        BINARY_NAME="cs-cli-x86_64-apple-darwin"
        ;;
    arm64|aarch64)
        BINARY_NAME="cs-cli-aarch64-apple-darwin"
        ;;
    *)
        echo "[ERROR] Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Get the latest version from GitHub releases
echo "[INFO] Fetching latest version..."
LATEST_VERSION=$(curl -s https://api.github.com/repos/postman-cs/cs-cli/releases/latest | grep -o '"tag_name": *"[^"]*"' | grep -o 'v[^"]*')

if [ -z "$LATEST_VERSION" ]; then
    echo "[ERROR] Could not determine latest version"
    exit 1
fi

BINARY_URL="https://github.com/postman-cs/cs-cli/releases/download/${LATEST_VERSION}/${BINARY_NAME}"

echo "[DOWNLOAD] Downloading CS-CLI binary for $ARCH..."
TEMP_BINARY="/tmp/cs-cli"

if command -v curl &> /dev/null; then
    if ! curl -L -f -o "$TEMP_BINARY" "$BINARY_URL"; then
        echo ""
        echo "[ERROR] Failed to download CS-CLI binary"
        echo "This might mean:"
        echo "  1. No internet connection"
        echo "  2. GitHub is temporarily unavailable"
        echo "  3. The binary for your system ($ARCH) isn't available yet"
        echo ""
        echo "Please try again later or contact support"
        exit 1
    fi
else
    echo "[ERROR] curl is not available"
    exit 1
fi

echo "[INSTALL] Installing CS-CLI..."
# Create local bin directory if it doesn't exist
mkdir -p "$HOME/.local/bin"

# Make the binary executable and move it to the local bin directory
chmod +x "$TEMP_BINARY"
mv "$TEMP_BINARY" "$HOME/.local/bin/cs-cli"

echo "[SETUP] CS-CLI installed successfully!"

# Add ~/.local/bin to PATH if not already there
SHELL_RC=""
if [ "$SHELL" = "/bin/zsh" ] || [ "$SHELL" = "/usr/bin/zsh" ]; then
    SHELL_RC="$HOME/.zshrc"
elif [ "$SHELL" = "/bin/bash" ] || [ "$SHELL" = "/usr/bin/bash" ]; then
    SHELL_RC="$HOME/.bash_profile"
fi

if [ -n "$SHELL_RC" ]; then
    if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$SHELL_RC" 2>/dev/null; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
        echo "[SETUP] Added ~/.local/bin to PATH in $SHELL_RC"
    fi
fi

# Export for current session
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