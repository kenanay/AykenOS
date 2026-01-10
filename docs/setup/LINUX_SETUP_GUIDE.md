# AykenOS Linux Development Setup Guide

**Author:** Kenan AY  
**Platform:** Linux (Ubuntu, Debian, Fedora, Arch, etc.)  
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

## Distribution-Specific Instructions

### Ubuntu/Debian
```bash
# Update package list
sudo apt update

# Install essential build tools
sudo apt install -y build-essential git make cmake

# Install cross-compilation tools
sudo apt install -y gcc-multilib nasm qemu-system-x86

# Install cross-compiler (if available)
sudo apt install -y gcc-x86-64-linux-gnu

# Alternative: Install from source (see Manual Build section)
```

### Fedora/RHEL/CentOS
```bash
# Install development tools
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y git make cmake

# Install cross-compilation tools
sudo dnf install -y nasm qemu-system-x86 glibc-devel.i686

# Build cross-compiler manually (see Manual Build section)
```

### Arch Linux
```bash
# Install base development tools
sudo pacman -S base-devel git make cmake

# Install cross-compilation tools
sudo pacman -S nasm qemu-system-x86

# Install cross-compiler from AUR
yay -S x86_64-elf-gcc x86_64-elf-binutils
# or
paru -S x86_64-elf-gcc x86_64-elf-binutils
```

### openSUSE
```bash
# Install development pattern
sudo zypper install -t pattern devel_basis

# Install additional tools
sudo zypper install git make cmake nasm qemu-x86

# Build cross-compiler manually (see Manual Build section)
```

## Manual Cross-Compiler Build

If your distribution doesn't provide x86_64-elf-gcc:

```bash
# Install dependencies
# Ubuntu/Debian:
sudo apt install -y build-essential bison flex libgmp3-dev libmpc-dev libmpfr-dev texinfo

# Fedora:
sudo dnf install -y gcc gcc-c++ bison flex gmp-devel libmpc-devel mpfr-devel texinfo

# Create build directory
mkdir -p ~/cross-compiler
cd ~/cross-compiler

# Set environment variables
export PREFIX="/usr/local/cross"
export TARGET=x86_64-elf
export PATH="$PREFIX/bin:$PATH"

# Download sources
wget https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.gz
wget https://ftp.gnu.org/gnu/gcc/gcc-12.2.0/gcc-12.2.0.tar.gz

# Build binutils
tar -xzf binutils-2.40.tar.gz
mkdir build-binutils && cd build-binutils
../binutils-2.40/configure --target=$TARGET --prefix="$PREFIX" --with-sysroot --disable-nls --disable-werror
make -j$(nproc)
sudo make install

# Build GCC
cd ~/cross-compiler
tar -xzf gcc-12.2.0.tar.gz
mkdir build-gcc && cd build-gcc
../gcc-12.2.0/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers
make all-gcc -j$(nproc)
make all-target-libgcc -j$(nproc)
sudo make install-gcc
sudo make install-target-libgcc

# Add to PATH
echo 'export PATH="/usr/local/cross/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## Verification

### Check Installation
```bash
# Run validation script
./validate_toolchain.sh --verbose

# Manual verification
x86_64-elf-gcc --version
x86_64-elf-ld --version
nasm --version
qemu-system-x86_64 --version
make --version
```

### Expected Output
```
[OK] x86_64-elf-gcc found - Cross-compiler for kernel
[OK] x86_64-elf-ld found - Cross-linker for kernel
[OK] nasm found - Assembly compiler
[OK] make found - Build system
[OK] qemu-system-x86_64 found - Emulator for testing
```

## Build and Test

### Standard Build
```bash
# Clean build
make clean
make all

# Quick development cycle
make dev

# Test in QEMU
make run
```

### Advanced Testing
```bash
# Run integration tests
./qemu_integration_tests.sh --verbose

# Full validation suite
./final_validation_report.sh --verbose

# Specific tests
./ring3_validation_test.sh
./devfs_validation_test.sh
./syscall_roundtrip_test.sh
```

## Performance Optimization

### Parallel Builds
```bash
# Use all CPU cores
export MAKEFLAGS="-j$(nproc)"
make clean && make all
```

### Compiler Optimizations
```bash
# Optimize for your CPU
export CFLAGS="-O2 -march=native"
export CXXFLAGS="-O2 -march=native"
```

### QEMU Acceleration
```bash
# Enable KVM acceleration (if available)
export QEMU_OPTS="-enable-kvm -cpu host"
make run

# Check KVM availability
ls -la /dev/kvm
```

## Troubleshooting

### Common Issues

#### 1. "Permission denied" for /dev/kvm
```bash
# Add user to kvm group
sudo usermod -a -G kvm $USER
# Log out and back in, or:
newgrp kvm
```

#### 2. Cross-compiler not found
```bash
# Check if installed
which x86_64-elf-gcc

# Check PATH
echo $PATH

# Add cross-compiler to PATH
export PATH="/usr/local/cross/bin:$PATH"
```

#### 3. QEMU not starting
```bash
# Check QEMU installation
qemu-system-x86_64 --version

# Try without KVM
export QEMU_OPTS=""
make run

# Check for missing dependencies
ldd $(which qemu-system-x86_64)
```

#### 4. Build errors
```bash
# Check for missing headers
sudo apt install linux-libc-dev  # Ubuntu/Debian
sudo dnf install kernel-headers   # Fedora

# Clean and rebuild
make clean
make all
```

## Development Environment

### IDE Setup

#### VS Code
```json
// .vscode/settings.json
{
    "C_Cpp.default.compilerPath": "/usr/local/cross/bin/x86_64-elf-gcc",
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

#### Vim/Neovim
```vim
" Add to .vimrc or init.vim
set path+=kernel/include,bootloader/efi
let g:ale_c_gcc_executable = 'x86_64-elf-gcc'
let g:ale_c_gcc_options = '-std=c11 -Wall -Wextra'
```

#### CLion/Qt Creator
- Set custom toolchain to x86_64-elf-gcc
- Add include paths: kernel/include, bootloader/efi
- Set C standard to C11

### Debugging

#### GDB Setup
```bash
# Install GDB for cross-debugging
sudo apt install gdb-multiarch  # Ubuntu/Debian
sudo dnf install gdb             # Fedora

# Debug with QEMU
make debug
# In another terminal:
gdb-multiarch kernel.elf
(gdb) target remote localhost:1234
(gdb) continue
```

#### QEMU Monitor
```bash
# Start QEMU with monitor
qemu-system-x86_64 -kernel kernel.elf -monitor stdio

# Useful monitor commands:
(qemu) info registers
(qemu) info mem
(qemu) x/10i $pc
```

## Distribution Comparison

| Distribution | Package Manager | Cross-Compiler | Setup Difficulty | Performance |
|--------------|----------------|----------------|------------------|-------------|
| **Ubuntu/Debian** | apt | Available | ⭐⭐⭐⭐⭐ Easy | ⭐⭐⭐⭐ Good |
| **Arch Linux** | pacman/AUR | AUR packages | ⭐⭐⭐⭐ Easy | ⭐⭐⭐⭐⭐ Excellent |
| **Fedora** | dnf | Manual build | ⭐⭐⭐ Medium | ⭐⭐⭐⭐ Good |
| **openSUSE** | zypper | Manual build | ⭐⭐⭐ Medium | ⭐⭐⭐⭐ Good |
| **Gentoo** | portage | Custom build | ⭐⭐ Hard | ⭐⭐⭐⭐⭐ Excellent |

## Advanced Configuration

### Custom Toolchain
```bash
# Set custom cross-compiler prefix
export CROSS_COMPILE=x86_64-elf-
export CC=${CROSS_COMPILE}gcc
export LD=${CROSS_COMPILE}ld
export AS=${CROSS_COMPILE}as
```

### Build System Integration
```bash
# Add to ~/.bashrc
export AYKEN_TOOLCHAIN_PATH="/usr/local/cross/bin"
export PATH="$AYKEN_TOOLCHAIN_PATH:$PATH"
export MAKEFLAGS="-j$(nproc)"
```

### QEMU Networking (for future network stack testing)
```bash
# Set up TAP interface
sudo ip tuntap add dev tap0 mode tap
sudo ip link set tap0 up
sudo ip addr add 192.168.100.1/24 dev tap0

# Use with QEMU
export QEMU_OPTS="-netdev tap,id=net0,ifname=tap0,script=no,downscript=no -device e1000,netdev=net0"
```

## Conclusion

Linux provides the most native and efficient development environment for AykenOS:

- ✅ **Native performance** - No emulation overhead
- ✅ **Excellent toolchain support** - Most distributions provide cross-compilers
- ✅ **KVM acceleration** - Best QEMU performance
- ✅ **Professional development tools** - GDB, Valgrind, perf, etc.
- ✅ **Package management** - Easy dependency installation

**Recommendation:** Linux (especially Ubuntu/Debian or Arch) is the optimal platform for AykenOS development.