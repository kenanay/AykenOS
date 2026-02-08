# Faz 2 - Runtime / Dispatcher
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

Objectives (Faz 2):
- Hiyerarşik DSL’den gelen istekleri veri konteynerlerine (tabular, log/text) yöneten hafif dispatcher.
- ABDF/BCIB v0.2 ile uyumlu kal; BCIB üretimi/okuması opsiyonel, aynı handler seti DSL çağrıları için de kullanılır.
- RAM içi meta-depo: konteyner kayıtları (tip, şema, izin, embedding/metrik placeholder).
- Ring0 minimal syscall yüzeyini kullanarak (map/unmap/switch/submit_execution/wait_result/interrupt_return/time_query/cap_bind/cap_revoke/exit) tamamen kullanıcı modunda çalışmak; POSIX-benzeri file/syscall’lar kaldırılacak.

Akış (pseudo):
- REPL → parser → `DispatchRequest { ctx, action, args }`
- Dispatcher:
  - validate ctx (konteyner var mı? şema uyumlu mu?)
  - route to handler (create/add/query/info/render_stub/ai_stub)
  - return result or error string
- Opsiyonel BCIB yolu: parse header/version, decode ops, dispatch aynı handler’lara.

Runtime state (min):
- context: current container (name, type)
- registries: containers (tabular/log/text), schemas, meta, optional ui_scenes
- output sink (console), logger
- ai_stub flag/handler (log-only)

Handlers (min):
- tabular/log/text: create/add/query (tek kolon filtre veya basit AND)
- sys.info (opsiyonel)
- ui.render stub (log; çizim yoksa “not implemented”)
- ai.ask stub (log)

Ring0 arayüzü (execution-centric):
- Yeni syscall seti: map/unmap/switch/submit_execution/wait_result/interrupt_return/time_query/cap_bind/cap_revoke/exit.
- Capability token’ları: `cap_bind`/`cap_revoke` ile execution context’e yetki bağlama; scheduler/policy Ring3’te kalır.
- Ring0 tarafında sadece mekanizma (MMU/interrupt/context switch), hiçbir VFS/DevFS/AI politika kodu bulunmaz.

Validation:
- Bağlam zorunlu: `>>` çağrılarında ctx yoksa hata.
- Şema/alan doğrulama (tabular); filter parse hataları temiz mesaj.
- BCIB varsa: magic/version, opcode aralığı, arg uzunluk sınırı.

Logging/testing:
- Debug log: ctx/action/args; sonuç sayısı veya hata mesajı.
- Unit testler: happy path (create+add+query), bağlam yok hatası, bozuk filter, BCIB unknown opcode (varsa), ui.render/ai.ask stub dönüşü.

Dependencies:
- Meta/ABDF yardımcıları (string pool/offset okuma) yalnızca gerekiyorsa.
- CLI parser isteklerini DispatchRequest’e çevirir; BCIB yolunda aynı handler seti kullanılır.
