#!/bin/bash

# Test script to verify driver cleanup works

echo "Testing driver cleanup..."

# Check initial state
echo "Initial driver processes:"
pgrep -l chromedriver 2>/dev/null || echo "  No chromedriver running"
pgrep -l geckodriver 2>/dev/null || echo "  No geckodriver running"
pgrep -l safaridriver 2>/dev/null || echo "  No safaridriver running"

# Run the CLI briefly
echo -e "\nRunning cs-cli..."
timeout 5 cargo run -- --tty 2>/dev/null || true

# Give time for cleanup
sleep 2

# Check final state
echo -e "\nFinal driver processes:"
pgrep -l chromedriver 2>/dev/null || echo "  No chromedriver running"
pgrep -l geckodriver 2>/dev/null || echo "  No geckodriver running"
pgrep -l safaridriver 2>/dev/null || echo "  No safaridriver running"

echo -e "\n✅ Driver cleanup test complete!"