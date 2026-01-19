#!/bin/bash

set -e

echo "Running SCP tests..."
echo ""

echo "=== Test 1: Setup Script Test ==="
./test-setup.sh

echo ""
echo "==================================="
echo "All tests passed!"
echo "==================================="
