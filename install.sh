#!/bin/bash
set -e

echo "==============================="
echo " Omni Forge Installation Script"
echo "==============================="

if ! command -v cargo &> /dev/null; then
    echo "Error: Rust (cargo) is not installed."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Building Omni Forge Release Binary..."
cargo build --release --workspace

echo "Installing binary..."
cp target/release/omniforge ./omniforge

echo "Build Complete!"
echo "Run './omniforge help' to verify installation."
