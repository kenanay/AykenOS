# AykenOS macOS Development Setup Guide
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Author:** Kenan AY  
**Platform:** macOS (Intel & Apple Silicon)  
**Updated:** February 10, 2026

## Quick Start

```bash
# 1. Clone and enter project
git clone <repository> AykenOS
cd AykenOS

# 2. Install dependencies
brew install qemu nasm clang make

# 3. Build kernel
make clean && make

# 4. Create EFI.img (macOS-specific)
./build_efi.sh

# 5. Test with QEMU
timeout 10 qemu-system-x86_64 \
  -machine q35 \
  -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
  -drive if=pflash,format=raw,file=ovmf_vars.fd \
  -drive if=ide,format=raw,file=EFI.img \
  -m 256M \
  -serial file:test_ring3_serial.log \
  -debugcon file:test_ring3_debugcon.log \
  -global isa-debugcon.iobase=0xe9 \
  -nographic \
  -no-reboot || true

# 6. Check boot log
tail -50 test_ring3_debugcon.log | tr -d '\000'
```

## Detailed Setup Instructions

### 1. Prerequisites

#### Install Homebrew (if not already installed)
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Install Xcode Command Line Tools
```bash
xcode-select --install
```

### 2. Install Development Tools

#### Core Build Tools
```bash
# Essential build tools
brew install make cmake git

# Compiler (Clang is included with Xcode)
# AykenOS uses Clang with --target=x86_64-elf

# Assembly and emulation
brew install nasm qemu

# Optional: LLVM for additional tools
brew install llvm
```

**Note:** AykenOS uses Clang with `--target=x86_64-elf` flag, which is available by default on macOS. No separate cross-compiler installation needed!

### 3. macOS-Specific Build Process

#### EFI.img Creation

macOS doesn't have Linux's `mkfs.vfat` tool. Use the provided `build_efi.sh` script:

```bash
#!/bin/bash
# build_efi.sh - macOS-compatible EFI.img creation

set -e

echo "EFI.img oluşturuluyor..."

# Clean old files
rm -f EFI.img EFI.dmg EFI_raw.dmg

# Create 64MB FAT32 disk image
hdiutil create -size 64m -fs MS-DOS -volname "EFI" -o EFI.dmg

# Mount
hdiutil attach EFI.dmg >/dev/null

# Prevent AppleDouble files
export COPYFILE_DISABLE=1

# Create EFI directory structure and copy files
mkdir -p /Volumes/EFI/EFI/BOOT
rm -f /Volumes/EFI/EFI/BOOT/._* /Volumes/EFI/EFI/BOOT/*
cp -X bootloader/efi/BOOTX64.EFI /Volumes/EFI/EFI/BOOT/
cp -X kernel.elf /Volumes/EFI/EFI/BOOT/
cp -X kernel.elf /Volumes/EFI/  # Root copy for bootloader

# Create startup.nsh for automatic boot
echo "FS0:" > /Volumes/EFI/startup.nsh
echo "cd EFI\BOOT" >> /Volumes/EFI/startup.nsh
echo "BOOTX64.EFI" >> /Volumes/EFI/startup.nsh

# Unmount
hdiutil detach /Volumes/EFI >/dev/null

# Convert to raw format (QEMU-compatible)
hdiutil convert EFI.dmg -format UDRW -o EFI_raw
mv EFI_raw.dmg EFI.img
rm -f EFI.dmg

echo "EFI.img hazır!"
echo "Kernel hash:"
shasum -a 256 kernel.elf
```

**Usage:**
```bash
# After building kernel
make clean && make

# Create EFI.img
chmod +x build_efi.sh
./build_efi.sh

# Verify kernel in image
hdiutil attach EFI.img
shasum -a 256 /Volumes/EFI/kernel.elf
hdiutil detach /Volumes/EFI
```

### 4. QEMU Configuration (CRITICAL)

**Important:** Always include `-global isa-debugcon.iobase=0xe9` for debug output!

**Correct QEMU Command:**
```bash
# Clean old logs
rm -f test_ring3_debugcon.log test_ring3_serial.log test_ring3_qemu.err

# Run QEMU with proper debugcon configuration
timeout 10 qemu-system-x86_64 \
  -machine q35 \
  -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
  -drive if=pflash,format=raw,file=ovmf_vars.fd \
  -drive if=ide,format=raw,file=EFI.img \
  -m 256M \
  -serial file:test_ring3_serial.log \
  -debugcon file:test_ring3_debugcon.log \
  -global isa-debugcon.iobase=0xe9 \
  -nographic \
  -no-reboot \
  -d cpu_reset 2>test_ring3_qemu.err || true

# View debug output
tail -50 test_ring3_debugcon.log | tr -d '\000'
```

**Key Flags:**
- `-machine q35`: Better hardware emulation
- `-drive if=ide`: UEFI compatibility
- `-global isa-debugcon.iobase=0xe9`: **CRITICAL** - Enables kernel debug output
- `-debugcon file:...`: Capture debug output to file
- `-serial file:...`: Capture serial output
- `-d cpu_reset`: Log CPU resets

**Without `-global isa-debugcon.iobase=0xe9`, debugcon log will be empty!**

### 5. Platform-Specific Considerations

#### Apple Silicon (M1/M2/M3) Macs
```bash
# Ensure Rosetta 2 is installed (for x86_64 emulation)
softwareupdate --install-rosetta

# QEMU with Apple Silicon optimization
brew install qemu

# OVMF firmware location
ls /opt/homebrew/share/qemu/edk2-x86_64-code.fd
```

#### Intel Macs
```bash
# Standard installation works directly
brew install x86_64-elf-gcc x86_64-elf-binutils nasm qemu
```

### 4. Verify Installation

#### Run Validation Script
```bash
./validate_toolchain.sh --verbose
```

#### Expected Output
```
[OK] x86_64-elf-gcc found - Cross-compiler for kernel
[OK] x86_64-elf-ld found - Cross-linker for kernel  
[OK] nasm found - Assembly compiler
[OK] make found - Build system
[OK] qemu-system-x86_64 found - Emulator for testing
```

#### Manual Verification
```bash
# Check cross-compiler
x86_64-elf-gcc --version
x86_64-elf-ld --version

# Check QEMU
qemu-system-x86_64 --version

# Check NASM
nasm --version
```

### 5. Build and Test

#### Clean Build
```bash
make clean
make all
```

#### Test in QEMU
```bash
make run
# or
./qemu_test_runner.sh --verbose
```

#### Advanced Testing
```bash
# Run full integration tests
./qemu_integration_tests.sh

# Run validation suite
./final_validation_report.sh --verbose
```

## Troubleshooting

### Common Issues

#### 1. "x86_64-elf-gcc not found"
```bash
# Check if installed via Homebrew
brew list | grep gcc

# If not found, install manually or use alternative
brew install gcc
# Then use gcc with appropriate flags for cross-compilation
```

#### 2. "Permission denied" errors
```bash
# Make scripts executable
chmod +x *.sh

# Fix ownership if needed
sudo chown -R $(whoami) .
```

#### 3. QEMU boot issues on Apple Silicon
```bash
# Use specific QEMU flags for Apple Silicon
export QEMU_OPTS="-accel hvf -cpu host"
make run
```

#### 4. Build errors with Apple Clang
```bash
# Use GNU GCC instead of Apple Clang
export CC=gcc-12
export CXX=g++-12
make clean && make all
```

### Performance Optimization

#### Apple Silicon Optimization
```bash
# Use Homebrew's optimized packages
brew install --HEAD qemu

# Enable hardware acceleration
export QEMU_OPTS="-accel hvf"
```

#### Parallel Builds
```bash
# Use all CPU cores for faster builds
export MAKEFLAGS="-j$(sysctl -n hw.ncpu)"
make clean && make all
```

## Development Workflow

### Recommended Setup
```bash
# 1. Daily validation
./validate_toolchain.sh

# 2. Quick development cycle
make dev  # Clean, build, and test

# 3. Full validation (before commits)
./final_validation_report.sh --verbose
```

### IDE Integration

#### VS Code Setup
```json
// .vscode/settings.json
{
    "C_Cpp.default.compilerPath": "/usr/local/bin/x86_64-elf-gcc",
    "C_Cpp.default.includePath": [
        "${workspaceFolder}/kernel/include",
        "${workspaceFolder}/bootloader/efi"
    ],
    "C_Cpp.default.defines": [
        "__x86_64__",
        "__ELF__"
    ]
}
```

#### Xcode Integration
```bash
# Generate Xcode project (if using CMake)
cmake -G Xcode .
open AykenOS.xcodeproj
```

## Platform Comparison

| Feature | macOS Intel | macOS Apple Silicon | Linux | Windows |
|---------|-------------|-------------------|-------|---------|
| **Native Build** | ✅ Fast | ✅ Fast | ✅ Fastest | ⚠️ WSL Required |
| **QEMU Performance** | ✅ Good | ✅ Excellent (HVF) | ✅ Good | ✅ Good |
| **Toolchain Setup** | 🍺 Homebrew | 🍺 Homebrew | 📦 Package Manager | 🔧 Manual/WSL |
| **Development Experience** | ✅ Excellent | ✅ Excellent | ✅ Excellent | ✅ Good |

## Advanced Configuration

### Custom Toolchain Paths
```bash
# Add to ~/.zshrc or ~/.bash_profile
export CROSS_COMPILE=x86_64-elf-
export QEMU_SYSTEM_X86_64=/usr/local/bin/qemu-system-x86_64
export NASM=/usr/local/bin/nasm
```

### Build Optimization
```bash
# Optimize for your Mac
export CFLAGS="-O2 -march=native"
export MAKEFLAGS="-j$(sysctl -n hw.ncpu)"
```

## Conclusion

macOS provides an excellent development environment for AykenOS with:
- ✅ **Native Unix tools** - Better than Windows
- ✅ **Homebrew package management** - Easy toolchain setup  
- ✅ **Hardware acceleration** - Excellent QEMU performance on Apple Silicon
- ✅ **Professional development tools** - Xcode, VS Code, etc.

**Recommendation:** macOS is one of the best platforms for AykenOS development, especially on Apple Silicon Macs with hardware-accelerated virtualization.
