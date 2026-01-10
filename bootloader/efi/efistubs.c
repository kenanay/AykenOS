#include <efi.h>
#include <efilib.h>

// Minimal stub implementations to satisfy EFI build when gnu-efi libs are absent.

EFI_GUID AcpiTableGuid = {0xeb9d2d30, 0x2d88, 0x11d3, {0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d}};
EFI_GUID gEfiSimpleFileSystemProtocolGuid = {0x0964e5b2, 0x6459, 0x11d2, {0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b}};

VOID EFIAPI SetMem(IN VOID *Buffer, IN UINTN Size, IN UINT8 Value)
{
    UINT8 *p = (UINT8 *)Buffer;
    for (UINTN i = 0; i < Size; i++) {
        p[i] = Value;
    }
}

INTN EFIAPI CompareMem(IN CONST VOID *Dest, IN CONST VOID *Src, IN UINTN len)
{
    const UINT8 *a = (const UINT8 *)Dest;
    const UINT8 *b = (const UINT8 *)Src;
    for (UINTN i = 0; i < len; i++) {
        if (a[i] != b[i]) {
            return (a[i] < b[i]) ? -1 : 1;
        }
    }
    return 0;
}

INTN EFIAPI CompareGuid(IN EFI_GUID *g1, IN EFI_GUID *g2)
{
    return CompareMem(g1, g2, sizeof(EFI_GUID));
}

VOID EFIAPI InitializeLib(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
    (void)ImageHandle;
    (void)SystemTable;
}

UINTN EFIAPI Print(IN CONST CHAR16 *fmt, ...)
{
    (void)fmt;
    return 0;
}

// libc shims for bootloader objects
void *memset(void *s, int c, size_t n)
{
    UINT8 *p = (UINT8 *)s;
    for (size_t i = 0; i < n; i++) {
        p[i] = (UINT8)c;
    }
    return s;
}
