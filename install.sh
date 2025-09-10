#!/bin/bash

# CS-Transcript-CLI Installer
# One-command setup for non-technical users

set -e  # Exit on any error

echo "🚀 Setting up CS-Transcript-CLI..."
echo ""

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "📦 Installing Homebrew (this manages software on your Mac)..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add Homebrew to PATH for M1/M2 Macs
    if [[ -f "/opt/homebrew/bin/brew" ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
fi

# Install git if not present
if ! command -v git &> /dev/null; then
    echo "📦 Installing git..."
    brew install git
fi

# Create tools directory if it doesn't exist
INSTALL_DIR="$HOME/customer-tools"
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

# Clone or update the repository
if [ -d "cs-transcript-cli" ]; then
    echo "📥 Updating CS-Transcript-CLI..."
    cd cs-transcript-cli
    git pull
else
    echo "📥 Downloading CS-Transcript-CLI..."
    git clone https://github.com/jaredboynton/cs-transcript-cli
    cd cs-transcript-cli
fi

# Make setup executable and run it
chmod +x setup || true
echo "🔧 Running setup..."
./setup

# Create a wrapper script that runs from Desktop
echo "🔗 Creating global 'cs-cli' command..."
cat > "$INSTALL_DIR/cs-cli-wrapper" << 'EOF'
#!/bin/bash
# Change to Desktop so output folders are easy to find
cd "$HOME/Desktop" 2>/dev/null || cd "$HOME"
# Run the actual CLI
exec "$HOME/customer-tools/cs-transcript-cli/cli" "$@"
EOF

chmod +x "$INSTALL_DIR/cs-cli-wrapper"

# Install to system or user bin
sudo ln -sf "$INSTALL_DIR/cs-cli-wrapper" /usr/local/bin/cs-cli 2>/dev/null || {
    # If sudo fails, try user's local bin
    mkdir -p "$HOME/.local/bin"
    ln -sf "$INSTALL_DIR/cs-cli-wrapper" "$HOME/.local/bin/cs-cli"
    
    # Add to PATH if not already there
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bash_profile"
        export PATH="$HOME/.local/bin:$PATH"
    fi
}

echo ""
echo "✅ Installation complete!"
echo ""
echo "🎉 You're ready to find customer insights!"
echo ""
echo "To get started, just type:"
echo "  cs-cli"
echo ""
echo "The tool will ask you 3 simple questions:"
echo "  1. What customer are you looking for?"
echo "  2. How far back should I look?"
echo "  3. What would you like to analyze?"
echo ""
echo "That's it! No complex commands needed."
echo ""
echo "📍 Files will appear on your Desktop in a folder like 'ct_customername'"
