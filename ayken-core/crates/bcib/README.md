# BCIB (Binary CLI Instruction Buffer)

AykenOS CLI komutları için binary instruction format.

## Özellikler

- **Compact**: 8-byte instruction format
- **Versioned**: Header ile format versiyonlama
- **Extensible**: Yeni opcode'lar eklenebilir
- **Data-oriented**: ABDF ile entegre çalışır
- **String Pool**: Efficient string referencing

## Instruction Set

### Context Management
- `CtxSelect` - Aktif veri konteynerini seç

### Data Operations
- `DataCreate` - Yeni veri konteyneri oluştur
- `DataAdd` - Konteynere veri ekle
- `DataQuery` - Veri sorgula
- `DataUpdate` - Veri güncelle
- `DataDelete` - Veri sil

### UI Operations
- `UiRender` - UI sahnesi render et
- `UiEvent` - UI olayı gönder

### AI Operations
- `AiAsk` - AI modülüne soru sor

### System Operations
- `SysInfo` - Sistem bilgisi al
- `End` - Execution sonlandır

## Kullanım

```rust
use bcib::{BcibHeader, BcibInstruction, BcibOpcode};

// Header oluştur
let mut header = BcibHeader::new();
header.instruction_count = 2;

// Instruction'lar oluştur
let select = BcibInstruction::new(
    BcibOpcode::CtxSelect, 
    0, // flags
    0, // container_id
    0, 0 // unused args
);

let query = BcibInstruction::new(
    BcibOpcode::DataQuery,
    0,
    0, // query string index
    0, 0
);
```

## Format Spesifikasyonu

Detaylı format spesifikasyonu için [BCIB Spec](../../docs/bcib/bcib-spec.md) dosyasına bakın.

## Lisans

AykenOS Project - Open Source