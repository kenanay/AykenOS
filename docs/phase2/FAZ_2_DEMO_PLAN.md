# Faz 2 - CLI Demo Plan (No AI)
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

Boot/data setup:
- RAM içi konteynerler: `data.users` (3-4 kayıt), `syslog` (5-6 kayıt).
- Meta kayıtları: tip=tabular/log, şema, izin placeholder (embedding alanı boş).
- (Opsiyonel) ui.scene.demo stub kaydı.

REPL flow (hiyerarşik DSL):
1) `> data.users`
2) `>> create schema=[id:int,name:string,age:int]` (şema varsa no-op)
3) `>> add {"id":3,"name":"Mehmet","age":40}`
4) `>> query filter="age >= 30"`
5) `> syslog` / `>> query filter="level == ERROR"` (basit eşleşme)
6) (Ops) `> ui.scene.demo` / `>> render` -> stub/log
7) (Ops) `> ai` / `>> ask "..."` -> stub mesaj
8) Hatalar: bağlam seçmeden `>>`, bozuk JSON, bilinmeyen alan adı.

Output format:
- Tabular sonuç: basit tablo veya key/value satırları.
- Log sonuç: satır listeleme.
- Stub çıktıları: “[ui.render] not implemented”, “[ai.ask] stub”.

Başarı ölçütleri:
- Parser → dispatcher → handler akışı tüm adımlarda çalışır.
- Bağlam/hata mesajları anlaşılır; çökme yok.
- Opsiyonel BCIB yolu varsa aynı senaryo çalışır, yoksa DSL → direkt çağrı yeterli.***
