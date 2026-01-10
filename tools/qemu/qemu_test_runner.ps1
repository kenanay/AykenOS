# AykenOS QEMU Test Runner
# Author: Kenan AY
# Purpose: Advanced QEMU boot testing with log analysis and automation

param(
    [int]$Timeout = 30,
    [switch]$Verbose,
    [switch]$SaveLogs,
    [string]$TestName = "boot-test",
    [switch]$Interactive
)

$ErrorActionPreference = "Continue"

# Test configuration
$script:TestConfig = @{
    QemuArgs = @(
        "-drive", "format=raw,file=EFI.img",
        "-serial", "stdio",
        "-m", "256M",
        "-no-reboot",
        "-no-shutdown"
    )
    SuccessPatterns = @(
        "AykenOS.*INIT",
        "Kernel.*init.*done",
        "kmain.*starting",
        "EARLY INIT.*done",
        "Scheduler.*ready"
    )
    ErrorPatterns = @(
        "PANIC",
        "ERROR",
        "FATAL",
        "Triple fault",
        "General Protection Fault"
    )
    TimeoutSeconds = $Timeout
}

function Write-TestLog {
    param([string]$Message, [string]$Level = "INFO")
    
    $timestamp = Get-Date -Format "HH:mm:ss.fff"
    $color = switch ($Level) {
        "SUCCESS" { "Green" }
        "ERROR" { "Red" }
        "WARNING" { "Yellow" }
        "INFO" { "Cyan" }
        default { "White" }
    }
    
    Write-Host "[$timestamp] [$Level] $Message" -ForegroundColor $color
}

function Start-QemuTest {
    param([string]$TestName)
    
    Write-TestLog "Starting QEMU test: $TestName" "INFO"
    
    # Ensure EFI image exists
    if (-not (Test-Path "EFI.img")) {
        Write-TestLog "EFI.img not found, creating..." "WARNING"
        try {
            if (Test-Path "make_efi_img.ps1") {
                & .\make_efi_img.ps1
            } else {
                make efi-img
            }
            Write-TestLog "EFI.img created successfully" "SUCCESS"
        } catch {
            Write-TestLog "Failed to create EFI.img: $_" "ERROR"
            return $false
        }
    }
    
    # Prepare log files
    $outputLog = "${TestName}_output.log"
    $errorLog = "${TestName}_error.log"
    $analysisLog = "${TestName}_analysis.log"
    
    # Clean old logs
    Remove-Item $outputLog, $errorLog, $analysisLog -ErrorAction SilentlyContinue
    
    # Configure QEMU arguments
    $qemuArgs = $script:TestConfig.QemuArgs
    if (-not $Interactive) {
        $qemuArgs += @("-display", "none")
    }
    
    Write-TestLog "QEMU command: qemu-system-x86_64 $($qemuArgs -join ' ')" "INFO"
    
    try {
        # Start QEMU process
        $qemuProcess = Start-Process -FilePath "qemu-system-x86_64" -ArgumentList $qemuArgs -PassThru -RedirectStandardOutput $outputLog -RedirectStandardError $errorLog
        
        Write-TestLog "QEMU process started (PID: $($qemuProcess.Id))" "INFO"
        
        # Monitor the test
        $testResult = Monitor-QemuExecution -Process $qemuProcess -OutputLog $outputLog -ErrorLog $errorLog -AnalysisLog $analysisLog
        
        # Cleanup
        if (-not $qemuProcess.HasExited) {
            Write-TestLog "Terminating QEMU process..." "INFO"
            $qemuProcess.Kill()
            $qemuProcess.WaitForExit(5000)
        }
        
        # Generate report
        Generate-TestReport -TestName $TestName -Result $testResult -OutputLog $outputLog -ErrorLog $errorLog -AnalysisLog $analysisLog
        
        # Cleanup logs if not saving
        if (-not $SaveLogs) {
            Remove-Item $outputLog, $errorLog, $analysisLog -ErrorAction SilentlyContinue
        }
        
        return $testResult.Success
        
    } catch {
        Write-TestLog "QEMU test failed with exception: $_" "ERROR"
        return $false
    }
}

function Monitor-QemuExecution {
    param(
        [System.Diagnostics.Process]$Process,
        [string]$OutputLog,
        [string]$ErrorLog,
        [string]$AnalysisLog
    )
    
    $startTime = Get-Date
    $lastOutputSize = 0
    $bootSuccess = $false
    $errorDetected = $false
    $bootStages = @()
    
    Write-TestLog "Monitoring QEMU execution (timeout: $($script:TestConfig.TimeoutSeconds)s)..." "INFO"
    
    while (((Get-Date) - $startTime).TotalSeconds -lt $script:TestConfig.TimeoutSeconds) {
        # Check if process exited
        if ($Process.HasExited) {
            Write-TestLog "QEMU process exited with code: $($Process.ExitCode)" "WARNING"
            break
        }
        
        # Analyze output
        if (Test-Path $OutputLog) {
            $currentOutput = Get-Content $OutputLog -Raw -ErrorAction SilentlyContinue
            if ($currentOutput -and $currentOutput.Length -gt $lastOutputSize) {
                $newContent = $currentOutput.Substring($lastOutputSize)
                $lastOutputSize = $currentOutput.Length
                
                # Check for success patterns
                foreach ($pattern in $script:TestConfig.SuccessPatterns) {
                    if ($newContent -match $pattern) {
                        $match = $matches[0]
                        Write-TestLog "Boot stage detected: $match" "SUCCESS"
                        $bootStages += @{
                            Timestamp = Get-Date
                            Stage = $match
                            Pattern = $pattern
                        }
                        $bootSuccess = $true
                    }
                }
                
                # Check for error patterns
                foreach ($pattern in $script:TestConfig.ErrorPatterns) {
                    if ($newContent -match $pattern) {
                        $match = $matches[0]
                        Write-TestLog "Error detected: $match" "ERROR"
                        $errorDetected = $true
                        break
                    }
                }
                
                if ($Verbose) {
                    $newContent.Split("`n") | ForEach-Object {
                        if ($_.Trim()) {
                            Write-Host "  QEMU: $_" -ForegroundColor Gray
                        }
                    }
                }
            }
        }
        
        # Early exit on error
        if ($errorDetected) {
            Write-TestLog "Stopping test due to error detection" "ERROR"
            break
        }
        
        Start-Sleep -Milliseconds 500
    }
    
    $duration = ((Get-Date) - $startTime).TotalSeconds
    Write-TestLog "Test completed in $([math]::Round($duration, 2)) seconds" "INFO"
    
    # Write analysis log
    $analysis = @{
        TestName = $TestName
        Duration = $duration
        Success = $bootSuccess -and -not $errorDetected
        BootStages = $bootStages
        ErrorDetected = $errorDetected
        ProcessExited = $Process.HasExited
        ExitCode = if ($Process.HasExited) { $Process.ExitCode } else { $null }
    }
    
    $analysis | ConvertTo-Json -Depth 3 | Out-File $AnalysisLog -Encoding UTF8
    
    return $analysis
}

function Generate-TestReport {
    param(
        [string]$TestName,
        [hashtable]$Result,
        [string]$OutputLog,
        [string]$ErrorLog,
        [string]$AnalysisLog
    )
    
    Write-Host "`n" + "="*60 -ForegroundColor Cyan
    Write-Host "QEMU Test Report: $TestName" -ForegroundColor Cyan
    Write-Host "="*60 -ForegroundColor Cyan
    
    Write-Host "`nTest Results:" -ForegroundColor White
    Write-Host "  Status: $(if ($Result.Success) { '✓ PASS' } else { '✗ FAIL' })" -ForegroundColor $(if ($Result.Success) { 'Green' } else { 'Red' })
    Write-Host "  Duration: $([math]::Round($Result.Duration, 2)) seconds" -ForegroundColor White
    Write-Host "  Boot Stages Detected: $($Result.BootStages.Count)" -ForegroundColor White
    Write-Host "  Errors Detected: $(if ($Result.ErrorDetected) { 'Yes' } else { 'No' })" -ForegroundColor $(if ($Result.ErrorDetected) { 'Red' } else { 'Green' })
    
    if ($Result.BootStages.Count -gt 0) {
        Write-Host "`nBoot Stages:" -ForegroundColor Green
        foreach ($stage in $Result.BootStages) {
            $timestamp = $stage.Timestamp.ToString("HH:mm:ss.fff")
            Write-Host "  [$timestamp] $($stage.Stage)" -ForegroundColor Green
        }
    }
    
    if (Test-Path $ErrorLog) {
        $errorContent = Get-Content $ErrorLog -Raw
        if ($errorContent -and $errorContent.Trim()) {
            Write-Host "`nQEMU Errors:" -ForegroundColor Red
            $errorContent.Split("`n") | ForEach-Object {
                if ($_.Trim()) {
                    Write-Host "  $_" -ForegroundColor Red
                }
            }
        }
    }
    
    Write-Host "`nLog Files:" -ForegroundColor Cyan
    if ($SaveLogs) {
        Write-Host "  Output: $OutputLog" -ForegroundColor White
        Write-Host "  Errors: $ErrorLog" -ForegroundColor White
        Write-Host "  Analysis: $AnalysisLog" -ForegroundColor White
    } else {
        Write-Host "  Logs cleaned up (use -SaveLogs to preserve)" -ForegroundColor Gray
    }
    
    Write-Host "="*60 -ForegroundColor Cyan
}

function Test-QemuAvailability {
    if (-not (Get-Command "qemu-system-x86_64" -ErrorAction SilentlyContinue)) {
        Write-TestLog "QEMU not found in PATH" "ERROR"
        Write-Host "Please install QEMU:" -ForegroundColor Yellow
        Write-Host "  Windows: Download from https://www.qemu.org/download/" -ForegroundColor White
        Write-Host "  WSL: sudo apt install qemu-system-x86" -ForegroundColor White
        return $false
    }
    
    $qemuVersion = qemu-system-x86_64 --version 2>&1 | Select-Object -First 1
    Write-TestLog "QEMU found: $qemuVersion" "SUCCESS"
    return $true
}

# Main execution
Write-Host "AykenOS QEMU Test Runner" -ForegroundColor Green
Write-Host "Author: Kenan AY" -ForegroundColor Gray
Write-Host ""

if (-not (Test-QemuAvailability)) {
    exit 1
}

$testSuccess = Start-QemuTest -TestName $TestName

if ($testSuccess) {
    Write-TestLog "Test completed successfully" "SUCCESS"
    exit 0
} else {
    Write-TestLog "Test failed" "ERROR"
    exit 1
}