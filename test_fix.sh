#!/bin/bash

echo "Testing GitHub gist storage initialization fix..."

# Run the specific test
cargo test github_gist_storage::tests::test_gist_storage_initialization --lib

echo "Test completed."