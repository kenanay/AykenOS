# Faz 1 Doğrulama Raporu

**Oluşturan:** Kenan AY  
**Tarih:** 01 Ocak 2026  
**Durum:** %85 — Bootloader, paging, syscalls, scheduler, DevFS (temel) tamam; Ring3 IRET yolu kodda mevcut, QEMU/runtime testi bekliyor.

---

## 1) Genel Özet

| Bileşen | Dosya | Durum | Not |
| --- | --- | --- | --- |
| UEFI PML4 | bootloader/efi/paging.c | ✅ | 1GB identity + higher-half + CR3 |
| Kernel Paging | kernel/mm/paging.c | ✅ | PML4 devralma, user PML4 klonu, identity drop |
| Syscall Dispatcher | kernel/sys/syscall.c | ✅ | INT 0x80, read/write/open/close/exit |
| Timer Preemption | kernel/arch/x86_64/timer.c | ✅ | PIT→`sched_yield` |
| Scheduler | kernel/sched/sched.c | ✅ | ready/blocked, `sched_add_task` dolu |
| Ring3 Transition | kernel/arch/x86_64/context_switch.asm | ⏳ | IRET yolu kodda; test pending |
| VFS | kernel/fs/vfs.c | ✅ | TAR initrd okuma |
| DevFS | kernel/fs/devfs.c | ⚠️ | /dev/null, /dev/zero, /dev/console; giriş sürücüleri yok |

---

## 2) Ring3 Durumu
- IRET tabanlı geçiş kodu mevcut: CS=0x23 hedeflenirse SS,RSP,RFLAGS,CS,RIP push + `iretq`; TSS.rsp0 scheduler’da güncelleniyor.  
- `switch_to_user_mode` helper’ı kullanılmıyor ve selector sabitleri ters (CS/SS); kullanılacaksa CS=0x23, SS=0x1B olarak düzeltilmeli veya kaldırılmalı.  
- Runtime doğrulama (QEMU boot + kullanıcı modunda `int 0x80`/interrupt dönüşü) henüz yapılmadı.

---

## 3) DevFS Durumu
- Mevcut: `/dev/null`, `/dev/zero`, `/dev/console`.  
- Eksik: klavye/serial gibi giriş aygıt sürücüleri; VFS mount entegrasyonu.  
- Dosya: `kernel/fs/devfs.c`.

---

## 4) Build/Test Durumu
- Kod incelendi; derleme/QEMU entegrasyon testi yapılmadı.  
- Windows’ta toolchain/QEMU kurulumu gerekiyor (WSL önerilir).

---

## 5) Sonuç
Faz 1’in kritik parçaları kodda hazır; Ring3 yolu ve DevFS temel düğümleri var. Kapanış için QEMU’da kullanıcı modunda çalışma ve syscalls/interruptların doğrulanması gerekir. README/PROJECT_STATUS_REPORT bu duruma göre güncellenmiştir.
