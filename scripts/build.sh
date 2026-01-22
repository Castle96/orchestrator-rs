#!/bin/bash

# Build script for ARM Hypervisor Platform

set -e

TARGET="${1:-aarch64-unknown-linux-gnu}"
RELEASE="${2:-release}"

echo "Building ARM Hypervisor Platform for target: $TARGET"
echo "Build mode: $RELEASE"

# Add target if not already installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Installing target: $TARGET"
    rustup target add "$TARGET"
fi

# Build
if [ "$RELEASE" = "release" ]; then
    cargo build --target "$TARGET" --release
else
    cargo build --target "$TARGET"
fi

echo "Build complete!"
echo "Binary location: target/$TARGET/$RELEASE/api-server"
