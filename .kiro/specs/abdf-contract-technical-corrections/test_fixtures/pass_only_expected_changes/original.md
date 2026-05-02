# Test Document - Original

## 🧵 String Pool Section

Current implementation uses null-terminated strings.

Example: `"hello\0world\0"`

## 🔒 Header Structure

Fields:
- `magic: u32`
- `version: u16`
- `checksum: u64, // XXH3 hash`
- `total_size: u64`

## 📐 Binary Layout (Preserved)

This section should NOT change.

```
[Header: 64B]
[Directory: N×32B]
[Segments: variable]
```

## 🎯 Success Criteria (Preserved)

- [ ] Parser implementation complete
- [ ] Validation tests pass
- [ ] Documentation updated
