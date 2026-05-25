## Summary
<!-- Kisa ve net aciklama -->

## Type
- [ ] `feat:` Yeni ozellik
- [ ] `fix:` Bug duzeltme
- [ ] `refactor:` Davranis degistirmeyen iyilestirme
- [ ] `ci:` CI/tooling degisikligi
- [ ] `docs:` Dokumantasyon

## Validation Checklist
- [ ] `./scripts/dev_loop.sh` -> PASS
- [ ] `./scripts/check_naming_compliance.sh` -> PASS
- [ ] `./scripts/check_evidence_isolation.sh` -> PASS
- [ ] Zorunlu remote CI/evidence gate'leri PASS

## Evidence
<!-- Istege bagli: log kesiti, test ciktisi -->

```
# Buraya evidence ciktisi yazilabilir
```

## Constitutional Compliance
- [ ] NON_OVERRIDABLE kurallarina uygun
- [ ] Phase Matrix kontrolu yapildi
- [ ] Memory contract ihlali yok
- [ ] Korunmus governance degisikligi varsa Kenan AY maintainer karar kaydi bagli

## Authority Boundary

- `CODEOWNERS`, `@kenanay` icin hesap verebilir sahiplik kaydidir; bagimsiz
  self-review iddiasi degildir.
- Zorunlu remote constitutional CI/evidence sonucu, maintainer kararinin
  yerine gecmez ve maintainer karari da basarisiz CI sonucunu bypass edemez.
- Aktif freeze sirasinda yeni feature mainline'a merge edilmez.

## Notes
<!-- Ek bilgiler, baglam, trade-off'lar -->
