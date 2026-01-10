# AykenOS Toolchain and QEMU Validation Script
# Author: Kenan AY
# Purpose: Automated toolchain detection and QEMU boot validation for Windows/WSL

param(
    [switch]$SkipQemu,
    [switch]$Verbose,
    [int]$QemuTimeout = 30
)

$ErrorActionPreference = "Continue"

# Color output functions
function Write-Success { param($msg) Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Error { param($msg) Write-Host "[ERROR] $msg" -ForegroundColor Red }
function Write-Warning { param($msg) Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }

# Validation results
$script:ValidationResults = @{
    ToolchainValid = $false
    QemuValid = $false
    BuildValid = $false
    Errors = @()
    Warnings = @()
}

function Test-Command {
    param([string]$Command)
    try {
        $null = Get-Command $Command -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

function Get-CommandVersion {
    param([string]$Command, [string]$VersionArg = "--version")
    try {
        $output = & $Command $VersionArg 2>&1
        return $output[0]
    } catch {
        return "Version unknown"
    }
}

function Test-Toolchain {
    Write-Info "Validating toolchain components..."
    
    $tools = @(
        @{ Name = "x86_64-elf-gcc"; Required = $true; Description = "Cross-compiler for kernel" },
        @{ Name = "x86_64-elf-ld"; Required = $true; Description = "Cross-linker for kernel" },
        @{ Name = "clang"; Required = $true; Description = "UEFI bootloader compiler" },
        @{ Name = "nasm"; Required = $true; Description = "Assembly compiler" },
        @{ Name = "make"; Required = $true; Description = "Build system" },
        @{ Name = "qemu-system-x86_64"; Required = $false; Description = "Emulator for testing" }
    )
    
    $allRequired = $true
    
    foreach ($tool in $tools) {
        if (Test-Command $tool.Name) {
            $version = Get-CommandVersion $tool.Name
            Write-Success "$($tool.Name) found - $version"
            if ($Verbose) {
                Write-Host "  Description: $($tool.Description)" -ForegroundColor Gray
            }
        } else {
            if ($tool.Required) {
                Write-Error "$($tool.Name) not found - $($tool.Description)"
                $script:ValidationResults.Errors += "Missing required tool: $($tool.Name)"
                $allRequired = $false
            } else {
                Write-Warning "$($tool.Name) not found - $($tool.Description)"
                $script:ValidationResults.Warnings += "Missing optional tool: $($tool.Name)"
            }
        }
    }
    
    # Check for WSL if tools are missing
    if (-not $allRequired -and (Test-Command "wsl")) {
        Write-Info "WSL detected. Checking WSL toolchain..."
        try {
            $wslGcc = wsl which x86_64-elf-gcc 2>$null
            if ($wslGcc) {
                Write-Success "x86_64-elf-gcc found in WSL: $wslGcc"
                Write-Info "Consider using WSL for compilation: wsl make all"
            }
        } catch {
            Write-Warning "Could not check WSL toolchain"
        }
    }
    
    $script:ValidationResults.ToolchainValid = $allRequired
    return $allRequired
}

function Test-BuildSystem {
    Write-Info "Testing build system..."
    
    # Check required files
    $requiredFiles = @("Makefile", "linker.ld", "kernel/kernel.c", "bootloader/efi/efi_main.c")
    $filesOk = $true
    
    foreach ($file in $requiredFiles) {
        if (Test-Path $file) {
            Write-Success "Found: $file"
        } else {
            Write-Error "Missing: $file"
            $script:ValidationResults.Errors += "Missing required file: $file"
            $filesOk = $false
        }
    }
    
    if (-not $filesOk) {
        return $false
    }
    
    # Test make clean
    Write-Info "Testing make clean..."
    try {
        $cleanOutput = make clean 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "make clean successful"
        } else {
            Write-Warning "make clean returned exit code $LASTEXITCODE"
        }
    } catch {
        Write-Error "make clean failed: $_"
        $script:ValidationResults.Errors += "make clean failed"
        return $false
    }
    
    # Test make all
    Write-Info "Testing make all..."
    try {
        $buildOutput = make all 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "make all successful"
            
            # Check output files
            if ((Test-Path "kernel.elf") -and (Test-Path "bootloader/efi/BOOTX64.EFI")) {
                Write-Success "Build artifacts created successfully"
                $script:ValidationResults.BuildValid = $true
                return $true
            } else {
                Write-Error "Build completed but artifacts missing"
                $script:ValidationResults.Errors += "Build artifacts not created"
                return $false
            }
        } else {
            Write-Error "make all failed with exit code $LASTEXITCODE"
            $script:ValidationResults.Errors += "Build failed"
            if ($Verbose) {
                Write-Host "Build output:" -ForegroundColor Gray
                Write-Host $buildOutput -ForegroundColor Gray
            }
            return $false
        }
    } catch {
        Write-Error "Build system test failed: $_"
        $script:ValidationResults.Errors += "Build system test failed"
        return $false
    }
}

function Test-QemuBoot {
    if ($SkipQemu) {
        Write-Info "Skipping QEMU validation (--SkipQemu specified)"
        return $true
    }
    
    if (-not (Test-Command "qemu-system-x86_64")) {
        Write-Warning "QEMU not found, skipping boot validation"
        $script:ValidationResults.Warnings += "QEMU not available for boot testing"
        return $true
    }
    
    Write-Info "Testing QEMU boot validation..."
    
    # Create EFI image if needed
    if (-not (Test-Path "EFI.img")) {
        Write-Info "Creating EFI image..."
        try {
            if (Test-Path "make_efi_img.ps1") {
                & .\make_efi_img.ps1
            } elseif (Test-Command "make") {
                make efi-img
            } else {
                Write-Error "Cannot create EFI image - no creation method available"
                return $false
            }
        } catch {
            Write-Error "Failed to create EFI image: $_"
            return $false
        }
    }
    
    # Run QEMU with timeout
    Write-Info "Starting QEMU boot test (timeout: ${QemuTimeout}s)..."
    
    $qemuArgs = @(
        "-drive", "format=raw,file=EFI.img",
        "-serial", "stdio",
        "-display", "none",
        "-no-reboot",
        "-no-shutdown"
    )
    
    try {
        $qemuProcess = Start-Process -FilePath "qemu-system-x86_64" -ArgumentList $qemuArgs -PassThru -RedirectStandardOutput "qemu_output.log" -RedirectStandardError "qemu_error.log"
        
        $bootSuccess = $false
        $startTime = Get-Date
        
        while (((Get-Date) - $startTime).TotalSeconds -lt $QemuTimeout) {
            if ($qemuProcess.HasExited) {
                break
            }
            
            # Check for boot success indicators in output
            if (Test-Path "qemu_output.log") {
                $output = Get-Content "qemu_output.log" -Raw -ErrorAction SilentlyContinue
                if ($output -match "AykenOS|EARLY INIT|Kernel.*init|kmain") {
                    $bootSuccess = $true
                    Write-Success "Boot success detected in QEMU output"
                    break
                }
            }
            
            Start-Sleep -Milliseconds 500
        }
        
        # Clean shutdown
        if (-not $qemuProcess.HasExited) {
            $qemuProcess.Kill()
            $qemuProcess.WaitForExit(5000)
        }
        
        if ($bootSuccess) {
            Write-Success "QEMU boot validation passed"
            $script:ValidationResults.QemuValid = $true
            
            if ($Verbose -and (Test-Path "qemu_output.log")) {
                Write-Info "QEMU output:"
                Get-Content "qemu_output.log" | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
            }
        } else {
            Write-Warning "QEMU boot validation inconclusive (no clear success indicators)"
            $script:ValidationResults.Warnings += "QEMU boot validation inconclusive"
            
            if (Test-Path "qemu_error.log") {
                $errorContent = Get-Content "qemu_error.log" -Raw
                if ($errorContent) {
                    Write-Warning "QEMU errors detected:"
                    Write-Host $errorContent -ForegroundColor Yellow
                }
            }
        }
        
        # Cleanup
        Remove-Item "qemu_output.log" -ErrorAction SilentlyContinue
        Remove-Item "qemu_error.log" -ErrorAction SilentlyContinue
        
        return $bootSuccess
        
    } catch {
        Write-Error "QEMU boot test failed: $_"
        $script:ValidationResults.Errors += "QEMU boot test failed"
        return $false
    }
}

function Write-ValidationReport {
    Write-Host "`n" + "="*60 -ForegroundColor Cyan
    Write-Host "AykenOS Validation Report" -ForegroundColor Cyan
    Write-Host "="*60 -ForegroundColor Cyan
    
    Write-Host "`nValidation Results:" -ForegroundColor White
    Write-Host "  Toolchain: $(if ($script:ValidationResults.ToolchainValid) { '[OK] PASS' } else { '[ERROR] FAIL' })" -ForegroundColor $(if ($script:ValidationResults.ToolchainValid) { 'Green' } else { 'Red' })
    Write-Host "  Build System: $(if ($script:ValidationResults.BuildValid) { '[OK] PASS' } else { '[ERROR] FAIL' })" -ForegroundColor $(if ($script:ValidationResults.BuildValid) { 'Green' } else { 'Red' })
    Write-Host "  QEMU Boot: $(if ($script:ValidationResults.QemuValid) { '[OK] PASS' } else { '[WARN] SKIP/WARN' })" -ForegroundColor $(if ($script:ValidationResults.QemuValid) { 'Green' } else { 'Yellow' })
    
    if ($script:ValidationResults.Errors.Count -gt 0) {
        Write-Host "`nErrors:" -ForegroundColor Red
        foreach ($error in $script:ValidationResults.Errors) {
            Write-Host "  • $error" -ForegroundColor Red
        }
    }
    
    if ($script:ValidationResults.Warnings.Count -gt 0) {
        Write-Host "`nWarnings:" -ForegroundColor Yellow
        foreach ($warning in $script:ValidationResults.Warnings) {
            Write-Host "  • $warning" -ForegroundColor Yellow
        }
    }
    
    $overallSuccess = $script:ValidationResults.ToolchainValid -and $script:ValidationResults.BuildValid
    
    Write-Host "`nOverall Status: $(if ($overallSuccess) { '[OK] READY FOR DEVELOPMENT' } else { '[ERROR] SETUP REQUIRED' })" -ForegroundColor $(if ($overallSuccess) { 'Green' } else { 'Red' })
    
    if (-not $overallSuccess) {
        Write-Host "`nNext Steps:" -ForegroundColor Cyan
        Write-Host "  1. Install missing tools (see BUILD_FIXES.md)" -ForegroundColor White
        Write-Host "  2. Consider using WSL2 for cross-compilation" -ForegroundColor White
        Write-Host "  3. Run validation again: .\validate_toolchain.ps1" -ForegroundColor White
    } else {
        Write-Host "`nReady to develop! Try:" -ForegroundColor Green
        Write-Host "  make clean && make all && make run" -ForegroundColor White
    }
    
    Write-Host "="*60 -ForegroundColor Cyan
}

# Main execution
Write-Host "AykenOS Toolchain & QEMU Validation" -ForegroundColor Green
Write-Host "Author: Kenan AY" -ForegroundColor Gray
Write-Host ""

$toolchainOk = Test-Toolchain
if ($toolchainOk) {
    $buildOk = Test-BuildSystem
    if ($buildOk) {
        Test-QemuBoot | Out-Null
    }
}

Write-ValidationReport

# Exit with appropriate code
$overallSuccess = $script:ValidationResults.ToolchainValid -and $script:ValidationResults.BuildValid
exit $(if ($overallSuccess) { 0 } else { 1 })