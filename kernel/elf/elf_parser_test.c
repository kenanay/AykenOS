/**
 * @file elf_parser_test.c
 * @brief Unit tests for ELF64 parser
 * 
 * This file contains unit tests for ELF64 validation and parsing functions.
 * Phase 10-A: Basic validation tests
 * Phase 10-B: Property-based tests (recommended)
 * 
 * @author Kenan AY
 * @date 2026-02-27
 */

#include <elf/elf64.h>
#include <elf/parser.h>
#include <errno.h>
#include "../drivers/console/fb_console.h"

/**
 * @brief Property 1: ELF Magic Validation
 * 
 * For any byte sequence, the ELF validator should accept it if and only if
 * the first four bytes are 0x7F, 'E', 'L', 'F'.
 * 
 * Validates: Requirements 1.1
 * 
 * Note: This is a basic unit test. Full property-based testing with
 * random input generation is recommended for Phase 10-B.
 */
static void test_elf_magic_validation(void)
{
    fb_print("[TEST] Property 1: ELF Magic Validation\n");
    
    /* Test 1: Valid ELF magic */
    uint8_t valid_elf[64] = {0};
    valid_elf[0] = 0x7F;
    valid_elf[1] = 'E';
    valid_elf[2] = 'L';
    valid_elf[3] = 'F';
    valid_elf[4] = ELFCLASS64;  /* 64-bit */
    valid_elf[5] = ELFDATA2LSB; /* Little-endian */
    valid_elf[6] = EV_CURRENT;  /* Version */
    
    /* Fill in minimal ELF header fields */
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
        fb_print("[TEST] ✗ Valid ELF magic rejected (error: ");
        fb_print_int(result);
        fb_print(")\n");
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
    
    /* Test 3: Invalid magic byte 1 */
    uint8_t invalid_magic1[64];
    for (int i = 0; i < 64; i++) invalid_magic1[i] = valid_elf[i];
    invalid_magic1[1] = 'X';
    
    result = elf64_validate_minimal(invalid_magic1, sizeof(invalid_magic1));
    if (result == -EINVAL) {
        fb_print("[TEST] ✓ Invalid magic byte 1 rejected\n");
    } else {
        fb_print("[TEST] ✗ Invalid magic byte 1 not rejected\n");
        return;
    }
    
    /* Test 4: Invalid magic byte 2 */
    uint8_t invalid_magic2[64];
    for (int i = 0; i < 64; i++) invalid_magic2[i] = valid_elf[i];
    invalid_magic2[2] = 'X';
    
    result = elf64_validate_minimal(invalid_magic2, sizeof(invalid_magic2));
    if (result == -EINVAL) {
        fb_print("[TEST] ✓ Invalid magic byte 2 rejected\n");
    } else {
        fb_print("[TEST] ✗ Invalid magic byte 2 not rejected\n");
        return;
    }
    
    /* Test 5: Invalid magic byte 3 */
    uint8_t invalid_magic3[64];
    for (int i = 0; i < 64; i++) invalid_magic3[i] = valid_elf[i];
    invalid_magic3[3] = 'X';
    
    result = elf64_validate_minimal(invalid_magic3, sizeof(invalid_magic3));
    if (result == -EINVAL) {
        fb_print("[TEST] ✓ Invalid magic byte 3 rejected\n");
    } else {
        fb_print("[TEST] ✗ Invalid magic byte 3 not rejected\n");
        return;
    }
    
    fb_print("[TEST] ✓ Property 1: ELF Magic Validation PASSED\n");
}

/**
 * @brief Property 3: Entry Point Extraction
 * 
 * For any valid ELF binary, the extracted entry point should equal
 * the e_entry field from the ELF header.
 * 
 * Validates: Requirements 1.3
 * 
 * Note: This is a basic unit test. Full property-based testing with
 * random entry point values is recommended for Phase 10-B.
 */
static void test_entry_point_extraction(void)
{
    fb_print("[TEST] Property 3: Entry Point Extraction\n");
    
    /* Test 1: Standard entry point (0x400000) */
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
        fb_print("[TEST] ✗ Entry point extraction failed (expected 0x400000, got 0x");
        fb_print_hex(entry);
        fb_print(")\n");
        return;
    }
    
    /* Test 2: Different entry point (0x401000) */
    ehdr->e_entry = 0x401000;
    entry = elf64_get_entry(elf_data);
    if (entry == 0x401000) {
        fb_print("[TEST] ✓ Entry point 0x401000 extracted correctly\n");
    } else {
        fb_print("[TEST] ✗ Entry point extraction failed (expected 0x401000, got 0x");
        fb_print_hex(entry);
        fb_print(")\n");
        return;
    }
    
    /* Test 3: High address entry point (0x00007FFFFFFFE000) */
    ehdr->e_entry = 0x00007FFFFFFFE000ULL;
    entry = elf64_get_entry(elf_data);
    if (entry == 0x00007FFFFFFFE000ULL) {
        fb_print("[TEST] ✓ Entry point 0x00007FFFFFFFE000 extracted correctly\n");
    } else {
        fb_print("[TEST] ✗ Entry point extraction failed\n");
        return;
    }
    
    fb_print("[TEST] ✓ Property 3: Entry Point Extraction PASSED\n");
}

/**
 * @brief Additional validation tests
 * 
 * Tests for class validation, machine validation, and bounds checking.
 */
static void test_additional_validation(void)
{
    fb_print("[TEST] Additional ELF Validation Tests\n");
    
    /* Create valid base ELF */
    uint8_t elf_data[128] = {0};
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
    
    /* Test: Invalid class (32-bit) */
    uint8_t invalid_class[128];
    for (int i = 0; i < 128; i++) invalid_class[i] = elf_data[i];
    invalid_class[4] = ELFCLASS32;
    
    int result = elf64_validate_minimal(invalid_class, sizeof(invalid_class));
    if (result == -ENOEXEC) {
        fb_print("[TEST] ✓ 32-bit ELF rejected\n");
    } else {
        fb_print("[TEST] ✗ 32-bit ELF not rejected\n");
        return;
    }
    
    /* Test: Invalid machine (not x86_64) */
    uint8_t invalid_machine[128];
    for (int i = 0; i < 128; i++) invalid_machine[i] = elf_data[i];
    ((elf64_ehdr_t *)invalid_machine)->e_machine = EM_NONE;
    
    result = elf64_validate_minimal(invalid_machine, sizeof(invalid_machine));
    if (result == -ENOEXEC) {
        fb_print("[TEST] ✓ Non-x86_64 ELF rejected\n");
    } else {
        fb_print("[TEST] ✗ Non-x86_64 ELF not rejected\n");
        return;
    }
    
    /* Test: Too small size */
    result = elf64_validate_minimal(elf_data, 10);
    if (result == -EINVAL) {
        fb_print("[TEST] ✓ Too small ELF rejected\n");
    } else {
        fb_print("[TEST] ✗ Too small ELF not rejected\n");
        return;
    }
    
    /* Test: NULL pointer */
    result = elf64_validate_minimal(0, 100);
    if (result == -EINVAL) {
        fb_print("[TEST] ✓ NULL pointer rejected\n");
    } else {
        fb_print("[TEST] ✗ NULL pointer not rejected\n");
        return;
    }
    
    fb_print("[TEST] ✓ Additional Validation Tests PASSED\n");
}

/**
 * @brief Run all ELF parser tests
 */
void test_elf_parser(void)
{
    fb_print("\n=== ELF Parser Tests (Phase 10-A) ===\n");
    
    test_elf_magic_validation();
    test_entry_point_extraction();
    test_additional_validation();
    
    fb_print("=== ELF Parser Tests COMPLETE ===\n\n");
}
