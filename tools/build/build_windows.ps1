# AykenOS Windows Build Script
# Bu script Windows ortamında AykenOS'u derlemek için gerekli araçları kontrol eder ve derleme yapar

Write-Host "AykenOS Windows Build Script" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green

# Gerekli araçları kontrol et
$tools_missing = $false

# Rust kontrolü
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "✓ Rust/Cargo found" -ForegroundColor Green
} else {
    Write-Host "✗ Rust/Cargo not found. Please install from https://rustup.rs/" -ForegroundColor Red
    $tools_missing = $true
}

# LLVM/Clang kontrolü (EFI bootloader için)
if (Get-Command clang -ErrorAction SilentlyContinue) {
    Write-Host "✓ Clang found" -ForegroundColor Green
} else {
    Write-Host "✗ Clang not found. Please install LLVM from https://llvm.org/" -ForegroundColor Red
    $tools_missing = $true
}

# Cross-compiler kontrolü
if (Get-Command x86_64-elf-gcc -ErrorAction SilentlyContinue) {
    Write-Host "✓ x86_64-elf-gcc found" -ForegroundColor Green
} else {
    Write-Host "✗ x86_64-elf-gcc not found." -ForegroundColor Yellow
    Write-Host "  You can install it via:" -ForegroundColor Yellow
    Write-Host "  - WSL2 + Ubuntu: apt install gcc-multilib" -ForegroundColor Yellow
    Write-Host "  - MSYS2: pacman -S mingw-w64-x86_64-gcc" -ForegroundColor Yellow
    Write-Host "  - Or use WSL for cross-compilation" -ForegroundColor Yellow
    $tools_missing = $true
}

# NASM kontrolü
if (Get-Command nasm -ErrorAction SilentlyContinue) {
    Write-Host "✓ NASM found" -ForegroundColor Green
} else {
    Write-Host "✗ NASM not found. Please install from https://www.nasm.us/" -ForegroundColor Red
    $tools_missing = $true
}

if ($tools_missing) {
    Write-Host "`nSome required tools are missing. Please install them first." -ForegroundColor Red
    Write-Host "For a complete setup, consider using WSL2 with Ubuntu." -ForegroundColor Yellow
    exit 1
}

Write-Host "`nAll tools found! Starting build..." -ForegroundColor Green

# Rust bileşenlerini derle
Write-Host "`nBuilding Rust components..." -ForegroundColor Cyan
Set-Location "ayken-core"
cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Rust build failed!" -ForegroundColor Red
    exit 1
}
Set-Location ".."

Write-Host "✓ Rust components built successfully" -ForegroundColor Green

# Kernel ve bootloader derleme (eğer araçlar varsa)
if (Get-Command make -ErrorAction SilentlyContinue) {
    Write-Host "`nBuilding kernel and bootloader..." -ForegroundColor Cyan
    make clean
    make all
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Kernel and bootloader built successfully" -ForegroundColor Green
        
        # EFI image oluştur
        Write-Host "`nCreating EFI image..." -ForegroundColor Cyan
        make efi-img
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ EFI image created successfully" -ForegroundColor Green
            Write-Host "`nBuild completed! You can now:" -ForegroundColor Green
            Write-Host "  - Run with QEMU: make run" -ForegroundColor White
            Write-Host "  - Create USB boot: .\make_usb_boot.ps1" -ForegroundColor White
        }
    } else {
        Write-Host "✗ Kernel/bootloader build failed" -ForegroundColor Red
    }
} else {
    Write-Host "`nMake not found. Kernel build skipped." -ForegroundColor Yellow
    Write-Host "Consider using WSL2 for full build support." -ForegroundColor Yellow
}

Write-Host "`nBuild script completed." -ForegroundColor Green