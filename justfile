#!/usr/bin/env just --justfile
# CS-CLI Build Commands - Simplified for AI assistants and developers
# Run `just` to see all available commands

# Load .env file
set dotenv-load := true

# Default target architecture for macOS
target := "aarch64-apple-darwin"

# Show all available commands
default:
    @just --list

# Run development version (with automatic code signing)
# Without args: launches interactive TUI
# With args: runs non-interactive mode (e.g., "just run Fiserv 30 both")
run *args: sign-debug
    cargo run --target {{target}} -- {{args}}

# Run without signing (faster, but auth features won't work)
run-unsigned *args:
    cargo run --target {{target}} -- {{args}}

# Build and sign debug binary for development
sign-debug:
    ./build.sh --sign-debug

# Build release version
build:
    ./build.sh

# Build release with PKG installer
release:
    ./build.sh --pkg

# Build and install to ~/.local/bin
install:
    ./build.sh --install

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run specific test
test-one name:
    cargo test {{name}} -- --nocapture

# Run GitHub OAuth tests
test-oauth:
    cargo test github_oauth

# Run integration tests with real API (requires browser session)
test-integration:
    USE_REAL_API=true cargo test -- --ignored --nocapture

# Run unit tests only
test-unit:
    cargo test --lib

# =============================================================================
# ADVANCED TESTING OPTIONS
# =============================================================================

# Compile but don't run tests
test-no-run:
    cargo test --no-run

# Run all tests regardless of failure
test-no-fail-fast:
    cargo test --no-fail-fast

# Quiet output (one character per test)
test-quiet:
    cargo test -q

# Verbose output levels (1-3)
test-verbose-level level="1":
    cargo test -v{{level}}

# Test with specific features
test-features features:
    cargo test --features {{features}}

# Test with all features enabled
test-all-features:
    cargo test --all-features

# Test in release mode
test-release:
    cargo test --release

# Test all packages in workspace
test-workspace:
    cargo test --workspace

# Test without network access
test-offline:
    cargo test --offline

# Test all targets (excludes doctests)
test-all-targets:
    cargo test --all-targets

# Test only library documentation
test-doc:
    cargo test --doc

# Number of test threads
test-threads threads:
    cargo test -- --test-threads {{threads}}

# Skip tests matching pattern
test-skip pattern:
    cargo test -- --skip {{pattern}}

# Exact match for test names
test-exact name:
    cargo test {{name}} -- --exact

# Include ignored tests
test-include-ignored:
    cargo test -- --include-ignored

# Run only ignored tests
test-ignored-only:
    cargo test -- --ignored

# Shuffle test execution order
test-shuffle:
    cargo test -- --shuffle

# =============================================================================
# FLEXIBLE TESTING INTERFACE
# =============================================================================

# Run cargo test with custom arguments (e.g., "just test-args '--release --features slack'")
test-args *args:
    cargo test {{args}}

# Run cargo test with custom test binary arguments (e.g., "just test-binary-args '--test-threads 4 --skip integration'")
test-binary-args *args:
    cargo test -- {{args}}

# Combined: custom cargo args + test binary args (e.g., "just test-combined '--features slack' '--test-threads 4'")
test-combined cargo_args test_args:
    cargo test {{cargo_args}} -- {{test_args}}

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Check formatting without applying
fmt-check:
    cargo fmt -- --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Clean build artifacts
clean:
    cargo clean
    rm -rf dist/

# Full clean including .env credentials (careful!)
clean-all: clean
    rm -f .env

# Launch interactive TUI mode for development
dev:
    just run

# Quick non-interactive test with a specific customer
dev-quick customer="Fiserv" days="7":
    just run {{customer}} {{days}} both

# Show current git status
status:
    git status

# Create a new git commit (runs tests first)
commit message: test
    git add -A
    git commit -m "{{message}}"

# View recent Gong transcripts for a customer
view customer="Fiserv":
    ls -la ~/Desktop/ct_{{customer}}/

# Open latest transcript for a customer
open-latest customer="Fiserv":
    open ~/Desktop/ct_{{customer}}/$(ls -t ~/Desktop/ct_{{customer}}/ | head -1)

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

# Watch for file changes and rebuild
watch:
    cargo watch -x check -x test -x run

# Run with debug logging
debug *args:
    RUST_LOG=cs_cli=debug just run {{args}}

# Run with trace logging (very verbose)
trace *args:
    RUST_LOG=cs_cli=trace just run {{args}}

# Quick test after code changes
quick: fmt check test-unit

# Full test suite before committing
precommit: fmt-check lint test

# Update dependencies
update:
    cargo update

# Build documentation
docs:
    cargo doc --open

# Show binary size
size:
    @echo "Binary sizes:"
    @ls -lh target/*/release/cs-cli 2>/dev/null || echo "No release builds found"
    @ls -lh target/*/debug/cs-cli 2>/dev/null || echo "No debug builds found"

# Regression test suite
test-regression:
    ./tests/run_tests.sh

# Help for common issues
doctor:
    @echo "Common fixes:"
    @echo ""
    @echo "Authentication issues:"
    @echo "  - Make sure you're logged into Gong in Safari/Chrome"
    @echo "  - Run 'just sign-debug' to sign the debug binary"
    @echo ""
    @echo "GitHub OAuth issues:"
    @echo "  - Check GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET in .env"
    @echo "  - Run 'just check-env' to verify settings"
    @echo ""
    @echo "Build issues:"
    @echo "  - Run 'just clean' to clean build artifacts"
    @echo "  - Run 'just setup' to reinstall dependencies"