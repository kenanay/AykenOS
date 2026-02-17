#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"
$IMG = "EFI.img"

Write-Host "[*] Creating EFI Disk Image ($IMG)..." -ForegroundColor Cyan

# Strategy 1: Try WSL (Preferred if mtools missing on Host)
if (Get-Command "wsl" -ErrorAction SilentlyContinue) {
    Write-Host "  -> Using WSL to generate image..." -ForegroundColor Gray
    
    # Check for mtools in WSL and attempt auto-install if missing
    wsl which mformat >$null 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "mtools not found in WSL. Attempting automatic installation..."
        wsl sh -c "if command -v apt-get >/dev/null 2>&1; then (command -v sudo >/dev/null 2>&1 && sudo apt-get update && sudo apt-get install -y mtools) || (apt-get update && apt-get install -y mtools); fi"
    }

    # Run the shell script inside WSL
    wsl bash "tools/build/make_efi_img.sh"
    if ($LASTEXITCODE -eq 0) {
        exit 0
    }
    Write-Warning "WSL build failed. Checking for native Windows tools..."
}

# Strategy 2: Try Native mtools (if installed via Chocolatey/MSYS2 and in PATH)
if ((Get-Command "mformat" -ErrorAction SilentlyContinue) -and (Get-Command "mcopy" -ErrorAction SilentlyContinue)) {
    Write-Host "  -> Using native mtools..." -ForegroundColor Gray
    
    # Create empty file (64MB)
    $f = [System.IO.File]::Create($IMG)
    $f.SetLength(64 * 1024 * 1024)
    $f.Close()
    
    # Run mtools commands
    # Note: Quoting "::" to avoid PowerShell parsing issues
    mformat -i $IMG "::"
    mmd -i $IMG "::EFI"
    mmd -i $IMG "::EFI/BOOT"
    mcopy -i $IMG "bootloader/efi/BOOTX64.EFI" "::EFI/BOOT/"
    mcopy -i $IMG "kernel.elf" "::"
    
    Write-Host "[*] EFI.img ready!" -ForegroundColor Green
    exit 0
}

Write-Error "Could not find 'mtools' (or WSL build failed). Cannot create EFI image."
Write-Host "Please install mtools (e.g. via MSYS2 or Chocolatey) or ensure 'mtools' is installed inside WSL."
Write-Host "WSL Hint: If 'sudo' is missing, try: apt-get update && apt-get install -y mtools"
exit 1
