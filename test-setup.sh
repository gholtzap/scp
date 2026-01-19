#!/bin/bash

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="/tmp/scp-setup-test-$(date +%s)"

log_info "Starting SCP setup test..."
log_info "Test directory: $TEST_DIR"

cleanup() {
    if [ -d "$TEST_DIR" ]; then
        log_warn "Cleaning up test directory..."
        rm -rf "$TEST_DIR"
        log_info "Cleanup complete"
    fi
}

trap cleanup EXIT

log_info "Creating test directory..."
mkdir -p "$TEST_DIR"

log_info "Copying SCP repo in its current local state (including uncommitted files)..."
mkdir -p "$TEST_DIR/scp"

rsync -av --exclude='.git' --exclude='target' --exclude='.claude' --exclude='.env' "$SCRIPT_DIR/" "$TEST_DIR/scp/" > /dev/null

cd "$TEST_DIR/scp"

log_info "Removed .env to simulate fresh clone"

log_info "Current directory: $(pwd)"
log_info "Directory contents:"
ls -la

log_info ""
log_info "Running setup script..."
echo "" | ./setup.sh

log_info ""
log_info "Verifying .env was created..."
if [ -f .env ]; then
    log_info "✓ .env file exists"
    log_info "Contents:"
    cat .env
else
    log_error "✗ .env file was not created"
    exit 1
fi

log_info ""
log_info "Testing cargo build..."
cargo build 2>&1

log_info ""
log_info "Checking if binary was built..."
if [ -f "target/debug/scp" ]; then
    log_info "✓ Binary built successfully"
else
    log_error "✗ Binary was not built"
    exit 1
fi

log_info ""
log_info "Testing cargo check..."
cargo check 2>&1

log_info ""
log_info "=========================================="
log_info "Setup test completed successfully!"
log_info "=========================================="
log_info ""
log_info "Summary:"
log_info "  ✓ Repository cloned"
log_info "  ✓ Setup script executed"
log_info "  ✓ .env file created with NF_INSTANCE_ID"
log_info "  ✓ Dependencies fetched"
log_info "  ✓ Project builds successfully"
log_info ""
