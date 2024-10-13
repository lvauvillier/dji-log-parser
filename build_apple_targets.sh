#!/bin/bash

set -e

# Function to build for a specific target
build_target() {
    target=$1
    echo "Building for $target..."
    cargo build --release --target $target
    echo "Build for $target completed."
    echo
}

# Build for Apple targets
build_target "aarch64-apple-darwin"
build_target "x86_64-apple-darwin"

# Verify the builds
echo "Verifying builds..."
ls -R target/*/release/libdji_log_parser.a || true

echo "All builds completed."