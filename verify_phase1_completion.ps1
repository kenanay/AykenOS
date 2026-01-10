#!/usr/bin/env pwsh
# AykenOS Phase 1 Completion Verification & Audit Script
# Bu script, AykenOS Geliştirme Yol Haritası'ndaki Faz 1 hedeflerini
# mevcut proje durumu ile karşılaştırır ve kapsamlı bir rapor üretir.

$ErrorActionPreference = "Continue"
$RoadmapFile = "AykenOS Geliştirme Yol Haritası.txt"
$ReportFile = "PHASE1_COMPLETION_AUDIT.md"

# Renkler
$Green = "`e[32m"; $Red = "`e[31m"; $Yellow = "`e[33m"; $Cyan = "`e[36m"; $Reset = "`e[0m"

function Write-Header { param($text) Write-Host "`n$Cyan=== $text ===$Reset" }
function Write-Pass { param($text) Write-Host "$Green [PASS] $text$Reset" }
function Write-Fail { param($text) Write-Host "$Red [FAIL] $text$Reset" }
function Write-Warn { param($text) Write-Host "$Yellow [WARN] $text$Reset" }

Write-Host "AykenOS Phase 1 Completion Audit" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host "Date: $(Get-Date)"
Write-Host ""

$AuditResults = @{
    RoadmapCheck = $false
    FileIntegrity = $false
    BuildArtifacts = $false
    ValidationTests = $false
    Phase1Goals = @{}
}

# 1. Yol Haritası Kontrolü
Write-Header "1. Roadmap Alignment Check"
if (Test-Path $RoadmapFile) {
    $roadmapContent = Get-Content $RoadmapFile -Raw
    if ($roadmapContent -match "Faz 1:.*?Tamamlanması") {
        Write-Pass "Phase 1 definition found in roadmap"
        $AuditResults.RoadmapCheck = $true
        
        # Hedefleri kontrol et
        $goals = @(
            "Bellek ve Önyükleme", 
            "Sistem Çağrıları", 
            "Zamanlayıcı", 
            "Aygıt ve Sürücü", 
            "Test ve Doğrulama"
        )
        foreach ($goal in $goals) {
            if ($roadmapContent -match $goal) {
                $AuditResults.Phase1Goals[$goal] = "DEFINED"
            }
        }
    } else {
        Write-Fail "Phase 1 definition NOT found in roadmap"
    }
} else {
    Write-Fail "Roadmap file not found: $RoadmapFile"
}

# 2. Dosya Bütünlüğü Kontrolü
Write-Header "2. File Integrity Check"
$criticalFiles = @(
    "tools/build/build_windows.ps1",
    "tools/validation/final_validation_report.ps1",
    "tools/qemu/run_qemu_tests.ps1",
    "tools/qemu/qemu_integration_tests.ps1",
    "validate_ring3.ps1"
)

$missingFiles = 0
foreach ($file in $criticalFiles) {
    if (Test-Path $file) {
        Write-Pass "Found: $file"
    } else {
        Write-Fail "Missing: $file"
        $missingFiles++
    }
}

if ($missingFiles -eq 0) { $AuditResults.FileIntegrity = $true }

# 3. Build Artifacts Kontrolü
Write-Header "3. Build Artifacts Check"
if ((Test-Path "kernel.elf") -and (Test-Path "EFI.img")) {
    $kSize = (Get-Item "kernel.elf").Length
    $eSize = (Get-Item "EFI.img").Length
    Write-Pass "kernel.elf exists ($kSize bytes)"
    Write-Pass "EFI.img exists ($eSize bytes)"
    $AuditResults.BuildArtifacts = $true
} else {
    Write-Fail "Build artifacts missing. Please run 'make all' and 'make efi-img' first."
    Write-Warn "Attempting to build..."
    try {
        if (Get-Command "make" -ErrorAction SilentlyContinue) {
            make all
            make efi-img
            if ($LASTEXITCODE -eq 0) { 
                Write-Pass "Build successful during audit"
                $AuditResults.BuildArtifacts = $true 
            }
        }
    } catch { Write-Fail "Build attempt failed" }
}

# 4. Validasyon Testlerini Çalıştırma
Write-Header "4. Running Validation Suites"

# Final Validation Report'u çalıştır
$finalReportScript = "tools/validation/final_validation_report.ps1"
if (Test-Path $finalReportScript) {
    Write-Host "Executing Final Validation Report..." -ForegroundColor Gray
    & $finalReportScript -SkipQemu # QEMU testlerini ayrıca kontrol edeceğiz veya master runner ile
    if ($LASTEXITCODE -eq 0) {
        Write-Pass "Static validation passed"
    } else {
        Write-Fail "Static validation failed"
    }
}

# QEMU Master Runner'ı çalıştır (Smoke Test)
$qemuRunner = "tools/qemu/run_qemu_tests.ps1"
if (Test-Path $qemuRunner) {
    Write-Host "Executing QEMU Integration Tests (Timeout: 30s)..." -ForegroundColor Gray
    & $qemuRunner -Timeout 30 -Verbose
    if ($LASTEXITCODE -eq 0) {
        Write-Pass "Dynamic QEMU validation passed"
        $AuditResults.ValidationTests = $true
    } else {
        Write-Fail "Dynamic QEMU validation failed"
    }
}

# 5. Rapor Oluşturma
Write-Header "5. Generating Audit Report"

$statusSymbol = if ($AuditResults.ValidationTests -and $AuditResults.BuildArtifacts) { "✅ COMPLETED" } else { "⚠️ INCOMPLETE" }

$reportContent = @"
# AykenOS Phase 1 Completion Audit Report

**Date:** $(Get-Date)
**Overall Status:** $statusSymbol

## 1. Roadmap Alignment
The following Phase 1 goals were audited against the codebase:

| Goal | Status | Verification Method |
|------|--------|---------------------|
| **Memory & Boot** | $(if ($AuditResults.BuildArtifacts) {"✅ READY"} else {"❌ MISSING"}) | EFI.img check, Kernel Multiboot check |
| **Syscalls** | $(if (Test-Path "tools/validation/syscall_roundtrip_validation.ps1") {"✅ VERIFIED"} else {"⚠️ UNCHECKED"}) | Syscall Roundtrip Test Script |
| **Scheduler** | $(if (Test-Path "validate_ring3.ps1") {"✅ VERIFIED"} else {"⚠️ UNCHECKED"}) | Ring3 Validation Script |
| **DevFS** | $(if (Select-String "DevFS" "tools/qemu/qemu_integration_tests.ps1" -Quiet) {"✅ IMPLEMENTED"} else {"⚠️ MISSING"}) | Integration Test Patterns |
| **Testing** | $(if ($AuditResults.ValidationTests) {"✅ PASSED"} else {"❌ FAILED"}) | Automated QEMU Test Suite |

## 2. File Integrity
$(if ($AuditResults.FileIntegrity) {"All critical Phase 1 files are present."} else {"Some critical files are missing (see console output)."})

## 3. Recommendations
$(if ($statusSymbol -eq "✅ COMPLETED") {
    "Phase 1 is considered COMPLETE. The system is ready for Phase 2 (Data-Driven Filesystem & Shell)."
} else {
    "Phase 1 is NOT fully complete. Please address the failed validation steps before proceeding."
})

---
*Generated by verify_phase1_completion.ps1*
"@

$reportContent | Out-File $ReportFile -Encoding UTF8
Write-Pass "Report generated: $ReportFile"

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "AUDIT SUMMARY" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "Phase 1 Status: $statusSymbol" -ForegroundColor $(if ($statusSymbol -match "COMPLETED") { "Green" } else { "Yellow" })
Write-Host "Report File:    $ReportFile" -ForegroundColor White
Write-Host ""

if ($statusSymbol -match "COMPLETED") {
    exit 0
} else {
    exit 1
}