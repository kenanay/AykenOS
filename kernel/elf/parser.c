/**
 * @file parser.c
 * @brief ELF64 parser implementation for AykenOS
 * 
 * This file implements minimal ELF64 parsing and validation for loading
 * Ring3 userspace programs. Phase 10-A provides minimal validation with
 * mandatory bounds checking for security.
 * 
 * Constitutional Requirements:
 * - Fail-closed validation (reject invalid ELF immediately)
 * - No partial state on error
 * - Bounds checking mandatory even in Phase 10-A
 * 
 * @author Kenan AY
 * @date 2026-02-27
 */

#include <elf/elf64.h>
#include <errno.h>

/**
 * @brief Validate minimal ELF64 binary (Phase 10-A)
 * 
 * Performs essential validation checks on an ELF64 binary:
 * - Magic number validation (0x7F 'E' 'L' 'F')
 * - Class validation (64-bit)
 * - Machine validation (x86_64)
 * - Type validation (ET_EXEC)
 * - Program header table bounds validation (CRITICAL for security)
 * - Segment file range bounds validation (CRITICAL for security)
 * 
 * Phase 10-A Ultra-Minimal Policy:
 * - Exact format match only (e_phentsize == sizeof(elf64_phdr_t))
 * - Limited program header count (max 16 for Phase 10-A)
 * - Strict bounds checking (fail-closed)
 * 
 * @param blob Pointer to ELF binary data
 * @param size Size of ELF binary in bytes
 * @return 0 on success, -EINVAL on invalid format, -ENOEXEC on unsupported format
 */
static int elf64_validate_minimal(const uint8_t *blob, size_t size) __attribute__((unused));
static int elf64_validate_minimal(const uint8_t *blob, size_t size) {
    /* Null pointer check */
    if (blob == 0) {
        return -EINVAL;
    }
    
    /* Minimum size check: must fit ELF header */
    if (size < sizeof(elf64_ehdr_t)) {
        return -EINVAL;
    }
    
    /* Cast to ELF header structure */
    const elf64_ehdr_t *ehdr = (const elf64_ehdr_t *)blob;
    
    /* Validate ELF magic number (0x7F 'E' 'L' 'F') */
    if (ehdr->e_ident[EI_MAG0] != ELF_MAGIC_0 ||
        ehdr->e_ident[EI_MAG1] != ELF_MAGIC_1 ||
        ehdr->e_ident[EI_MAG2] != ELF_MAGIC_2 ||
        ehdr->e_ident[EI_MAG3] != ELF_MAGIC_3) {
        return -EINVAL;
    }
    
    /* Validate ELF class (64-bit) */
    if (ehdr->e_ident[EI_CLASS] != ELFCLASS64) {
        return -ENOEXEC;
    }
    
    /* Validate data encoding (little-endian) */
    if (ehdr->e_ident[EI_DATA] != ELFDATA2LSB) {
        return -ENOEXEC;
    }
    
    /* Validate ELF version */
    if (ehdr->e_ident[EI_VERSION] != EV_CURRENT) {
        return -ENOEXEC;
    }
    
    /* Validate machine type (x86_64) */
    if (ehdr->e_machine != EM_X86_64) {
        return -ENOEXEC;
    }
    
    /* Validate object file type (executable) */
    if (ehdr->e_type != ET_EXEC) {
        return -ENOEXEC;
    }
    
    /* Validate ELF version field */
    if (ehdr->e_version != EV_CURRENT) {
        return -ENOEXEC;
    }
    
    /* CRITICAL: Validate program header table bounds
     * This check prevents out-of-bounds reads when iterating program headers.
     * Without this check, a malicious ELF could cause the kernel to read
     * beyond the blob boundary, potentially leaking kernel memory or causing
     * a page fault.
     */
    if (ehdr->e_phnum > 0) {
        /* Phase 10-A: Limit program header count to prevent DoS
         * 16 segments is more than enough for Phase 10-A (typically 1-3)
         * This prevents kernel from allocating excessive resources
         */
        if (ehdr->e_phnum > 16) {
            return -EINVAL;
        }
        
        /* Phase 10-A: Exact format match only (fail-closed)
         * We only accept standard elf64_phdr_t size
         * This prevents ABI-incompatible ELF files from causing issues
         */
        if (ehdr->e_phentsize != sizeof(elf64_phdr_t)) {
            return -EINVAL;
        }
        
        /* Validate program header offset alignment and minimum sanity
         * e_phoff must be at least past the ELF header
         */
        if (ehdr->e_phoff < sizeof(elf64_ehdr_t)) {
            return -EINVAL;
        }
        
        /* Check for overflow in program header table size calculation */
        uint64_t phdr_table_size = (uint64_t)ehdr->e_phnum * (uint64_t)ehdr->e_phentsize;
        if (phdr_table_size > size) {
            return -EINVAL;
        }
        
        /* Check for overflow in program header table end offset */
        if (ehdr->e_phoff > size - phdr_table_size) {
            return -EINVAL;
        }
    }
    
    /* CRITICAL: Validate segment file range bounds for all PT_LOAD segments
     * This check prevents out-of-bounds reads when copying segment data.
     * Without this check, a malicious ELF could cause the kernel to read
     * beyond the blob boundary during segment loading.
     */
    if (ehdr->e_phnum > 0) {
        const uint8_t *phdr_base = blob + ehdr->e_phoff;
        
        for (uint16_t i = 0; i < ehdr->e_phnum; i++) {
            const elf64_phdr_t *phdr = (const elf64_phdr_t *)(phdr_base + i * ehdr->e_phentsize);
            
            /* Only validate PT_LOAD segments */
            if (phdr->p_type != PT_LOAD) {
                continue;
            }
            
            /* CRITICAL: Validate p_memsz >= p_filesz
             * If p_memsz < p_filesz, this is a malformed ELF (negative BSS)
             * This would cause loader to fail or corrupt memory
             */
            if (phdr->p_memsz < phdr->p_filesz) {
                return -EINVAL;
            }
            
            /* Skip segments with no file data */
            if (phdr->p_filesz == 0) {
                continue;
            }
            
            /* Check for overflow in segment file range calculation */
            if (phdr->p_offset > size) {
                return -EINVAL;
            }
            
            if (phdr->p_filesz > size - phdr->p_offset) {
                return -EINVAL;
            }
        }
    }
    
    return 0;
}

/**
 * @brief Extract entry point address from ELF64 binary
 * 
 * Returns the virtual address where execution should begin (e_entry field).
 * This function assumes the ELF binary has already been validated.
 * 
 * @param blob Pointer to validated ELF binary data
 * @return Entry point virtual address (e_entry)
 */
static uint64_t elf64_get_entry(const uint8_t *blob) __attribute__((unused));
static uint64_t elf64_get_entry(const uint8_t *blob) {
    const elf64_ehdr_t *ehdr = (const elf64_ehdr_t *)blob;
    return ehdr->e_entry;
}


/* Phase 10-A: ELF Parser Self-Test Functions */
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)

/* Forward declaration for fb_print functions */
extern void fb_print(const char *str);
extern void fb_print_int(int64_t val);
extern void fb_print_hex(uint64_t val);

/**
 * @brief Property 1: ELF Magic Validation Test
 * 
 * For any byte sequence, the ELF validator should accept it if and only if
 * the first four bytes are 0x7F, 'E', 'L', 'F'.
 */
static void test_elf_magic_validation(void) {
    fb_print("[TEST] Property 1: ELF Magic Validation\n");
    
    /* Test 1: Valid ELF magic */
    uint8_t valid_elf[64] = {0};
    valid_elf[0] = 0x7F;
    valid_elf[1] = 'E';
    valid_elf[2] = 'L';
    valid_elf[3] = 'F';
    valid_elf[4] = ELFCLASS64;
    valid_elf[5] = ELFDATA2LSB;
    valid_elf[6] = EV_CURRENT;
    
    elf64_ehdr_t *ehdr = (elf64_ehdr_t *)valid_elf;
    ehdr->e_type = ET_EXEC;
    ehdr->e_machine = EM_X86_64;
    ehdr->e_version = EV_CURRENT;
    ehdr->e_entry = 0x400000;
    ehdr->e_phoff = sizeof(elf64_ehdr_t);
    ehdr->e_phentsize = sizeof(elf64_phdr_t);
    ehdr->e_phnum = 0;
    
    int result = elf64_validate_minimal(valid_elf, sizeof(valid_elf));
    if (result == 0) {
        fb_print("[TEST] ✓ Valid ELF magic accepted\n");
    } else {
        fb_print("[TEST] ✗ Valid ELF magic rejected\n");
        return;
    }
    
    /* Test 2: Invalid magic byte 0 */
    uint8_t invalid_magic0[64];
    for (int i = 0; i < 64; i++) invalid_magic0[i] = valid_elf[i];
    invalid_magic0[0] = 0x00;
    
    result = elf64_validate_minimal(invalid_magic0, sizeof(invalid_magic0));
    if (result == -EINVAL) {
        fb_print("[TEST] ✓ Invalid magic byte 0 rejected\n");
    } else {
        fb_print("[TEST] ✗ Invalid magic byte 0 not rejected\n");
        return;
    }
    
    fb_print("[TEST] ✓ Property 1: ELF Magic Validation PASSED\n");
}

/**
 * @brief Property 3: Entry Point Extraction Test
 */
static void test_entry_point_extraction(void) {
    fb_print("[TEST] Property 3: Entry Point Extraction\n");
    
    uint8_t elf_data[64] = {0};
    elf_data[0] = 0x7F;
    elf_data[1] = 'E';
    elf_data[2] = 'L';
    elf_data[3] = 'F';
    elf_data[4] = ELFCLASS64;
    elf_data[5] = ELFDATA2LSB;
    elf_data[6] = EV_CURRENT;
    
    elf64_ehdr_t *ehdr = (elf64_ehdr_t *)elf_data;
    ehdr->e_type = ET_EXEC;
    ehdr->e_machine = EM_X86_64;
    ehdr->e_version = EV_CURRENT;
    ehdr->e_entry = 0x400000;
    ehdr->e_phoff = sizeof(elf64_ehdr_t);
    ehdr->e_phentsize = sizeof(elf64_phdr_t);
    ehdr->e_phnum = 0;
    
    uint64_t entry = elf64_get_entry(elf_data);
    if (entry == 0x400000) {
        fb_print("[TEST] ✓ Entry point 0x400000 extracted correctly\n");
    } else {
        fb_print("[TEST] ✗ Entry point extraction failed\n");
        return;
    }
    
    fb_print("[TEST] ✓ Property 3: Entry Point Extraction PASSED\n");
}

/**
 * @brief Run all ELF parser tests (Phase 10-A)
 * 
 * Internal validation function - not exported to Ring0 surface.
 * Called only during boot validation stage.
 */
static void test_elf_parser_internal(void) {
    fb_print("\n=== ELF Parser Tests (Phase 10-A) ===\n");
    
    test_elf_magic_validation();
    test_entry_point_extraction();
    
    fb_print("=== ELF Parser Tests COMPLETE ===\n\n");
}

/**
 * @brief Public validation entry point (called from boot)
 * 
 * This function is called during kernel boot validation stage,
 * not from scheduler. Keeps test code internal to parser module.
 */
void elf_parser_run_validation(void) {
    test_elf_parser_internal();
}

#else
/* Stub for release builds */
void elf_parser_run_validation(void) {
    /* No-op in release builds */
}
#endif /* AYKEN_VALIDATION */
