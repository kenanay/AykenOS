# AykenOS Faz 1 - Özet ve Durum

**Oluşturan:** Kenan AY  
**Tarih:** 01 Ocak 2026  
**Durum:** Faz 1 çoğunlukla tamamlandı; Ring3 geçişi koddaki IRET yolunda test bekliyor.

---

## Faz 1 Hedefleri (Kritik)
1) Bellek & Boot: UEFI PML4 + higher-half paging  
2) Syscalls & Kullanıcı Modu: INT 0x80, temel çağrılar  
3) Zamanlayıcı & Çoklu Görev: Preemption, görev ekleme  
4) Aygıt/DevFS: Temel /dev düğümleri  
5) Test: QEMU/derleme ile doğrulama

---

## Uygulanan Değişiklikler
- **Bootloader:** `bootloader/efi/paging.c` PML4 oluşturuyor, 1GB identity + higher-half kernel/FB map, CR3 yüklüyor, `boot_info.pml4_phys` set.
- **Kernel Paging:** `kernel/mm/paging.c` PML4 devralma, user PML4 klonlama, identity drop.
- **Syscall:** `kernel/sys/syscall.c` INT 0x80 (DPL=3), read/write/open/close/exit.
- **Scheduler:** `kernel/sched/sched.c` ready/blocked kuyrukları, preemption; `sched_add_task` dolu.
- **Process/Ring3:** `kernel/arch/x86_64/context_switch.asm` CS=0x23 ise IRET çerçevesi kurup CPL=3’e geçiyor; TSS.rsp0 güncelleniyor. `proc_create_user_process` user CS/SS, PML4 ve stack kuruyor. `switch_to_user_mode` helper’ı kullanılmıyor ve selector’ları ters; scheduler path’i doğru.
- **DevFS:** `kernel/fs/devfs.c` /dev/null, /dev/zero, /dev/console kayıtlı; ops API var. Gerçek input sürücüleri yok.
- **VFS/Console:** TAR initrd’den okuma; framebuffer konsol aktif.

---

## Doğrulama Durumu
- Kod incelemesi: Ring3 IRET yolu ve TSS güncellemesi eklendi.  
- Eksik/Test Bekleyen: QEMU’da kullanıcı modu + `int 0x80` çağrısı ve kesme dönüşlerinin doğrulanması; `switch_to_user_mode` fonksiyonunda SS/CS sabitleri ters (kullanılmıyorsa kaldırılabilir).  
- Build: Araç zinciri ve QEMU entegrasyon testi yapılmadı; README/PROJECT_STATUS_REPORT bunu yansıtıyor.
- Mimari uyumsuzluk (Faz 2 borcu): POSIX-benzeri syscall yüzeyi + VFS/DevFS ve scheduler politikaları Ring0’da; minimal 10 syscall yüzeyi ve politika-in-Ring3 ayrıştırması Faz 2’de yapılacak.

---

## Kalan İşler (Faz 1 Kapanışı için)
1) QEMU’da derleme/boot: Ring3 kullanıcı süreci çalışıp `int 0x80` ile syscall yapabiliyor mu kontrol et.  
2) `switch_to_user_mode` helper’ı ya düzelt (CS=0x23, SS=0x1B) ya da kaldır; scheduler path tek kaynak olsun.  
3) DevFS: gerçek giriş sürücüleri (klavye/serial) yok; Faz 2’ye taşınabilir.  
4) Mimari geçişi hazırla: POSIX syscall/VFS/AI runtime gibi Ring0’daki politika katmanlarını Faz 2’de user-mode runtime’a taşı; Ring0 yalnızca mekanizma kalsın.  
5) Raporlar: PROJECT_STATUS_REPORT/README ile tutarlı tek durum — “Kodda Ring3 yolu mevcut, test bekliyor; Faz 1 %~85”.

---

## Satır/Sürüm Notları
- proc.c ~700 satır; devfs.c ~200+ satır; context_switch.asm ~100 satır. (Raporlar arasında farklı satır sayıları kaldırıldı.)
- Tarih standardı: 01.01.2026 kullanılmalı.

---

## Sonuç
Faz 1’in kod tarafındaki kritik bileşenleri hazır; kullanıcı moduna gerçek geçiş kodda var ancak henüz koşum/test doğrulaması yapılmadı. Raporlar test bekleyen durumu yansıtacak şekilde güncellendi. Faz 1’i kapatmak için QEMU doğrulaması ve küçük temizlik (switch_to_user_mode selector düzeltmesi) gereklidir.*** End Patch
