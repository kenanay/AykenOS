# Architecture Decision Records

Bu dizin freeze donemindeki mimari karar kayitlarinin resmi arsividir.
Aktif tek-maintainer modeli
`20260525-single-maintainer-authority-model.md` ile tanimlanir. Dizin adi
tarihsel kayit yolu olarak korunur; var olmayan bir inceleme kurulu iddiasi
uretmez.

## Rules

1. Her karar ayrı dosya olmalı.
2. Karar dosyasi RFC/Waiver linki veya `N/A` gerekcesi icermeli.
3. Karar sonucu acik olmali: `approved | rejected | deferred`.
4. Karar tarihi ve gecerli insan otoritesinin imzasi zorunlu.
5. Bagimsiz reviewer ancak atanabilir, farkli insan otoritesi gercekte
   mevcutsa iddia edilebilir.

## File Naming

`YYYYMMDD-<decision-title>.md`

## Current Records

1. `20260214-scheduler-arbitration-contract.md` - scheduler arbitration contract strict mode
2. `20260305-runtime-integration-order.md` - phase-gated runtime integration order freeze
3. `20260305-two-level-authority-scheduler.md` - distributed two-level scheduler authority freeze
4. `20260525-single-maintainer-authority-model.md` - current repository governance authority model
