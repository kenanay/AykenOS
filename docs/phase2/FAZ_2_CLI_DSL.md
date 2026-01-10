# Faz 2 - CLI DSL and Parser

Scope (Faz 2):
- Hiyerarşik DSL (> / >> / >[ ]) ile veri konteynerlerini (tabular, log/text) yönetmek.
- Parser → runtime çağrıları; BCIB üretimi opsiyonel (ileride TinyLLM/Shell LLM için köprü).
- REPL: açık hatalar, yardım ve listeleme komutları.

Grammar (sketch):
- Prompt sembolleri:
  - `>`   : global/bağlam seçimi (örn. `> data.users`)
  - `>>`  : seçili bağlamda aksiyon (örn. `>> add {...}`, `>> query "age>30"`)
  - `>[ ]`: paralel/pipe benzeri ifade (opsiyonel, bu fazda log-only olabilir)
- Komut örnekleri:
  - `> data.users` (bağlam seç)
  - `>> create schema=[id:int,name:string,age:int]`
  - `>> add {"id":1,"name":"Ahmet","age":34}`
  - `>> query filter="age > 30"`
  - `> sys.hw` / `>> info` (opsiyonel stub)
  - `> ui.scene.sysdash` / `>> render` (stub/log)
  - `> ai` / `>> ask "..."` (stub; Faz 3 için köprü)

Parser davranışı:
- Toleranslı boşluk; JSON benzeri obje (add), key=value listeleri (schema/filter) desteklenir.
- Bağlam zorunlu: `>>` çağrıları öncesi `>` seçilmemişse hata verir.
- Hata mesajları okunabilir: “bilinmeyen aksiyon”, “bozuk JSON”, “eksik bağlam” vb.

Çıktı/Rota:
- Faz 2’de parser doğrudan runtime API’sine çağrı yapabilir.
- BCIB çıkışı (string pool + opcode) isteğe bağlı; format v0.2 notları ile hizalı.

REPL:
- Prompt: `AykenOS> `, bağlam seçilince `AykenOS[data.users]> ` biçiminde gösterim önerisi.
- Komutlar: exit/quit, help, list.data (konteynerler), list.meta (şema/izin bilgisi).
- Help: sembol kullanımı, örnek akışlar, desteklenen veri tipleri.***
