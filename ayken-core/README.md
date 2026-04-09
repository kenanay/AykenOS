# Ayken Core - AI Sistemi

**Oluşturan:** Kenan AY  
**Proje:** AykenOS  
**Son Güncelleme:** 09 Nisan 2026

AykenOS'un Rust tabanlı AI core sistemi. Yapay zeka modelleri için veri formatları, builder araçları ve runtime bileşenlerini içerir.

**Güncel Durum:**
- Phase-15 OFFICIALLY CLOSED (2026-04-09) — BCIB Execution Engine v3 tamamlandı
- `ayken-core/crates/bcib/` — BCIB v0.2 format (frozen, backward-compatible)
- BCIB v3 runtime: `userspace/bcib-runtime/` (Phase-15 deliverable)
- `ayken-core` ayrı gate'e tabi: `ayken-core/` ve `ayken/` Phase-15 kapsamı dışındadır

---

## 🎯 Genel Bakış

Ayken Core, AykenOS'un yapay zeka yeteneklerinin temelini oluşturur. İki ana format ve bunların araçlarını içerir:

1. **ABDF (Ayken Binary Data Format):** AI/ML modelleri için yüksek performanslı binary veri formatı
2. **BCIB (Binary CLI Instruction Buffer):** CLI komutları ve execution graph'ler için compact binary format

---

## 📦 Crate'ler

### 1. ABDF (Ayken Binary Data Format)

**Konum:** `crates/abdf/`

**Açıklama:** AI/ML modelleri için optimize edilmiş binary veri formatı

**Özellikler:**
- 🚀 **Yüksek Performans:** CPU ve GPU dostu memory layout
- 🔧 **Extensible:** Segment tabanlı genişletilebilir yapı
- 📊 **AI/ML Desteği:** Model ağırlıkları, aktivasyonlar ve metadata
- 🔒 **Versiyonlu:** Header versiyonlama ile geriye uyumluluk
- 💾 **Compact:** Verimli binary encoding

**Kullanım:**
```rust
use abdf::{AbdfHeader, AbdfSegment};

// ABDF dosyası oku
let data = std::fs::read("model.abdf")?;
let header = AbdfHeader::parse(&data)?;

// Segment'leri işle
for segment in header.segments() {
    match segment.segment_type() {
        SegmentType::Weights => { /* ... */ },
        SegmentType::Metadata => { /* ... */ },
        _ => {}
    }
}
```

**Dokümantasyon:**
- [ABDF Spesifikasyonu](docs/abdf/abdf-spec.md)
- [Metadata Yapısı](docs/abdf/metadata.md)

---

### 2. ABDF Builder

**Konum:** `crates/abdf-builder/`

**Açıklama:** ABDF formatında veri oluşturma ve okuma araçları

**Özellikler:**
- 🏗️ **Builder API:** Fluent API ile kolay ABDF oluşturma
- 📝 **Meta-veri Desteği:** String pool ve metadata tablosu
- 🔄 **Encode/Decode:** Binary encoding ve decoding
- ✅ **Validation:** Format doğrulama ve hata kontrolü

**Kullanım:**
```rust
use abdf_builder::AbdfBuilder;

// ABDF dosyası oluştur
let mut builder = AbdfBuilder::new();

builder
    .add_metadata("model_name", "TinyLLM-v1")
    .add_metadata("version", "1.0.0")
    .add_weights_segment(weights_data)
    .add_config_segment(config_data);

let abdf_data = builder.build()?;
std::fs::write("model.abdf", abdf_data)?;
```

**Özellikler:**
- Meta-veri tablosu yönetimi
- String pool optimizasyonu
- Segment ekleme ve yönetimi
- Binary serialization

---

### 3. BCIB (Binary CLI Instruction Buffer)

**Konum:** `crates/bcib/`

**Açıklama:** CLI komutları ve execution graph'ler için binary format

**Özellikler:**
- 📦 **Compact Format:** Verimli binary encoding
- 🔗 **Execution Graph:** Komut bağımlılıkları ve paralel execution
- 🎯 **Veri-Odaklı:** Data-centric komut yapısı
- 🔢 **Versiyonlu Header:** Format versiyonlama

**Kullanım:**
```rust
use bcib::{BcibBuffer, BcibInstruction};

// BCIB buffer oluştur
let mut buffer = BcibBuffer::new();

// Instruction ekle
buffer.add_instruction(BcibInstruction::DataCreate {
    name: "users".to_string(),
    data_type: DataType::Tabular,
    schema: schema_data,
});

buffer.add_instruction(BcibInstruction::DataQuery {
    target: "users".to_string(),
    filter: "role == 'admin'".to_string(),
});

// Binary'ye serialize et
let binary = buffer.to_bytes()?;
```

**Instruction Tipleri:**
- `DataCreate`: Veri konteyneri oluşturma
- `DataQuery`: Veri sorgulama
- `DataAdd`: Veri ekleme
- `DataUpdate`: Veri güncelleme
- `DataDelete`: Veri silme
- `Execute`: Execution graph yürütme

---

## 🚀 Kullanım

### Tüm Crate'leri Derleme

```bash
# Workspace root'ta
cargo build

# Release mode
cargo build --release
```

### Belirli Bir Crate'i Derleme

```bash
# ABDF
cargo build -p abdf

# ABDF Builder
cargo build -p abdf-builder

# BCIB
cargo build -p bcib
```

### Testleri Çalıştırma

```bash
# Tüm testler
cargo test

# Belirli bir crate
cargo test -p abdf
cargo test -p bcib

# Verbose output
cargo test -- --nocapture
```

### Dokümantasyon Oluşturma

```bash
# Tüm crate'ler için dokümantasyon
cargo doc --open

# Belirli bir crate
cargo doc -p abdf --open
```

---

## 📚 Dokümantasyon

### Format Spesifikasyonları

- **[ABDF Specification](docs/abdf/abdf-spec.md)** - ABDF format detayları
- **[ABDF Metadata](docs/abdf/metadata.md)** - Metadata yapısı
- **[BCIB Specification](docs/bcib/bcib-spec.md)** - BCIB format detayları

### API Dokümantasyonu

```bash
cargo doc --open
```

### Örnekler

Her crate'in `examples/` dizininde kullanım örnekleri bulunur:

```bash
# ABDF builder örneği
cargo run --example abdf_builder

# BCIB executor örneği
cargo run --example bcib_executor
```

---

## 🏗️ Mimari

### ABDF Format Yapısı

```
┌─────────────────────────────────────┐
│ ABDF Header                         │
│ - Magic: "ABDF"                     │
│ - Version: u32                      │
│ - Segment Count: u32                │
│ - Metadata Offset: u64              │
└─────────────────────────────────────┘
┌─────────────────────────────────────┐
│ Segment 1: Weights                  │
│ - Type: u32                         │
│ - Size: u64                         │
│ - Data: [u8]                        │
└─────────────────────────────────────┘
┌─────────────────────────────────────┐
│ Segment 2: Config                   │
│ - Type: u32                         │
│ - Size: u64                         │
│ - Data: [u8]                        │
└─────────────────────────────────────┘
┌─────────────────────────────────────┐
│ Metadata Table                      │
│ - Entry Count: u32                  │
│ - Entries: [(key, value)]           │
└─────────────────────────────────────┘
```

### BCIB Format Yapısı

```
┌─────────────────────────────────────┐
│ BCIB Header                         │
│ - Magic: "BCIB"                     │
│ - Version: u32                      │
│ - Instruction Count: u32            │
│ - Graph Offset: u64                 │
└─────────────────────────────────────┘
┌─────────────────────────────────────┐
│ Instruction 1                       │
│ - Opcode: u8                        │
│ - Args Length: u32                  │
│ - Args: [u8]                        │
└─────────────────────────────────────┘
┌─────────────────────────────────────┐
│ Instruction 2                       │
│ - Opcode: u8                        │
│ - Args Length: u32                  │
│ - Args: [u8]                        │
└─────────────────────────────────────┘
┌─────────────────────────────────────┐
│ Execution Graph                     │
│ - Node Count: u32                   │
│ - Edges: [(from, to)]               │
└─────────────────────────────────────┘
```

---

## 🔧 Geliştirme

### Workspace Yapısı

```
ayken-core/
├── Cargo.toml          # Workspace konfigürasyonu
├── Cargo.lock          # Bağımlılık kilidi
├── README.md           # Bu dosya
│
├── crates/
│   ├── abdf/           # ABDF format
│   ├── abdf-builder/   # ABDF builder
│   └── bcib/           # BCIB format
│
├── docs/               # Dokümantasyon
│   ├── abdf/
│   ├── bcib/
│   └── api/
│
└── target/             # Build çıktıları
```

### Bağımlılıklar

**ABDF:**
- `serde` - Serialization/deserialization
- `bincode` - Binary encoding

**ABDF Builder:**
- `abdf` - ABDF format
- `serde` - Serialization

**BCIB:**
- `serde` - Serialization/deserialization
- `petgraph` - Graph data structures

### Kod Standartları

```bash
# Format kontrolü
cargo fmt --check

# Linting
cargo clippy -- -D warnings

# Test coverage
cargo tarpaulin
```

---

## 🧪 Test

### Unit Testler

Her crate kendi unit testlerini içerir:

```bash
# Tüm unit testler
cargo test

# Belirli bir test
cargo test test_abdf_header
```

### Integration Testler

```bash
# Integration testler
cargo test --test integration_tests
```

### Benchmark'lar

```bash
# Benchmark'ları çalıştır
cargo bench
```

---

## 📊 Performans

### ABDF

- **Encoding:** ~100 MB/s
- **Decoding:** ~150 MB/s
- **Memory Overhead:** ~5% (metadata için)

### BCIB

- **Encoding:** ~200 MB/s
- **Decoding:** ~250 MB/s
- **Execution Overhead:** ~1-2μs per instruction

---

## 🔗 AykenOS Entegrasyonu

Ayken Core, AykenOS'un aşağıdaki bileşenleriyle entegre olur:

### Kernel Entegrasyonu

- **Ring0:** ABDF/BCIB format validation
- **Ring3:** AI runtime ve BCIB execution engine

### Userspace Entegrasyonu

- **AI Runtime:** ABDF model loading
- **BCIB Runtime:** BCIB execution
- **Semantic CLI:** BCIB instruction generation

### Build Sistemi

```bash
# AykenOS ile birlikte derleme
cd ..
make all

# Sadece Rust bileşenleri
cd ayken-core
cargo build
```

---

## 🎯 Gelecek Hedefler

### Kısa Vadeli

- [ ] ABDF v2 format (gelişmiş compression)
- [ ] BCIB optimizer (execution graph optimization)
- [ ] Streaming API (büyük dosyalar için)
- [ ] Python bindings

### Orta Vadeli

- [ ] GPU acceleration (CUDA/OpenCL)
- [ ] Distributed execution (multi-node BCIB)
- [ ] Model quantization (ABDF)
- [ ] JIT compilation (BCIB)

### Uzun Vadeli

- [ ] Neural architecture search (NAS) desteği
- [ ] Federated learning (distributed ABDF)
- [ ] Hardware-specific optimizations
- [ ] Cloud integration

---

## 📝 Lisans

AykenOS Source-Available License (ASAL v1.0)

- ✅ Topluluk ve kişisel kullanım için ücretsiz
- ✅ Kod görülebilir, incelenebilir, değiştirilebilir
- ❌ Ticari kullanım için özel lisans gerekir

**Hak Sahibi:** Kenan AY — AykenOS Project

---

## 🤝 Katkıda Bulunma

Katkılarınızı bekliyoruz! Lütfen aşağıdaki adımları izleyin:

1. Fork edin
2. Feature branch oluşturun (`git checkout -b feature/amazing-feature`)
3. Değişikliklerinizi commit edin (`git commit -m 'Add amazing feature'`)
4. Branch'inizi push edin (`git push origin feature/amazing-feature`)
5. Pull Request açın

### Kod Standartları

- Rust formatting: `cargo fmt`
- Linting: `cargo clippy`
- Testler: `cargo test`
- Dokümantasyon: Her public API için

---

## 📞 İletişim

**Proje Sahibi:** Kenan AY  
**Proje:** AykenOS  
**Repository:** AykenOS/ayken-core

---

**Oluşturan:** Kenan AY  
**Son Güncelleme:** 15 Ocak 2026

**© 2026 Kenan AY - AykenOS Project**
