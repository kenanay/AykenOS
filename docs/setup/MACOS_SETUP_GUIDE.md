# AykenOS macOS Development Setup Guide

**Author:** Kenan AY  
**Platform:** macOS (Intel & Apple Silicon)  
**Updated:** January 2026

## Quick Start

```bash
# 1. Clone and enter project
git clone <repository> AykenOS
cd AykenOS

# 2. Run automated setup
./setup_and_validate.sh --auto-install

# 3. Validate environment
./validate_toolchain.sh --verbose
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

# Cross-compilation toolchain
brew install x86_64-elf-gcc x86_64-elf-binutils

# Assembly and emulation
brew install nasm qemu

# Optional: Additional tools
brew install llvm clang
```

#### Alternative: Manual Cross-Compiler Build
If Homebrew doesn't have x86_64-elf-gcc:

```bash
# Install dependencies
brew install gmp mpfr libmpc

# Create build directory
mkdir -p ~/cross-compiler
cd ~/cross-compiler

# Download and build binutils
wget https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.gz
tar -xzf binutils-2.40.tar.gz
mkdir build-binutils && cd build-binutils
../binutils-2.40/configure --target=x86_64-elf --prefix=/usr/local/cross --disable-nls
make -j$(nproc)
sudo make install

# Download and build GCC
cd ~/cross-compiler
wget https://ftp.gnu.org/gnu/gcc/gcc-12.2.0/gcc-12.2.0.tar.gz
tar -xzf gcc-12.2.0.tar.gz
mkdir build-gcc && cd build-gcc
../gcc-12.2.0/configure --target=x86_64-elf --prefix=/usr/local/cross --disable-nls --enable-languages=c --without-headers
make all-gcc -j$(nproc)
make all-target-libgcc -j$(nproc)
sudo make install-gcc
sudo make install-target-libgcc

# Add to PATH
echo 'export PATH="/usr/local/cross/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### 3. Platform-Specific Considerations

#### Apple Silicon (M1/M2/M3) Macs
```bash
# Ensure Rosetta 2 is installed (for x86_64 emulation)
softwareupdate --install-rosetta

# Use arch command for x86_64 compatibility if needed
arch -x86_64 brew install x86_64-elf-gcc

# QEMU with Apple Silicon optimization
brew install qemu --HEAD
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