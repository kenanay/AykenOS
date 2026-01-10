#pragma once
// Stub efilib.h to allow building without gnu-efi headers

#include <efi.h>

extern EFI_GUID AcpiTableGuid;

UINTN EFIAPI Print(IN CONST CHAR16 *fmt, ...);
INTN EFIAPI CompareGuid(IN EFI_GUID *g1, IN EFI_GUID *g2);
INTN EFIAPI CompareMem(IN CONST VOID *buf1, IN CONST VOID *buf2, IN UINTN len);

static inline void InitializeLib(EFI_HANDLE image, EFI_SYSTEM_TABLE *systab) {
    (void)image; (void)systab;
}
