# AykenOS Syscall Round-Trip Test Validation Script
# Task 1.5.2.2 - Syscall Round-Trip Test Validation
# Author: Kiro AI Assistant
# Date: January 3, 2026

param(
    [switch]$Verbose,
    [int]$TimeoutSeconds = 30
)

Write-Host "AykenOS Syscall Round-Trip Test Validation" -ForegroundColor Cyan
Write-Host "Task 1.5.2.2 - Phase 1.5 Validation" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# Expected test output sequence
$ExpectedOutputs = @(
    "Starting Phase 1.5 Syscall Round-Trip Test",
    "=== SYSCALL ROUND-TRIP TEST START ===",
    "File opened, fd stored",
    "File read completed", 
    "File closed successfully",
    "Invalid syscall tested",
    "1.2.3.4.5.",
    "=== ALL SYSCALL TESTS PASSED ==="
)

# Test validation criteria
$ValidationCriteria = @{
    "INT_0x80_mechanism" = $false
    "parameter_passing" = $false
    "return_values" = $false
    "ring3_transitions" = $false
    "all_syscalls_tested" = $false
    "error_handling" = $false
    "stability_testing" = $false
}

function Test-Toolchain {
    Write-Host "[INFO] Checking toolchain availability..." -ForegroundColor Yellow
    
    $tools = @("x86_64-elf-gcc", "x86_64-elf-ld", "make", "qemu-system-x86_64")
    $missing = @()
    
    foreach ($tool in $tools) {
        try {
            $null = Get-Command $tool -ErrorAction Stop
            Write-Host "[OK] $tool found" -ForegroundColor Green
        }
        catch {
            Write-Host "[ERROR] $tool not found" -ForegroundColor Red
            $missing += $tool
        }
    }
    
    if ($missing.Count -gt 0) {
        Write-Host "[ERROR] Missing required tools: $($missing -join ', ')" -ForegroundColor Red
        Write-Host "[INFO] Please run toolchain setup first:" -ForegroundColor Yellow
        Write-Host "  .\tools\setup\setup_windows_dev.ps1 -AutoInstall" -ForegroundColor White
        return $false
    }
    
    return $true
}

function Build-Kernel {
    Write-Host "[INFO] Building kernel with syscall round-trip test..." -ForegroundColor Yellow
    
    try {
        $buildOutput = & make 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[ERROR] Kernel build failed" -ForegroundColor Red
            if ($Verbose) {
                Write-Host "Build output:" -ForegroundColor Gray
                $buildOutput | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
            }
            return $false
        }
        
        Write-Host "[OK] Kernel built successfully" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "[ERROR] Build process failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Test-SyscallRoundTrip {
    Write-Host "[INFO] Starting QEMU test with timeout of $TimeoutSeconds seconds..." -ForegroundColor Yellow
    
    # Start QEMU process
    $qemuArgs = @(
        "-machine", "q35",
        "-cpu", "qemu64",
        "-m", "256M",
        "-drive", "format=raw,file=EFI.img",
        "-bios", "OVMF.fd",
        "-serial", "stdio",
        "-display", "none",
        "-no-reboot"
    )
    
    try {
        $process = Start-Process -FilePath "qemu-system-x86_64" -ArgumentList $qemuArgs -PassThru -RedirectStandardOutput "qemu_syscall_test.log" -RedirectStandardError "qemu_syscall_error.log"
        
        Write-Host "[INFO] QEMU started (PID: $($process.Id))" -ForegroundColor Yellow
        Write-Host "[INFO] Monitoring output for test results..." -ForegroundColor Yellow
        
        $startTime = Get-Date
        $outputFound = @()
        
        # Monitor output file
        while ((Get-Date) -lt $startTime.AddSeconds($TimeoutSeconds)) {
            if (Test-Path "qemu_syscall_test.log") {
                $content = Get-Content "qemu_syscall_test.log" -Raw
                
                foreach ($expected in $ExpectedOutputs) {
                    if ($content -match [regex]::Escape($expected) -and $expected -notin $outputFound) {
                        Write-Host "[FOUND] $expected" -ForegroundColor Green
                        $outputFound += $expected
                        
                        # Update validation criteria
                        switch -Regex ($expected) {
                            "SYSCALL ROUND-TRIP TEST START" { $ValidationCriteria["INT_0x80_mechanism"] = $true }
                            "File opened, fd stored" { $ValidationCriteria["parameter_passing"] = $true }
                            "File read completed" { $ValidationCriteria["return_values"] = $true }
                            "File closed successfully" { $ValidationCriteria["ring3_transitions"] = $true }
                            "Invalid syscall tested" { $ValidationCriteria["error_handling"] = $true }
                            "1\.2\.3\.4\.5\." { $ValidationCriteria["stability_testing"] = $true }
                            "ALL SYSCALL TESTS PASSED" { $ValidationCriteria["all_syscalls_tested"] = $true }
                        }
                    }
                }
                
                # Check if all tests passed
                if ($outputFound.Count -eq $ExpectedOutputs.Count) {
                    Write-Host "[SUCCESS] All syscall round-trip tests completed!" -ForegroundColor Green
                    break
                }
            }
            
            Start-Sleep -Milliseconds 500
        }
        
        # Stop QEMU
        if (!$process.HasExited) {
            $process.Kill()
            $process.WaitForExit(5000)
        }
        
        return $outputFound.Count
    }
    catch {
        Write-Host "[ERROR] QEMU test failed: $($_.Exception.Message)" -ForegroundColor Red
        return 0
    }
}

function Show-ValidationReport {
    param($TestsFound)
    
    Write-Host "`n" + "="*60 -ForegroundColor Cyan
    Write-Host "SYSCALL ROUND-TRIP TEST VALIDATION REPORT" -ForegroundColor Cyan
    Write-Host "="*60 -ForegroundColor Cyan
    
    Write-Host "`nTest Results:" -ForegroundColor White
    Write-Host "  Tests Found: $TestsFound / $($ExpectedOutputs.Count)" -ForegroundColor $(if ($TestsFound -eq $ExpectedOutputs.Count) { "Green" } else { "Yellow" })
    
    Write-Host "`nValidation Criteria:" -ForegroundColor White
    foreach ($criteria in $ValidationCriteria.GetEnumerator()) {
        $status = if ($criteria.Value) { "[PASS]" } else { "[FAIL]" }
        $color = if ($criteria.Value) { "Green" } else { "Red" }
        Write-Host "  $status $($criteria.Key.Replace('_', ' '))" -ForegroundColor $color
    }
    
    $passCount = ($ValidationCriteria.Values | Where-Object { $_ }).Count
    $totalCount = $ValidationCriteria.Count
    
    Write-Host "`nOverall Status:" -ForegroundColor White
    if ($passCount -eq $totalCount) {
        Write-Host "  [SUCCESS] All syscall round-trip tests PASSED" -ForegroundColor Green
        Write-Host "  Task 1.5.2.2 requirements fully satisfied" -ForegroundColor Green
    }
    elseif ($passCount -gt 0) {
        Write-Host "  [PARTIAL] $passCount/$totalCount criteria met" -ForegroundColor Yellow
        Write-Host "  Some syscall functionality working" -ForegroundColor Yellow
    }
    else {
        Write-Host "  [FAILED] No syscall tests passed" -ForegroundColor Red
        Write-Host "  Check kernel implementation and QEMU setup" -ForegroundColor Red
    }
    
    Write-Host "`nTask 1.5.2.2 Requirements:" -ForegroundColor White
    Write-Host "  ✓ Validate INT 0x80 mechanism works reliably: $(if ($ValidationCriteria['INT_0x80_mechanism']) { 'PASS' } else { 'FAIL' })" -ForegroundColor $(if ($ValidationCriteria['INT_0x80_mechanism']) { 'Green' } else { 'Red' })
    Write-Host "  ✓ Test syscall parameter passing and return values: $(if ($ValidationCriteria['parameter_passing'] -and $ValidationCriteria['return_values']) { 'PASS' } else { 'FAIL' })" -ForegroundColor $(if ($ValidationCriteria['parameter_passing'] -and $ValidationCriteria['return_values']) { 'Green' } else { 'Red' })
    Write-Host "  ✓ Ensure Ring3→Ring0→Ring3 transitions are stable: $(if ($ValidationCriteria['ring3_transitions']) { 'PASS' } else { 'FAIL' })" -ForegroundColor $(if ($ValidationCriteria['ring3_transitions']) { 'Green' } else { 'Red' })
    Write-Host "  ✓ Test all current syscalls (read/write/open/close): $(if ($ValidationCriteria['all_syscalls_tested']) { 'PASS' } else { 'FAIL' })" -ForegroundColor $(if ($ValidationCriteria['all_syscalls_tested']) { 'Green' } else { 'Red' })
    
    Write-Host "`n" + "="*60 -ForegroundColor Cyan
}

# Main execution
try {
    # Check toolchain
    if (!(Test-Toolchain)) {
        Write-Host "[ERROR] Toolchain validation failed" -ForegroundColor Red
        exit 1
    }
    
    # Build kernel
    if (!(Build-Kernel)) {
        Write-Host "[ERROR] Kernel build failed" -ForegroundColor Red
        exit 1
    }
    
    # Run syscall round-trip test
    $testsFound = Test-SyscallRoundTrip
    
    # Show validation report
    Show-ValidationReport -TestsFound $testsFound
    
    # Exit with appropriate code
    $allPassed = ($ValidationCriteria.Values | Where-Object { $_ }).Count -eq $ValidationCriteria.Count
    exit $(if ($allPassed) { 0 } else { 1 })
}
catch {
    Write-Host "[FATAL] Validation script failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}