# QEMU + OVMF Boot Chain Agresif Debug Planı

**Amaç**: UEFI shell → startup.nsh → BOOTX64.EFI → kernel handoff zincirinde tam olarak nerede koptuğunu tek tek izole etmek.

---

## 1. Önce boot zincirini dört parçaya ayır

Her koşuda **sadece bir halkayı** doğrula:

1. OVMF açılıyor mu?
2. `startup.nsh` gerçekten çalışıyor mu?
3. `BOOTX64.EFI` gerçekten çağrılıyor mu?
4. kernel entry'ye gerçekten geçiliyor mu?

**Hepsini aynı anda debug etme. Bu hata büyütür.**

---

## 2. startup.nsh için sıfır şüpheli sürüm kullan

İlk test dosyası **sadece bu** olsun:

```bash
echo STARTUP_OK
stall 3000000
fs0:
\EFI\BOOT\BOOTX64.EFI
```

Burada hedef:
- serial log'da `STARTUP_OK` görünmeli
- görünmüyorsa `startup.nsh` hiç çalışmıyor

**Bu aşamada kernel umursanmaz.**

---

## 3. EFI image layout'ı körlemesine varsayma, doğrula

Doğrulanacak yapı:
```
/
  startup.nsh
  EFI/
    BOOT/
      BOOTX64.EFI
```

Kontrol listesi:
- `startup.nsh` root'ta mı?
- `BOOTX64.EFI` gerçekten `EFI/BOOT/BOOTX64.EFI` altında mı?
- dosya isimleri doğru mu?
- image üretim script'i sessizce yanlış path'e kopyalıyor mu?

---

## 4. Shell'e düşünce manuel komut testi yap

Shell açıldığında sırayla:

```
fs0:
ls
type startup.nsh
\EFI\BOOT\BOOTX64.EFI
```

### Karar ağacı:

- `fs0:` yoksa → disk mapping sorunu
- `ls` içinde dosyalar yoksa → image layout bozuk
- `type startup.nsh` hata verirse → dosya bozuk
- `BOOTX64.EFI` manuel çalışıyorsa → sorun sadece otomatik startup hattı
- `BOOTX64.EFI` manuel de çalışmıyorsa → bootloader binary / EFI application sorunu

---

## 5. Bootloader'a tek-byte marker koy

`BOOTX64.EFI` entry'sine en erken noktada çok kaba ama kesin marker koy:

- serial
- gerekiyorsa UEFI console output
- debugcon varsa o da

Örnek mantık:

```c
"[UEFI_BOOT_START]"
```

sonra loader ELF açmadan önce:

```c
"[UEFI_LOAD_BEGIN]"
```

kernel entry çağrısından hemen önce:

```c
"[UEFI_JUMP_KERNEL]"
```

**Hedef**: `BOOTX64.EFI` gerçekten çalışıyor mu?

---

## 6. Kernel entry'ye en erken marker koy

Kernel entry'de ilk satırlara:

- debugcon tek karakter
- serial tek satır

Örnek mantık:

```c
K0
[[AYKEN_KERNEL_ENTRY]]
```

Böylece şu ayrımı yaparsın:
- bootloader çalıştı ama kernel'e geçmedi mi?
- yoksa kernel'e geçti ama sonra mı düştü?

---

## 7. Output kanallarını tek tek test et

Şu an debugcon güvenilmez görünüyor. O yüzden sırayı böyle kur:

1. **serial**
2. UEFI shell echo
3. debugcon

Yani önce:
- serial'da `STARTUP_OK`
- sonra serial'da `UEFI_BOOT_START`
- sonra serial'da `KERNEL_ENTRY`

**Debugcon'u yardımcı kanal gibi düşün, ana kanıt kanalı gibi değil.**

---

## 8. QEMU komutunu sadeleştir

Karışık test script yerine tek bir çıplak komut kullan ve her denemede aynı olsun.

Gerekli şeyler:
- readonly `OVMF_CODE.fd`
- writable `OVMF_VARS_run.fd`
- `EFI.img`
- serial log
- debugcon log
- `-nographic`
- yeterli timeout

**Ama ilk doğrulamada 10 farklı seçenek ekleme. Sade tut.**

---

## 9. Her koşudan sonra tek soru sor

Her koşuda **sadece bir karar** ver:

- `startup.nsh` çalıştı mı?
- `BOOTX64.EFI` çalıştı mı?
- kernel entry görüldü mü?
- late init görüldü mü?

**Bir koşudan dört sonuç çıkarmaya çalışma.**

---

## 10. BCIB marker'larına dönmeden önce minimum başarı kriteri

Şunlar görünmeden BCIB debug'a geri dönme:

```
STARTUP_OK
UEFI_BOOT_START
AYKEN_KERNEL_ENTRY
[K][LATE]...
```

**Bu dördü yoksa BCIB log'ları yok diye yorum yapmak erken olur.**

---

## Hızlı karar tablosu

### Senaryo A
`STARTUP_OK` yok → `startup.nsh` çalışmıyor

### Senaryo B
`STARTUP_OK` var, `UEFI_BOOT_START` yok → `BOOTX64.EFI` çağrısı kırık

### Senaryo C
`UEFI_BOOT_START` var, `AYKEN_KERNEL_ENTRY` yok → bootloader → kernel handoff kırık

### Senaryo D
`AYKEN_KERNEL_ENTRY` var, `[K][LATE]8.1 BCIB_WORKER_CREATE` yok → kernel init path / late init hattı kırık

### Senaryo E
`[K][LATE]8.1 ...` var ama `[U][BW_START]` yok → artık gerçek blocker scheduler / first schedule

---

## Şu an en doğru uygulama sırası

1. `startup.nsh` içine `STARTUP_OK`
2. bootloader entry marker
3. kernel earliest entry marker
4. late init marker
5. sonra BCIB worker marker
6. en son userspace `[U][BW_START]`

**Bu sırayı bozma.**

---

## Implementation Checklist

- [ ] `startup.nsh` minimal test version
- [ ] EFI image layout verification
- [ ] Manual shell test procedure
- [ ] Bootloader entry markers
- [ ] Kernel entry markers
- [ ] Serial output priority
- [ ] Simplified QEMU command
- [ ] Single-decision test runs
- [ ] Minimum success criteria met
- [ ] BCIB markers only after boot chain verified
