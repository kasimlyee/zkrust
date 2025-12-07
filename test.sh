#!/bin/bash

# Set your device IP
export DEVICE_IP="192.168.1.201"

echo "Running zkrust Phase 1 tests..."
echo

# Unit tests
echo "=== Running unit tests ==="
cargo test --workspace

echo
echo "=== Running connection example ==="
cargo run --example connect

echo
echo "=== Running device control example ==="
cargo run --example device_control

echo
echo "All tests completed!"