#!/bin/bash

# Build PKG installer for CS-CLI
# Creates .app bundle and installer that sets up PATH automatically

set -e

echo "================================"
echo "CS-CLI PKG Installer Builder"
echo "================================"

# Configuration
APP_NAME="Postman CS-CLI"
BUNDLE_ID="com.postman.cs-cli"
VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
SIGNING_IDENTITY="Developer ID Application: Jared Boynton (RGSZTAM229)"
INSTALLER_IDENTITY="Developer ID Installer: Jared Boynton (RGSZTAM229)"

# Step 1: Build and compress binary with UPX
echo ""
echo "Step 1: Building and compressing binary..."

# Build optimized binary
RUSTFLAGS='--cfg reqwest_unstable -C target-cpu=native -C opt-level=3' \
    cargo build --release

# Check for UPX
if command -v upx &> /dev/null; then
    echo "Compressing with UPX..."
    cp target/release/cs-cli target/release/cs-cli.backup
    upx --best --lzma --force-macos target/release/cs-cli || {
        echo "⚠️  UPX compression failed. Using uncompressed binary."
        cp target/release/cs-cli.backup target/release/cs-cli
    }
    echo "✓ Binary size: $(du -h target/release/cs-cli | cut -f1)"
else
    echo "⚠️  UPX not found. Install with: brew install upx"
    echo "   Continuing without compression..."
fi

# Step 2: Create .app bundle
echo ""
echo "Step 2: Creating ${APP_NAME}.app bundle..."

# Clean and create bundle structure
rm -rf "${APP_NAME}.app"
mkdir -p "${APP_NAME}.app/Contents/MacOS"
mkdir -p "${APP_NAME}.app/Contents/Resources"

# Copy binary
cp target/release/cs-cli "${APP_NAME}.app/Contents/MacOS/cs-cli"
chmod +x "${APP_NAME}.app/Contents/MacOS/cs-cli"

# Create Info.plist
cat > "${APP_NAME}.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>cs-cli</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>Postman CS-CLI</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
</dict>
</plist>
EOF

# Copy icon if it exists
if [ -f "AppIcon.icns" ]; then
    cp AppIcon.icns "${APP_NAME}.app/Contents/Resources/"
    echo "✓ Added app icon"
else
    echo "⚠️  AppIcon.icns not found. App will use default icon."
fi

# Sign the app bundle
if security find-identity -v -p codesigning | grep -q "$SIGNING_IDENTITY"; then
    echo "Signing ${APP_NAME}.app..."
    codesign --force --deep --options runtime --timestamp \
        --sign "$SIGNING_IDENTITY" \
        "${APP_NAME}.app"
    echo "✓ Signed ${APP_NAME}.app"
else
    echo "⚠️  Signing identity not found. App will trigger Gatekeeper warnings."
fi

# Step 3: Create installer package structure
echo ""
echo "Step 3: Creating PKG installer..."

# Clean and create package directories
rm -rf pkg-build
mkdir -p pkg-build/root/Applications
mkdir -p pkg-build/scripts

# Copy app to package root (rename to standard CS-CLI.app for simplicity)
cp -R "${APP_NAME}.app" "pkg-build/root/Applications/CS-CLI.app"

# Create postinstall script (runs after installation)
cat > pkg-build/scripts/postinstall << 'POSTINSTALL'
#!/bin/bash

# CS-CLI Post-Installation Script
# Sets up command-line access without requiring sudo

# Get the current user (installer runs as root, but we need the actual user)
CURRENT_USER=$(stat -f "%Su" /dev/console)
USER_HOME=$(eval echo ~$CURRENT_USER)

# Create user bin directory
sudo -u "$CURRENT_USER" mkdir -p "$USER_HOME/.local/bin"

# Create wrapper script (not symlink, to avoid permission issues)
cat > "$USER_HOME/.local/bin/cs-cli" << 'EOF'
#!/bin/bash
exec "/Applications/CS-CLI.app/Contents/MacOS/cs-cli" "$@"
EOF

# Set ownership and permissions
chown "$CURRENT_USER:staff" "$USER_HOME/.local/bin/cs-cli"
chmod +x "$USER_HOME/.local/bin/cs-cli"

# Function to add PATH to a shell config file
add_to_path() {
    local config_file="$1"
    if [ -f "$config_file" ]; then
        if ! grep -q "/.local/bin" "$config_file"; then
            echo '' >> "$config_file"
            echo '# Added by CS-CLI installer' >> "$config_file"
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$config_file"
        fi
    fi
}

# Add to all possible shell configs
sudo -u "$CURRENT_USER" bash -c "$(declare -f add_to_path); add_to_path '$USER_HOME/.zshrc'"
sudo -u "$CURRENT_USER" bash -c "$(declare -f add_to_path); add_to_path '$USER_HOME/.bash_profile'"
sudo -u "$CURRENT_USER" bash -c "$(declare -f add_to_path); add_to_path '$USER_HOME/.bashrc'"

exit 0
POSTINSTALL

chmod +x pkg-build/scripts/postinstall

# Step 4: Build the package
echo ""
echo "Step 4: Building installer package..."

# Build component package
pkgbuild --root pkg-build/root \
    --scripts pkg-build/scripts \
    --identifier "${BUNDLE_ID}" \
    --version "${VERSION}" \
    --install-location "/" \
    "cs-cli-component.pkg"

# Create distribution XML
cat > distribution.xml << EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>Postman CS-CLI ${VERSION}</title>
    <organization>com.postman</organization>
    <domains enable_anywhere="true"/>
    <allow-external-scripts/>
    <options customize="never" require-scripts="false" hostArchitectures="x86_64,arm64"/>
    <welcome>Welcome to CS-CLI!

This installer will:
• Install CS-CLI.app to Applications
• Set up the 'cs-cli' Terminal command
• Configure your PATH automatically

No administrator password required!</welcome>
    <license file="LICENSE.txt"/>
    <pkg-ref id="${BUNDLE_ID}">
        <bundle-version/>
    </pkg-ref>
    <choices-outline>
        <line choice="default">
            <line choice="${BUNDLE_ID}"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="${BUNDLE_ID}" visible="false">
        <pkg-ref id="${BUNDLE_ID}"/>
    </choice>
    <pkg-ref id="${BUNDLE_ID}" version="${VERSION}" onConclusion="none">cs-cli-component.pkg</pkg-ref>
</installer-gui-script>
EOF

# Create a simple LICENSE.txt if it doesn't exist
if [ ! -f "LICENSE.txt" ]; then
    cat > LICENSE.txt << 'EOF'
CS-CLI is provided as-is for use by authorized personnel only.

By installing this software, you agree to use it in compliance with your organization's policies.
EOF
fi

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    PKG_ARCH="arm64"
elif [ "$ARCH" = "x86_64" ]; then
    PKG_ARCH="x86_64"
else
    PKG_ARCH="universal"
fi

# Build final product package
PKG_NAME="cs-cli-${PKG_ARCH}-${VERSION}.pkg"
productbuild --distribution distribution.xml \
    --resources . \
    --package-path . \
    "$PKG_NAME"

# Sign the installer package
echo "Signing installer package..."
productsign --sign "$INSTALLER_IDENTITY" \
    "$PKG_NAME" \
    "${PKG_NAME}.signed" || {
    echo "⚠️  Failed to sign installer package"
}

if [ -f "${PKG_NAME}.signed" ]; then
    mv "${PKG_NAME}.signed" "$PKG_NAME"
    echo "✓ Signed installer package"
fi

# Clean up
rm -rf pkg-build
rm -f cs-cli-component.pkg
rm -f distribution.xml

# Final summary
echo ""
echo "================================"
echo "✅ Build Complete!"
echo "================================"
echo ""
echo "Created: $PKG_NAME ($(du -h "$PKG_NAME" | cut -f1))"
echo ""
echo "This installer will:"
echo "  • Install CS-CLI.app to /Applications"
echo "  • Create cs-cli command in ~/.local/bin"
echo "  • Add ~/.local/bin to PATH in shell configs"
echo "  • No sudo required!"
echo ""
echo "Users just double-click the PKG to install everything!"