# Windows/WSL Setup Guide for AykenOS Development
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Author:** Kenan AY  
**Project:** AykenOS - Advanced AI-Integrated Operating System  
**Updated:** January 2026

## Overview

This guide provides comprehensive setup instructions for AykenOS development on Windows systems, including native Windows development and Windows Subsystem for Linux (WSL) configurations.

## Quick Start

```powershell
# Automated setup (recommended)
.\setup_and_validate.ps1

# Manual validation
.\validate_toolchain.ps1 -Verbose
```

## Setup Options

### Option 1: WSL2 + Ubuntu (Recommended)

WSL2 provides the most compatible development environment for AykenOS.

#### 1.1 Install WSL2

```powershell
# Install WSL2 with Ubuntu
wsl --install Ubuntu

# Restart your computer when prompted
# Set up Ubuntu username and password
```

#### 1.2 Install Development Tools in WSL

```bash
# Update package list
sudo apt update && sudo apt upgrade -y

# Install essential build tools
sudo apt install -y \
    gcc-multilib \
    build-essential \
    nasm \
    clang \
    make \
    qemu-system-x86 \
    git \
    curl \
    wget

# Verify installation
gcc --version
clang --version
nasm --version
qemu-system-x86_64 --version
```

#### 1.3 Cross-Compiler Setup (Optional but Recommended)

```bash
# Install cross-compiler dependencies
sudo apt install -y \
    libgmp3-dev \
    libmpfr-dev \
    libmpc-dev \
    flex \
    bison \
    texinfo

# Build cross-compiler (this takes time)
mkdir -p ~/cross-compiler
cd ~/cross-compiler

# Download and build binutils
wget https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.gz
tar -xzf binutils-2.40.tar.gz
cd binutils-2.40
./configure --target=x86_64-elf --prefix=/usr/local/cross --disable-nls
make -j$(nproc)
sudo make install
cd ..

# Add to PATH
echo 'export PATH="/usr/local/cross/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify cross-compiler
x86_64-elf-gcc --version
```

#### 1.4 Development Workflow

```bash
# Navigate to project (assuming it's in Windows)
cd /mnt/c/path/to/aykenos

# Validate setup
./validate_toolchain.sh --verbose

# Build and test
make clean && make all && make run
```

### Option 2: Native Windows Development

For developers who prefer native Windows development.

#### 2.1 Install Required Tools

**Using winget (Windows Package Manager):**

```powershell
# Install LLVM/Clang
winget install LLVM.LLVM

# Install NASM
winget install NASM.NASM

# Install QEMU
winget install SoftwareFreedomConservancy.QEMU

# Install Git (if not already installed)
winget install Git.Git

# Refresh PATH
$env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
```

**Manual Installation:**

1. **LLVM/Clang**: Download from https://llvm.org/builds/
   - Install to default location (C:\Program Files\LLVM)
   - Add to PATH: `C:\Program Files\LLVM\bin`

2. **NASM**: Download from https://www.nasm.us/pub/nasm/releasebuilds/
   - Install to `C:\nasm`
   - Add to PATH: `C:\nasm`

3. **QEMU**: Download from https://www.qemu.org/download/
   - Install to default location
   - Add to PATH: `C:\Program Files\qemu`

#### 2.2 Cross-Compiler Options

**Option A: Use WSL for Cross-Compilation**
```powershell
# Use WSL for kernel compilation, Windows for bootloader
wsl make kernel
make bootloader
make efi-img
```

**Option B: MinGW-w64 Cross-Compiler**
```powershell
# Install MinGW-w64
winget install Mingw-w64.Mingw-w64

# Update Makefile to use MinGW cross-compiler
# KERNEL_CC = x86_64-w64-mingw32-gcc
# KERNEL_LD = x86_64-w64-mingw32-ld
```

### Option 3: Docker Development Environment

For consistent cross-platform development.

#### 3.1 Docker Setup

```powershell
# Install Docker Desktop
winget install Docker.DockerDesktop

# Create development container
docker run -it --rm -v ${PWD}:/workspace ubuntu:20.04 bash

# Inside container
apt update && apt install -y gcc-multilib nasm clang make qemu-system-x86
cd /workspace
make clean && make all
```

#### 3.2 Docker Compose (Optional)

Create `docker-compose.yml`:

```yaml
version: '3.8'
services:
  aykenos-dev:
    image: ubuntu:20.04
    volumes:
      - .:/workspace
    working_dir: /workspace
    command: bash
    stdin_open: true
    tty: true
    environment:
      - DEBIAN_FRONTEND=noninteractive
```

## Validation and Testing

### Automated Validation

```powershell
# Complete validation
.\validate_toolchain.ps1 -Verbose

# Build validation only
.\validate_toolchain.ps1 -SkipQemu

# QEMU boot test
.\qemu_test_runner.ps1 -Verbose -SaveLogs
```

### Manual Validation Steps

1. **Toolchain Check:**
   ```powershell
   clang --version
   nasm --version
   qemu-system-x86_64 --version
   
   # For WSL
   wsl x86_64-elf-gcc --version
   ```

2. **Build Test:**
   ```powershell
   make clean
   make all
   
   # Check outputs
   ls kernel.elf
   ls bootloader/efi/BOOTX64.EFI
   ```

3. **QEMU Test:**
   ```powershell
   make efi-img
   make run
   ```

## Troubleshooting

### Common Issues

#### Issue 1: "x86_64-elf-gcc not found"

**Solutions:**
- Use WSL2 with Ubuntu (recommended)
- Build cross-compiler from source
- Use system GCC with appropriate flags

```powershell
# Quick fix: Use WSL for kernel compilation
wsl sudo apt install gcc-multilib
# Update Makefile to use: wsl x86_64-elf-gcc
```

#### Issue 2: "QEMU boot timeout"

**Solutions:**
- Increase timeout: `.\qemu_test_runner.ps1 -Timeout 60`
- Check hardware acceleration: Enable Hyper-V or WSL2
- Use interactive mode: `.\qemu_test_runner.ps1 -Interactive`

#### Issue 3: "Permission denied" errors

**Solutions:**
- Run PowerShell as Administrator
- Check Windows Defender exclusions
- Use WSL2 for file operations

#### Issue 4: "EFI image creation fails"

**Solutions:**
- Install mtools: `wsl sudo apt install mtools`
- Use PowerShell version: `.\make_efi_img.ps1`
- Check disk space and permissions

### Performance Optimization

#### WSL2 Performance

```bash
# Configure WSL2 memory and CPU limits
# Create/edit ~/.wslconfig
[wsl2]
memory=4GB
processors=4
swap=2GB
```

#### Windows Defender Exclusions

Add these paths to Windows Defender exclusions:
- Project directory
- WSL2 file system (`\\wsl$\Ubuntu\`)
- Build output directories

## Integration with IDEs

### Visual Studio Code

1. Install WSL extension
2. Open project in WSL: `code .` (from WSL terminal)
3. Install C/C++ extension
4. Configure build tasks in `.vscode/tasks.json`

### CLion

1. Configure WSL toolchain
2. Set up remote development
3. Configure CMake (if using CMake wrapper)

## Continuous Integration

### GitHub Actions Example

```yaml
name: AykenOS Windows Build
on: [push, pull_request]

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup WSL
      uses: Vampire/setup-wsl@v1
      with:
        distribution: Ubuntu-20.04
    
    - name: Install dependencies
      run: |
        wsl sudo apt update
        wsl sudo apt install -y gcc-multilib nasm clang make qemu-system-x86
    
    - name: Validate toolchain
      run: wsl ./validate_toolchain.sh --verbose
    
    - name: Build and test
      run: |
        wsl make clean
        wsl make all
        wsl ./qemu_test_runner.sh --save-logs
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: build-artifacts
        path: |
          kernel.elf
          bootloader/efi/BOOTX64.EFI
          EFI.img
          *_*.log
```

## Best Practices

### Development Workflow

1. **Use WSL2 for compilation** - Most compatible environment
2. **Native Windows for editing** - Better IDE integration
3. **Automated validation** - Run `.\validate_toolchain.ps1` regularly
4. **Version control** - Use Git with proper `.gitignore`
5. **Regular testing** - Use `make dev` for quick build-test cycles

### File System Considerations

- Keep source code on Windows filesystem for IDE performance
- Use WSL2 for compilation and testing
- Avoid mixing Windows and WSL file operations
- Use `\\wsl$\Ubuntu\` path for Windows access to WSL files

### Security Considerations

- Add project directory to Windows Defender exclusions
- Use Windows Firewall rules for QEMU if needed
- Keep WSL2 updated: `wsl --update`

## Support and Resources

### Documentation
- [BUILD_FIXES_COMPLETE.md](BUILD_FIXES_COMPLETE.md) - Complete build guide
- [README.md](README.md) - Project overview
- [Microsoft WSL Documentation](https://docs.microsoft.com/en-us/windows/wsl/)

### Community
- Report issues in project repository
- Check existing documentation for solutions
- Use validation scripts for debugging

### Quick Reference

```powershell
# Essential commands
.\setup_and_validate.ps1          # Complete setup
.\validate_toolchain.ps1 -Verbose # Validate environment
make help                         # Show all targets
make dev                          # Quick build and test
make validate-full                # Complete validation
```

---

**Happy coding on Windows! 🚀**
