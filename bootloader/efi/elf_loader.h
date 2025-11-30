#ifndef AYKEN_ELF_LOADER_H
#define AYKEN_ELF_LOADER_H

#include <efi.h>
#include <efilib.h>
#include "../../kernel/include/boot_info.h"

// ELF64 tanımları
#define EI_NIDENT 16
typedef UINT16  Elf64_Half;
typedef UINT32  Elf64_Word;
typedef UINT64  Elf64_Xword;
typedef UINT64  Elf64_Addr;
typedef UINT64  Elf64_Off;

typedef struct {
    UINT8 e_ident[EI_NIDENT];
    Elf64_Half e_type;
    Elf64_Half e_machine;
    Elf64_Word e_version;
    Elf64_Addr e_entry;
    Elf64_Off  e_phoff;
    Elf64_Off  e_shoff;
    Elf64_Word e_flags;
    Elf64_Half e_ehsize;
    Elf64_Half e_phentsize;
    Elf64_Half e_phnum;
    Elf64_Half e_shentsize;
    Elf64_Half e_shnum;
    Elf64_Half e_shstrndx;
} Elf64_Ehdr;

typedef struct {
    Elf64_Word p_type;
    Elf64_Word p_flags;
    Elf64_Off  p_offset;
    Elf64_Addr p_vaddr;
    Elf64_Addr p_paddr;
    Elf64_Xword p_filesz;
    Elf64_Xword p_memsz;
    Elf64_Xword p_align;
} Elf64_Phdr;

#define PT_LOAD 1

EFI_STATUS elf_load_kernel(EFI_HANDLE ImageHandle,
                           EFI_SYSTEM_TABLE *SystemTable,
                           CHAR16 *kernel_path,
                           ayken_boot_info_t *boot_info,
                           UINT64 *kernel_entry_out);

#endif
