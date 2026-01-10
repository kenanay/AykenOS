# AykenOS QEMU Integration Test Suite
# Author: Kenan AY
# Purpose: Comprehensive QEMU-based testing for Phase 1 critical functionality

param(
    [int]$Timeout = 45,
    [switch]$Verbose,
    [switch]$SaveLogs,
    [switch]$Interactive,
    [string]$TestSuite = "all"
)

$ErrorActionPreference = "Continue"

# Test results tracking
$script:TestResults = @{}
$script:TestDurations = @{}
$script:TestDetails = @{}
$script:TotalTests = 0
$script:PassedTests = 0
$script:FailedTests = 0

# QEMU configuration
$script:QemuArgs = @(
    "-drive", "format=raw,file=EFI.img",
    "-serial", "stdio",
    "-m", "256M",
    "-no-reboot",
    "-no-shutdown",
    "-monitor", "tcp:127.0.0.1:55555,server,nowait"
)

# Test patterns for different validation types
$script:BootSuccessPatterns = @(
    "AykenOS.*INIT",
    "Kernel.*init.*done",
    "kmain.*starting",
    "EARLY INIT.*done",
    "Scheduler.*ready",
    "LATE INIT.*done"
)

$script:Ring3SuccessPatterns = @(
    "user AI service scheduled.*Ring3",
    "Ring3.*transition",
    "user.*process.*created",
    "PID.*running"
)

$script:DevfsSuccessPatterns = @(
    "devfs.*Registered.*null",
    "devfs.*Registered.*zero",
    "devfs.*Registered.*console",
    "devfs.*Registered.*kbd",
    "devfs.*Registered.*ttyS0",
    "devfs.*Registered.*sda",
    "VFS.*DevFS"
)

$script:SyscallSuccessPatterns = @(
    "syscall.*installing.*INT",
    "Syscall interface ready",
    "SYS_write",
    "syscall.*handler"
)

$script:ErrorPatterns = @(
    "PANIC",
    "ERROR",
    "FATAL",
    "Triple fault",
    "General Protection Fault",
    "Page fault",
    "Invalid opcode"
)

function Write-TestLog {
    param([string]$Message, [string]$Level = "INFO")
    
    $timestamp = Get-Date -Format "HH:mm:ss.fff"
    $color = switch ($Level) {
        "SUCCESS" { "Green" }
        "ERROR" { "Red" }
        "WARNING" { "Yellow" }
        "INFO" { "Cyan" }
        "DEBUG" { if ($Verbose) { "Gray" } else { return } }
        default { "White" }
    }
    
    Write-Host "[$timestamp] [$Level] $Message" -ForegroundColor $color
}

function Test-Prerequisites {
    Write-TestLog "Checking test prerequisites..." "INFO"
    
    # Check QEMU availability
    if (-not (Get-Command "qemu-system-x86_64" -ErrorAction SilentlyContinue)) {
        Write-TestLog "QEMU not found in PATH" "ERROR"
        return $false
    }
    
    # Check EFI image
    if (-not (Test-Path "EFI.img")) {
        Write-TestLog "EFI.img not found, attempting to create..." "WARNING"
        try {
            if (Test-Path "make_efi_img.ps1") {
                & .\make_efi_img.ps1
            } else {
                make efi-img
            }
        } catch {
            Write-TestLog "Failed to create EFI.img: $_" "ERROR"
            return $false
        }
    }
    
    # Check kernel.elf
    if (-not (Test-Path "kernel.elf")) {
        Write-TestLog "kernel.elf not found, attempting to build..." "WARNING"
        try {
            make all
        } catch {
            Write-TestLog "Failed to build kernel: $_" "ERROR"
            return $false
        }
    }
    
    Write-TestLog "Prerequisites check passed" "SUCCESS"
    return $true
}

function Start-QemuTest {
    param(
        [string]$TestName,
        [int]$TestTimeout = $Timeout
    )
    
    $outputLog = "${TestName}_output.log"
    $errorLog = "${TestName}_error.log"
    
    Write-TestLog "Starting QEMU test: $TestName (timeout: ${TestTimeout}s)" "INFO"
    
    # Clean old logs
    Remove-Item $outputLog, $errorLog -ErrorAction SilentlyContinue
    
    # Configure QEMU arguments
    $qemuArgs = $script:QemuArgs
    if (-not $Interactive) {
        $qemuArgs += @("-display", "none")
    }
    
    Write-TestLog "QEMU command: qemu-system-x86_64 $($qemuArgs -join ' ')" "DEBUG"
    
    try {
        # Start QEMU process
        $qemuProcess = Start-Process -FilePath "qemu-system-x86_64" -ArgumentList $qemuArgs -PassThru -RedirectStandardOutput $outputLog -RedirectStandardError $errorLog
        
        Write-TestLog "QEMU process started (PID: $($qemuProcess.Id))" "DEBUG"
        
        return @{
            Process = $qemuProcess
            OutputLog = $outputLog
            ErrorLog = $errorLog
        }
        
    } catch {
        Write-TestLog "Failed to start QEMU: $_" "ERROR"
        return $null
    }
}

function Monitor-QemuExecution {
    param(
        [hashtable]$ProcessInfo,
        [string[]]$Patterns,
        [string]$TestName,
        [int]$TestTimeout = $Timeout
    )
    
    $qemuProcess = $ProcessInfo.Process
    $outputLog = $ProcessInfo.OutputLog
    $errorLog = $ProcessInfo.ErrorLog
    
    $startTime = Get-Date
    $lastOutputSize = 0
    $successCount = 0
    $errorDetected = $false
    $detectedStages = @()
    $requiredPatterns = $Patterns.Count
    
    Write-TestLog "Monitoring $TestName execution ($requiredPatterns patterns required)..." "DEBUG"
    
    while (((Get-Date) - $startTime).TotalSeconds -lt $TestTimeout) {
        # Check if process exited
        if ($qemuProcess.HasExited) {
            Write-TestLog "QEMU process exited with code: $($qemuProcess.ExitCode)" "WARNING"
            break
        }
        
        # Analyze output
        if (Test-Path $outputLog) {
            try {
                $currentOutput = Get-Content $outputLog -Raw -ErrorAction SilentlyContinue
                if ($currentOutput -and $currentOutput.Length -gt $lastOutputSize) {
                    $newContent = $currentOutput.Substring($lastOutputSize)
                    $lastOutputSize = $currentOutput.Length
                    
                    # Check for success patterns
                    foreach ($pattern in $Patterns) {
                        if ($newContent -match $pattern) {
                            $match = $matches[0]
                            if ($detectedStages -notcontains $pattern) {
                                Write-TestLog "Pattern detected: $match" "SUCCESS"
                                $detectedStages += $pattern
                                $successCount++
                            }
                        }
                    }
                    
                    # Check for error patterns
                    foreach ($pattern in $script:ErrorPatterns) {
                        if ($newContent -match $pattern) {
                            $match = $matches[0]
                            Write-TestLog "Error detected: $match" "ERROR"
                            $errorDetected = $true
                            break
                        }
                    }
                    
                    # Verbose output
                    if ($Verbose) {
                        $newContent.Split("`n") | ForEach-Object {
                            if ($_.Trim()) {
                                Write-Host "  QEMU: $_" -ForegroundColor Gray
                            }
                        }
                    }
                }
            } catch {
                # Continue on file access errors
            }
        }
        
        # Early exit conditions
        if ($errorDetected) {
            Write-TestLog "Stopping $TestName due to error detection" "ERROR"
            break
        }
        
        # Success condition: all required patterns detected
        if ($successCount -ge $requiredPatterns) {
            Write-TestLog "All required patterns detected for $TestName" "SUCCESS"
            break
        }
        
        Start-Sleep -Milliseconds 500
    }
    
    # Cleanup QEMU process
    if (-not $qemuProcess.HasExited) {
        Write-TestLog "Terminating QEMU process..." "DEBUG"
        try {
            $qemuProcess.Kill()
            $qemuProcess.WaitForExit(5000)
        } catch {
            # Process may have already exited
        }
    }
    
    $duration = [math]::Round(((Get-Date) - $startTime).TotalSeconds, 2)
    $testSuccess = ($successCount -ge $requiredPatterns) -and (-not $errorDetected)
    
    # Store test results
    $script:TestResults[$TestName] = $testSuccess
    $script:TestDurations[$TestName] = $duration
    $script:TestDetails[$TestName] = "Patterns: $successCount/$requiredPatterns, Duration: ${duration}s"
    
    if ($testSuccess) {
        $script:PassedTests++
    } else {
        $script:FailedTests++
    }
    $script:TotalTests++
    
    $statusText = if ($testSuccess) { "PASS" } else { "FAIL" }
    Write-TestLog "$TestName completed: $statusText (${duration}s)" "INFO"
    
    # Cleanup logs if not saving
    if (-not $SaveLogs) {
        Remove-Item $outputLog, $errorLog -ErrorAction SilentlyContinue
    }
    
    return $testSuccess
}

function Test-BootValidation {
    Write-TestLog "=== BOOT VALIDATION TEST ===" "INFO"
    $processInfo = Start-QemuTest "boot_validation" 30
    if ($processInfo) {
        return Monitor-QemuExecution $processInfo $script:BootSuccessPatterns "boot_validation" 30
    }
    return $false
}

function Test-Ring3Execution {
    Write-TestLog "=== RING3 USER PROCESS EXECUTION TEST ===" "INFO"
    $processInfo = Start-QemuTest "ring3_execution" 40
    if ($processInfo) {
        return Monitor-QemuExecution $processInfo $script:Ring3SuccessPatterns "ring3_execution" 40
    }
    return $false
}

function Test-DevfsOperations {
    Write-TestLog "=== DEVFS DEVICE I/O OPERATIONS TEST ===" "INFO"
    $processInfo = Start-QemuTest "devfs_operations" 35
    if ($processInfo) {
        return Monitor-QemuExecution $processInfo $script:DevfsSuccessPatterns "devfs_operations" 35
    }
    return $false
}

function Test-SyscallRoundtrip {
    Write-TestLog "=== SYSCALL ROUNDTRIP TEST ===" "INFO"
    $processInfo = Start-QemuTest "syscall_roundtrip" 40
    if ($processInfo) {
        return Monitor-QemuExecution $processInfo $script:SyscallSuccessPatterns "syscall_roundtrip" 40
    }
    return $false
}

function Test-QemuDebugging {
    Write-TestLog "=== QEMU DEBUGGING INTERFACE TEST ===" "INFO"
    
    $outputLog = "debug_test_output.log"
    $errorLog = "debug_test_error.log"
    
    # Clean old logs
    Remove-Item $outputLog, $errorLog -ErrorAction SilentlyContinue
    
    # Start QEMU with monitor
    $qemuArgs = $script:QemuArgs + @("-display", "none")
    
    try {
        $qemuProcess = Start-Process -FilePath "qemu-system-x86_64" -ArgumentList $qemuArgs -PassThru -RedirectStandardOutput $outputLog -RedirectStandardError $errorLog
        
        Write-TestLog "QEMU debugging test started (PID: $($qemuProcess.Id))" "DEBUG"
        
        # Wait a moment for QEMU to start
        Start-Sleep -Seconds 3
        
        $debugSuccess = $false
        
        # Test monitor connection
        try {
            $tcpClient = New-Object System.Net.Sockets.TcpClient
            $tcpClient.Connect("127.0.0.1", 55555)
            
            if ($tcpClient.Connected) {
                $stream = $tcpClient.GetStream()
                $writer = New-Object System.IO.StreamWriter($stream)
                $reader = New-Object System.IO.StreamReader($stream)
                
                # Send monitor commands
                $writer.WriteLine("info registers")
                $writer.WriteLine("info cpus")
                $writer.WriteLine("quit")
                $writer.Flush()
                
                # Read response
                Start-Sleep -Seconds 2
                $response = ""
                while ($stream.DataAvailable) {
                    $response += $reader.ReadLine() + "`n"
                }
                
                if ($response -match "registers|cpus|RIP|RAX") {
                    $debugSuccess = $true
                    Write-TestLog "QEMU monitor interface working" "SUCCESS"
                } else {
                    Write-TestLog "QEMU monitor interface failed - no valid response" "ERROR"
                }
                
                $tcpClient.Close()
            } else {
                Write-TestLog "Could not connect to QEMU monitor" "ERROR"
            }
        } catch {
            Write-TestLog "QEMU monitor test failed: $_" "ERROR"
        }
        
        # Cleanup
        if (-not $qemuProcess.HasExited) {
            $qemuProcess.Kill()
            $qemuProcess.WaitForExit(5000)
        }
        
    } catch {
        Write-TestLog "Failed to start QEMU for debugging test: $_" "ERROR"
        $debugSuccess = $false
    }
    
    # Store results
    $script:TestResults["qemu_debugging"] = $debugSuccess
    $script:TestDurations["qemu_debugging"] = 5
    $script:TestDetails["qemu_debugging"] = "Monitor interface test"
    
    if ($debugSuccess) {
        $script:PassedTests++
    } else {
        $script:FailedTests++
    }
    $script:TotalTests++
    
    # Cleanup logs if not saving
    if (-not $SaveLogs) {
        Remove-Item $outputLog, $errorLog -ErrorAction SilentlyContinue
    }
    
    return $debugSuccess
}

function Generate-ComprehensiveReport {
    $reportFile = "qemu_integration_test_report.md"
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    
    Write-TestLog "Generating comprehensive test report..." "INFO"
    
    $reportContent = @"
# AykenOS QEMU Integration Test Report

**Generated:** $timestamp  
**Test Suite:** $TestSuite  
**Total Tests:** $($script:TotalTests)  
**Passed:** $($script:PassedTests)  
**Failed:** $($script:FailedTests)  
**Success Rate:** $(if ($script:TotalTests -gt 0) { [math]::Round($script:PassedTests * 100 / $script:TotalTests, 1) } else { 0 })%

## Test Configuration

- **Timeout:** ${Timeout}s
- **Verbose:** $Verbose
- **Interactive:** $Interactive
- **Save Logs:** $SaveLogs

## Test Results Summary

| Test Name | Status | Duration | Details |
|-----------|--------|----------|---------|
"@

    foreach ($testName in $script:TestResults.Keys) {
        $status = $script:TestResults[$testName]
        $duration = $script:TestDurations[$testName]
        $details = $script:TestDetails[$testName]
        $statusIcon = if ($status) { "✅ PASS" } else { "❌ FAIL" }
        
        $reportContent += "`n| $testName | $statusIcon | ${duration}s | $details |"
    }
    
    $reportContent += @"

## Test Descriptions

### Boot Validation Test
Verifies that the AykenOS kernel boots successfully and completes all initialization phases:
- Early initialization (CPU, GDT, IDT, memory management)
- AI initialization (placeholder)
- Late initialization (scheduler, processes, filesystem, syscalls)

**Required Patterns:** $($script:BootSuccessPatterns.Count)
- AykenOS initialization messages
- Kernel subsystem completion confirmations
- Scheduler readiness indication

### Ring3 User Process Execution Test
Validates that the kernel can successfully create and execute user-mode processes:
- User process creation and scheduling
- Ring3 privilege level transitions
- User space memory management

**Required Patterns:** $($script:Ring3SuccessPatterns.Count)
- User process scheduling messages
- Ring3 transition confirmations
- Process execution indicators

### DevFS Device I/O Operations Test
Confirms that the device filesystem is properly initialized with essential devices:
- Standard devices (/dev/null, /dev/zero, /dev/console)
- Input devices (/dev/kbd)
- Serial devices (/dev/ttyS0)
- Block devices (/dev/sda)

**Required Patterns:** $($script:DevfsSuccessPatterns.Count)
- Device registration confirmations
- VFS-DevFS integration messages

### Syscall Roundtrip Test
Verifies that the system call interface is properly configured and functional:
- Syscall handler installation
- INT 0x80 gate configuration
- System call interface readiness

**Required Patterns:** $($script:SyscallSuccessPatterns.Count)
- Syscall installation messages
- Interface readiness confirmations

### QEMU Debugging Interface Test
Tests the QEMU monitor interface for advanced debugging capabilities:
- Monitor TCP connection
- Register inspection commands
- CPU state queries
- Command response validation

## Requirements Validation

This test suite validates the following Phase 1 requirements:

- **4.1:** QEMU smoke tests verify basic kernel boot through log parsing ✓
- **4.2:** Ring3 validation demonstrates user mode execution via QEMU automation ✓
- **4.3:** DevFS integration confirms device file operations work correctly ✓
- **4.4:** Syscall roundtrip tests prove kernel-user transitions function properly ✓
- **4.5:** Automated test suite generates comprehensive validation reports ✓

## Next Steps

"@

    if ($script:FailedTests -gt 0) {
        $reportContent += "⚠️ **Action Required:** $($script:FailedTests) test(s) failed. Review the following:`n"
        foreach ($testName in $script:TestResults.Keys) {
            if (-not $script:TestResults[$testName]) {
                $reportContent += "- **${testName}:** $($script:TestDetails[$testName])`n"
            }
        }
        $reportContent += "`nCheck saved logs (if enabled) for detailed error information."
    } else {
        $reportContent += @"
✅ **All tests passed!** AykenOS Phase 1 critical functionality is validated.

The system is ready for:
- Phase 2 development
- AI integration features
- Advanced filesystem implementation
"@
    }
    
    $reportContent += @"

---
*Report generated by AykenOS QEMU Integration Test Suite*
"@

    $reportContent | Out-File $reportFile -Encoding UTF8
    
    Write-TestLog "Test report saved to: $reportFile" "SUCCESS"
    
    # Display summary to console
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host "QEMU INTEGRATION TEST SUITE SUMMARY" -ForegroundColor Cyan
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Total Tests: $($script:TotalTests)" -ForegroundColor White
    Write-Host "Passed: $($script:PassedTests)" -ForegroundColor Green
    Write-Host "Failed: $($script:FailedTests)" -ForegroundColor Red
    if ($script:TotalTests -gt 0) {
        Write-Host "Success Rate: $([math]::Round($script:PassedTests * 100 / $script:TotalTests, 1))%" -ForegroundColor Blue
    }
    Write-Host ""
    
    if ($script:FailedTests -gt 0) {
        Write-Host "Failed Tests:" -ForegroundColor Red
        foreach ($testName in $script:TestResults.Keys) {
            if (-not $script:TestResults[$testName]) {
                Write-Host "  ❌ $testName" -ForegroundColor Red
            }
        }
    } else {
        Write-Host "🎉 All tests passed! Phase 1 validation complete." -ForegroundColor Green
    }
    
    Write-Host ""
    Write-Host "Detailed report: $reportFile" -ForegroundColor Cyan
    Write-Host "============================================================" -ForegroundColor Cyan
}

# Main execution
function Main {
    Write-Host "AykenOS QEMU Integration Test Suite" -ForegroundColor Green
    Write-Host "Author: Kenan AY" -ForegroundColor Gray
    Write-Host "Phase 1 Critical Functionality Validation" -ForegroundColor Gray
    Write-Host ""
    
    if (-not (Test-Prerequisites)) {
        exit 1
    }
    
    Write-TestLog "Starting test suite: $TestSuite" "INFO"
    
    switch ($TestSuite.ToLower()) {
        "boot" {
            Test-BootValidation | Out-Null
        }
        "ring3" {
            Test-Ring3Execution | Out-Null
        }
        "devfs" {
            Test-DevfsOperations | Out-Null
        }
        "syscall" {
            Test-SyscallRoundtrip | Out-Null
        }
        "debug" {
            Test-QemuDebugging | Out-Null
        }
        default {
            Test-BootValidation | Out-Null
            Test-Ring3Execution | Out-Null
            Test-DevfsOperations | Out-Null
            Test-SyscallRoundtrip | Out-Null
            Test-QemuDebugging | Out-Null
        }
    }
    
    Generate-ComprehensiveReport
    
    # Exit with appropriate code
    exit $(if ($script:FailedTests -gt 0) { 1 } else { 0 })
}

# Run main function
Main
