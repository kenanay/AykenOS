/**
 * @file parser.h
 * @brief ELF64 parser interface for AykenOS
 * 
 * This file declares the public interface for ELF64 parsing and validation.
 * 
 * @author Kenan AY
 * @date 2026-02-27
 */

#ifndef AYKEN_ELF_PARSER_H
#define AYKEN_ELF_PARSER_H

#include <stdint.h>
#include <stddef.h>

/**
 * @brief Validate minimal ELF64 binary (Phase 10-A)
 * 
 * Performs essential validation checks on an ELF64 binary including
 * magic number, class, machine type, and critical bounds checking.
 * 
 * @param blob Pointer to ELF binary data
 * @param size Size of ELF binary in bytes
 * @return 0 on success, -EINVAL on invalid format, -ENOEXEC on unsupported format
 */
int elf64_validate_minimal(const uint8_t *blob, size_t size);

/**
 * @brief Extract entry point address from ELF64 binary
 * 
 * Returns the virtual address where execution should begin (e_entry field).
 * This function assumes the ELF binary has already been validated.
 * 
 * @param blob Pointer to validated ELF binary data
 * @return Entry point virtual address (e_entry)
 */
uint64_t elf64_get_entry(const uint8_t *blob);

/**
 * @brief Run ELF parser validation tests (Phase 10-A)
 * 
 * Internal validation function called during kernel boot.
 * Only available in validation builds (AYKEN_VALIDATION=1).
 * Not part of production Ring0 export surface.
 */
void elf_parser_run_validation(void);

#endif /* AYKEN_ELF_PARSER_H */
