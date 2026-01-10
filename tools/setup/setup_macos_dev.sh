#!/bin/bash
# AykenOS macOS Development Environment Setup
# Author: Kenan AY

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Parse arguments
AUTO_INSTALL=false
VERBOSE=false
SKIP_VALIDATION=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --auto-install)
            AUTO_INSTALL=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --skip-validation)
            SKIP_VALIDATION=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

write_status() {
    local message="$1"
    local status="${2:-INFO}"
    local color
    
    case "$status" in
        "OK") color="$GREEN" ;;
        "ERROR") color="$RED" ;;
        "WARN") color="$YELLOW" ;;
        "INFO") color="$BLUE" ;;
        *) color="$NC" ;;
    esac
    
    echo -e "${color}[$status]${NC} $message"
}

write_section() {
    local title="$1"
    echo ""
    echo -e "${CYAN}$(printf '=%.0s' {1..60})${NC}"
    echo -e "${CYAN}$title${NC}"
    echo -e "${CYAN}$(printf '=%.0s' {1..60})${NC}"
}

# Detect macOS version and architecture
detect_platform() {
    MACOS_VERSION=$(sw_vers -productVersion)
    ARCH=$(uname -m)
    
    write_status "Detected macOS $MACOS_VERSION on $ARCH" "INFO"
    
    if [[ "$ARCH" == "arm64" ]]; then
        APPLE_SILICON=true
        write_status "Apple Silicon Mac detected" "INFO"
    else
        APPLE_SILICON=false
        write_status "Intel Mac detected" "INFO"
    fi
}

# Check if Homebrew is installed
check_homebrew() {
    if command -v brew >/dev/null 2>&1; then
        write_status "Homebrew found: $(brew --version | head -1)" "OK"
        return 0
    else
        write_status "Homebrew not found" "ERROR"
        return 1
    fi
}

# Install Homebrew
install_homebrew() {
    write_status "Installing Homebrew..." "INFO"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add to PATH for Apple Silicon
    if [[ "$APPLE_SILICON" == "true" ]]; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
}

# Check development tools
check_tools() {
    write_status "Checking development tools..." "INFO"
    
    local tools=(
        "git:Version control"
        "make:Build system"
        "gcc:C compiler"
        "clang:LLVM compiler"
    )
    
    local missing_tools=()
    
    for tool_info in "${tools[@]}"; do
        IFS=':' read -r tool desc <<< "$tool_info"
        if command -v "$tool" >/dev/null 2>&1; then
            write_status "$tool found - $desc" "OK"
        else
            write_status "$tool missing - $desc" "ERROR"
            missing_tools+=("$tool")
        fi
    done
    
    return ${#missing_tools[@]}
}

# Check cross-compilation tools
check_cross_tools() {
    write_status "Checking cross-compilation tools..." "INFO"
    
    local cross_tools=(
        "x86_64-elf-gcc:Cross compiler"
        "x86_64-elf-ld:Cross linker"
        "nasm:Assembly compiler"
        "qemu-system-x86_64:Emulator"
    )
    
    local missing_cross_tools=()
    
    for tool_info in "${cross_tools[@]}"; do
        IFS=':' read -r tool desc <<< "$tool_info"
        if command -v "$tool" >/dev/null 2>&1; then
            write_status "$tool found - $desc" "OK"
        else
            write_status "$tool missing - $desc" "ERROR"
            missing_cross_tools+=("$tool")
        fi
    done
    
    return ${#missing_cross_tools[@]}
}

# Install basic development tools
install_basic_tools() {
    write_status "Installing basic development tools..." "INFO"
    
    # Install Xcode Command Line Tools
    if ! xcode-select -p >/dev/null 2>&1; then
        write_status "Installing Xcode Command Line Tools..." "INFO"
        xcode-select --install
        
        # Wait for installation to complete
        write_status "Please complete Xcode Command Line Tools installation and press Enter to continue..." "WARN"
        read -r
    else
        write_status "Xcode Command Line Tools already installed" "OK"
    fi
    
    # Install basic tools via Homebrew
    local basic_packages=(
        "make"
        "git"
        "cmake"
    )
    
    for package in "${basic_packages[@]}"; do
        write_status "Installing $package..." "INFO"
        brew install "$package" || write_status "Failed to install $package" "WARN"
    done
}

# Install cross-compilation tools
install_cross_tools() {
    write_status "Installing cross-compilation tools..." "INFO"
    
    # Try to install via Homebrew first
    local cross_packages=(
        "nasm"
        "qemu"
    )
    
    for package in "${cross_packages[@]}"; do
        write_status "Installing $package..." "INFO"
        brew install "$package" || write_status "Failed to install $package" "WARN"
    done
    
    # Try to install x86_64-elf-gcc
    write_status "Installing x86_64-elf cross-compiler..." "INFO"
    if brew install x86_64-elf-gcc 2>/dev/null; then
        write_status "x86_64-elf-gcc installed via Homebrew" "OK"
    else
        write_status "x86_64-elf-gcc not available via Homebrew" "WARN"
        write_status "You may need to build it manually or use system GCC" "INFO"
        
        # Offer to install alternative
        if [[ "$AUTO_INSTALL" == "true" ]]; then
            install_alternative_cross_compiler
        else
            write_status "Run with --auto-install to attempt manual cross-compiler build" "INFO"
        fi
    fi
}

# Install alternative cross-compiler
install_alternative_cross_compiler() {
    write_status "Attempting to build cross-compiler manually..." "INFO"
    
    # Install dependencies
    brew install gmp mpfr libmpc
    
    # Create build directory
    local build_dir="$HOME/cross-compiler-build"
    mkdir -p "$build_dir"
    cd "$build_dir"
    
    # Build binutils
    write_status "Building binutils..." "INFO"
    if [[ ! -f "binutils-2.40.tar.gz" ]]; then
        curl -O https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.gz
    fi
    
    tar -xzf binutils-2.40.tar.gz
    mkdir -p build-binutils
    cd build-binutils
    
    ../binutils-2.40/configure \
        --target=x86_64-elf \
        --prefix=/usr/local/cross \
        --disable-nls \
        --disable-werror
    
    make -j$(sysctl -n hw.ncpu)
    sudo make install
    
    # Build GCC
    cd "$build_dir"
    write_status "Building GCC..." "INFO"
    if [[ ! -f "gcc-12.2.0.tar.gz" ]]; then
        curl -O https://ftp.gnu.org/gnu/gcc/gcc-12.2.0/gcc-12.2.0.tar.gz
    fi
    
    tar -xzf gcc-12.2.0.tar.gz
    mkdir -p build-gcc
    cd build-gcc
    
    ../gcc-12.2.0/configure \
        --target=x86_64-elf \
        --prefix=/usr/local/cross \
        --disable-nls \
        --enable-languages=c \
        --without-headers
    
    make all-gcc -j$(sysctl -n hw.ncpu)
    make all-target-libgcc -j$(sysctl -n hw.ncpu)
    sudo make install-gcc
    sudo make install-target-libgcc
    
    # Add to PATH
    echo 'export PATH="/usr/local/cross/bin:$PATH"' >> ~/.zshrc
    export PATH="/usr/local/cross/bin:$PATH"
    
    write_status "Cross-compiler built and installed" "OK"
}

# Apple Silicon specific setup
setup_apple_silicon() {
    if [[ "$APPLE_SILICON" == "true" ]]; then
        write_status "Configuring Apple Silicon optimizations..." "INFO"
        
        # Install Rosetta 2 if needed
        if ! /usr/bin/pgrep oahd >/dev/null 2>&1; then
            write_status "Installing Rosetta 2..." "INFO"
            softwareupdate --install-rosetta --agree-to-license
        else
            write_status "Rosetta 2 already installed" "OK"
        fi
        
        # Set up QEMU with hardware acceleration
        echo 'export QEMU_OPTS="-accel hvf -cpu host"' >> ~/.zshrc
        write_status "QEMU hardware acceleration configured" "OK"
    fi
}

# Test build
test_build() {
    if [[ "$SKIP_VALIDATION" == "true" ]]; then
        return 0
    fi
    
    write_status "Testing build system..." "INFO"
    
    if make clean >/dev/null 2>&1 && make all >/dev/null 2>&1; then
        write_status "Build test: SUCCESS" "OK"
        return 0
    else
        write_status "Build test: FAILED" "ERROR"
        return 1
    fi
}

# Main setup function
main() {
    write_section "AykenOS macOS Development Environment Setup"
    
    # Detect platform
    detect_platform
    
    # Check and install Homebrew
    if ! check_homebrew; then
        if [[ "$AUTO_INSTALL" == "true" ]]; then
            install_homebrew
        else
            write_status "Please install Homebrew first: https://brew.sh" "ERROR"
            exit 1
        fi
    fi
    
    # Check tools
    check_tools
    basic_tools_missing=$?
    
    check_cross_tools
    cross_tools_missing=$?
    
    # Install missing tools if auto-install is enabled
    if [[ "$AUTO_INSTALL" == "true" ]]; then
        if [[ "$basic_tools_missing" -gt 0 ]]; then
            install_basic_tools
        fi
        
        if [[ "$cross_tools_missing" -gt 0 ]]; then
            install_cross_tools
        fi
        
        # Apple Silicon specific setup
        setup_apple_silicon
    fi
    
    # Test build
    if ! test_build; then
        write_status "Build test failed. Check your toolchain installation." "WARN"
    fi
    
    # Final status
    write_section "Setup Complete"
    
    if [[ "$basic_tools_missing" -eq 0 && "$cross_tools_missing" -eq 0 ]]; then
        write_status "READY FOR DEVELOPMENT!" "OK"
        echo ""
        echo "Next steps:"
        echo "  1. Build: make clean && make all"
        echo "  2. Test:  make run"
        echo "  3. Validate: ./validate_toolchain.sh"
    else
        write_status "SETUP INCOMPLETE" "WARN"
        echo ""
        echo "Manual installation required:"
        echo "  1. Install missing tools via Homebrew"
        echo "  2. See MACOS_SETUP_GUIDE.md for detailed instructions"
        echo "  3. Run this script with --auto-install for automatic setup"
    fi
    
    echo ""
    echo "Platform-specific notes:"
    if [[ "$APPLE_SILICON" == "true" ]]; then
        echo "  • Apple Silicon optimizations enabled"
        echo "  • QEMU hardware acceleration available"
        echo "  • Rosetta 2 installed for x86_64 compatibility"
    else
        echo "  • Intel Mac - standard x86_64 toolchain"
        echo "  • Native performance for cross-compilation"
    fi
    
    echo ""
    echo "For detailed setup instructions, see MACOS_SETUP_GUIDE.md"
}

# Run main function
main "$@"