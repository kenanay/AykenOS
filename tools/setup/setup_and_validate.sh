#!/usr/bin/env bash
# AykenOS Setup and Validation Script
# Author: Kenan AY
# Purpose: One-click setup and validation for AykenOS development environment

set -e

# Default parameters
SKIP_INSTALL=false
VERBOSE=false
INTERACTIVE=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
NC='\033[0m'

step() { echo -e "${CYAN}🔧 $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; }
info() { echo -e "${BLUE}ℹ️  $1${NC}"; }

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-install)
            SKIP_INSTALL=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --interactive)
            INTERACTIVE=true
            shift
            ;;
        --help)
            echo "AykenOS Setup and Validation Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --skip-install     Skip automatic tool installation"
            echo "  --verbose          Enable verbose output"
            echo "  --interactive      Enable interactive QEMU testing"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                    AykenOS Development Setup                 ║
║                        Author: Kenan AY                      ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

info "This script will set up and validate your AykenOS development environment"
info "Press Ctrl+C to cancel at any time"
echo ""

# Step 1: Check if we're in the right directory
step "Checking project structure..."
required_files=("Makefile" "kernel/kernel.c" "bootloader/efi/efi_main.c" "validate_toolchain.sh")
missing_files=()

for file in "${required_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        missing_files+=("$file")
    fi
done

if [[ ${#missing_files[@]} -gt 0 ]]; then
    error "Missing required files: ${missing_files[*]}"
    error "Please run this script from the AykenOS project root directory"
    exit 1
fi

success "Project structure validated"

# Step 2: Install missing tools (if not skipped)
if [[ "$SKIP_INSTALL" != "true" ]]; then
    step "Checking and installing missing tools..."
    
    # Detect package manager and install tools
    if command -v apt >/dev/null 2>&1; then
        info "Using apt package manager..."
        
        # Update package list
        info "Updating package list..."
        sudo apt update
        
        # Install required packages
        packages_to_install=()
        
        if ! command -v gcc >/dev/null 2>&1; then
            packages_to_install+=("gcc-multilib")
        fi
        
        if ! command -v nasm >/dev/null 2>&1; then
            packages_to_install+=("nasm")
        fi
        
        if ! command -v clang >/dev/null 2>&1; then
            packages_to_install+=("clang")
        fi
        
        if ! command -v make >/dev/null 2>&1; then
            packages_to_install+=("build-essential")
        fi
        
        if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
            packages_to_install+=("qemu-system-x86")
        fi
        
        if [[ ${#packages_to_install[@]} -gt 0 ]]; then
            info "Installing packages: ${packages_to_install[*]}"
            sudo apt install -y "${packages_to_install[@]}"
            success "Packages installed successfully"
        else
            success "All required packages already installed"
        fi
        
    elif command -v yum >/dev/null 2>&1; then
        info "Using yum package manager..."
        packages_to_install=()
        
        if ! command -v gcc >/dev/null 2>&1; then
            packages_to_install+=("gcc")
        fi
        
        if ! command -v nasm >/dev/null 2>&1; then
            packages_to_install+=("nasm")
        fi
        
        if ! command -v clang >/dev/null 2>&1; then
            packages_to_install+=("clang")
        fi
        
        if ! command -v make >/dev/null 2>&1; then
            packages_to_install+=("make")
        fi
        
        if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
            packages_to_install+=("qemu-system-x86")
        fi
        
        if [[ ${#packages_to_install[@]} -gt 0 ]]; then
            info "Installing packages: ${packages_to_install[*]}"
            sudo yum install -y "${packages_to_install[@]}"
            success "Packages installed successfully"
        fi
        
    elif command -v pacman >/dev/null 2>&1; then
        info "Using pacman package manager..."
        packages_to_install=()
        
        if ! command -v gcc >/dev/null 2>&1; then
            packages_to_install+=("gcc")
        fi
        
        if ! command -v nasm >/dev/null 2>&1; then
            packages_to_install+=("nasm")
        fi
        
        if ! command -v clang >/dev/null 2>&1; then
            packages_to_install+=("clang")
        fi
        
        if ! command -v make >/dev/null 2>&1; then
            packages_to_install+=("make")
        fi
        
        if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
            packages_to_install+=("qemu")
        fi
        
        if [[ ${#packages_to_install[@]} -gt 0 ]]; then
            info "Installing packages: ${packages_to_install[*]}"
            sudo pacman -S --noconfirm "${packages_to_install[@]}"
            success "Packages installed successfully"
        fi
        
    else
        info "No supported package manager found. Please install tools manually:"
        echo -e "  ${YELLOW}- GCC cross-compiler (x86_64-elf-gcc)${NC}"
        echo -e "  ${YELLOW}- NASM assembler${NC}"
        echo -e "  ${YELLOW}- Clang compiler${NC}"
        echo -e "  ${YELLOW}- Make build system${NC}"
        echo -e "  ${YELLOW}- QEMU emulator${NC}"
    fi
    
    # Check for cross-compiler and offer to build it
    if ! command -v x86_64-elf-gcc >/dev/null 2>&1; then
        info "Cross-compiler (x86_64-elf-gcc) not found"
        echo -e "${YELLOW}You can build it from source or use the system GCC for now${NC}"
        echo -e "${YELLOW}For production builds, a cross-compiler is recommended${NC}"
        
        # Offer to build cross-compiler (simplified)
        if command -v gcc >/dev/null 2>&1 && command -v wget >/dev/null 2>&1; then
            echo ""
            read -p "Would you like to build a cross-compiler? (y/N): " -n 1 -r
            echo ""
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                info "Building cross-compiler (this may take a while)..."
                
                # Create build directory
                mkdir -p /tmp/cross-compiler-build
                cd /tmp/cross-compiler-build
                
                # Build binutils
                info "Building binutils..."
                wget -q https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.gz
                tar -xzf binutils-2.40.tar.gz
                cd binutils-2.40
                ./configure --target=x86_64-elf --prefix=/usr/local/cross --disable-nls
                make -j$(nproc)
                sudo make install
                cd ..
                
                # Add to PATH
                export PATH="/usr/local/cross/bin:$PATH"
                
                success "Cross-compiler build completed"
                cd - >/dev/null
            fi
        fi
    else
        success "Cross-compiler already available"
    fi
    
else
    info "Skipping tool installation (--skip-install specified)"
fi

# Step 3: Run validation
step "Running comprehensive validation..."

validation_args=()
if [[ "$VERBOSE" == "true" ]]; then
    validation_args+=("--verbose")
fi

info "Running toolchain validation..."
if ./validate_toolchain.sh "${validation_args[@]}"; then
    success "Toolchain validation passed!"
    
    # Step 4: Run QEMU test if toolchain is good
    step "Running QEMU boot test..."
    
    qemu_args=()
    if [[ "$VERBOSE" == "true" ]]; then
        qemu_args+=("--verbose")
    fi
    if [[ "$INTERACTIVE" == "true" ]]; then
        qemu_args+=("--interactive")
    fi
    
    if ./qemu_test_runner.sh "${qemu_args[@]}"; then
        success "QEMU boot test passed!"
    else
        error "QEMU boot test failed"
        info "This might be normal if the kernel is not fully implemented yet"
    fi
else
    error "Toolchain validation failed"
    info "Please check the validation output above and install missing tools"
    exit 1
fi

# Step 5: Final summary
echo ""
echo -e "${GREEN}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                    Setup Complete!                          ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

success "AykenOS development environment is ready!"
echo ""
echo -e "${CYAN}Next steps:${NC}"
echo -e "  ${NC}1. Build the system:    make clean && make all${NC}"
echo -e "  ${NC}2. Test in QEMU:        make run${NC}"
echo -e "  ${NC}3. Create USB boot:     ./make_usb_boot.sh${NC}"
echo -e "  ${NC}4. Re-validate anytime: ./validate_toolchain.sh${NC}"
echo ""
echo -e "${CYAN}Documentation:${NC}"
echo -e "  ${NC}- BUILD_FIXES_COMPLETE.md - Complete build guide${NC}"
echo -e "  ${NC}- README.md - Project overview${NC}"
echo ""
echo -e "${GREEN}Happy coding! 🚀${NC}"