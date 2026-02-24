#!/bin/bash

# Stellar Contracts Build Script

set -e

echo "Building Stellara Smart Contracts..."
echo "======================================"

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Build all contracts in release mode
echo "Building contracts for Soroban..."
cargo build --release --target wasm32-unknown-unknown

# List generated WASM binaries
echo ""
echo "Generated WASM Binaries:"
echo "======================="
ls -lh target/wasm32-unknown-unknown/release/*.wasm

echo ""
echo "Build complete! Contracts ready for deployment."
echo ""
echo "Next steps:"
echo "1. Review DEPLOYMENT.md for deployment instructions"
echo "2. Fund your Stellar account via testnet faucet"
echo "3. Deploy contracts using stellar CLI"
