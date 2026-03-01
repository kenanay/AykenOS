#!/usr/bin/env python3
"""Unit test for userspace/minimal/minimal.elf structure."""

from __future__ import annotations

import struct
import sys
from pathlib import Path

ELF_MAGIC = b"\x7fELF"
ELFCLASS64 = 2
ELFDATA2LSB = 1
ET_EXEC = 2
EM_X86_64 = 62
PT_LOAD = 1
PF_X = 0x1
PF_W = 0x2
PF_R = 0x4
EXPECTED_ENTRY = 0x400000


def fail(msg: str) -> int:
    print(f"[FAIL] {msg}", file=sys.stderr)
    return 1


def main() -> int:
    elf_path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("minimal.elf")
    if not elf_path.is_file():
        return fail(f"ELF not found: {elf_path}")

    blob = elf_path.read_bytes()
    if len(blob) < 64:
        return fail("ELF file is too small for ELF64 header")

    if blob[:4] != ELF_MAGIC:
        return fail("ELF magic mismatch")
    if blob[4] != ELFCLASS64:
        return fail(f"ELF class is not 64-bit: got {blob[4]}")
    if blob[5] != ELFDATA2LSB:
        return fail(f"ELF data encoding is not little-endian: got {blob[5]}")

    (
        _e_ident,
        e_type,
        e_machine,
        _e_version,
        e_entry,
        e_phoff,
        _e_shoff,
        _e_flags,
        e_ehsize,
        e_phentsize,
        e_phnum,
        _e_shentsize,
        _e_shnum,
        _e_shstrndx,
    ) = struct.unpack_from("<16sHHIQQQIHHHHHH", blob, 0)

    if e_ehsize != 64:
        return fail(f"unexpected ELF header size: {e_ehsize}")
    if e_type != ET_EXEC:
        return fail(f"unexpected ELF type: {e_type} (expected ET_EXEC={ET_EXEC})")
    if e_machine != EM_X86_64:
        return fail(
            f"unexpected ELF machine: {e_machine} (expected EM_X86_64={EM_X86_64})"
        )
    if e_entry != EXPECTED_ENTRY:
        return fail(f"unexpected ELF entry: 0x{e_entry:x} (expected 0x{EXPECTED_ENTRY:x})")
    if e_phentsize < 56:
        return fail(f"unexpected program header size: {e_phentsize}")
    if e_phoff + (e_phnum * e_phentsize) > len(blob):
        return fail("program header table exceeds file size")

    load_segments = []
    for idx in range(e_phnum):
        off = e_phoff + (idx * e_phentsize)
        p_type, p_flags, p_offset, p_vaddr, _p_paddr, p_filesz, p_memsz, _p_align = (
            struct.unpack_from("<IIQQQQQQ", blob, off)
        )
        if p_type == PT_LOAD:
            load_segments.append((p_flags, p_offset, p_vaddr, p_filesz, p_memsz))

    if len(load_segments) != 1:
        return fail(f"expected exactly one PT_LOAD segment, got {len(load_segments)}")

    p_flags, _p_offset, p_vaddr, _p_filesz, p_memsz = load_segments[0]
    expected_flags = PF_R | PF_X
    if p_flags != expected_flags:
        return fail(
            f"PT_LOAD flags mismatch: got 0x{p_flags:x}, expected RX (0x{expected_flags:x})"
        )
    if p_flags & PF_W:
        return fail("PT_LOAD segment is writable; expected RX-only")
    if not (p_vaddr <= EXPECTED_ENTRY < (p_vaddr + p_memsz)):
        return fail("entry point is not inside PT_LOAD segment range")

    print("[OK] minimal.elf structure validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
