#!/bin/bash
# Build Runtime_Bridge QEMU test binaries
# These tests prove Task 5 syscall path enforcement

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
USERSPACE_DIR="$PROJECT_ROOT/userspace"
BUILD_DIR="$PROJECT_ROOT/build/runtime-bridge-tests"

echo "Building Runtime_Bridge QEMU test binaries..."

# Create build directory
mkdir -p "$BUILD_DIR"

# Compiler flags for freestanding x86_64
CFLAGS="-m64 -ffreestanding -nostdlib -nostdinc -fno-builtin -fno-stack-protector -mno-red-zone -O2"
LDFLAGS="-nostdlib -static -Wl,--build-id=none"

# Build allowed path test
echo "Building runtime_bridge_allowed_test..."
gcc $CFLAGS -c "$USERSPACE_DIR/runtime_bridge_allowed_test.c" -o "$BUILD_DIR/runtime_bridge_allowed_test.o"
ld $LDFLAGS -Ttext=0x400000 "$BUILD_DIR/runtime_bridge_allowed_test.o" -o "$BUILD_DIR/runtime_bridge_allowed_test.elf"

# Build forbidden path test
echo "Building runtime_bridge_forbidden_test..."
gcc $CFLAGS -c "$USERSPACE_DIR/runtime_bridge_forbidden_test.c" -o "$BUILD_DIR/runtime_bridge_forbidden_test.o"
ld $LDFLAGS -Ttext=0x400000 "$BUILD_DIR/runtime_bridge_forbidden_test.o" -o "$BUILD_DIR/runtime_bridge_forbidden_test.elf"

echo "✓ Runtime_Bridge test binaries built:"
echo "  - $BUILD_DIR/runtime_bridge_allowed_test.elf"
echo "  - $BUILD_DIR/runtime_bridge_forbidden_test.elf"
