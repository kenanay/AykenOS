# ============================================================
# AykenOS USB Boot Creator (Windows PowerShell)
# USB belleğe bootable AykenOS yazdırır
# ============================================================

param(
    [Parameter(Mandatory=$false)]
    [string]$DiskNumber,
    
    [Parameter(Mandatory=$false)]
    [switch]$AutoConfirm
)

# Yönetici kontrolü
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "HATA: Bu script yönetici olarak çalıştırılmalı!" -ForegroundColor Red
    Write-Host "Sağ tık -> 'Yönetici olarak çalıştır'" -ForegroundColor Yellow
    exit 1
}

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  AykenOS USB Boot Creator" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# EFI.img kontrolü
if (-not (Test-Path "EFI.img")) {
    Write-Host "HATA: EFI.img bulunamadı!" -ForegroundColor Red
    Write-Host "Önce 'make efi-img' komutunu çalıştırın." -ForegroundColor Yellow
    exit 1
}

$imgSize = (Get-Item "EFI.img").Length / 1MB
Write-Host "[OK] EFI.img bulundu ($([math]::Round($imgSize, 2)) MB)" -ForegroundColor Green
Write-Host ""

# Disk listesi
Write-Host "Mevcut diskler:" -ForegroundColor Yellow
Get-Disk | Format-Table Number, FriendlyName, Size, PartitionStyle -AutoSize

Write-Host ""
Write-Host "UYARI: Seçilen disk tamamen silinecek!" -ForegroundColor Red
Write-Host ""

# Disk seçimi
if (-not $DiskNumber) {
    $DiskNumber = Read-Host "USB disk numarasını girin (örn: 1, 2, 3)"
}

# Disk kontrolü
try {
    $disk = Get-Disk -Number $DiskNumber -ErrorAction Stop
} catch {
    Write-Host "HATA: Disk $DiskNumber bulunamadı!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Seçilen disk:" -ForegroundColor Yellow
Write-Host "  Numara: $($disk.Number)"
Write-Host "  İsim: $($disk.FriendlyName)"
Write-Host "  Boyut: $([math]::Round($disk.Size / 1GB, 2)) GB"
Write-Host "  Tür: $($disk.BusType)"
Write-Host ""

# Onay
if (-not $AutoConfirm) {
    $confirm = Read-Host "Bu diski silmek istediğinizden emin misiniz? (EVET yazın)"
    if ($confirm -ne "EVET") {
        Write-Host "İşlem iptal edildi." -ForegroundColor Yellow
        exit 0
    }
}

Write-Host ""
Write-Host "USB hazırlanıyor..." -ForegroundColor Cyan

# DiskPart script oluştur
$diskpartScript = @"
select disk $DiskNumber
clean
convert gpt
create partition primary
format fs=fat32 quick label="AykenOS"
assign
exit
"@

$scriptPath = "$env:TEMP\ayken_diskpart.txt"
$diskpartScript | Out-File -FilePath $scriptPath -Encoding ASCII

# DiskPart çalıştır
Write-Host "[1/4] Disk temizleniyor ve formatlanıyor..." -ForegroundColor Yellow
$result = diskpart /s $scriptPath 2>&1

if ($LASTEXITCODE -ne 0) {
    Write-Host "HATA: Disk formatlanamadı!" -ForegroundColor Red
    Write-Host $result
    Remove-Item $scriptPath -ErrorAction SilentlyContinue
    exit 1
}

Remove-Item $scriptPath -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# Yeni sürücü harfini bul
$partition = Get-Partition -DiskNumber $DiskNumber | Where-Object { $_.Type -eq 'Basic' }
$driveLetter = $partition.DriveLetter

if (-not $driveLetter) {
    Write-Host "HATA: Sürücü harfi atanamadı!" -ForegroundColor Red
    exit 1
}

Write-Host "[OK] Disk hazır: $($driveLetter):\" -ForegroundColor Green
Write-Host ""

# EFI.img mount et
Write-Host "[2/4] EFI.img mount ediliyor..." -ForegroundColor Yellow

try {
    $mountResult = Mount-DiskImage -ImagePath (Resolve-Path "EFI.img").Path -PassThru -ErrorAction Stop
    $mountedDrive = ($mountResult | Get-Volume).DriveLetter
    
    if (-not $mountedDrive) {
        throw "Mount edilen sürücü bulunamadı"
    }
    
    Write-Host "[OK] EFI.img mount edildi: $($mountedDrive):\" -ForegroundColor Green
    Write-Host ""
    
    # Dosyaları kopyala
    Write-Host "[3/4] Dosyalar kopyalanıyor..." -ForegroundColor Yellow
    
    $source = "$($mountedDrive):\"
    $dest = "$($driveLetter):\"
    
    # EFI klasörünü kopyala
    if (Test-Path "$source\EFI") {
        Copy-Item -Path "$source\EFI" -Destination $dest -Recurse -Force
        Write-Host "  [OK] EFI klasörü kopyalandı" -ForegroundColor Green
    }
    
    # kernel.elf kopyala
    if (Test-Path "$source\kernel.elf") {
        Copy-Item -Path "$source\kernel.elf" -Destination $dest -Force
        Write-Host "  [OK] kernel.elf kopyalandı" -ForegroundColor Green
    }
    
    Write-Host ""
    
    # Unmount
    Write-Host "[4/4] Temizleniyor..." -ForegroundColor Yellow
    Dismount-DiskImage -ImagePath (Resolve-Path "EFI.img").Path | Out-Null
    
} catch {
    Write-Host "HATA: Dosya kopyalama başarısız!" -ForegroundColor Red
    Write-Host $_.Exception.Message
    Dismount-DiskImage -ImagePath (Resolve-Path "EFI.img").Path -ErrorAction SilentlyContinue | Out-Null
    exit 1
}

# Doğrulama
Write-Host ""
Write-Host "Doğrulanıyor..." -ForegroundColor Cyan

$bootFile = "$($driveLetter):\EFI\BOOT\BOOTX64.EFI"
$kernelFile = "$($driveLetter):\kernel.elf"

if ((Test-Path $bootFile) -and (Test-Path $kernelFile)) {
    Write-Host "[OK] BOOTX64.EFI bulundu" -ForegroundColor Green
    Write-Host "[OK] kernel.elf bulundu" -ForegroundColor Green
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Green
    Write-Host "  USB HAZIR!" -ForegroundColor Green
    Write-Host "============================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Sonraki adımlar:" -ForegroundColor Yellow
    Write-Host "  1. USB'yi güvenli çıkar"
    Write-Host "  2. Hedef bilgisayara tak"
    Write-Host "  3. BIOS'ta Secure Boot'u kapat"
    Write-Host "  4. USB'den boot et"
    Write-Host ""
    Write-Host "Detaylı bilgi için: USB_BOOT_GUIDE.md" -ForegroundColor Cyan
} else {
    Write-Host "UYARI: Bazı dosyalar eksik!" -ForegroundColor Yellow
    if (-not (Test-Path $bootFile)) {
        Write-Host "  [X] BOOTX64.EFI bulunamadı" -ForegroundColor Red
    }
    if (-not (Test-Path $kernelFile)) {
        Write-Host "  [X] kernel.elf bulunamadı" -ForegroundColor Red
    }
}

Write-Host ""
