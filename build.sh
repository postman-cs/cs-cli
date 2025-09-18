#!/bin/bash
#
# Production Build & Release Script for Postman CS-CLI
#
# This script handles compilation, code signing, and packaging
# for macOS targets (both Intel and Apple Silicon).
#
# Prerequisites:
#   - Rust and rustup installed.
#   - For macOS signing and notarization, set the following in .env file or as environment variables:
#     SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
#     INSTALLER_IDENTITY="Developer ID Installer: Your Name (TEAMID)"

set -euo pipefail

# Always source .env file for configuration
if [ -f ".env" ]; then
    echo "Loading environment from .env file..."
    source .env

    # Verify critical environment variables are loaded
    if [ -n "${SIGNING_IDENTITY:-}" ]; then
        echo "✓ Signing identity loaded: ${SIGNING_IDENTITY}"
    else
        echo "⚠ No signing identity found in .env (binaries will not be signed)"
    fi
else
    echo "⚠ No .env file found. Using default configuration."
    echo "  Create .env from .env.example for custom settings."
fi

# --- Configuration ---
BINARY_NAME="cs-cli"
APP_NAME="Postman CS-CLI"
BUNDLE_ID="com.postman.cs-cli"
VERSION=$(awk -F ' = ' '/^version/ { gsub(/"/, "", $2); print $2 }' Cargo.toml)

# Target definitions - Apple Silicon only
TARGETS_DEFAULT="aarch64-apple-darwin"
TARGETS_ALL="aarch64-apple-darwin"

# --- Script State ---
SELECTED_TARGETS=""
DO_PKG_BUILD=false
DO_SIGN_DEBUG=false

# --- Helper Functions ---
log() {
    local color_reset='\033[0m'
    local color_red='\033[0;31m'
    local color_green='\033[0;32m'
    local color_yellow='\033[0;33m'
    local color_blue='\033[0;34m'
    local level="$1"
    shift
    local msg="$*"

    case "$level" in
        INFO) echo -e "${color_blue}==>${color_reset} ${msg}" ;;
        SUCCESS) echo -e "${color_green} ✓ ${color_reset} ${msg}" ;;
        WARN) echo -e "${color_yellow} ⚠ ${color_reset} ${msg}" ;;
        ERROR) echo -e "${color_red} ✗ ${color_reset} ${msg}" >&2; exit 1 ;;
        STEP) echo -e "\n${color_blue}==>${color_reset} \033[1m${msg}\033[0m" ;;
        *) echo "$msg" ;;
    esac
}

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --pkg                         Build PKG installer for macOS"
    echo "  --sign-debug                  Sign debug binary for 'cargo run' (development)"
    echo "  --help                        Show this help"
    echo ""
    echo "Default target: $TARGETS_DEFAULT (Apple Silicon macOS)"
    echo "Note: CS-CLI uses the system's Chrome browser for guided authentication"
    exit 1
}

# --- Core Logic Functions ---

# Function removed - no longer bundling lightpanda
# CS-CLI now uses the system's installed Chrome browser for guided authentication

parse_args() {
    SELECTED_TARGETS="$TARGETS_DEFAULT" # Default to Apple Silicon

    if [[ $# -eq 0 ]]; then
        return
    fi

    while [[ $# -gt 0 ]]; do
        case $1 in
            --pkg)
                DO_PKG_BUILD=true
                shift
                ;;
            --sign-debug)
                DO_SIGN_DEBUG=true
                shift
                ;;
            --help)
                usage
                ;;
            *)
                log ERROR "Unknown option: $1"
                ;;
        esac
    done
}

build_target() {
    local target="$1"
    log INFO "Building for target: $target"

    if ! rustup target list --installed | grep -q "$target"; then
        log WARN "Target '$target' not installed. Adding it now..."
        rustup target add "$target"
    fi

    # Define target-specific build environment and flags
    local rust_flags="--cfg reqwest_unstable -C opt-level=3"

    case "$target" in
        aarch64-apple-darwin)
            rust_flags+=" -C target-feature=+neon,+aes,+sha2"
            ;;
        x86_64-apple-darwin)
            rust_flags+=" -C target-feature=+sse4.2,+avx2"
            ;;
    esac

    # Execute the build
    RUSTFLAGS="$rust_flags" cargo build --release --target "$target"
    
    # Copy artifact to dist directory
    local output_dir="dist/binaries"
    local output_name="${BINARY_NAME}-${target}"
    local source_path="target/$target/release/$BINARY_NAME"

    if [ -f "$source_path" ]; then
        cp "$source_path" "$output_dir/$output_name"
        log SUCCESS "Built artifact: $output_dir/$output_name"
    else
        log ERROR "Build failed for target $target. Artifact not found at $source_path."
    fi
}

create_macos_binary() {
    log STEP "Preparing macOS binary for distribution..."
    
    local arm_bin="dist/binaries/${BINARY_NAME}-aarch64-apple-darwin"
    local macos_bin="dist/binaries/${BINARY_NAME}-macos"

    if [ ! -f "$arm_bin" ]; then
        log ERROR "Apple Silicon binary not found. Cannot proceed."
    fi

    # Copy the Apple Silicon binary as the main macOS binary
    cp "$arm_bin" "$macos_bin"
    log SUCCESS "Created macOS binary at $macos_bin"
}

sign_file() {
    local file_path="$1"
    log INFO "Signing $file_path..."

    # This pulls from the environment (loaded from .env)
    if [ -z "${SIGNING_IDENTITY:-}" ]; then
        log WARN "SIGNING_IDENTITY not configured in .env file. Skipping code signing."
        log INFO "To enable signing, add SIGNING_IDENTITY=\"Developer ID Application: Your Name (TEAMID)\" to .env"
        return
    fi

    if ! security find-identity -v -p codesigning | grep -q "$SIGNING_IDENTITY"; then
        log WARN "Signing identity '$SIGNING_IDENTITY' not found in keychain. Skipping."
        log INFO "Run 'security find-identity -v -p codesigning' to see available identities."
        return
    fi

    codesign --force --options runtime --timestamp \
        --identifier "$BUNDLE_ID" \
        --sign "$SIGNING_IDENTITY" "$file_path"

    codesign --verify --verbose "$file_path"
    log SUCCESS "Successfully signed $file_path"
}

sign_debug_binary() {
    log STEP "Signing debug binary for development use..."

    # Always use Apple Silicon target for consistency
    local target_dir="target/aarch64-apple-darwin/debug"
    local debug_binary="$target_dir/$BINARY_NAME"

    # Check for default debug directory first (cargo build without --target)
    if [ -f "target/debug/$BINARY_NAME" ]; then
        debug_binary="target/debug/$BINARY_NAME"
    fi

    if [ ! -f "$debug_binary" ]; then
        log WARN "Debug binary not found at $debug_binary"
        log INFO "Building debug binary for Apple Silicon..."
        
        # Use same RUSTFLAGS as release build for consistency
        RUSTFLAGS='--cfg reqwest_unstable -C target-feature=+neon,+aes,+sha2' \
            cargo build --target aarch64-apple-darwin

        # Re-check after build
        if [ ! -f "$debug_binary" ]; then
            log ERROR "Failed to find or build debug binary"
        fi
    fi

    # Sign the debug binary with bundle ID
    if [ -z "${SIGNING_IDENTITY:-}" ]; then
        log WARN "SIGNING_IDENTITY not configured in .env file. Skipping code signing."
        log INFO "To enable signing, add SIGNING_IDENTITY=\"Developer ID Application: Your Name (TEAMID)\" to .env"
        return
    fi

    if ! security find-identity -v -p codesigning | grep -q "$SIGNING_IDENTITY"; then
        log WARN "Signing identity '$SIGNING_IDENTITY' not found in keychain. Skipping."
        return
    fi

    codesign --force --options runtime --timestamp \
        --identifier "$BUNDLE_ID" \
        --sign "$SIGNING_IDENTITY" "$debug_binary"

    codesign --verify --verbose "$debug_binary"
    log SUCCESS "Successfully signed $debug_binary with bundle ID: $BUNDLE_ID"

    log SUCCESS "Debug binary signed and ready for development use"
    log INFO "You can now use: cargo run --target aarch64-apple-darwin -- [args]"
}

create_app_bundle() {
    log STEP "Creating macOS app bundle (launcher)..."

    local app_name="${APP_NAME}.app"
    local app_path="dist/$app_name"

    # Create app bundle structure
    rm -rf "$app_path"
    mkdir -p "$app_path/Contents/MacOS"
    mkdir -p "$app_path/Contents/Resources"

    # Create launcher script that opens Terminal and runs cs-cli
    cat > "$app_path/Contents/MacOS/${BINARY_NAME}-launcher" << 'LAUNCHER'
#!/bin/bash
# Launch Terminal and run cs-cli from /usr/local/bin

osascript <<EOF
tell application "Terminal"
    activate
    do script "clear && echo 'Starting CS Deep Research CLI...' && echo '' && /usr/local/bin/cs-cli; exit"
end tell
EOF
LAUNCHER
    chmod +x "$app_path/Contents/MacOS/${BINARY_NAME}-launcher"

    # Copy icon
    if [ -f "AppIcon.icns" ]; then
        cp "AppIcon.icns" "$app_path/Contents/Resources/AppIcon.icns"
    else
        log WARN "AppIcon.icns not found. App will use default icon."
    fi

    # Create Info.plist
    cat > "$app_path/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${BINARY_NAME}-launcher</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>LSUIElement</key>
    <false/>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
EOF

    # Sign the app bundle
    if [ -n "${SIGNING_IDENTITY:-}" ]; then
        if security find-identity -v -p codesigning | grep -q "$SIGNING_IDENTITY"; then
            log INFO "Signing app bundle..."
            codesign --force --options runtime --timestamp \
                --sign "$SIGNING_IDENTITY" \
                --deep "$app_path"
            codesign --verify --verbose "$app_path"
            log SUCCESS "App bundle signed."
        else
            log WARN "Signing identity not found. App bundle will be unsigned."
        fi
    fi

    log SUCCESS "Created app bundle: $app_path"
}

package_releases() {
    log STEP "Creating release ZIP packages..."
    mkdir -p "dist/release"

    # macOS App ZIP
    local app_name="${APP_NAME}.app"
    local app_path="dist/$app_name"
    if [ -d "$app_path" ]; then
        local zip_name="dist/release/${BINARY_NAME}-macos-v${VERSION}.zip"
        (cd dist && zip -r "../$zip_name" "$app_name")
        log SUCCESS "Created macOS app release package: $zip_name"
    fi
}

build_macos_pkg() {
    log STEP "Building macOS PKG Installer..."

    # Check for required files
    local app_name="${APP_NAME}.app"
    local app_path="dist/$app_name"
    local macos_bin="dist/binaries/${BINARY_NAME}-macos"

    if [ ! -d "$app_path" ]; then
        log WARN "App bundle not found. Skipping PKG build."
        return
    fi

    if [ ! -f "$macos_bin" ]; then
        log WARN "macOS binary not found. Skipping PKG build."
        return
    fi

    if [ -z "${INSTALLER_IDENTITY:-}" ]; then
        log WARN "INSTALLER_IDENTITY environment variable not set. PKG will not be signed."
    fi

    local pkg_build_dir="dist/pkg-build"
    local component_pkg="dist/cs-cli-component.pkg"
    local final_pkg_name="dist/release/${BINARY_NAME}-macos-v${VERSION}.pkg"

    # 1. Setup build directories
    rm -rf "$pkg_build_dir"
    mkdir -p "$pkg_build_dir/root/Applications"
    mkdir -p "$pkg_build_dir/root/usr/local/bin"

    # 2. Copy the app bundle to /Applications
    cp -R "$app_path" "$pkg_build_dir/root/Applications/"

    # 3. Copy the binary to /usr/local/bin
    cp "$macos_bin" "$pkg_build_dir/root/usr/local/bin/${BINARY_NAME}"
    chmod +x "$pkg_build_dir/root/usr/local/bin/${BINARY_NAME}"

    # 3.5. Copy the Swift keychain helper if it exists
    # Find the helper in the Apple Silicon target directory
    helper_path=""
    potential_path="target/aarch64-apple-darwin/release/build"
    if [ -d "$potential_path" ]; then
        found_helper=$(find "$potential_path" -name "keychain_helper" -type f 2>/dev/null | head -1)
        if [ -n "$found_helper" ]; then
            helper_path="$found_helper"
        fi
    fi

    if [ -n "$helper_path" ]; then
        log INFO "Copying Swift keychain helper..."
        cp "$helper_path" "$pkg_build_dir/root/usr/local/bin/keychain_helper"
        chmod +x "$pkg_build_dir/root/usr/local/bin/keychain_helper"

        # Sign the helper if we have signing identity
        if [ -n "$SIGNING_IDENTITY" ]; then
            codesign --force --sign "$SIGNING_IDENTITY" \
                     --options runtime \
                     --timestamp \
                     "$pkg_build_dir/root/usr/local/bin/keychain_helper"
        fi
    else
        log INFO "Swift keychain helper not found (TouchID support has been removed)"
    fi

    # 4. Create welcome.html file
    cat > "dist/welcome.html" << 'WELCOME_HTML'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 20px; line-height: 1.6; }
        h1 { color: #333; font-size: 24px; margin-bottom: 10px; }
        h2 { color: #666; font-size: 18px; margin-top: 25px; margin-bottom: 10px; }
        p { color: #555; margin-bottom: 15px; }
        ul { color: #555; margin-bottom: 15px; }
        li { margin-bottom: 8px; }
        .highlight { background-color: #f8f9fa; padding: 15px; border-radius: 8px; margin: 15px 0; }
        .note { font-style: italic; color: #777; font-size: 14px; }
        code { background-color: #f4f4f4; padding: 2px 5px; border-radius: 3px; }
    </style>
</head>
<body>
    <h1>Welcome to CS Deep Research CLI</h1>

    <p>This tool helps Customer Success teams find critical insights hidden in customer conversations.</p>

    <h2>What you'll discover:</h2>
    <ul>
        <li>Issues your customers forgot to follow up on</li>
        <li>Problems brewing beneath the surface</li>
        <li>Exact quotes to reference in your next call</li>
        <li>Opportunities to shift from vendor to trusted advisor</li>
    </ul>

    <div class="highlight">
        <p><strong>Installation Complete!</strong> We've installed:</p>
        <ul>
            <li>The app launcher in your Applications folder</li>
            <li>The <code>cs-cli</code> command in <code>/usr/local/bin</code></li>
        </ul>
        <p>You can either click the app in Applications or type <code>cs-cli</code> in Terminal.</p>
    </div>

    <h2>Important:</h2>
    <p>Make sure you're logged into Gong in your browser before using the tool. Results will be saved to folders on your Desktop for easy access.</p>

    <p class="note">Built by your technical colleagues who believe in your success.</p>
</body>
</html>
WELCOME_HTML

    # 5. Build the component package
    pkgbuild --root "$pkg_build_dir/root" \
        --identifier "${BUNDLE_ID}" \
        --version "${VERSION}" \
        --install-location "/" \
        "$component_pkg"
    log SUCCESS "Component package built."

    # 6. Copy icon to dist for installer
    if [ -f "AppIcon.icns" ]; then
        cp AppIcon.icns "dist/AppIcon.icns"
    fi

    # 7. Create distribution file
    local distribution_file="dist/distribution.xml"
    cat > "$distribution_file" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>${APP_NAME} ${VERSION}</title>
    <welcome file="welcome.html"/>
    <background file="AppIcon.icns" mime-type="image/icns" alignment="center" scaling="none"/>
    <options customize="never" require-scripts="false" hostArchitectures="x86_64,arm64"/>
    <pkg-ref id="${BUNDLE_ID}" version="${VERSION}" onConclusion="none">cs-cli-component.pkg</pkg-ref>
</installer-gui-script>
EOF

    # 8. Build the final product package with icon
    productbuild --distribution "$distribution_file" \
        --package-path "dist" \
        --resources "dist" \
        "$final_pkg_name"
    log SUCCESS "Product package built: $final_pkg_name"

    # Create version-agnostic copy for consistent latest download URLs
    local generic_pkg_name="dist/release/${BINARY_NAME}-macos.pkg"
    cp "$final_pkg_name" "$generic_pkg_name"
    log SUCCESS "Created generic package: $generic_pkg_name"

    # 9. Sign the final package
    if [ -n "${INSTALLER_IDENTITY:-}" ]; then
        if security find-identity -v | grep -q "$INSTALLER_IDENTITY"; then
            log INFO "Signing installer packages..."
            local signed_pkg_name="${final_pkg_name}.signed"
            productsign --sign "$INSTALLER_IDENTITY" "$final_pkg_name" "$signed_pkg_name"
            mv "$signed_pkg_name" "$final_pkg_name"
            
            # Also sign the generic package
            local signed_generic_name="${generic_pkg_name}.signed"
            productsign --sign "$INSTALLER_IDENTITY" "$generic_pkg_name" "$signed_generic_name"
            mv "$signed_generic_name" "$generic_pkg_name"
            log SUCCESS "Installer packages signed."
        else
            log WARN "Installer identity '$INSTALLER_IDENTITY' not found in keychain. Skipping."
        fi
    fi

    # 10. Cleanup
    rm -rf "$pkg_build_dir" "$component_pkg" "$distribution_file"
}

# --- Main Execution ---
main() {
    parse_args "$@"

    # Handle special case for signing debug binary
    if $DO_SIGN_DEBUG; then
        sign_debug_binary
        exit 0
    fi

    log STEP "Starting build for CS-CLI v${VERSION}"
    log INFO "Targets: $SELECTED_TARGETS"

    # Chrome browser detection is now handled at runtime

    rm -rf dist
    mkdir -p dist/binaries dist/release
    xattr -w com.apple.metadata:com_apple_backup_excludeItem com.apple.backupd dist 2>/dev/null || true
    cargo clean

    for target in $SELECTED_TARGETS; do
        build_target "$target"
    done

    # Create macOS distribution binary and sign it
    create_macos_binary
    if [ -f "dist/binaries/${BINARY_NAME}-macos" ]; then
        sign_file "dist/binaries/${BINARY_NAME}-macos"
    fi

    # Create the app bundle (launcher)
    create_app_bundle

    # Build PKG installer if requested
    if $DO_PKG_BUILD; then
        build_macos_pkg
    fi

    package_releases

    log STEP "Build Complete!"
    log INFO "Output directory: dist/"
    echo "Release packages:"
    ls -lh dist/release/
}

main "$@"