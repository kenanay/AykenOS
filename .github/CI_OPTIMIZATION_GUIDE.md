# CI Optimization Guide

## 🎯 Hedef
Full rebuild (10-15 dk) → Incremental + Cache (3-5 dk)

## ⚡ Optimization Katmanları

### 1. Cache Stratejisi
```yaml
- uses: actions/cache@v3
  with:
    path: |
      out/build
      out/cache
    key: ayken-build-${{ runner.os }}-${{ hashFiles('**/Makefile', '**/*.c', '**/*.h', '**/*.rs') }}
    restore-keys: |
      ayken-build-${{ runner.os }}-
```

**Kazanç**: İlk build sonrası 60-80% hız artışı

### 2. Incremental Build
```bash
# ❌ Yavaş
make clean && make kernel.elf

# ✅ Hızlı
make kernel.elf  # sadece değişen dosyalar
```

**Kazanç**: 5-10 dk → 1-2 dk

### 3. Paralel Execution
GitHub Actions otomatik olarak farklı job'ları paralel çalıştırır:
- `naming.yml` ║ `evidence.yml` ║ `dev-loop.yml`

**Kazanç**: 3x job = 3x paralel = ~3x hız

### 4. Path-Based Triggering
```yaml
on:
  push:
    paths:
      - 'kernel/**'
      - 'ayken_core/**'
      - 'Makefile'
```

**Kazanç**: Gereksiz build'leri engeller

### 5. Fail-Fast Strategy
```yaml
strategy:
  fail-fast: true
```

**Kazanç**: Bir job patlarsa diğerleri iptal → kaynak tasarrufu

### 6. Timeout Optimization
```yaml
jobs:
  devloop:
    timeout-minutes: 10  # 15 yerine 10
```

**Kazanç**: Takılı job'ları erken keser

## 📊 Beklenen Sonuçlar

| Metrik | Önce | Sonra | Kazanç |
|--------|------|-------|--------|
| İlk build | 10-15 dk | 10-15 dk | - |
| Incremental | 10-15 dk | 2-4 dk | 70-80% |
| Paralel job | Seri | Paralel | 3x |
| Cache hit | Yok | Var | 60-80% |

## 🚀 Uygulama

### Adım 1: Mevcut workflow'ları koru
`dev-loop.yml` → temel, her zaman çalışır

### Adım 2: Optimized versiyonu test et
`dev-loop-optimized.yml` → cache + incremental

### Adım 3: Sonuçları karşılaştır
```bash
# GitHub Actions → Actions tab
# İki workflow'un süresini karşılaştır
```

### Adım 4: Geçiş
Optimized versiyon stabil olunca:
- `dev-loop.yml` → sil
- `dev-loop-optimized.yml` → `dev-loop.yml` olarak yeniden adlandır

## ⚠️ Dikkat Edilecekler

1. **Cache invalidation**: Makefile değişince cache temizlenir
2. **Incremental risk**: Bazen `make clean` gerekebilir
3. **Disk kullanımı**: Cache 500MB-1GB olabilir

## 🔍 Monitoring

```bash
# Cache boyutu
du -sh out/build out/cache

# Build süresi
time make kernel.elf
```
