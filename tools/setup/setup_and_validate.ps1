# AykenOS Setup and Validation Script
# Author: Kenan AY
# Purpose: One-click setup and validation for AykenOS development environment

param(
    [switch]$SkipInstall,
    [switch]$Verbose,
    [switch]$Interactive
)

$ErrorActionPreference = "Continue"

function Write-Step { param($msg) Write-Host "🔧 $msg" -ForegroundColor Cyan }
function Write-Success { param($msg) Write-Host "✅ $msg" -ForegroundColor Green }
function Write-Error { param($msg) Write-Host "❌ $msg" -ForegroundColor Red }
function Write-Info { param($msg) Write-Host "ℹ️  $msg" -ForegroundColor Blue }

Write-Host @"
╔══════════════════════════════════════════════════════════════╗
║                    AykenOS Development Setup                 ║
║                        Author: Kenan AY                      ║
╚══════════════════════════════════════════════════════════════╝
"@ -ForegroundColor Green

Write-Info "This script will set up and validate your AykenOS development environment"
Write-Info "Press Ctrl+C to cancel at any time"
Write-Host ""

# Step 1: Check if we're in the right directory
Write-Step "Checking project structure..."
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$requiredFiles = @("Makefile", "kernel/kernel.c", "bootloader/efi/efi_main.c", "tools/validation/validate_toolchain.ps1")
$missingFiles = @()

foreach ($file in $requiredFiles) {
    if (-not (Test-Path (Join-Path $projectRoot $file))) {
        $missingFiles += $file
    }
}

if ($missingFiles.Count -gt 0) {
    Write-Error "Missing required files: $($missingFiles -join ', ')"
    Write-Error "Please run this script from a sub-directory of the AykenOS project root"
    exit 1
}

Write-Success "Project structure validated"

# Step 2: Install missing tools (if not skipped)
if (-not $SkipInstall) {
    Write-Step "Checking and installing missing tools..."
    
    # Check for winget
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        Write-Info "Using winget for tool installation..."
        
        # Install LLVM/Clang
        if (-not (Get-Command clang -ErrorAction SilentlyContinue)) {
            Write-Info "Installing LLVM/Clang..."
            try {
                winget install LLVM.LLVM --silent --accept-source-agreements --accept-package-agreements
                Write-Success "LLVM/Clang installed"
            } catch {
                Write-Error "Failed to install LLVM/Clang: $_"
            }
        }
        
        # Install NASM
        if (-not (Get-Command nasm -ErrorAction SilentlyContinue)) {
            Write-Info "Installing NASM..."
            try {
                winget install NASM.NASM --silent --accept-source-agreements --accept-package-agreements
                Write-Success "NASM installed"
            } catch {
                Write-Error "Failed to install NASM: $_"
            }
        }
        
        # Install QEMU
        if (-not (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue)) {
            Write-Info "Installing QEMU..."
            try {
                winget install SoftwareFreedomConservancy.QEMU --silent --accept-source-agreements --accept-package-agreements
                Write-Success "QEMU installed"
            } catch {
                Write-Error "Failed to install QEMU: $_"
            }
        }
        
        # Refresh PATH
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
        
    } else {
        Write-Info "winget not available. Please install tools manually:"
        Write-Host "  - LLVM/Clang: https://llvm.org/builds/" -ForegroundColor Yellow
        Write-Host "  - NASM: https://www.nasm.us/pub/nasm/releasebuilds/" -ForegroundColor Yellow
        Write-Host "  - QEMU: https://www.qemu.org/download/" -ForegroundColor Yellow
        Write-Host "  - Cross-compiler: Use WSL2 with Ubuntu" -ForegroundColor Yellow
    }
    
    # Check for WSL and suggest cross-compiler installation
    if (Get-Command wsl -ErrorAction SilentlyContinue) {
        Write-Info "WSL detected. Checking for cross-compiler..."
        try {
            $wslGcc = wsl which x86_64-elf-gcc 2>$null
            if (-not $wslGcc) {
                Write-Info "Installing cross-compiler in WSL..."
                wsl sudo apt update
                wsl sudo apt install -y gcc-multilib build-essential nasm
                Write-Success "Cross-compiler installed in WSL"
            } else {
                Write-Success "Cross-compiler already available in WSL"
            }
        } catch {
            Write-Error "Failed to set up WSL cross-compiler: $_"
            Write-Info "You can install manually with: wsl sudo apt install gcc-multilib"
        }
    } else {
        Write-Info "WSL not detected. Consider installing WSL2 for cross-compilation:"
        Write-Host "  wsl --install Ubuntu" -ForegroundColor Yellow
    }
} else {
    Write-Info "Skipping tool installation (--SkipInstall specified)"
}

# Step 3: Run validation
Write-Step "Running comprehensive validation..."

$validationArgs = @()
if ($Verbose) { $validationArgs += "-Verbose" }

try {
    Write-Info "Running toolchain validation..."
    & (Join-Path $projectRoot "tools/validation/validate_toolchain.ps1") @validationArgs
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Toolchain validation passed!"
        
        # Step 4: Run QEMU test if toolchain is good
        Write-Step "Running QEMU boot test..."
        
        $qemuArgs = @()
        if ($Verbose) { $qemuArgs += "--verbose" }
        if ($Interactive) { $qemuArgs += "--interactive" }
        
        & (Join-Path $projectRoot "tools/qemu/qemu_test_runner.ps1") @qemuArgs
        
        if ($LASTEXITCODE -eq 0) {
            Write-Success "QEMU boot test passed!"
        } else {
            Write-Error "QEMU boot test failed"
            Write-Info "This might be normal if the kernel is not fully implemented yet"
        }
    } else {
        Write-Error "Toolchain validation failed"
        Write-Info "Please check the validation output above and install missing tools"
        exit 1
    }
} catch {
    Write-Error "Validation failed with exception: $_"
    exit 1
}

# Step 5: Final summary
Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║                    Setup Complete!                          ║" -ForegroundColor Green
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Green

Write-Success "AykenOS development environment is ready!"
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Build the system:    'make clean', then 'make all'" -ForegroundColor White
Write-Host "  2. Test in QEMU:        make run" -ForegroundColor White
Write-Host "  3. Create USB boot:     (see docs for '.ps1' script)" -ForegroundColor White
Write-Host "  4. Re-validate anytime: (see docs for '.ps1' script)" -ForegroundColor White
Write-Host ""
Write-Host "Documentation:" -ForegroundColor Cyan
Write-Host "  - docs/development/BUILD_FIXES_COMPLETE.md - Complete build guide" -ForegroundColor White
Write-Host "  - README.md - Project overview" -ForegroundColor White
Write-Host ""