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

/*
 * Minimal validation and entry extraction helpers are intentionally private
 * to parser.c to keep Ring0 export surface stable.
 */

/**
 * @brief Run ELF parser validation tests (Phase 10-A)
 * 
 * Internal validation function called during kernel boot.
 * Only available in validation builds (AYKEN_VALIDATION=1).
 * Not part of production Ring0 export surface.
 */
void elf_parser_run_validation(void);

#endif /* AYKEN_ELF_PARSER_H */
