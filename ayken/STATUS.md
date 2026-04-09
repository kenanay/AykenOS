# Ayken Toolchain — Status

**STATUS:** experimental
**CI:** disabled
**DEFAULT:** no
**PHASE:** parked (Phase-15 kapandı — Phase-16 Faz B bekliyor)

---

## Mevcut Durum

`ayken/` crate'i AykenOS ana build zincirine dahil değildir.
Workspace Cargo.toml'a üye değildir. CI gate'lerinde çalıştırılmaz.

`tools/ayken-cli/` altında Faz A wrapper CLI implement edildi (2026-04-09).
Faz B ve sonrası Phase-15 kapanışından sonra başlatılacak — **Phase-15 OFFICIALLY CLOSED** (2026-04-09).

---

## Neden Pasif?

- Ana build zinciri `KERNEL_CC = clang` kullanıyor (Makefile:15)
- `ayken/` crate'i native dependency build ekosistemiyle tam uyumlu değil
- Phase-15 kapsamı `userspace/bcib-runtime/` çekirdeğine odaklıydı
- Erken production'a almak determinism ve CI stability riskidir

---

## Gelecek Potansiyeli

Ayken ileride şu rollere evrilebilir:

1. **Governed build entrypoint** — `ayken build`, `ayken test`, `ayken ci`
   toolchain version pinning, deterministic env setup, CI/local parity

2. **BCIB-native toolchain** — `ayken dsl compile`, `ayken bcib verify`
   DSL → BCIB pipeline, verification + packaging, evidence production

3. **Enforcement compiler** — capability-aware compile-time checks,
   unsafe pattern reject, boundary enforcement at compile time

4. **Self-hosted execution** — AykenOS içinde Ayken çalışır,
   execution planning kendi yüzeyinde yönetilir

---

## Aktif Etme Koşulları

Ayken'i production'a almak için:

- [x] Phase-15 kapanışı tamamlanmış olmalı ✅ (2026-04-09, ci-freeze#24213727039)
- [ ] Native dependency build ekosistemiyle tam uyum (cc-rs, blake3, vb.)
- [ ] CI gate'lerinde deterministic davranış kanıtı
- [ ] Ayrı bir spec ile governance onayı (Phase-16)

---

## Lokal Kullanım (isteğe bağlı)

```bash
cd ayken
cargo build
cargo test
```

CI'da veya default build'de kullanılmaz.
