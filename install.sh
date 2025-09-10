#!/bin/bash

# CS-Transcript-CLI Installer
# One-command setup for non-technical users

set -e  # Exit on any error

echo "Setting up CS-Transcript-CLI..."
echo ""
echo "NOTE: This installer may prompt for your password to install required software."
echo "This is normal and secure - it's needed to install Homebrew and system tools."
echo ""

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "[INSTALL] Installing Homebrew (this manages software on your Mac)..."
    echo "[SUDO] You'll be asked for your Mac password to install Homebrew."
    echo "      This is required for system-level software installation."
    echo ""
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add Homebrew to PATH for M1/M2 Macs
    if [[ -f "/opt/homebrew/bin/brew" ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    elif [[ -f "/usr/local/bin/brew" ]]; then
        eval "$(/usr/local/bin/brew shellenv)"
    fi
fi

# Install git if not present
if ! command -v git &> /dev/null; then
    echo "[INSTALL] Installing git..."
    brew install git
fi

# Install uv if not present (needed for Python environment)
if ! command -v uv &> /dev/null; then
    echo "[INSTALL] Installing uv (Python environment manager)..."
    brew install uv
fi

# Create tools directory if it doesn't exist
INSTALL_DIR="$HOME/customer-tools"
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

# Clone or update the repository
if [ -d "cs-cli" ]; then
    echo "[UPDATE] Updating CS-Transcript-CLI..."
    cd cs-cli
    git pull
else
    echo "[DOWNLOAD] Downloading CS-Transcript-CLI..."
    git clone https://github.com/jaredboynton/cs-cli
    cd cs-cli
fi

# Setup Python dependencies using uv
echo "[SETUP] Installing Python dependencies..."

# Ensure Python 3.12 is available
echo "[SETUP] Ensuring Python 3.12 is available..."
uv python install 3.12 >/dev/null 2>&1 || true

# Initialize uv project and install dependencies
echo "[SETUP] Installing project dependencies..."
if [ ! -f "uv.lock" ]; then
    # First time setup - create lock file
    uv sync
else
    # Already initialized - just sync dependencies
    uv sync
fi

# Make cli executable
chmod +x cli || true

echo "[SETUP] Dependencies installed successfully!"

# Create a wrapper script that runs from Desktop
echo "[CONFIG] Creating global 'cs-cli' command..."
cat > "$INSTALL_DIR/cs-cli-wrapper" << 'EOF'
#!/bin/bash
# Change to Desktop so output folders are easy to find
cd "$HOME/Desktop" 2>/dev/null || cd "$HOME"
# Run the actual CLI
exec "$HOME/customer-tools/cs-cli/cli" "$@"
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
echo "All required software has been installed:"
echo "  - Homebrew (package manager)"
echo "  - Git (version control)"
echo "  - UV (Python environment manager)"
echo "  - Python 3.12 and all dependencies"
echo ""
echo "You're ready to find customer insights!"
echo ""

# Try to source the shell config to make cs-cli immediately available
if [ "$PATH_ADDED" = true ]; then
    echo "To use cs-cli right now, run one of these commands:"
    echo "  source ~/.zshrc        (if you use zsh - most common)"
    echo "  source ~/.bash_profile (if you use bash)"
    echo ""
    echo "Or simply restart Terminal and the command will be available."
    echo ""
fi

echo "To get started, type:"
echo "  cs-cli"
echo ""
echo "The tool will ask you 3 simple questions:"
echo "  1. What customer are you looking for?"
echo "  2. How far back should I look?"
echo "  3. What would you like to analyze?"
echo ""
echo "That's it! No complex commands needed."
echo ""
echo "Files will appear on your Desktop in a folder like 'ct_customername'"
