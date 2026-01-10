# AykenOS Ring3 Validation Launcher
# Author: Kenan AY
# Purpose: Simple launcher for Task 1.5.2.3 - QEMU integration testing
# This script provides an easy way to run the comprehensive Ring3 validation

param(
    [switch]$Quick,
    [switch]$Verbose,
    [switch]$SaveLogs,
    [switch]$Interactive,
    [switch]$Help
)

if ($Help) {
    Write-Host "AykenOS Ring3 Validation Launcher" -ForegroundColor Green
    Write-Host "Task 1.5.2.3: QEMU integration testing" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Usage: .\run_ring3_validation.ps1 [OPTIONS]" -ForegroundColor White
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Cyan
    Write-Host "  -Quick        Run quick validation (10 iterations, 30s timeout)" -ForegroundColor White
    Write-Host "  -Verbose      Enable verbose output" -ForegroundColor White
    Write-Host "  -SaveLogs     Save all log files" -ForegroundColor White
    Write-Host "  -Interactive  Enable QEMU display" -ForegroundColor White
    Write-Host "  -Help         Show this help message" -ForegroundColor White
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\run_ring3_validation.ps1                    # Full validation" -ForegroundColor White
    Write-Host "  .\run_ring3_validation.ps1 -Quick             # Quick validation" -ForegroundColor White
    Write-Host "  .\run_ring3_validation.ps1 -Verbose -SaveLogs # Detailed logging" -ForegroundColor White
    exit 0
}

Write-Host "AykenOS Ring3 Validation Launcher" -ForegroundColor Green
Write-Host "Task 1.5.2.3: QEMU integration testing" -ForegroundColor Gray
Write-Host ""

# Check if comprehensive test runner exists
if (-not (Test-Path "tools/validation/comprehensive_ring3_test_runner.ps1")) {
    Write-Host "Error: Comprehensive test runner not found!" -ForegroundColor Red
    Write-Host "Expected: tools/validation/comprehensive_ring3_test_runner.ps1" -ForegroundColor Red
    exit 1
}

# Prepare arguments based on mode
$testArgs = @()

if ($Quick) {
    Write-Host "Running quick Ring3 validation..." -ForegroundColor Yellow
    $testArgs += "-Timeout", 30
    $testArgs += "-StabilityIterations", 10
} else {
    Write-Host "Running comprehensive Ring3 validation..." -ForegroundColor Cyan
    $testArgs += "-Timeout", 60
    $testArgs += "-StabilityIterations", 100
}

if ($Verbose) { $testArgs += "-Verbose" }
if ($SaveLogs) { $testArgs += "-SaveLogs" }
if ($Interactive) { $testArgs += "-Interactive" }

Write-Host "Launching comprehensive Ring3 test runner..." -ForegroundColor Info
Write-Host ""

# Execute the comprehensive test runner
try {
    & .\tools\validation\comprehensive_ring3_test_runner.ps1 @testArgs
    $exitCode = $LASTEXITCODE
    
    Write-Host ""
    if ($exitCode -eq 0) {
        Write-Host "🎉 Ring3 validation completed successfully!" -ForegroundColor Green
        Write-Host "✅ Task 1.5.2.3 requirements satisfied" -ForegroundColor Green
    } else {
        Write-Host "❌ Ring3 validation failed" -ForegroundColor Red
        Write-Host "⚠️  Task 1.5.2.3 incomplete" -ForegroundColor Yellow
    }
    
    exit $exitCode
    
} catch {
    Write-Host "Error running Ring3 validation: $_" -ForegroundColor Red
    exit 1
}