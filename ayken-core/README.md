# Ayken Core

AykenOS'un temel veri formatları ve runtime bileşenleri.

## Crate'ler

### ABDF (Ayken Binary Data Format)
- **Konum:** `crates/abdf/`
- **Açıklama:** Yüksek performanslı binary veri formatı
- **Özellikler:** 
  - CPU ve GPU dostu layout
  - AI/ML veri desteği
  - Extensible segment yapısı

### ABDF Builder
- **Konum:** `crates/abdf-builder/`
- **Açıklama:** ABDF formatında veri oluşturma ve okuma araçları
- **Özellikler:**
  - Meta-veri tablosu desteği
  - String pool yönetimi
  - Encode/decode API'si

### BCIB (Binary CLI Instruction Buffer)
- **Konum:** `crates/bcib/`
- **Açıklama:** CLI komutları için binary format
- **Özellikler:**
  - Veri odaklı komut yapısı
  - Versiyonlu header
  - Compact instruction set

## Kullanım

```bash
# Tüm crate'leri test et
cargo test

# Belirli bir crate'i test et
cargo test -p abdf

# Build kontrolü
cargo check
```

## Dokümantasyon

- [ABDF Specification](docs/abdf/abdf-spec.md)
- [ABDF Metadata](docs/abdf/metadata.md)

## Lisans

AykenOS Project - Open Source