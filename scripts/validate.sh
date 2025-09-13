#!/bin/bash
# Step 6.3 Validation Runner Script
# 
# This script runs the comprehensive validation suite to compare the Rust
# implementation against the Python version.

set -e

echo "🚀 CS-CLI Step 6.3 Validation Suite"
echo "===================================="
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Run this script from the cs-cli-rust directory"
    exit 1
fi

# Check dependencies
echo "🔍 Checking dependencies..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust."
    exit 1
fi

# Build the project first
echo "🔨 Building project..."
cargo check --quiet

# Run the validation suite
echo "🧪 Running validation tests..."
echo

# Run with increased verbosity for better output
RUST_LOG=info cargo test --test run_validation -- --nocapture

# Check if validation report was generated
if [ -f "validation_report_step_6_3.md" ]; then
    echo
    echo "📊 Validation report generated: validation_report_step_6_3.md"
    echo
    echo "📋 Report Summary:"
    head -20 validation_report_step_6_3.md
    echo
    echo "📄 Full report available in validation_report_step_6_3.md"
else
    echo "⚠️  Warning: Validation report not generated"
fi

# Run performance benchmarks separately (optional)
echo
read -p "🏃 Run performance benchmarks? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "⚡ Running performance benchmarks..."
    cargo bench --bench validation_benchmarking
fi

echo
echo "✅ Validation suite completed!"
echo "📝 Review the validation report for detailed results"
