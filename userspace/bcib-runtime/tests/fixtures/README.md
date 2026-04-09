# BCIB v0.2 Golden Fixtures

Bu dizin, v0.2 corpus'tan alınan golden fixture BCIB binary dosyalarını içerir.

## Fixture Listesi

| Dosya | Açıklama | Beklenen Sonuç |
|-------|----------|----------------|
| `nop_end.bcib` | Minimal: Nop (0x00) + End (0x01), v0.2 header | parse OK, plan OK |
| `data_create_query.bcib` | DataCreate (0x10) + DataQuery (0x12) + End (0x01) | parse OK, plan OK (capability gerekli) |
| `data_add.bcib` | DataCreate (0x10) + DataAdd (0x11) + End (0x01) | parse OK, plan OK (capability gerekli) |
| `ui_render.bcib` | UiRender (0x20) + End (0x01) | parse OK, plan OK (capability gerekli) |
| `ai_ask.bcib` | AiAsk (0x30) + End (0x01) | parse OK, plan OK (capability gerekli) |
| `invalid_magic.bcib` | Geçersiz magic bytes | parse FAIL: BCIB_ERR_INVALID_GRAPH |
| `unsupported_version.bcib` | Version 0x0004 (gelecek, desteklenmiyor) | parse FAIL: BCIB_ERR_UNSUPPORTED_VERSION |

## Binary Format

```
Header (16 bytes):
  magic:         [u8; 4]  = b"BCIB"
  version:       u16 LE   = 0x0002 (v0.2) veya 0x0003 (v3)
  flags:         u16 LE   = 0x0000
  section_count: u16 LE
  reserved:      [u8; 4]  = 0x00000000
  tail:          [u8; 2]  = 0x0000

Section Table (section_count × 8 bytes):
  section_id: u16 LE
  offset:     u32 LE
  length:     u16 LE

Instruction Encoding:
  opcode:        u8
  operand_count: u8
  operands:      [u32 LE; operand_count]
```

## CI Davranışı

Fixture uyumsuzluğu → CI FAIL (Requirements 12.3, 12.4)
