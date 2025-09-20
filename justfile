#!/usr/bin/env just --justfile
# CS-CLI Build Commands
# Run `just` to see all available commands

# Load .env file
set dotenv-load := true

# Default target architecture for macOS
target := "aarch64-apple-darwin"

# Show all available commands
default:
    @just --list

# =============================================================================
# BUILD COMMANDS
# =============================================================================

# Build and sign debug binary for development
sign-debug:
    ./build.sh --sign-debug

# Build release version (incremental build + signing)
build:
    cargo build --release --target {{target}}
    @if [ -n "${SIGNING_IDENTITY}" ]; then \
        echo "Signing release binary..."; \
        codesign --force --options runtime --timestamp \
            --identifier "com.postman.cs-cli" \
            --sign "${SIGNING_IDENTITY}" \
            "target/{{target}}/release/cs-cli" && \
        echo "Binary signed successfully"; \
    else \
        echo "Warning: SIGNING_IDENTITY not set, binary will not be signed"; \
    fi

# Full build with clean and complete packaging (uses build.sh)
fullbuild:
    ./build.sh

# Build release with PKG installer
release:
    ./build.sh --pkg

# Build and install to ~/.local/bin
install:
    ./build.sh --install

# Check code without building
check:
    cargo check

# Clean build artifacts
clean:
    cargo clean
    rm -rf dist/

# Build documentation
docs:
    cargo doc --open

# Show binary size
size:
    @echo "Binary sizes:"
    @ls -lh target/*/release/cs-cli 2>/dev/null || echo "No release builds found"
    @ls -lh target/*/debug/cs-cli 2>/dev/null || echo "No debug builds found"

# =============================================================================
# TEST COMMANDS
# =============================================================================

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run unit tests only
test-unit:
    cargo test --lib

# Run integration tests with real API (requires browser session)
test-integration:
    USE_REAL_API=true cargo test -- --ignored --nocapture

# Run GitHub OAuth tests
test-oauth:
    cargo test github_oauth

# Regression test suite
test-regression:
    ./tests/run_tests.sh

# Run cargo test with custom arguments
test-args *args:
    cargo test {{args}}

# =============================================================================
# DEVELOPMENT COMMANDS
# =============================================================================

# Run development version (with automatic code signing)
run *args:
    cargo build --target {{target}}
    @if [ -n "${SIGNING_IDENTITY}" ]; then \
        echo "Signing debug binary..."; \
        codesign --force --options runtime --timestamp \
            --identifier "com.postman.cs-cli" \
            --sign "${SIGNING_IDENTITY}" \
            "target/{{target}}/debug/cs-cli" && \
        echo "Debug binary signed successfully"; \
    else \
        echo "Warning: SIGNING_IDENTITY not set, binary will not be signed"; \
    fi
    cargo run --target {{target}} -- {{args}}

# Run without signing (faster, but auth features won't work)
run-unsigned *args:
    cargo run --target {{target}} -- {{args}}

# Launch interactive TUI mode for development with debug logging
dev:
    RUST_LOG=cs_cli=debug just run

# Run with debug logging
debug *args:
    RUST_LOG=cs_cli=debug just run {{args}}

# Watch for file changes and rebuild
watch:
    cargo watch -x check -x test -x run

# =============================================================================
# SETUP & VERIFICATION
# =============================================================================

# Setup development environment
setup:
    @echo "Setting up development environment..."
    @test -f .env || cp .env.example .env
    @echo "✓ .env file ready (edit it with your credentials)"
    rustup target add aarch64-apple-darwin
    @echo "✓ Apple Silicon target installed"
    @echo ""
    @echo "Next steps:"
    @echo "1. Edit .env with your signing identity and GitHub OAuth credentials"
    @echo "2. Run 'just check-env' to verify setup"
    @echo "3. Run 'just run' to start the application"

# Verify environment setup
check-env:
    @echo "Checking environment setup..."
    @echo "SIGNING_IDENTITY: ${SIGNING_IDENTITY:-Not set}"
    @echo "GITHUB_CLIENT_ID: ${GITHUB_CLIENT_ID:-Not set}"
    @echo "GITHUB_CLIENT_SECRET: ${GITHUB_CLIENT_SECRET:+Set}"
    @echo "RUSTFLAGS: ${RUSTFLAGS:-Not set}"
    @which cargo > /dev/null && echo "✓ cargo found" || echo "✗ cargo not found"
    @which rustc > /dev/null && echo "✓ rustc found" || echo "✗ rustc not found"
    @test -f .env && echo "✓ .env file exists" || echo "✗ .env file not found"