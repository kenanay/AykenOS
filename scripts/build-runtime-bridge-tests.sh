#!/usr/bin/env bash
# Build Runtime_Bridge QEMU test binaries.
#
# This script is intentionally environment-aware. On macOS/arm64 it must not use
# the native Mach-O linker; the proof artifacts are freestanding x86_64 ELF.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
USERSPACE_DIR="$PROJECT_ROOT/userspace"
BUILD_DIR="$PROJECT_ROOT/build/runtime-bridge-tests"

HOST_OS="$(uname -s 2>/dev/null || echo unknown)"
HOST_ARCH="$(uname -m 2>/dev/null || echo unknown)"
TARGET_TRIPLE="${RUNTIME_BRIDGE_TARGET_TRIPLE:-x86_64-unknown-none}"

log() {
    printf '[build-runtime-bridge] %s\n' "$*"
}

die_unsupported() {
    printf '[build-runtime-bridge] unsupported: %s\n' "$*" >&2
    exit 2
}

need_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        die_unsupported "required tool missing: ${tool}"
    fi
}

readelf_tool() {
    if command -v llvm-readelf >/dev/null 2>&1; then
        printf 'llvm-readelf\n'
    elif command -v readelf >/dev/null 2>&1; then
        printf 'readelf\n'
    else
        printf '\n'
    fi
}

validate_elf() {
    local elf="$1"
    local reader="$2"

    if [[ ! -s "$elf" ]]; then
        die_unsupported "artifact missing or empty: ${elf}"
    fi

    if command -v file >/dev/null 2>&1; then
        file "$elf" | tee "${elf}.file.txt"
        if ! file "$elf" | grep -Eq 'ELF 64-bit.*(x86-64|x86_64|AMD x86-64)'; then
            die_unsupported "artifact is not an x86_64 ELF: ${elf}"
        fi
    fi

    if [[ -n "$reader" ]]; then
        "$reader" -h "$elf" > "${elf}.readelf.txt"
        if ! grep -Eq 'Machine:[[:space:]]*(Advanced Micro Devices X86-64|AMD x86-64|X86-64)' "${elf}.readelf.txt"; then
            die_unsupported "readelf machine check failed for ${elf}"
        fi
    else
        log "warning: llvm-readelf/readelf missing; file(1) validation only"
    fi
}

build_one() {
    local name="$1"
    local source="$USERSPACE_DIR/${name}.c"
    local object="$BUILD_DIR/${name}.o"
    local elf="$BUILD_DIR/${name}.elf"

    log "building ${name} (${TARGET_TRIPLE})"
    clang \
        "--target=${TARGET_TRIPLE}" \
        -m64 \
        -ffreestanding \
        -nostdlib \
        -nostdinc \
        -fno-builtin \
        -fno-stack-protector \
        -mno-red-zone \
        -O2 \
        -c "$source" \
        -o "$object"

    ld.lld -nostdlib -static -e _start "$object" -o "$elf"
}

need_tool clang
need_tool ld.lld

mkdir -p "$BUILD_DIR"

log "host_os=${HOST_OS}"
log "host_arch=${HOST_ARCH}"
log "target=${TARGET_TRIPLE}"
log "clang=$(clang --version 2>/dev/null | head -n1 || echo unknown)"
log "ld.lld=$(ld.lld --version 2>/dev/null | head -n1 || echo unknown)"

{
    echo "host_os=${HOST_OS}"
    echo "host_arch=${HOST_ARCH}"
    echo "target=${TARGET_TRIPLE}"
    echo "mode=build-verify"
    if [[ "$HOST_OS" == "Darwin" && "$HOST_ARCH" == "arm64" ]]; then
        echo "note=macos_arm64_cross_compile_requires_ld_lld"
    fi
} > "$BUILD_DIR/build-environment.txt"

build_one runtime_bridge_allowed_test
build_one runtime_bridge_forbidden_test

READER="$(readelf_tool)"
validate_elf "$BUILD_DIR/runtime_bridge_allowed_test.elf" "$READER"
validate_elf "$BUILD_DIR/runtime_bridge_forbidden_test.elf" "$READER"

log "artifacts built:"
log "  $BUILD_DIR/runtime_bridge_allowed_test.elf"
log "  $BUILD_DIR/runtime_bridge_forbidden_test.elf"
