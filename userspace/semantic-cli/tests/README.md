# Loop Engine Test Suite

Bu dizin, D3 Loop Support Design için kapsamlı test paketini içerir. Testler üç kategoriye ayrılmıştır:

## Test Kategorileri

### 1. Core Tests (`loop_core.rs`)
**Stabil, üretim hazır testler**
- 10.1: Temel loop execution fonksiyonalitesi
- 10.2: Collection determinism

Bu testler varsayılan olarak çalışır:
```bash
cargo test --test loop_core
```

### 2. Safety Analysis Tests (`loop_spec_safety.rs`)
**Spec implementation testleri - Feature gate arkasında**
- 10.3: Safety analysis sistemi

Bu testler sadece feature flag ile çalışır:
```bash
cargo test --test loop_spec_safety --features d3_loop_spec
```

### 3. Optimization Tests (`loop_spec_opt.rs`)
**Spec implementation testleri - Feature gate arkasında**
- 10.4: Optimization sistemleri (unrolling, hot loop detection, JIT)
- Integration testleri

Bu testler sadece feature flag ile çalışır:
```bash
cargo test --test loop_spec_opt --features d3_loop_spec
```

## Hızlı Komutlar

### Tüm core testleri çalıştır (stabil)
```bash
cargo test -p semantic-cli --test loop_core
```

### Tüm spec testleri çalıştır (geliştirilmekte)
```bash
cargo test -p semantic-cli --features d3_loop_spec --test loop_spec_safety --test loop_spec_opt
```

### Tüm testleri çalıştır
```bash
# Core testler
cargo test -p semantic-cli --test loop_core

# Spec testler
cargo test -p semantic-cli --features d3_loop_spec --test loop_spec_safety --test loop_spec_opt
```

## Test Durumu

### ✅ Çalışan Testler
- **Core Tests**: 20/20 test geçiyor
- **Feature Gate**: Doğru çalışıyor (feature olmadan spec testler çalışmıyor)

### ⚠️ Bilinen Sorunlar
- Bazı spec testlerde API uyumsuzlukları var (monitoring, JIT integration)
- Bu testler `#[ignore]` ile işaretlenmiş ve gelecekte düzeltilecek

## Mimari

### Test Organizasyonu
```
tests/
├── loop_core.rs           # Stabil core testler (10.1, 10.2)
├── loop_spec_safety.rs    # Safety analysis testleri (10.3)
├── loop_spec_opt.rs       # Optimization testleri (10.4 + integration)
└── README.md              # Bu dosya
```

### Feature Gate Stratejisi
- `d3_loop_spec` feature flag ile spec testler kontrol edilir
- Core testler her zaman çalışır (production ready)
- Spec testler sadece geliştirme sırasında çalışır

Bu yaklaşım, repo'yu "bugün çalışan" ve "gelecek spec" testleri arasında temiz bir ayrım sağlar.