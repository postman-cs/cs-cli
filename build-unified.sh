#!/bin/bash
#
# Production Build & Release Script for Postman CS-CLI
#
# This script handles cross-platform compilation, code signing, and packaging
# for macOS, Linux, and Windows targets.
#
# Prerequisites:
#   - Rust and rustup installed.
#   - Cross-compilation toolchains installed (e.g., via Homebrew):
#     `brew install mingw-w64 x86_64-linux-musl`
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
TARGETS_LINUX="x86_64-unknown-linux-musl"
TARGETS_WINDOWS="x86_64-pc-windows-gnu"
TARGETS_ALL="$TARGETS_MACOS $TARGETS_LINUX $TARGETS_WINDOWS"

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
    echo "  --all                         Build all supported targets (default)"
    echo "  --macos                       Build macOS targets only"
    echo "  --linux                       Build Linux targets only"
    echo "  --windows                     Build Windows targets only"
    echo "  --pkg                         Build PKG installer for macOS (requires macOS build)"
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
                requested_targets+="$(echo "$2" | tr ',' ' ')"
                shift 2
                ;;
            --all)
                requested_targets+=" $TARGETS_ALL"
                shift
                ;;
            --macos)
                requested_targets+=" $TARGETS_MACOS"
                shift
                ;;
            --linux)
                requested_targets+=" $TARGETS_LINUX"
                shift
                ;;
            --windows)
                requested_targets+=" $TARGETS_WINDOWS"
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

    # Deduplicate and trim whitespace
    SELECTED_TARGETS=$(echo "$requested_targets" | tr ' ' '\n' | sort -u | tr '\n' ' ' | sed 's/ $//')
}

build_target() {
    local target="$1"
    log INFO "Building for target: $target"

    if ! rustup target list --installed | grep -q "$target"; then
        log WARN "Target '$target' not installed. Adding it now..."
        rustup target add "$target"
    fi

    # Define target-specific build environment and flags using a safe array. No eval.
    local cargo_env=()
    local rust_flags="--cfg reqwest_unstable -C opt-level=3"
    
    case "$target" in
        aarch64-apple-darwin)
            rust_flags+=" -C target-feature=+neon,+aes,+sha2"
            ;;
        x86_64-apple-darwin | x86_64-unknown-linux-musl | x86_64-pc-windows-gnu)
            rust_flags+=" -C target-feature=+sse4.2,+avx2"
            ;;
    esac

    # Handle cross-compilation toolchains. This assumes they are in the PATH.
    # No hardcoded /opt/homebrew paths.
    if [[ "$target" == "x86_64-unknown-linux-musl" ]]; then
        cargo_env+=(
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc"
            "OPENSSL_STATIC=1"
        )
    elif [[ "$target" == "x86_64-pc-windows-gnu" ]]; then
        cargo_env+=(
            "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc"
        )
    fi

    # Execute the build safely
    RUSTFLAGS="$rust_flags" env "${cargo_env[@]}" cargo build --release --target "$target"
    
    # Copy artifact to dist directory
    local output_dir="dist/binaries"
    local output_name="${BINARY_NAME}-${target}"
    local source_path="target/$target/release/$BINARY_NAME"
    
    if [[ "$target" == *"-windows-"* ]]; then
        source_path+=".exe"
        output_name+=".exe"
    fi

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

package_releases() {
    log STEP "Creating release ZIP packages..."
    mkdir -p "dist/release"

    # macOS ZIP
    local universal_bin="dist/binaries/${BINARY_NAME}-macos-universal"
    if [ -f "$universal_bin" ]; then
        local zip_name="dist/release/${BINARY_NAME}-macos-universal-v${VERSION}.zip"
        zip -j "$zip_name" "$universal_bin"
        log SUCCESS "Created macOS release package: $zip_name"
    fi

    # Linux ZIP
    local linux_bin="dist/binaries/${BINARY_NAME}-x86_64-unknown-linux-musl"
    if [ -f "$linux_bin" ]; then
        local zip_name="dist/release/${BINARY_NAME}-linux-amd64-v${VERSION}.zip"
        zip -j "$zip_name" "$linux_bin"
        log SUCCESS "Created Linux release package: $zip_name"
    fi

    # Windows ZIP
    local windows_bin="dist/binaries/${BINARY_NAME}-x86_64-pc-windows-gnu.exe"
    if [ -f "$windows_bin" ]; then
        local zip_name="dist/release/${BINARY_NAME}-windows-amd64-v${VERSION}.zip"
        zip -j "$zip_name" "$windows_bin"
        log SUCCESS "Created Windows release package: $zip_name"
    fi
}

build_macos_pkg() {
    log STEP "Building macOS PKG Installer..."

    # Use the universal binary we've already built and signed. No recompiling.
    local universal_bin="dist/binaries/${BINARY_NAME}-macos-universal"
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

    # 1. Setup build directories - put binary in temp location for postinstall to handle
    rm -rf "$pkg_build_dir"
    mkdir -p "$pkg_build_dir/root/tmp/cs-cli-installer"
    mkdir -p "$pkg_build_dir/scripts"

    # 2. Copy the binary to temp location
    cp "$universal_bin" "$pkg_build_dir/root/tmp/cs-cli-installer/${BINARY_NAME}"
    chmod +x "$pkg_build_dir/root/tmp/cs-cli-installer/${BINARY_NAME}"

    # 3. Create postinstall script for user-local installation
    cat > "$pkg_build_dir/scripts/postinstall" << 'POSTINSTALL'
#!/bin/bash
set -e

BIN_NAME="cs-cli"
USER_BIN_DIR="$HOME/.local/bin"
TEMP_BINARY="/tmp/cs-cli-installer/$BIN_NAME"

# Create user's local bin directory
mkdir -p "$USER_BIN_DIR"

# Move binary from temp to user's bin directory
cp "$TEMP_BINARY" "$USER_BIN_DIR/$BIN_NAME"
chmod +x "$USER_BIN_DIR/$BIN_NAME"

# Clean up temp files
rm -rf "/tmp/cs-cli-installer"

# Add to PATH in shell configs if not already there
for shell_config in "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.bashrc"; do
    if [ -f "$shell_config" ]; then
        if ! grep -q "/.local/bin" "$shell_config"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_config"
        fi
    fi
done

# Create shell configs if they don't exist and add PATH
if [ ! -f "$HOME/.zshrc" ]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' > "$HOME/.zshrc"
fi

echo "Successfully installed $BIN_NAME to $USER_BIN_DIR"
echo "Added $USER_BIN_DIR to PATH in shell configuration"
echo "Restart your terminal or run: source ~/.zshrc"

exit 0
POSTINSTALL
    chmod +x "$pkg_build_dir/scripts/postinstall"

    # 4. Build the component package
    pkgbuild --root "$pkg_build_dir/root" \
        --scripts "$pkg_build_dir/scripts" \
        --identifier "${BUNDLE_ID}" \
        --version "${VERSION}" \
        --install-location "/" \
        "$component_pkg"
    log SUCCESS "Component package built."

    # 5. Build the final product package
    productbuild --distribution <(cat <<EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>${APP_NAME} ${VERSION}</title>
    <options customize="never" require-scripts="false" hostArchitectures="x86_64,arm64"/>
    <pkg-ref id="${BUNDLE_ID}" version="${VERSION}" onConclusion="none">cs-cli-component.pkg</pkg-ref>
</installer-gui-script>
EOF
    ) \
    --package-path "dist" \
    "$final_pkg_name"
    log SUCCESS "Product package built: $final_pkg_name"

    # 6. Sign the final package
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

    # 7. Cleanup
    rm -rf "$pkg_build_dir" "$component_pkg"
}

# --- Main Execution ---
main() {
    parse_args "$@"

    if [ -z "$SELECTED_TARGETS" ]; then
        log INFO "No targets specified. Use --help for options."
        exit 0
    fi

    log STEP "Starting build for CS-CLI v${VERSION}"
    log INFO "Targets: $SELECTED_TARGETS"

    rm -rf dist
    mkdir -p dist/binaries dist/release
    xattr -w com.apple.metadata:com_apple_backup_excludeItem com.apple.backupd dist 2>/dev/null || true
    cargo clean

    for target in $SELECTED_TARGETS; do
        build_target "$target"
    done

    if [[ " $SELECTED_TARGETS " =~ " apple-darwin " ]]; then
        create_universal_macos
        if [ -f "dist/binaries/${BINARY_NAME}-macos-universal" ]; then
            sign_file "dist/binaries/${BINARY_NAME}-macos-universal"
        fi
        if $DO_PKG_BUILD; then
            build_macos_pkg
        fi
    fi

    package_releases

    log STEP "Build Complete!"
    log INFO "Output directory: dist/"
    echo "Release packages:"
    ls -lh dist/release/
}

main "$@"