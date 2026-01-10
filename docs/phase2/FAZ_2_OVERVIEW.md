# AykenOS Faz 2 Overview

Felsefe: "AI-native, veri konteyner OS". Faz 2 odağı; veri konteyneri + meta-FS, hiyerarşik DSL kabuk ve bunları çalıştıran hafif runtime çekirdeğini ayağa kaldırmak. AI/GL/çoklu mimari bu fazda yalnızca iskelet ve not seviyesinde.

Scope (Faz 2):
- P0: Veri konteyneri meta-katmanı (tip, şema, izin, embedding alanları); POSIX’le uyumlu hibrit görünüm.
- P0: Hiyerarşik DSL kabuk (> / >> / >[ ]) ve parser; data.<container> add/query akışı.
- P0: Hafif runtime/dispatcher (kullanıcı modu) → RAM içi konteyner/ABDF erişimi; BCIB yalnızca opsiyonel ara temsil.
- P0: Ring0 minimal syscall yüzeyi (10 syscall: map/unmap/switch/submit_execution/wait_result/interrupt_return/time_query/cap_bind/cap_revoke/exit); scheduler/VFS/DevFS/AI runtime politika katmanlarının Ring3’e taşınması, mevcut POSIX-benzeri syscall setinin kademeli kaldırılması.
- P1: Tabular + log/text veri modülleri (create/add/query + basit filtre), RAM içi depo.
- P1: Demo REPL (AI’sız), meta senkronizasyon kontrolleri.
- P2: UI render stub (ui.render log veya basit çizim), multi-arch notları, ai.ask stub.

Milestones:
- M1: Meta-model taslağı (konteyner tip/şema/izin/embedding) + dokümantasyon.
- M2: DSL grammar + parser + hata mesajları; prompt: `>`/`>>`/`>[ ]`.
- M3: Runtime/dispatcher + veri modülleri (tabular/log/text) RAM içi; unit testler.
- M4: Demo REPL senaryosu çalışır (create/add/query, hata durumları).
- M5 (opsiyonel): ui.render stub + ai.ask stub + notlar; multi-arch derleme notu.

Bağımlılıklar / Notlar:
- Faz 1 ABDF/BCIB taslakları rehber; Faz 2’de BCIB kullanımını opsiyonel tut, DSL’den doğrudan çağrıyı da destekle.
- Faz 1 kodu POSIX syscall + kernel içi VFS/DevFS/AI runtime içeriyor; Faz 2’de bunlar user-mode runtime’a aktarılıp Ring0 yalnız mekanizma hâline getirilecek.
- Build env: Rust/C toolchain; host’ta çalıştırılabilir hedef öncelikli. Cross/x86_64-aarch64 ve QEMU sadece not olarak.***
