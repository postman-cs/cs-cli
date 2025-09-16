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

if [ -f ".env" ]; then
    source .env
fi

# --- Configuration ---
BINARY_NAME="cs-cli"
APP_NAME="Postman CS-CLI"
BUNDLE_ID="com.postman.cs-cli"
VERSION=$(awk -F ' = ' '/^version/ { gsub(/"/, "", $2); print $2 }' Cargo.toml)

# Target definitions
TARGETS_MACOS="aarch64-apple-darwin x86_64-apple-darwin"
TARGETS_ALL="$TARGETS_MACOS"

# --- Script State ---
SELECTED_TARGETS=""
DO_PKG_BUILD=false

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
    echo "  --targets TARGET[,TARGET...]  Build specific targets (default: all)"
    echo "  --all                         Build all macOS targets (default)"
    echo "  --pkg                         Build PKG installer for macOS"
    echo "  --help                        Show this help"
    echo ""
    echo "Available targets: $TARGETS_ALL"
    exit 1
}

# --- Core Logic Functions ---

parse_args() {
    SELECTED_TARGETS="$TARGETS_ALL" # Default to all targets

    if [[ $# -eq 0 ]]; then
        return
    fi

    local requested_targets=""
    while [[ $# -gt 0 ]]; do
        case $1 in
            --targets)
                requested_targets+="$(echo "$2" | tr ',' ' ') "
                shift 2
                ;;
            --all)
                requested_targets+=" $TARGETS_ALL "
                shift
                ;;
            --pkg)
                DO_PKG_BUILD=true
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

    # Only update SELECTED_TARGETS if targets were explicitly requested
    if [ -n "$requested_targets" ]; then
        # Deduplicate and trim whitespace
        SELECTED_TARGETS=$(echo "$requested_targets" | tr ' ' '\n' | sort -u | tr '\n' ' ' | sed 's/ $//')
    fi
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

create_universal_macos() {
    if [[ ! " $SELECTED_TARGETS " =~ " x86_64-apple-darwin " ]] || [[ ! " $SELECTED_TARGETS " =~ " aarch64-apple-darwin " ]]; then
        log WARN "Not all macOS targets were built. Skipping universal binary creation."
        return
    fi
    
    log STEP "Creating universal macOS binary..."
    
    local intel_bin="dist/binaries/${BINARY_NAME}-x86_64-apple-darwin"
    local arm_bin="dist/binaries/${BINARY_NAME}-aarch64-apple-darwin"
    local universal_bin="dist/binaries/${BINARY_NAME}-macos-universal"

    if [ ! -f "$intel_bin" ] || [ ! -f "$arm_bin" ]; then
        log ERROR "Missing required binaries for universal package. Cannot proceed."
    fi

    lipo -create -output "$universal_bin" "$intel_bin" "$arm_bin"
    log SUCCESS "Created universal binary at $universal_bin"
}

sign_file() {
    local file_path="$1"
    log INFO "Signing $file_path..."

    # This pulls from the environment. Not hardcoded.
    if [ -z "${SIGNING_IDENTITY:-}" ]; then
        log WARN "SIGNING_IDENTITY environment variable not set. Skipping code signing."
        return
    fi

    if ! security find-identity -v -p codesigning | grep -q "$SIGNING_IDENTITY"; then
        log WARN "Signing identity '$SIGNING_IDENTITY' not found in keychain. Skipping."
        return
    fi

    codesign --force --options runtime --timestamp \
        --sign "$SIGNING_IDENTITY" "$file_path"
    
    codesign --verify --verbose "$file_path"
    log SUCCESS "Successfully signed $file_path"
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
    local universal_bin="dist/binaries/${BINARY_NAME}-macos-universal"

    if [ ! -d "$app_path" ]; then
        log WARN "App bundle not found. Skipping PKG build."
        return
    fi

    if [ ! -f "$universal_bin" ]; then
        log WARN "Universal binary not found. Skipping PKG build."
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
    cp "$universal_bin" "$pkg_build_dir/root/usr/local/bin/${BINARY_NAME}"
    chmod +x "$pkg_build_dir/root/usr/local/bin/${BINARY_NAME}"

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

    # 9. Sign the final package
    if [ -n "${INSTALLER_IDENTITY:-}" ]; then
        if security find-identity -v -p codesigning | grep -q "$INSTALLER_IDENTITY"; then
            log INFO "Signing installer package..."
            local signed_pkg_name="${final_pkg_name}.signed"
            productsign --sign "$INSTALLER_IDENTITY" "$final_pkg_name" "$signed_pkg_name"
            mv "$signed_pkg_name" "$final_pkg_name"
            log SUCCESS "Installer package signed."
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

    log STEP "Starting build for CS-CLI v${VERSION}"
    log INFO "Targets: $SELECTED_TARGETS"

    rm -rf dist
    mkdir -p dist/binaries dist/release
    xattr -w com.apple.metadata:com_apple_backup_excludeItem com.apple.backupd dist 2>/dev/null || true
    cargo clean

    for target in $SELECTED_TARGETS; do
        build_target "$target"
    done

    # Always create universal binary and sign it
    create_universal_macos
    if [ -f "dist/binaries/${BINARY_NAME}-macos-universal" ]; then
        sign_file "dist/binaries/${BINARY_NAME}-macos-universal"
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