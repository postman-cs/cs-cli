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

# Construct the binary name
BINARY_NAME="cs-cli-${PLATFORM}-${ARCH_NAME}"

# Create tools directory if it doesn't exist
INSTALL_DIR="$HOME/customer-tools"
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

# Download the latest release binary from GitHub
echo "[DOWNLOAD] Downloading CS-CLI binary..."
RELEASE_URL="https://github.com/postman-cs/cs-cli/releases/latest/download/${BINARY_NAME}"

if command -v curl &> /dev/null; then
    if ! curl -L -f -o "$INSTALL_DIR/cs-cli-binary" "$RELEASE_URL"; then
        echo ""
        echo "[ERROR] Failed to download CS-CLI binary"
        echo "This might mean:"
        echo "  1. No internet connection"
        echo "  2. GitHub is temporarily unavailable"
        echo "  3. The binary for your system (${PLATFORM}-${ARCH_NAME}) isn't available yet"
        echo ""
        echo "Please try again later or contact support"
        exit 1
    fi
elif command -v wget &> /dev/null; then
    if ! wget -O "$INSTALL_DIR/cs-cli-binary" "$RELEASE_URL"; then
        echo ""
        echo "[ERROR] Failed to download CS-CLI binary"
        echo "This might mean:"
        echo "  1. No internet connection"
        echo "  2. GitHub is temporarily unavailable"
        echo "  3. The binary for your system (${PLATFORM}-${ARCH_NAME}) isn't available yet"
        echo ""
        echo "Please try again later or contact support"
        exit 1
    fi
else
    echo "[ERROR] Cannot download files - curl is not available"
    echo ""
    echo "macOS should have curl pre-installed. This might mean:"
    echo "  1. You're using an extremely old version of macOS"
    echo "  2. Your system configuration has been modified"
    echo ""
    echo "Please contact support for help"
    exit 1
fi

# Make the binary executable
chmod +x "$INSTALL_DIR/cs-cli-binary"

echo "[SETUP] CS-CLI downloaded successfully!"

# Create a wrapper script that runs from Desktop
echo "[CONFIG] Creating global 'cs-cli' command..."
cat > "$INSTALL_DIR/cs-cli-wrapper" << 'EOF'
#!/bin/bash
# Change to Desktop so output folders are easy to find
cd "$HOME/Desktop" 2>/dev/null || cd "$HOME"
# Run the actual CLI
exec "$HOME/customer-tools/cs-cli-binary" "$@"
EOF

chmod +x "$INSTALL_DIR/cs-cli-wrapper"

# Install to user bin (avoids needing admin password)
mkdir -p "$HOME/.local/bin"
ln -sf "$INSTALL_DIR/cs-cli-wrapper" "$HOME/.local/bin/cs-cli"

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