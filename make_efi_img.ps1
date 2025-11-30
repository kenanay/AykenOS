$img = "EFI.img"
$sizeMB = 64

Write-Host "[*] EFI.img oluşturuluyor..."
fsutil file createnew $img ($sizeMB * 1024 * 1024)

Write-Host "[*] Sanal disk attach ediliyor..."
$disk = Mount-DiskImage -ImagePath $img -PassThru
$drive = ($disk | Get-DiskImage | Get-Volume).DriveLetter + ":"

Write-Host "[*] FAT32 formatlanıyor..."
Format-Volume -DriveLetter $drive.Substring(0,1) -FileSystem FAT32 -Force

Write-Host "[*] EFI klasörleri oluşturuluyor..."
New-Item -Path "$drive\EFI\BOOT" -ItemType Directory -Force | Out-Null

Write-Host "[*] BOOTX64.EFI kopyalanıyor..."
Copy-Item "bootloader\efi\BOOTX64.EFI" "$drive\EFI\BOOT\BOOTX64.EFI"

Write-Host "[*] kernel.elf kopyalanıyor..."
Copy-Item "kernel.elf" "$drive\kernel.elf"

Write-Host "[*] Disk unmount ediliyor..."
Dismount-DiskImage -ImagePath $img

Write-Host "[*] EFI.img hazır!"
