# AykenOS Roadmap (High-Level)

## Vision
AI-native, veri-odaklı OS: tiplenmiş veri konteynerleri + hiyerarşik DSL kabuk; TinyLLM ajanları kullanıcı modunda; Ring0 yalnızca donanım kapısı ve execution mekanizması.

## Fazlar (Özet)
- Faz 1: Çekirdek temel (UEFI boot, bellek yönetimi, Ring3 geçişi, DevFS iskeleti, temel POSIX-benzeri syscalls, preemptive sched). Durum: tamamlandı/stabilizasyon, QEMU/toolchain doğrulaması bekliyor.
- Faz 2: Ring0 küçültme ve runtime. Minimal 10 syscall yüzeyi (map/unmap/switch/submit_execution/wait_result/interrupt_return/time_query/cap_bind/cap_revoke/exit), scheduler/VFS/DevFS/AI runtime politikalarının kullanıcı moduna taşınması; veri-konteyner + DSL + runtime (AI’sız) ve BCIB v0.2 opsiyonel yol. Demo REPL.
- Faz 3: TinyLLM entegrasyonu kullanıcı modunda. Shell LLM / HW agent / Data LLM iskeleti; ai.ask komutları; BCIB/DSL köprüsü; güvenlik: insan-onaylı politika.
- Faz 4: Donanım genişleme + dashboard. ARM/RISC-V doğrulama, temel sürücüler; UI scene graph render (OpenGL); canlı telemetri panosu.
- Faz 5: Çoklu platform + optimizasyon + dokümantasyon + paketleme; topluluk/beta.
- Faz 6: Ağ yayını, yüksek seviye dil/ortam (WASM/Python), kapsamlı güvenlik politikaları, vizyon tamamlama.

## ABDF/BCIB (Faz 2 hedefleri)
- ABDF v0.2: MetaContainer (name_idx, type_idx, schema_idx, permissions, embedding_idx), SegmentKind meta tablosu, SegmentDescriptor meta_idx+offset+length; header version u16=2 (dokümantasyonda 0.2); builder/decoder uyumu.
- Yerleşim notu: meta_count = seg_count varsayımı; data section 8 byte hizalı ve segment offset’leri data section başlangıcına göredir.
- BCIB v0.2: DSL uyumlu opcode seti (data.create/add/query, ui.render stub, ai.ask stub, end), header (magic/version/opcount), validation. Header: `BCIB` + version=2 (dokümantasyonda 0.2) + instr_count; invalid opcode/header durumları validation ile yakalanır.

## DSL/Runtime (Faz 2)
- Hiyerarşik prompt: `>` bağlam, `>>` aksiyon, `>[ ]` opsiyonel parallel/pipe.
- Parser → dispatcher; RAM içi tabular/log/text konteynerleri; list/help/hata mesajları; demo REPL senaryosu.
- Ring0 minimal syscall yüzeyi; tüm politika (scheduler/VFS/DevFS/AI runtime) kullanıcı modunda.

## Güvenlik ve test
- Faz 2: AI kapalı, stub mesajlar; bağlam doğrulama, şema doğrulama, hatalara açık mesajlar.
- Faz 3+: AI önerileri insan onayı olmadan uygulanmaz; politika motoru planı.

## Notlar
- Versiyonlama: 0.x hızlı iterasyon; 1.0/2.0 format donduğunda.
- Multi-arch ve GL render Faz 4+, opsiyonel POC.
