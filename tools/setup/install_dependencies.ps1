# AykenOS Automated Dependency Installation Script
# Author: Kenan AY
# Purpose: Cross-platform dependency installation for AykenOS development

param(
    [switch]$Force,
    [switch]$SkipQemu,
    [switch]$Verbose,
    [string]$InstallMethod = "auto"  # auto, winget, manual, wsl
)

$ErrorActionPreference = "Continue"

function Write-Step { param($msg) Write-Host "🔧 $msg" -ForegroundColor Cyan }
function Write-Success { param($msg) Write-Host "✅ $msg" -ForegroundColor Green }
function Write-Error { param($msg) Write-Host "❌ $msg" -ForegroundColor Red }
function Write-Info { param($msg) Write-Host "ℹ️  $msg" -ForegroundColor Blue }
function Write-Warning { param($msg) Write-Host "⚠️  $msg" -ForegroundColor Yellow }

# Dependency configuration
$script:Dependencies = @{
    Required = @(
        @{ Name = "clang"; WingetId = "LLVM.LLVM"; Description = "LLVM/Clang compiler for UEFI bootloader" },
        @{ Name = "nasm"; WingetId = "NASM.NASM"; Description = "Netwide Assembler" },
        @{ Name = "make"; WingetId = "GnuWin32.Make"; Description = "GNU Make build system" }
    )
    Optional = @(
        @{ Name = "qemu-system-x86_64"; WingetId = "SoftwareFreedomConservancy.QEMU"; Description = "QEMU emulator for testing" },
        @{ Name = "git"; WingetId = "Git.Git"; Description = "Version control system" }
    )
    WSLPackages = @(
        "gcc-multilib",
        "build-essential", 
        "nasm",
        "clang",
        "make",
        "qemu-system-x86",
        "git",
        "curl",
        "wget"
    )
}

function Test-AdminRights {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-WSLAvailable {
    try {
        $wslVersion = wsl --version 2>$null
        return $true
    } catch {
        return $false
    }
}

function Test-WingetAvailable {
    try {
        $wingetVersion = winget --version 2>$null
        return $true
    } catch {
        return $false
    }
}

function Install-WSL2 {
    Write-Step "Installing WSL2 with Ubuntu..."
    
    if (-not (Test-AdminRights)) {
        Write-Error "Administrator rights required to install WSL2"
        Write-Info "Please run this script as Administrator or install WSL2 manually:"
        Write-Host "  wsl --install Ubuntu" -ForegroundColor Yellow
        return $false
    }
    
    try {
        # Enable WSL feature
        Write-Info "Enabling WSL feature..."
        Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux -NoRestart
        
        # Enable Virtual Machine Platform
        Write-Info "Enabling Virtual Machine Platform..."
        Enable-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform -NoRestart
        
        # Install Ubuntu
        Write-Info "Installing Ubuntu distribution..."
        wsl --install Ubuntu
        
        Write-Success "WSL2 installation initiated"
        Write-Warning "A restart may be required to complete WSL2 installation"
        Write-Info "After restart, run: wsl --set-default-version 2"
        
        return $true
    } catch {
        Write-Error "Failed to install WSL2: $_"
        return $false
    }
}

function Install-WSLDependencies {
    Write-Step "Installing dependencies in WSL..."
    
    if (-not (Test-WSLAvailable)) {
        Write-Error "WSL not available"
        return $false
    }
    
    try {
        # Check if Ubuntu is installed
        $wslDistros = wsl --list --quiet
        if (-not ($wslDistros -match "Ubuntu")) {
            Write-Warning "Ubuntu not found in WSL. Installing..."
            wsl --install Ubuntu
            Write-Info "Please set up Ubuntu and run this script again"
            return $false
        }
        
        Write-Info "Updating package list in WSL..."
        wsl sudo apt update
        
        Write-Info "Installing build dependencies..."
        $packageList = $script:Dependencies.WSLPackages -join " "
        wsl sudo apt install -y $packageList
        
        Write-Success "WSL dependencies installed successfully"
        
        # Verify installation
        Write-Info "Verifying WSL installation..."
        $gccVersion = wsl gcc --version 2>$null
        $clangVersion = wsl clang --version 2>$null
        $nasmVersion = wsl nasm --version 2>$null
        
        if ($gccVersion -and $clangVersion -and $nasmVersion) {
            Write-Success "WSL toolchain verification passed"
            if ($Verbose) {
                Write-Host "  GCC: $($gccVersion[0])" -ForegroundColor Gray
                Write-Host "  Clang: $($clangVersion[0])" -ForegroundColor Gray
                Write-Host "  NASM: $($nasmVersion[0])" -ForegroundColor Gray
            }
            return $true
        } else {
            Write-Warning "Some tools may not be properly installed in WSL"
            return $false
        }
        
    } catch {
        Write-Error "Failed to install WSL dependencies: $_"
        return $false
    }
}

function Install-WingetDependencies {
    Write-Step "Installing dependencies using winget..."
    
    if (-not (Test-WingetAvailable)) {
        Write-Error "winget not available"
        Write-Info "Please install App Installer from Microsoft Store or use manual installation"
        return $false
    }
    
    $installSuccess = $true
    
    # Install required dependencies
    foreach ($dep in $script:Dependencies.Required) {
        if ($Force -or -not (Get-Command $dep.Name -ErrorAction SilentlyContinue)) {
            Write-Info "Installing $($dep.Name)..."
            try {
                winget install $dep.WingetId --silent --accept-source-agreements --accept-package-agreements
                Write-Success "$($dep.Name) installed successfully"
            } catch {
                Write-Error "Failed to install $($dep.Name): $_"
                $installSuccess = $false
            }
        } else {
            Write-Success "$($dep.Name) already installed"
        }
    }
    
    # Install optional dependencies
    if (-not $SkipQemu) {
        foreach ($dep in $script:Dependencies.Optional) {
            if ($Force -or -not (Get-Command $dep.Name -ErrorAction SilentlyContinue)) {
                Write-Info "Installing $($dep.Name) (optional)..."
                try {
                    winget install $dep.WingetId --silent --accept-source-agreements --accept-package-agreements
                    Write-Success "$($dep.Name) installed successfully"
                } catch {
                    Write-Warning "Failed to install optional dependency $($dep.Name): $_"
                }
            } else {
                Write-Success "$($dep.Name) already installed"
            }
        }
    }
    
    if ($installSuccess) {
        # Refresh PATH
        Write-Info "Refreshing environment PATH..."
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
        Write-Success "winget dependencies installation completed"
    }
    
    return $installSuccess
}

function Install-ManualDependencies {
    Write-Step "Manual dependency installation guide..."
    
    Write-Info "Please install the following tools manually:"
    Write-Host ""
    
    Write-Host "Required Tools:" -ForegroundColor Yellow
    Write-Host "  1. LLVM/Clang:" -ForegroundColor White
    Write-Host "     Download: https://llvm.org/builds/" -ForegroundColor Gray
    Write-Host "     Install to: C:\Program Files\LLVM" -ForegroundColor Gray
    Write-Host "     Add to PATH: C:\Program Files\LLVM\bin" -ForegroundColor Gray
    Write-Host ""
    
    Write-Host "  2. NASM Assembler:" -ForegroundColor White
    Write-Host "     Download: https://www.nasm.us/pub/nasm/releasebuilds/" -ForegroundColor Gray
    Write-Host "     Install to: C:\nasm" -ForegroundColor Gray
    Write-Host "     Add to PATH: C:\nasm" -ForegroundColor Gray
    Write-Host ""
    
    Write-Host "  3. GNU Make:" -ForegroundColor White
    Write-Host "     Download: http://gnuwin32.sourceforge.net/packages/make.htm" -ForegroundColor Gray
    Write-Host "     Or use: winget install GnuWin32.Make" -ForegroundColor Gray
    Write-Host ""
    
    if (-not $SkipQemu) {
        Write-Host "Optional Tools:" -ForegroundColor Yellow
        Write-Host "  4. QEMU Emulator:" -ForegroundColor White
        Write-Host "     Download: https://www.qemu.org/download/" -ForegroundColor Gray
        Write-Host "     Install to default location" -ForegroundColor Gray
        Write-Host ""
    }
    
    Write-Host "Cross-Compiler Options:" -ForegroundColor Yellow
    Write-Host "  Option A: Use WSL2 (Recommended)" -ForegroundColor White
    Write-Host "    wsl --install Ubuntu" -ForegroundColor Gray
    Write-Host "    wsl sudo apt install gcc-multilib" -ForegroundColor Gray
    Write-Host ""
    Write-Host "  Option B: Build from source" -ForegroundColor White
    Write-Host "    See WINDOWS_WSL_SETUP_GUIDE.md for detailed instructions" -ForegroundColor Gray
    Write-Host ""
    
    Write-Info "After installation, run: .\validate_toolchain.ps1 -Verbose"
    
    return $true
}

function Get-InstallationMethod {
    if ($InstallMethod -ne "auto") {
        return $InstallMethod
    }
    
    # Auto-detect best installation method
    if (Test-WSLAvailable) {
        Write-Info "WSL detected - recommending WSL-based development"
        return "wsl"
    } elseif (Test-WingetAvailable) {
        Write-Info "winget detected - using package manager installation"
        return "winget"
    } else {
        Write-Info "No automated installation method available - using manual guide"
        return "manual"
    }
}

function Install-CrossCompiler {
    Write-Step "Setting up cross-compiler..."
    
    if (Test-WSLAvailable) {
        Write-Info "Installing cross-compiler in WSL..."
        try {
            # Check if cross-compiler already exists
            $crossGcc = wsl which x86_64-elf-gcc 2>$null
            if ($crossGcc) {
                Write-Success "Cross-compiler already available in WSL: $crossGcc"
                return $true
            }
            
            Write-Info "Cross-compiler not found. Installing build dependencies..."
            wsl sudo apt install -y libgmp3-dev libmpfr-dev libmpc-dev flex bison texinfo
            
            Write-Info "Building cross-compiler (this may take 10-30 minutes)..."
            Write-Warning "This is a long process. Consider using system GCC for now."
            
            # Offer choice
            $choice = Read-Host "Build cross-compiler now? (y/N)"
            if ($choice -match "^[Yy]") {
                # Build cross-compiler in WSL
                wsl bash -c @"
                mkdir -p ~/cross-compiler && cd ~/cross-compiler
                wget -q https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.gz
                tar -xzf binutils-2.40.tar.gz
                cd binutils-2.40
                ./configure --target=x86_64-elf --prefix=/usr/local/cross --disable-nls
                make -j\$(nproc)
                sudo make install
                echo 'export PATH="/usr/local/cross/bin:\$PATH"' >> ~/.bashrc
"@
                Write-Success "Cross-compiler build completed"
            } else {
                Write-Info "Skipping cross-compiler build. You can use system GCC for now."
                Write-Info "To build later, see WINDOWS_WSL_SETUP_GUIDE.md"
            }
            
            return $true
        } catch {
            Write-Error "Failed to set up cross-compiler: $_"
            return $false
        }
    } else {
        Write-Info "WSL not available. Cross-compiler setup skipped."
        Write-Info "Consider installing WSL2 for better compatibility."
        return $true
    }
}

function Validate-Installation {
    Write-Step "Validating installation..."
    
    try {
        if (Test-Path "validate_toolchain.ps1") {
            Write-Info "Running toolchain validation..."
            $validationArgs = @()
            if ($Verbose) { $validationArgs += "-Verbose" }
            if ($SkipQemu) { $validationArgs += "-SkipQemu" }
            
            & .\validate_toolchain.ps1 @validationArgs
            
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Installation validation passed!"
                return $true
            } else {
                Write-Warning "Installation validation had issues"
                Write-Info "Check the validation output above for details"
                return $false
            }
        } else {
            Write-Warning "Validation script not found - skipping validation"
            return $true
        }
    } catch {
        Write-Error "Validation failed: $_"
        return $false
    }
}

# Main execution
Write-Host @"
╔══════════════════════════════════════════════════════════════╗
║              AykenOS Dependency Installation                 ║
║                     Author: Kenan AY                         ║
╚══════════════════════════════════════════════════════════════╝
"@ -ForegroundColor Green

Write-Info "This script will install AykenOS development dependencies"
Write-Info "Installation method: $InstallMethod"
if ($SkipQemu) { Write-Info "QEMU installation will be skipped" }
Write-Host ""

# Determine installation method
$method = Get-InstallationMethod
Write-Info "Using installation method: $method"

$installSuccess = $false

switch ($method) {
    "wsl" {
        if (-not (Test-WSLAvailable)) {
            Write-Info "WSL not available, installing..."
            if (Install-WSL2) {
                Write-Info "WSL2 installed. Please restart and run this script again."
                exit 0
            } else {
                Write-Error "WSL2 installation failed"
                exit 1
            }
        }
        $installSuccess = Install-WSLDependencies
        if ($installSuccess) {
            Install-CrossCompiler | Out-Null
        }
    }
    "winget" {
        $installSuccess = Install-WingetDependencies
    }
    "manual" {
        $installSuccess = Install-ManualDependencies
    }
    default {
        Write-Error "Unknown installation method: $method"
        exit 1
    }
}

if ($installSuccess) {
    Write-Success "Dependency installation completed!"
    
    # Run validation
    if (Validate-Installation) {
        Write-Host ""
        Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Green
        Write-Host "║                 Installation Successful!                    ║" -ForegroundColor Green
        Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Green
        
        Write-Success "AykenOS development environment is ready!"
        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Cyan
        Write-Host "  1. Build the system:    make clean; make all" -ForegroundColor White
        Write-Host "  2. Test in QEMU:        make run" -ForegroundColor White
        Write-Host "  3. Validate anytime:    .\validate_toolchain.ps1" -ForegroundColor White
        Write-Host ""
        
        exit 0
    } else {
        Write-Warning "Installation completed but validation had issues"
        Write-Info "You may still be able to develop, but some features might not work"
        exit 1
    }
} else {
    Write-Error "Dependency installation failed"
    Write-Info "Please check the error messages above and try manual installation"
    Write-Info "See WINDOWS_WSL_SETUP_GUIDE.md for detailed instructions"
    exit 1
}