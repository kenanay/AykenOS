# AykenOS Branch Model

## Branch Yapısı

```
main          → stable, production-ready (KORUMAL)
dev           → entegrasyon branch (opsiyonel)
feature/*     → yeni özellikler
fix/*         → bug fix
refactor/*    → davranış değiştirmeyen iyileştirme
ci/*          → CI/tooling değişiklikleri
```

## Kurallar

### 1. main Branch
- ✅ **Korumalı**: Direkt push YOK
- ✅ **PR zorunlu**: Tüm değişiklikler PR ile
- ✅ **CI geçmeli**: naming + evidence + dev-loop + perf
- ✅ **Review gerekli**: En az 1 onay

### 2. PR Disiplini
- **Küçük PR'lar**: 1 PR = 1 sorumluluk
- **Commit standardı**: `feat:`, `fix:`, `refactor:`, `ci:`, `docs:`
- **Merge stratejisi**: Squash merge (temiz history)

### 3. CI Gereksinimleri
Her PR için zorunlu:
- ✅ Naming Compliance
- ✅ Evidence Isolation
- ✅ Dev Loop Validation
- ✅ Performance Gate (main'e push için)

### 4. Rollback Stratejisi
Squash merge sayesinde:
```bash
git revert <commit-hash>
```
→ Sistem hızlı toparlanır

## Workflow

```
1. Branch oluştur:
   git checkout -b feature/my-feature

2. Lokal test:
   ./scripts/dev_loop.sh
   ./scripts/check_naming_compliance.sh
   ./scripts/check_evidence_isolation.sh

3. Push + PR:
   git push -u origin feature/my-feature
   # GitHub'da PR aç

4. CI bekle:
   Tüm job'lar yeşil olmalı

5. Review + Merge:
   Squash merge ile main'e al
```

## Branch Protection (GitHub Settings)

```
Settings → Branches → main:
✅ Require pull request
✅ Require status checks:
   - naming
   - evidence
   - devloop
✅ Require up-to-date branch
✅ Dismiss stale approvals
```
