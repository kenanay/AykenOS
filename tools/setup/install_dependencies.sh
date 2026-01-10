#!/usr/bin/env bash
# AykenOS Automated Dependency Installation Script
# Author: Kenan AY
# Purpose: Cross-platform dependency installation for AykenOS development

set -e

# Default parameters
FORCE=false
SKIP_QEMU=false
VERBOSE=false
INSTALL_METHOD="auto"  # auto, apt, yum, pacman, manual

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
GRAY='\033[0;37m'
NC='\033[0m'

step() { echo -e "${CYAN}🔧 $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; }
info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }

# Dependency configuration
REQUIRED_PACKAGES_APT=(
    "gcc-multilib"
    "build-essential"
    "nasm"
    "clang"
    "make"
)

OPTIONAL_PACKAGES_APT=(
    "qemu-system-x86"
    "git"
    "curl"
    "wget"
)

REQUIRED_PACKAGES_YUM=(
    "gcc"
    "glibc-devel.i686"
    "nasm"
    "clang"
    "make"
)

OPTIONAL_PACKAGES_YUM=(
    "qemu-system-x86"
    "git"
    "curl"
    "wget"
)

REQUIRED_PACKAGES_PACMAN=(
    "gcc"
    "lib32-glibc"
    "nasm"
    "clang"
    "make"
)

OPTIONAL_PACKAGES_PACMAN=(
    "qemu"
    "git"
    "curl"
    "wget"
)

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --force)
            FORCE=true
            shift
            ;;
        --skip-qemu)
            SKIP_QEMU=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --install-method)
            INSTALL_METHOD="$2"
            shift 2
            ;;
        --help)
            echo "AykenOS Automated Dependency Installation Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --force              Force reinstallation of existing packages"
            echo "  --skip-qemu          Skip QEMU installation"
            echo "  --verbose            Enable verbose output"
            echo "  --install-method M   Installation method (auto, apt, yum, pacman, manual)"
            echo "  --help               Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

detect_package_manager() {
    if command_exists apt; then
        echo "apt"
    elif command_exists yum; then
        echo "yum"
    elif command_exists pacman; then
        echo "pacman"
    else
        echo "none"
    fi
}

get_installation_method() {
    if [[ "$INSTALL_METHOD" != "auto" ]]; then
        echo "$INSTALL_METHOD"
        return
    fi
    
    # Auto-detect package manager
    local pm=$(detect_package_manager)
    if [[ "$pm" != "none" ]]; then
        info "$pm package manager detected"
        echo "$pm"
    else
        info "No supported package manager found - using manual guide"
        echo "manual"
    fi
}

install_apt_dependencies() {
    step "Installing dependencies using apt..."
    
    if ! command_exists apt; then
        error "apt package manager not available"
        return 1
    fi
    
    # Update package list
    info "Updating package list..."
    sudo apt update
    
    # Install required packages
    local packages_to_install=()
    
    for package in "${REQUIRED_PACKAGES_APT[@]}"; do
        if [[ "$FORCE" == "true" ]] || ! dpkg -l | grep -q "^ii  $package "; then
            packages_to_install+=("$package")
        else
            success "$package already installed"
        fi
    done
    
    # Add optional packages
    if [[ "$SKIP_QEMU" != "true" ]]; then
        for package in "${OPTIONAL_PACKAGES_APT[@]}"; do
            if [[ "$FORCE" == "true" ]] || ! dpkg -l | grep -q "^ii  $package "; then
                packages_to_install+=("$package")
            else
                success "$package already installed"
            fi
        done
    fi
    
    if [[ ${#packages_to_install[@]} -gt 0 ]]; then
        info "Installing packages: ${packages_to_install[*]}"
        sudo apt install -y "${packages_to_install[@]}"
        success "apt packages installed successfully"
    else
        success "All required packages already installed"
    fi
    
    return 0
}

install_yum_dependencies() {
    step "Installing dependencies using yum..."
    
    if ! command_exists yum; then
        error "yum package manager not available"
        return 1
    fi
    
    # Install required packages
    local packages_to_install=()
    
    for package in "${REQUIRED_PACKAGES_YUM[@]}"; do
        if [[ "$FORCE" == "true" ]] || ! rpm -q "$package" >/dev/null 2>&1; then
            packages_to_install+=("$package")
        else
            success "$package already installed"
        fi
    done
    
    # Add optional packages
    if [[ "$SKIP_QEMU" != "true" ]]; then
        for package in "${OPTIONAL_PACKAGES_YUM[@]}"; do
            if [[ "$FORCE" == "true" ]] || ! rpm -q "$package" >/dev/null 2>&1; then
                packages_to_install+=("$package")
            else
                success "$package already installed"
            fi
        done
    fi
    
    if [[ ${#packages_to_install[@]} -gt 0 ]]; then
        info "Installing packages: ${packages_to_install[*]}"
        sudo yum install -y "${packages_to_install[@]}"
        success "yum packages installed successfully"
    else
        success "All required packages already installed"
    fi
    
    return 0
}

install_pacman_dependencies() {
    step "Installing dependencies using pacman..."
    
    if ! command_exists pacman; then
        error "pacman package manager not available"
        return 1
    fi
    
    # Update package database
    info "Updating package database..."
    sudo pacman -Sy
    
    # Install required packages
    local packages_to_install=()
    
    for package in "${REQUIRED_PACKAGES_PACMAN[@]}"; do
        if [[ "$FORCE" == "true" ]] || ! pacman -Q "$package" >/dev/null 2>&1; then
            packages_to_install+=("$package")
        else
            success "$package already installed"
        fi
    done
    
    # Add optional packages
    if [[ "$SKIP_QEMU" != "true" ]]; then
        for package in "${OPTIONAL_PACKAGES_PACMAN[@]}"; do
            if [[ "$FORCE" == "true" ]] || ! pacman -Q "$package" >/dev/null 2>&1; then
                packages_to_install+=("$package")
            else
                success "$package already installed"
            fi
        done
    fi
    
    if [[ ${#packages_to_install[@]} -gt 0 ]]; then
        info "Installing packages: ${packages_to_install[*]}"
        sudo pacman -S --noconfirm "${packages_to_install[@]}"
        success "pacman packages installed successfully"
    else
        success "All required packages already installed"
    fi
    
    return 0
}

install_manual_dependencies() {
    step "Manual dependency installation guide..."
    
    info "Please install the following tools manually:"
    echo ""
    
    echo -e "${YELLOW}Required Tools:${NC}"
    echo -e "  ${NC}1. GCC Cross-Compiler (x86_64-elf-gcc)${NC}"
    echo -e "     ${GRAY}Build from source or use system GCC${NC}"
    echo -e "     ${GRAY}See WINDOWS_WSL_SETUP_GUIDE.md for build instructions${NC}"
    echo ""
    
    echo -e "  ${NC}2. NASM Assembler${NC}"
    echo -e "     ${GRAY}Download: https://www.nasm.us/pub/nasm/releasebuilds/${NC}"
    echo -e "     ${GRAY}Or compile from source${NC}"
    echo ""
    
    echo -e "  ${NC}3. Clang Compiler${NC}"
    echo -e "     ${GRAY}Download: https://llvm.org/builds/${NC}"
    echo -e "     ${GRAY}Or compile from source${NC}"
    echo ""
    
    echo -e "  ${NC}4. GNU Make${NC}"
    echo -e "     ${GRAY}Usually available in build-essential packages${NC}"
    echo ""
    
    if [[ "$SKIP_QEMU" != "true" ]]; then
        echo -e "${YELLOW}Optional Tools:${NC}"
        echo -e "  ${NC}5. QEMU Emulator${NC}"
        echo -e "     ${GRAY}Download: https://www.qemu.org/download/${NC}"
        echo -e "     ${GRAY}Or compile from source${NC}"
        echo ""
    fi
    
    echo -e "${YELLOW}Package Manager Commands:${NC}"
    echo -e "  ${NC}Ubuntu/Debian:${NC}"
    echo -e "    ${GRAY}sudo apt install gcc-multilib nasm clang make qemu-system-x86${NC}"
    echo ""
    echo -e "  ${NC}RHEL/CentOS:${NC}"
    echo -e "    ${GRAY}sudo yum install gcc nasm clang make qemu-system-x86${NC}"
    echo ""
    echo -e "  ${NC}Arch Linux:${NC}"
    echo -e "    ${GRAY}sudo pacman -S gcc nasm clang make qemu${NC}"
    echo ""
    
    info "After installation, run: ./validate_toolchain.sh --verbose"
    
    return 0
}

install_cross_compiler() {
    step "Setting up cross-compiler..."
    
    # Check if cross-compiler already exists
    if command_exists x86_64-elf-gcc; then
        success "Cross-compiler already available: $(which x86_64-elf-gcc)"
        return 0
    fi
    
    info "Cross-compiler (x86_64-elf-gcc) not found"
    echo ""
    
    # Check if we have build dependencies
    if ! command_exists gcc || ! command_exists make; then
        warning "Build dependencies not available for cross-compiler build"
        info "You can use system GCC for now or install build dependencies first"
        return 0
    fi
    
    echo -e "${YELLOW}Cross-compiler options:${NC}"
    echo -e "  ${NC}1. Build from source (recommended for production)${NC}"
    echo -e "  ${NC}2. Use system GCC (quick start)${NC}"
    echo -e "  ${NC}3. Skip for now${NC}"
    echo ""
    
    read -p "Choose option (1/2/3): " -n 1 -r choice
    echo ""
    
    case $choice in
        1)
            build_cross_compiler
            ;;
        2)
            info "Using system GCC. Update Makefile if needed:"
            echo -e "  ${GRAY}KERNEL_CC = gcc${NC}"
            echo -e "  ${GRAY}KERNEL_LD = ld${NC}"
            ;;
        3)
            info "Skipping cross-compiler setup"
            ;;
        *)
            info "Invalid choice. Skipping cross-compiler setup"
            ;;
    esac
    
    return 0
}

build_cross_compiler() {
    step "Building cross-compiler from source..."
    
    warning "This process can take 10-30 minutes depending on your system"
    read -p "Continue? (y/N): " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Cross-compiler build cancelled"
        return 0
    fi
    
    # Install build dependencies
    info "Installing build dependencies..."
    case $(detect_package_manager) in
        "apt")
            sudo apt install -y libgmp3-dev libmpfr-dev libmpc-dev flex bison texinfo
            ;;
        "yum")
            sudo yum install -y gmp-devel mpfr-devel libmpc-devel flex bison texinfo
            ;;
        "pacman")
            sudo pacman -S --noconfirm gmp mpfr libmpc flex bison texinfo
            ;;
        *)
            warning "Unknown package manager. Please install build dependencies manually"
            ;;
    esac
    
    # Create build directory
    local build_dir="/tmp/cross-compiler-build"
    mkdir -p "$build_dir"
    cd "$build_dir"
    
    info "Building binutils..."
    if [[ ! -f "binutils-2.40.tar.gz" ]]; then
        wget -q https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.gz
    fi
    
    tar -xzf binutils-2.40.tar.gz
    cd binutils-2.40
    
    ./configure --target=x86_64-elf --prefix=/usr/local/cross --disable-nls
    make -j$(nproc)
    sudo make install
    
    # Add to PATH
    export PATH="/usr/local/cross/bin:$PATH"
    echo 'export PATH="/usr/local/cross/bin:$PATH"' >> ~/.bashrc
    
    success "Cross-compiler build completed"
    info "Added /usr/local/cross/bin to PATH"
    info "Restart your shell or run: source ~/.bashrc"
    
    # Verify installation
    if command_exists x86_64-elf-gcc; then
        local version=$(x86_64-elf-gcc --version | head -n1)
        success "Cross-compiler verification: $version"
    else
        warning "Cross-compiler not found in PATH. You may need to restart your shell"
    fi
    
    # Cleanup
    cd - >/dev/null
    rm -rf "$build_dir"
    
    return 0
}

validate_installation() {
    step "Validating installation..."
    
    if [[ -x "./validate_toolchain.sh" ]]; then
        info "Running toolchain validation..."
        
        local validation_args=()
        if [[ "$VERBOSE" == "true" ]]; then
            validation_args+=("--verbose")
        fi
        if [[ "$SKIP_QEMU" == "true" ]]; then
            validation_args+=("--skip-qemu")
        fi
        
        if ./validate_toolchain.sh "${validation_args[@]}"; then
            success "Installation validation passed!"
            return 0
        else
            warning "Installation validation had issues"
            info "Check the validation output above for details"
            return 1
        fi
    else
        warning "Validation script not found - skipping validation"
        return 0
    fi
}

# Main execution
echo -e "${GREEN}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║              AykenOS Dependency Installation                 ║
║                     Author: Kenan AY                         ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

info "This script will install AykenOS development dependencies"
info "Installation method: $INSTALL_METHOD"
if [[ "$SKIP_QEMU" == "true" ]]; then
    info "QEMU installation will be skipped"
fi
echo ""

# Determine installation method
method=$(get_installation_method)
info "Using installation method: $method"

install_success=false

case "$method" in
    "apt")
        if install_apt_dependencies; then
            install_success=true
        fi
        ;;
    "yum")
        if install_yum_dependencies; then
            install_success=true
        fi
        ;;
    "pacman")
        if install_pacman_dependencies; then
            install_success=true
        fi
        ;;
    "manual")
        if install_manual_dependencies; then
            install_success=true
        fi
        ;;
    *)
        error "Unknown installation method: $method"
        exit 1
        ;;
esac

if [[ "$install_success" == "true" ]]; then
    success "Dependency installation completed!"
    
    # Set up cross-compiler if using package manager
    if [[ "$method" != "manual" ]]; then
        install_cross_compiler
    fi
    
    # Run validation
    if validate_installation; then
        echo ""
        echo -e "${GREEN}"
        cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                 Installation Successful!                    ║
╚══════════════════════════════════════════════════════════════╝
EOF
        echo -e "${NC}"
        
        success "AykenOS development environment is ready!"
        echo ""
        echo -e "${CYAN}Next steps:${NC}"
        echo -e "  ${NC}1. Build the system:    make clean && make all${NC}"
        echo -e "  ${NC}2. Test in QEMU:        make run${NC}"
        echo -e "  ${NC}3. Validate anytime:    ./validate_toolchain.sh${NC}"
        echo ""
        
        exit 0
    else
        warning "Installation completed but validation had issues"
        info "You may still be able to develop, but some features might not work"
        exit 1
    fi
else
    error "Dependency installation failed"
    info "Please check the error messages above and try manual installation"
    info "See WINDOWS_WSL_SETUP_GUIDE.md for detailed instructions"
    exit 1
fi