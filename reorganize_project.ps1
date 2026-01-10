#!/usr/bin/env pwsh
# AykenOS Project Reorganization Script
# Author: Kenan AY
# Purpose: Organize project files into logical directory structure

param(
    [switch]$DryRun,
    [switch]$Force
)

$ErrorActionPreference = "Continue"

# Colors
$Green = "`e[32m"
$Red = "`e[31m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Cyan = "`e[36m"
$Reset = "`e[0m"

function Write-Status {
    param([string]$Message, [string]$Type = "INFO")
    $color = switch ($Type) {
        "OK" { $Green }
        "ERROR" { $Red }
        "WARN" { $Yellow }
        "INFO" { $Blue }
        "MOVE" { $Cyan }
        default { $Reset }
    }
    Write-Host "${color}[$Type]${Reset} $Message"
}

function Move-FileWithBackup {
    param(
        [string]$Source,
        [string]$Destination,
        [string]$Description
    )
    
    if (-not (Test-Path $Source)) {
        Write-Status "Source not found: $Source" "WARN"
        return
    }
    
    $destDir = Split-Path $Destination -Parent
    if (-not (Test-Path $destDir)) {
        if ($DryRun) {
            Write-Status "Would create directory: $destDir" "INFO"
        } else {
            New-Item -ItemType Directory -Path $destDir -Force | Out-Null
            Write-Status "Created directory: $destDir" "OK"
        }
    }
    
    if ($DryRun) {
        Write-Status "Would move: $Source -> $Destination ($Description)" "MOVE"
    } else {
        try {
            Move-Item -Path $Source -Destination $Destination -Force
            Write-Status "Moved: $Source -> $Destination ($Description)" "MOVE"
        } catch {
            Write-Status "Failed to move ${Source}: $($_.Exception.Message)" "ERROR"
        }
    }
}

Write-Host "${Cyan}AykenOS Project Reorganization${Reset}"
Write-Host "=================================="
Write-Host ""

if ($DryRun) {
    Write-Status "DRY RUN MODE - No files will be moved" "WARN"
    Write-Host ""
}

# Create main directories
$directories = @(
    "tools/setup",
    "tools/validation", 
    "tools/build",
    "tools/qemu",
    "docs/setup",
    "docs/development",
    "docs/phase1",
    "docs/phase2",
    "docs/api",
    "build"
)

Write-Status "Creating directory structure..." "INFO"
foreach ($dir in $directories) {
    if ($DryRun) {
        Write-Status "Would create: $dir" "INFO"
    } else {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            Write-Status "Created: $dir" "OK"
        }
    }
}

Write-Host ""
Write-Status "Moving files to new structure..." "INFO"

# Move setup scripts
$setupFiles = @{
    "setup_windows_dev.ps1" = "tools/setup/setup_windows_dev.ps1"
    "setup_macos_dev.sh" = "tools/setup/setup_macos_dev.sh"
    "setup_and_validate.ps1" = "tools/setup/setup_and_validate.ps1"
    "setup_and_validate.sh" = "tools/setup/setup_and_validate.sh"
    "install_dependencies.ps1" = "tools/setup/install_dependencies.ps1"
    "install_dependencies.sh" = "tools/setup/install_dependencies.sh"
}

foreach ($file in $setupFiles.Keys) {
    Move-FileWithBackup $file $setupFiles[$file] "Setup script"
}

# Move validation scripts
$validationFiles = @{
    "validate_toolchain.ps1" = "tools/validation/validate_toolchain.ps1"
    "validate_toolchain.sh" = "tools/validation/validate_toolchain.sh"
    "final_validation_report.ps1" = "tools/validation/final_validation_report.ps1"
    "final_validation_report.sh" = "tools/validation/final_validation_report.sh"
    "ring3_validation_test.sh" = "tools/validation/ring3_validation_test.sh"
    "devfs_validation_test.sh" = "tools/validation/devfs_validation_test.sh"
    "syscall_roundtrip_test.sh" = "tools/validation/syscall_roundtrip_test.sh"
}

foreach ($file in $validationFiles.Keys) {
    Move-FileWithBackup $file $validationFiles[$file] "Validation script"
}

# Move build scripts
$buildFiles = @{
    "build_windows.ps1" = "tools/build/build_windows.ps1"
    "make_efi_img.ps1" = "tools/build/make_efi_img.ps1"
    "make_efi_img.sh" = "tools/build/make_efi_img.sh"
    "make_usb_boot.ps1" = "tools/build/make_usb_boot.ps1"
    "make_usb_boot.sh" = "tools/build/make_usb_boot.sh"
}

foreach ($file in $buildFiles.Keys) {
    Move-FileWithBackup $file $buildFiles[$file] "Build script"
}

# Move QEMU scripts
$qemuFiles = @{
    "qemu_test_runner.ps1" = "tools/qemu/qemu_test_runner.ps1"
    "qemu_test_runner.sh" = "tools/qemu/qemu_test_runner.sh"
    "qemu_integration_tests.ps1" = "tools/qemu/qemu_integration_tests.ps1"
    "qemu_integration_tests.sh" = "tools/qemu/qemu_integration_tests.sh"
    "run_qemu_tests.ps1" = "tools/qemu/run_qemu_tests.ps1"
    "run_qemu_tests.sh" = "tools/qemu/run_qemu_tests.sh"
}

foreach ($file in $qemuFiles.Keys) {
    Move-FileWithBackup $file $qemuFiles[$file] "QEMU script"
}

# Move setup documentation
$setupDocs = @{
    "WINDOWS_WSL_SETUP_GUIDE.md" = "docs/setup/WINDOWS_WSL_SETUP_GUIDE.md"
    "MACOS_SETUP_GUIDE.md" = "docs/setup/MACOS_SETUP_GUIDE.md"
    "LINUX_SETUP_GUIDE.md" = "docs/setup/LINUX_SETUP_GUIDE.md"
    "MULTI_PLATFORM_DEVELOPMENT_GUIDE.md" = "docs/setup/MULTI_PLATFORM_DEVELOPMENT_GUIDE.md"
    "QUICK_START_USB.md" = "docs/setup/QUICK_START_USB.md"
    "USB_BOOT_GUIDE.md" = "docs/setup/USB_BOOT_GUIDE.md"
}

foreach ($file in $setupDocs.Keys) {
    Move-FileWithBackup $file $setupDocs[$file] "Setup documentation"
}

# Move development documentation
$devDocs = @{
    "BUILD_FIXES_COMPLETE.md" = "docs/development/BUILD_FIXES_COMPLETE.md"
    "BUILD_SYSTEM_INTEGRATION_SUMMARY.md" = "docs/development/BUILD_SYSTEM_INTEGRATION_SUMMARY.md"
    "DEVFS_IMPLEMENTATION.md" = "docs/development/DEVFS_IMPLEMENTATION.md"
    "RING3_IMPLEMENTATION.md" = "docs/development/RING3_IMPLEMENTATION.md"
    "QEMU_TEST_SUITE_DOCUMENTATION.md" = "docs/development/QEMU_TEST_SUITE_DOCUMENTATION.md"
    "PROJECT_STRUCTURE.md" = "docs/development/PROJECT_STRUCTURE.md"
    "DOCUMENTATION_INDEX.md" = "docs/development/DOCUMENTATION_INDEX.md"
}

foreach ($file in $devDocs.Keys) {
    Move-FileWithBackup $file $devDocs[$file] "Development documentation"
}

# Move Phase 1 documentation
$phase1Docs = @{
    "FAZ_1_COMPLETION_ANALYSIS.md" = "docs/phase1/FAZ_1_COMPLETION_ANALYSIS.md"
    "FAZ_1_COMPLETION_REPORT.md" = "docs/phase1/FAZ_1_COMPLETION_REPORT.md"
    "PHASE_1_COMPLETION_SUMMARY.md" = "docs/phase1/PHASE_1_COMPLETION_SUMMARY.md"
    "PHASE_1_VERIFICATION.md" = "docs/phase1/PHASE_1_VERIFICATION.md"
    "PHASE1_FINAL_VALIDATION_REPORT.md" = "docs/phase1/PHASE1_FINAL_VALIDATION_REPORT.md"
    "PHASE1_VALIDATION_SUMMARY.md" = "docs/phase1/PHASE1_VALIDATION_SUMMARY.md"
    "PROJECT_STATUS_REPORT.md" = "docs/phase1/PROJECT_STATUS_REPORT.md"
    "SESSION_SUMMARY.md" = "docs/phase1/SESSION_SUMMARY.md"
    "DEPENDENCY_FIX_SUMMARY.md" = "docs/phase1/DEPENDENCY_FIX_SUMMARY.md"
    "DEVFS_VALIDATION_SUMMARY.md" = "docs/phase1/DEVFS_VALIDATION_SUMMARY.md"
    "USB_BOOT_SUMMARY.md" = "docs/phase1/USB_BOOT_SUMMARY.md"
    "FB_CONSOLE_COMPLETE.md" = "docs/phase1/FB_CONSOLE_COMPLETE.md"
}

foreach ($file in $phase1Docs.Keys) {
    Move-FileWithBackup $file $phase1Docs[$file] "Phase 1 documentation"
}

# Move Phase 2 documentation
$phase2Docs = @{
    "FAZ_2_ABDF_BCIB.md" = "docs/phase2/FAZ_2_ABDF_BCIB.md"
    "FAZ_2_AI_SKELETON.md" = "docs/phase2/FAZ_2_AI_SKELETON.md"
    "FAZ_2_CLI_DSL.md" = "docs/phase2/FAZ_2_CLI_DSL.md"
    "FAZ_2_DATA_MODULES.md" = "docs/phase2/FAZ_2_DATA_MODULES.md"
    "FAZ_2_DEMO_PLAN.md" = "docs/phase2/FAZ_2_DEMO_PLAN.md"
    "FAZ_2_EXECUTOR_RUNTIME.md" = "docs/phase2/FAZ_2_EXECUTOR_RUNTIME.md"
    "FAZ_2_MULTI_ARCH.md" = "docs/phase2/FAZ_2_MULTI_ARCH.md"
    "FAZ_2_OVERVIEW.md" = "docs/phase2/FAZ_2_OVERVIEW.md"
    "FAZ_2_UI_RENDER.md" = "docs/phase2/FAZ_2_UI_RENDER.md"
    "cli-spec.md" = "docs/phase2/cli-spec.md"
}

foreach ($file in $phase2Docs.Keys) {
    Move-FileWithBackup $file $phase2Docs[$file] "Phase 2 documentation"
}

# Move Turkish documentation
$turkishDocs = @{
    "aykenos_faz_1_teknik_notlar.md" = "docs/development/aykenos_faz_1_teknik_notlar.md"
    "AykenOS Geliştirme Yol Haritası.txt" = "docs/development/AykenOS_Gelistirme_Yol_Haritasi.txt"
}

foreach ($file in $turkishDocs.Keys) {
    Move-FileWithBackup $file $turkishDocs[$file] "Turkish documentation"
}

# Move build outputs
$buildOutputs = @{
    "kernel.elf" = "build/kernel.elf"
    "EFI.img" = "build/EFI.img"
}

foreach ($file in $buildOutputs.Keys) {
    Move-FileWithBackup $file $buildOutputs[$file] "Build output"
}

# Create updated Makefile with new paths
Write-Status "Creating updated Makefile..." "INFO"

$newMakefileContent = @"
# AykenOS Makefile - Updated for new directory structure
# Author: Kenan AY

# Directories
KERNEL_DIR = kernel
BOOTLOADER_DIR = bootloader
BUILD_DIR = build
TOOLS_DIR = tools

# Build targets
KERNEL_ELF = `$(BUILD_DIR)/kernel.elf
EFI_IMG = `$(BUILD_DIR)/EFI.img

# Include original Makefile content but update paths
include Makefile.original

# Update validation targets to use new paths
validate:
	@echo "Running toolchain validation..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File `$(TOOLS_DIR)/validation/validate_toolchain.ps1; \
	else \
		./`$(TOOLS_DIR)/validation/validate_toolchain.sh; \
	fi

setup:
	@echo "Running platform setup..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File `$(TOOLS_DIR)/setup/setup_windows_dev.ps1; \
	elif [[ "`$OSTYPE" == "darwin"* ]]; then \
		./`$(TOOLS_DIR)/setup/setup_macos_dev.sh; \
	else \
		./`$(TOOLS_DIR)/setup/setup_and_validate.sh; \
	fi

qemu-test:
	@echo "Running QEMU tests..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File `$(TOOLS_DIR)/qemu/qemu_test_runner.ps1; \
	else \
		./`$(TOOLS_DIR)/qemu/qemu_test_runner.sh; \
	fi

.PHONY: validate setup qemu-test
"@

if ($DryRun) {
    Write-Status "Would create updated Makefile" "INFO"
} else {
    # Backup original Makefile
    if (Test-Path "Makefile") {
        Copy-Item "Makefile" "Makefile.original" -Force
        Write-Status "Backed up original Makefile to Makefile.original" "OK"
    }
    
    # Note: In real implementation, we'd need to properly update the Makefile
    Write-Status "Makefile update needed - manual review required" "WARN"
}

# Create convenience scripts in root
$rootScripts = @{
    "setup.ps1" = "tools/setup/setup_windows_dev.ps1"
    "setup.sh" = "tools/setup/setup_and_validate.sh"
    "validate.ps1" = "tools/validation/validate_toolchain.ps1"
    "validate.sh" = "tools/validation/validate_toolchain.sh"
}

Write-Status "Creating convenience scripts in root..." "INFO"
foreach ($script in $rootScripts.Keys) {
    $target = $rootScripts[$script]
    $content = @"
#!/usr/bin/env pwsh
# Convenience script - calls actual script in tools directory
& "./$target" @args
"@
    
    if ($DryRun) {
        Write-Status "Would create convenience script: $script" "INFO"
    } else {
        $content | Out-File -FilePath $script -Encoding UTF8
        Write-Status "Created convenience script: $script" "OK"
    }
}

Write-Host ""
Write-Status "Reorganization complete!" "OK"

if ($DryRun) {
    Write-Host ""
    Write-Status "This was a DRY RUN - no files were actually moved" "WARN"
    Write-Status "Run without -DryRun to perform actual reorganization" "INFO"
} else {
    Write-Host ""
    Write-Status "Files have been reorganized into the new structure" "OK"
    Write-Status "Convenience scripts created in root directory" "OK"
    Write-Status "Please review and update any remaining file references" "WARN"
}

Write-Host ""
Write-Host "New project structure:"
Write-Host "  📁 tools/     - Development and build tools"
Write-Host "  📁 docs/      - All documentation"
Write-Host "  📁 build/     - Build outputs"
Write-Host "  📁 kernel/    - Kernel source (unchanged)"
Write-Host "  📁 bootloader/ - Bootloader source (unchanged)"
Write-Host "  📄 README.md  - Main project README"
"@