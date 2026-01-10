#!/usr/bin/env pwsh
# AykenOS Windows Development Environment Setup
# Author: Kenan AY

param(
    [switch]$Force,
    [switch]$SkipValidation,
    [switch]$AutoInstall
)

$ErrorActionPreference = "Continue"

# Colors
$Green = "`e[32m"
$Red = "`e[31m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Reset = "`e[0m"

function Write-Status {
    param([string]$Message, [string]$Type = "INFO")
    $color = switch ($Type) {
        "OK" { $Green }
        "ERROR" { $Red }
        "WARN" { $Yellow }
        "INFO" { $Blue }
        default { $Reset }
    }
    Write-Host "${color}[$Type]${Reset} $Message"
}

Write-Host "${Blue}AykenOS Windows Development Environment Setup${Reset}"
Write-Host "=============================================="
Write-Host ""

# Check if WSL is available
Write-Status "Checking WSL availability..." "INFO"
try {
    $wslVersion = wsl --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Status "WSL is available" "OK"
        $hasWSL = $true
    } else {
        Write-Status "WSL not available or not configured" "WARN"
        $hasWSL = $false
    }
} catch {
    Write-Status "WSL not available" "WARN"
    $hasWSL = $false
}

# Check required tools
Write-Status "Checking development tools..." "INFO"

$tools = @{
    "make" = "Build system"
    "git" = "Version control"
    "powershell" = "Scripting environment"
}

$missingTools = @()
foreach ($tool in $tools.Keys) {
    if (Get-Command $tool -ErrorAction SilentlyContinue) {
        Write-Status "$tool found - $($tools[$tool])" "OK"
    } else {
        Write-Status "$tool missing - $($tools[$tool])" "ERROR"
        $missingTools += $tool
    }
}

# Check cross-compilation tools
Write-Status "Checking cross-compilation tools..." "INFO"

$crossTools = @{
    "x86_64-elf-gcc" = "Cross compiler"
    "x86_64-elf-ld" = "Cross linker"
    "nasm" = "Assembly compiler"
    "qemu-system-x86_64" = "Emulator"
}

$missingCrossTools = @()
foreach ($tool in $crossTools.Keys) {
    $found = $false
    
    # Check in PATH
    if (Get-Command $tool -ErrorAction SilentlyContinue) {
        Write-Status "$tool found in PATH - $($crossTools[$tool])" "OK"
        $found = $true
    }
    # Check in WSL if available
    elseif ($hasWSL) {
        try {
            $wslResult = wsl which $tool 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Status "$tool found in WSL - $($crossTools[$tool])" "OK"
                $found = $true
            }
        } catch {
            # Ignore WSL errors
        }
    }
    
    if (-not $found) {
        Write-Status "$tool missing - $($crossTools[$tool])" "ERROR"
        $missingCrossTools += $tool
    }
}

# Summary
Write-Host ""
Write-Status "Environment Assessment Summary" "INFO"
Write-Host "=============================="

if ($missingTools.Count -eq 0) {
    Write-Status "Basic development tools: COMPLETE" "OK"
} else {
    Write-Status "Basic development tools: MISSING ($($missingTools.Count) tools)" "ERROR"
}

if ($missingCrossTools.Count -eq 0) {
    Write-Status "Cross-compilation tools: COMPLETE" "OK"
} else {
    Write-Status "Cross-compilation tools: MISSING ($($missingCrossTools.Count) tools)" "ERROR"
}

if ($hasWSL) {
    Write-Status "WSL environment: AVAILABLE" "OK"
} else {
    Write-Status "WSL environment: NOT AVAILABLE" "WARN"
}

# Recommendations
Write-Host ""
Write-Status "Recommendations" "INFO"
Write-Host "==============="

if ($missingCrossTools.Count -gt 0) {
    Write-Host "To set up cross-compilation tools:"
    Write-Host ""
    
    if ($hasWSL) {
        Write-Host "${Yellow}Option 1: Use WSL (Recommended)${Reset}"
        Write-Host "  wsl sudo apt update"
        Write-Host "  wsl sudo apt install build-essential nasm qemu-system-x86"
        Write-Host "  wsl sudo apt install gcc-multilib"
        Write-Host ""
        Write-Host "  # For cross-compiler (if needed):"
        Write-Host "  wsl sudo apt install gcc-x86-64-linux-gnu"
        Write-Host ""
    }
    
    Write-Host "${Yellow}Option 2: Install tools directly on Windows${Reset}"
    Write-Host "  1. Install MSYS2: https://www.msys2.org/"
    Write-Host "  2. Install tools via MSYS2:"
    Write-Host "     pacman -S mingw-w64-x86_64-gcc"
    Write-Host "     pacman -S mingw-w64-x86_64-nasm"
    Write-Host "     pacman -S mingw-w64-x86_64-qemu"
    Write-Host ""
    
    Write-Host "${Yellow}Option 3: Use package managers${Reset}"
    Write-Host "  # With Chocolatey:"
    Write-Host "  choco install msys2 qemu"
    Write-Host ""
    Write-Host "  # With Scoop:"
    Write-Host "  scoop install msys2 qemu"
    Write-Host ""
}

if (-not $hasWSL) {
    Write-Host "${Yellow}To enable WSL (Recommended):${Reset}"
    Write-Host "  1. Enable WSL feature:"
    Write-Host "     dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart"
    Write-Host "  2. Enable Virtual Machine Platform:"
    Write-Host "     dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart"
    Write-Host "  3. Restart computer"
    Write-Host "  4. Install WSL2:"
    Write-Host "     wsl --install"
    Write-Host ""
}

# Auto-installation if requested
if ($AutoInstall) {
    Write-Host ""
    Write-Status "Auto-installation requested" "INFO"
    Write-Host "============================"
    
    if ($missingCrossTools.Count -gt 0 -and $hasWSL) {
        Write-Status "Installing missing tools via WSL..." "INFO"
        
        try {
            # Update package list
            Write-Status "Updating WSL package list..." "INFO"
            wsl sudo apt update
            
            # Install build-essential and basic tools
            if ($missingCrossTools -contains "x86_64-elf-gcc" -or $missingCrossTools -contains "x86_64-elf-ld") {
                Write-Status "Installing build-essential and gcc-multilib..." "INFO"
                wsl sudo apt install -y build-essential gcc-multilib
            }
            
            # Install NASM if missing
            if ($missingCrossTools -contains "nasm") {
                Write-Status "Installing NASM..." "INFO"
                wsl sudo apt install -y nasm
            }
            
            # Install QEMU if missing
            if ($missingCrossTools -contains "qemu-system-x86_64") {
                Write-Status "Installing QEMU..." "INFO"
                wsl sudo apt install -y qemu-system-x86
            }
            
            Write-Status "WSL tools installation completed" "OK"
            
            # Re-check tools after installation
            Write-Status "Re-checking installed tools..." "INFO"
            $stillMissing = @()
            foreach ($tool in $crossTools.Keys) {
                $found = $false
                
                # Check in PATH
                if (Get-Command $tool -ErrorAction SilentlyContinue) {
                    $found = $true
                }
                # Check in WSL
                elseif ($hasWSL) {
                    try {
                        $wslResult = wsl which $tool 2>$null
                        if ($LASTEXITCODE -eq 0) {
                            $found = $true
                        }
                    } catch {
                        # Ignore WSL errors
                    }
                }
                
                if (-not $found) {
                    $stillMissing += $tool
                }
            }
            
            if ($stillMissing.Count -eq 0) {
                Write-Status "All cross-compilation tools now available!" "OK"
            } else {
                Write-Status "Some tools still missing: $($stillMissing -join ', ')" "WARN"
            }
            
        } catch {
            Write-Status "Auto-installation failed: $($_.Exception.Message)" "ERROR"
        }
    } elseif ($missingCrossTools.Count -gt 0 -and -not $hasWSL) {
        Write-Status "WSL not available - cannot auto-install cross-compilation tools" "ERROR"
        Write-Status "Please install WSL2 first or use manual installation" "INFO"
    }
    
    # Install basic Windows tools if missing
    if ($missingTools -contains "make") {
        Write-Status "Attempting to install make via winget..." "INFO"
        try {
            winget install GnuWin32.Make --silent --accept-source-agreements --accept-package-agreements
            Write-Status "Make installation completed" "OK"
        } catch {
            Write-Status "Failed to install make via winget: $($_.Exception.Message)" "WARN"
        }
    }
}

# Test build if tools are available and auto-install was used
if ($AutoInstall -and -not $SkipValidation) {
    Write-Host ""
    Write-Status "Testing build system after auto-installation..." "INFO"
    
    try {
        if ($hasWSL) {
            Write-Status "Testing WSL build..." "INFO"
            $buildResult = wsl make clean 2>&1
            $buildResult += wsl make all 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Status "WSL build test: SUCCESS" "OK"
            } else {
                Write-Status "WSL build test: FAILED" "ERROR"
                if ($buildResult) {
                    Write-Host "Build output:"
                    Write-Host $buildResult
                }
            }
        }
    } catch {
        Write-Status "Build test failed: $($_.Exception.Message)" "ERROR"
    }
}

# Test build if tools are available
if ($missingCrossTools.Count -eq 0 -and -not $SkipValidation) {
    Write-Host ""
    Write-Status "Testing build system..." "INFO"
    
    try {
        if ($hasWSL) {
            Write-Status "Testing WSL build..." "INFO"
            $buildResult = wsl make clean 2>&1
            $buildResult += wsl make all 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Status "WSL build test: SUCCESS" "OK"
            } else {
                Write-Status "WSL build test: FAILED" "ERROR"
                if ($buildResult) {
                    Write-Host "Build output:"
                    Write-Host $buildResult
                }
            }
        } else {
            Write-Status "Testing native build..." "INFO"
            $buildResult = make clean 2>&1
            $buildResult += make all 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Status "Native build test: SUCCESS" "OK"
            } else {
                Write-Status "Native build test: FAILED" "ERROR"
                if ($buildResult) {
                    Write-Host "Build output:"
                    Write-Host $buildResult
                }
            }
        }
    } catch {
        Write-Status "Build test failed: $($_.Exception.Message)" "ERROR"
    }
}

# Final status
Write-Host ""
Write-Host "=============================================="
if ($missingTools.Count -eq 0 -and $missingCrossTools.Count -eq 0) {
    Write-Status "READY FOR DEVELOPMENT!" "OK"
    Write-Host ""
    Write-Host "Next steps:"
    Write-Host "  1. Build: make clean && make all"
    Write-Host "  2. Test:  make run"
    Write-Host "  3. Validate: .\validate_toolchain.ps1"
} elseif ($missingCrossTools.Count -eq 0) {
    Write-Status "CROSS-COMPILATION READY" "OK"
    Write-Host "Install missing basic tools and you'll be ready to develop!"
} else {
    Write-Status "SETUP REQUIRED" "WARN"
    Write-Host "Follow the recommendations above to complete your development environment."
}

Write-Host ""
Write-Host "For detailed setup instructions, see:"
Write-Host "  - WINDOWS_WSL_SETUP_GUIDE.md"
Write-Host "  - BUILD_FIXES_COMPLETE.md"