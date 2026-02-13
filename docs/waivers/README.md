# Waiver Registry

Bu dizin freeze dönemindeki waiver kayıtlarının tek kaynağıdır.

## Rules

1. Her waiver ayrı bir markdown dosyası olmalı.
2. `expiry_date` zorunlu.
3. Tracking issue link'i zorunlu.
4. Fix plan ve rollback plan zorunlu.
5. 90 günü aşan waiver otomatik violation kabul edilir.

## File Naming

`YYYYMMDD-<short-title>.md`

Örnek:
`20260213-toolchain-breaking-change.md`

## Required Sections

1. Metadata
2. Risk and impact
3. Gate impact
4. Expiry and closure plan
5. Approval
