/**
 * @file elf64.h
 * @brief ELF64 format structures and constants for AykenOS
 * 
 * This file defines the minimal ELF64 structures needed for loading
 * Ring3 userspace programs. Supports x86_64 architecture only.
 * 
 * Phase 10-A: Minimal validation and single PT_LOAD segment support
 * Phase 10-B: Full ELF parsing with multi-segment and BSS support
 * 
 * @author Kenan AY
 * @date 2026-02-27
 */

#ifndef AYKEN_ELF64_H
#define AYKEN_ELF64_H

#include <stdint.h>
#include <stddef.h>

/* ELF Identification Indices */
#define EI_MAG0         0   /* File identification byte 0 index */
#define EI_MAG1         1   /* File identification byte 1 index */
#define EI_MAG2         2   /* File identification byte 2 index */
#define EI_MAG3         3   /* File identification byte 3 index */
#define EI_CLASS        4   /* File class byte index */
#define EI_DATA         5   /* Data encoding byte index */
#define EI_VERSION      6   /* File version byte index */
#define EI_OSABI        7   /* OS ABI identification */
#define EI_ABIVERSION   8   /* ABI version */
#define EI_PAD          9   /* Byte index of padding bytes */
#define EI_NIDENT       16  /* Size of e_ident[] */

/* ELF Magic Number */
#define ELF_MAGIC_0     0x7F
#define ELF_MAGIC_1     'E'
#define ELF_MAGIC_2     'L'
#define ELF_MAGIC_3     'F'

/* ELF Class */
#define ELFCLASS32      1   /* 32-bit objects */
#define ELFCLASS64      2   /* 64-bit objects */

/* ELF Data Encoding */
#define ELFDATA2LSB     1   /* Little-endian */
#define ELFDATA2MSB     2   /* Big-endian */

/* ELF Version */
#define EV_NONE         0   /* Invalid version */
#define EV_CURRENT      1   /* Current version */

/* ELF Object File Types */
#define ET_NONE         0   /* No file type */
#define ET_REL          1   /* Relocatable file */
#define ET_EXEC         2   /* Executable file */
#define ET_DYN          3   /* Shared object file */
#define ET_CORE         4   /* Core file */

/* ELF Machine Types */
#define EM_NONE         0   /* No machine */
#define EM_X86_64       62  /* AMD x86-64 architecture */

/* Program Header Types */
#define PT_NULL         0   /* Program header table entry unused */
#define PT_LOAD         1   /* Loadable program segment */
#define PT_DYNAMIC      2   /* Dynamic linking information */
#define PT_INTERP       3   /* Program interpreter */
#define PT_NOTE         4   /* Auxiliary information */
#define PT_SHLIB        5   /* Reserved */
#define PT_PHDR         6   /* Entry for header table itself */
#define PT_TLS          7   /* Thread-local storage segment */

/* Program Header Flags */
#define PF_X            0x1 /* Segment is executable */
#define PF_W            0x2 /* Segment is writable */
#define PF_R            0x4 /* Segment is readable */

/**
 * @brief ELF64 File Header
 * 
 * The ELF header appears at the beginning of every ELF file and
 * provides information about the file's organization.
 */
typedef struct {
    uint8_t  e_ident[EI_NIDENT]; /* Magic number and other info */
    uint16_t e_type;              /* Object file type (ET_EXEC) */
    uint16_t e_machine;           /* Architecture (EM_X86_64) */
    uint32_t e_version;           /* Object file version */
    uint64_t e_entry;             /* Entry point virtual address */
    uint64_t e_phoff;             /* Program header table file offset */
    uint64_t e_shoff;             /* Section header table file offset */
    uint32_t e_flags;             /* Processor-specific flags */
    uint16_t e_ehsize;            /* ELF header size in bytes */
    uint16_t e_phentsize;         /* Program header table entry size */
    uint16_t e_phnum;             /* Program header table entry count */
    uint16_t e_shentsize;         /* Section header table entry size */
    uint16_t e_shnum;             /* Section header table entry count */
    uint16_t e_shstrndx;          /* Section header string table index */
} __attribute__((packed)) elf64_ehdr_t;

/**
 * @brief ELF64 Program Header
 * 
 * Program headers describe segments that are loaded into memory
 * at runtime. Each PT_LOAD segment defines a contiguous region
 * of memory to be loaded from the ELF file.
 */
typedef struct {
    uint32_t p_type;   /* Segment type (PT_LOAD, PT_DYNAMIC, etc.) */
    uint32_t p_flags;  /* Segment flags (PF_R, PF_W, PF_X) */
    uint64_t p_offset; /* Segment file offset */
    uint64_t p_vaddr;  /* Segment virtual address */
    uint64_t p_paddr;  /* Segment physical address (unused) */
    uint64_t p_filesz; /* Segment size in file */
    uint64_t p_memsz;  /* Segment size in memory */
    uint64_t p_align;  /* Segment alignment */
} __attribute__((packed)) elf64_phdr_t;

#endif /* AYKEN_ELF64_H */
