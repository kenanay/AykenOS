// bootloader/efi/elf_loader.c
#include "elf_loader.h"
#include "boot_info.h"
#include <efilib.h>

static EFI_GUID LoadedImageProtocolGuid = EFI_LOADED_IMAGE_PROTOCOL_GUID;
static EFI_GUID SimpleFsGuid = EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID;
static EFI_GUID BlockIoGuid = EFI_BLOCK_IO_PROTOCOL_GUID;

// debugcon (QEMU 0xE9)
static void debugcon_write(const char *s)
{
    while (s && *s) {
        __asm__ volatile("outb %0, %1" : : "a"(*s), "Nd"(0xE9));
        s++;
    }
}

static void debugcon_hex_u64(UINT64 v)
{
    const char *hex = "0123456789ABCDEF";
    for (int i = 15; i >= 0; --i) {
        UINT8 c = (UINT8)hex[(v >> (i * 4)) & 0xF];
        __asm__ volatile("outb %0, %1" : : "a"(c), "Nd"(0xE9));
    }
}

static void debugcon_status(const char *tag, EFI_STATUS st)
{
    debugcon_write(tag);
    debugcon_write(" status=0x");
    debugcon_hex_u64((UINT64)st);
    debugcon_write("\n");
}

static void debugcon_u64(const char *tag, UINT64 v)
{
    debugcon_write(tag);
    debugcon_write("0x");
    debugcon_hex_u64(v);
    debugcon_write("\n");
}

static ayken_phdr_t g_phdrs[AYKEN_MAX_PHDR];
static UINT64 g_phdr_count = 0;
static UINT64 g_load_base_phys = 0;
static UINT64 g_min_vaddr = 0;

UINT64 ayken_get_phdr_count(void) { return g_phdr_count; }
const ayken_phdr_t *ayken_get_phdrs(void) { return g_phdrs; }
UINT64 ayken_get_load_base_phys(void) { return g_load_base_phys; }
UINT64 ayken_get_min_vaddr(void) { return g_min_vaddr; }

EFI_STATUS elf_load_kernel(
    EFI_HANDLE ImageHandle,
    EFI_SYSTEM_TABLE *SystemTable,
    CHAR16 *kernel_path,
    ayken_boot_info_t *boot_info,
    UINT64 *kernel_entry_out
)
{
    EFI_STATUS Status;
    EFI_BOOT_SERVICES *BS = SystemTable->BootServices;

    EFI_LOADED_IMAGE_PROTOCOL *LoadedImage = NULL;
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *Volume = NULL;
    EFI_FILE_HANDLE RootFS = NULL;
    EFI_FILE_HANDLE KernelFile = NULL;


    // 1) LoadedImage
    Status = uefi_call_wrapper(
        BS->HandleProtocol, 3,
        ImageHandle,
        &LoadedImageProtocolGuid,
        (void**)&LoadedImage
    );
    if (EFI_ERROR(Status) || !LoadedImage) {
        debugcon_status("[B][LOADED_IMAGE_FAIL]", Status);
        return Status;
    }

    // 2) Best-effort connect on the loaded image device
    Status = uefi_call_wrapper(
        BS->ConnectController, 4,
        LoadedImage->DeviceHandle,
        NULL, NULL, TRUE
    );
    debugcon_status("[B][CONNECT_DEVICE]", Status);

    // 3) Enumerate FS handles and find one that can open kernel.elf
    EFI_HANDLE *Handles = NULL;
    UINTN HandleCount = 0;
    EFI_STATUS LastStatus = EFI_NOT_FOUND;

    debugcon_write("[B][FS_FALLBACK_START]\n");
    Status = uefi_call_wrapper(
        BS->LocateHandleBuffer, 5,
        ByProtocol,
        &SimpleFsGuid,
        NULL,
        &HandleCount,
        &Handles
    );
    debugcon_status("[B][FS_LOCATEHB]", Status);
    EFI_HANDLE *BlkHandles = NULL;
    UINTN BlkCount = 0;
    EFI_STATUS BlkStatus = uefi_call_wrapper(
        BS->LocateHandleBuffer, 5,
        ByProtocol,
        &BlockIoGuid,
        NULL,
        &BlkCount,
        &BlkHandles
    );
    debugcon_status("[B][BLK_LOCATEHB]", BlkStatus);
    if (!EFI_ERROR(BlkStatus) && BlkHandles) {
        uefi_call_wrapper(BS->FreePool, 1, BlkHandles);
    }
    if (EFI_ERROR(Status) || !Handles || HandleCount == 0) {
        debugcon_write("[B][FS_HANDLECOUNT_ZERO]\n");
        return EFI_NOT_FOUND;
    }
    debugcon_write("[B][FS_HANDLECOUNT_NONZERO]\n");

    for (UINTN i = 0; i < HandleCount; ++i) {
        EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *Fs = NULL;
        EFI_FILE_HANDLE Root = NULL;
        EFI_FILE_HANDLE File = NULL;

        Status = uefi_call_wrapper(
            BS->ConnectController, 4,
            Handles[i],
            NULL, NULL, TRUE
        );

        Status = uefi_call_wrapper(
            BS->HandleProtocol, 3,
            Handles[i],
            &SimpleFsGuid,
            (void**)&Fs
        );
        if (EFI_ERROR(Status) || !Fs) {
            LastStatus = Status;
            continue;
        }

        Status = uefi_call_wrapper(Fs->OpenVolume, 2, Fs, &Root);
        if (EFI_ERROR(Status) || !Root) {
            LastStatus = Status;
            continue;
        }

        Status = uefi_call_wrapper(
            Root->Open, 5,
            Root,
            &File,
            kernel_path,
            EFI_FILE_MODE_READ,
            0
        );
        if (EFI_ERROR(Status)) {
            LastStatus = Status;
            Status = uefi_call_wrapper(
                Root->Open, 5,
                Root,
                &File,
                L"\\kernel.elf",
                EFI_FILE_MODE_READ,
                0
            );
        }

        if (!EFI_ERROR(Status) && File) {
            Volume = Fs;
            RootFS = Root;
            KernelFile = File;
            break;
        }

        if (Root) {
            uefi_call_wrapper(Root->Close, 1, Root);
        }
    }

    uefi_call_wrapper(BS->FreePool, 1, Handles);

    if (!Volume) {
        debugcon_status("[B][FS_FALLBACK_FAIL]", LastStatus);
        return LastStatus;
    }

    debugcon_write("[B][FS_FALLBACK_OK]\n");

    // 4) Open volume + kernel if fallback didn't already open it
    if (!KernelFile) {
        Status = uefi_call_wrapper(Volume->OpenVolume, 2, Volume, &RootFS);
        if (EFI_ERROR(Status) || !RootFS) {
            debugcon_status("[B][OPEN_VOLUME_FAIL]", Status);
            return Status;
        }

        Status = uefi_call_wrapper(
            RootFS->Open, 5,
            RootFS,
            &KernelFile,
            kernel_path,
            EFI_FILE_MODE_READ,
            0
        );
        if (EFI_ERROR(Status)) {
            Status = uefi_call_wrapper(
                RootFS->Open, 5,
                RootFS,
                &KernelFile,
                L"\\kernel.elf",
                EFI_FILE_MODE_READ,
                0
            );
        }
        if (EFI_ERROR(Status) || !KernelFile) {
            debugcon_status("[B][KERNEL_OPEN_FAIL]", Status);
            return Status;
        }
    }

    debugcon_write("[B][KERNEL_OPEN_OK]\n");

    // ELF header read + magic check (stage 1)
    Elf64_Ehdr Eh;
    UINTN Size = sizeof(Eh);
    Status = uefi_call_wrapper(KernelFile->Read, 3, KernelFile, &Size, &Eh);
    if (EFI_ERROR(Status) || Size != sizeof(Eh)) {
        debugcon_status("[B][ELF_HDR_READ_FAIL]", Status);
        return EFI_LOAD_ERROR;
    }
    debugcon_write("[B][ELF_HDR_READ_OK]\n");

    if (Eh.e_ident[0] != 0x7F || Eh.e_ident[1] != 'E' ||
        Eh.e_ident[2] != 'L'  || Eh.e_ident[3] != 'F') {
        debugcon_write("[B][ELF_MAGIC_FAIL]\n");
        return EFI_LOAD_ERROR;
    }
    debugcon_write("[B][ELF_MAGIC_OK]\n");
    if (kernel_entry_out) { *kernel_entry_out = (UINT64)Eh.e_entry; }
    if (boot_info) { boot_info->kernel_entry = (UINT64)Eh.e_entry; }
    debugcon_u64("[B][ELF_E_ENTRY]=", (UINT64)Eh.e_entry);
    debugcon_u64("[B][ELF_PH_OFF]=", (UINT64)Eh.e_phoff);
    debugcon_u64("[B][ELF_PH_NUM]=", (UINT64)Eh.e_phnum);
    debugcon_u64("[B][ELF_PH_ENT]=", (UINT64)Eh.e_phentsize);

    // Program header table read (stage 2)
    if (Eh.e_phentsize == 0 || Eh.e_phnum == 0) {
        debugcon_write("[B][PH_TABLE_EMPTY]\n");
    } else {
        UINTN PhTableSize = (UINTN)(Eh.e_phentsize * Eh.e_phnum);
        Elf64_Phdr *PhTable = NULL;
        Status = uefi_call_wrapper(SystemTable->BootServices->AllocatePool, 3, EfiLoaderData, PhTableSize, (void**)&PhTable);
        if (EFI_ERROR(Status) || !PhTable) {
            debugcon_status("[B][PH_ALLOC_FAIL]", Status);
            return EFI_LOAD_ERROR;
        }

        Status = uefi_call_wrapper(KernelFile->SetPosition, 2, KernelFile, (UINT64)Eh.e_phoff);
        if (EFI_ERROR(Status)) {
            debugcon_status("[B][PH_SEEK_FAIL]", Status);
            uefi_call_wrapper(SystemTable->BootServices->FreePool, 1, PhTable);
            return EFI_LOAD_ERROR;
        }

        UINTN ReadSize = PhTableSize;
        Status = uefi_call_wrapper(KernelFile->Read, 3, KernelFile, &ReadSize, PhTable);
        if (EFI_ERROR(Status) || ReadSize != PhTableSize) {
            debugcon_status("[B][PH_READ_FAIL]", Status);
            uefi_call_wrapper(SystemTable->BootServices->FreePool, 1, PhTable);
            return EFI_LOAD_ERROR;
        }
        debugcon_write("[B][PH_READ_OK]\n");

        UINT64 load_count = 0;
        Elf64_Phdr *Ph0 = NULL;
        for (UINTN i = 0; i < Eh.e_phnum; ++i) {
            if (PhTable[i].p_type == 1) {
                load_count++;
                if (!Ph0) { Ph0 = &PhTable[i]; }
                if (load_count == 1) {
                    debugcon_u64("[B][PH0_TYPE]=", (UINT64)PhTable[i].p_type);
                    debugcon_u64("[B][PH0_OFF]=", (UINT64)PhTable[i].p_offset);
                    debugcon_u64("[B][PH0_VADDR]=", (UINT64)PhTable[i].p_vaddr);
                    debugcon_u64("[B][PH0_PADDR]=", (UINT64)PhTable[i].p_paddr);
                    debugcon_u64("[B][PH0_FILESZ]=", (UINT64)PhTable[i].p_filesz);
                    debugcon_u64("[B][PH0_MEMSZ]=", (UINT64)PhTable[i].p_memsz);
                    debugcon_u64("[B][PH0_ALIGN]=", (UINT64)PhTable[i].p_align);
                    debugcon_u64("[B][PH0_FLAGS]=", (UINT64)PhTable[i].p_flags);
                }
            }
        }
        debugcon_u64("[B][PH_LOAD_COUNT]=", load_count);

        if (Ph0) {
            UINT64 min_vaddr = 0;
            UINT64 min_vaddr_offset = 0;
            UINT64 max_vaddr = 0;
            int first = 1;

            for (UINTN i = 0; i < Eh.e_phnum; ++i) {
                if (PhTable[i].p_type != 1)
                    continue;
                UINT64 vstart = (UINT64)PhTable[i].p_vaddr;
                UINT64 vend = vstart + (UINT64)PhTable[i].p_memsz;
                if (first || vstart < min_vaddr) {
                    min_vaddr = vstart;
                    min_vaddr_offset = PhTable[i].p_offset;
                }
                if (first || vend > max_vaddr) max_vaddr = vend;
                first = 0;
            }

            UINT64 total_size = (max_vaddr > min_vaddr) ? (max_vaddr - min_vaddr) : 0;
            UINT64 total_size_aligned = (total_size + 0xFFFULL) & ~0xFFFULL;
            UINT64 alloc_size = total_size_aligned;
            UINTN pages = (UINTN)((alloc_size + 0xFFFULL) / 0x1000ULL);
            EFI_PHYSICAL_ADDRESS base_phys = 0;

            debugcon_u64("[B][KSEG_VBASE]=", min_vaddr);
            debugcon_u64("[B][KSEG_SIZE]=", total_size_aligned);
            debugcon_u64("[B][KSEG_PAGES]=", (UINT64)pages);
            debugcon_u64("[B][KSEG_MIN_OFF]=", min_vaddr_offset);

            Status = uefi_call_wrapper(SystemTable->BootServices->AllocatePages, 4,
                                       AllocateAnyPages, EfiLoaderData, pages, &base_phys);
            if (EFI_ERROR(Status)) {
                debugcon_status("[B][KSEG_ALLOC_FAIL]", Status);
                uefi_call_wrapper(SystemTable->BootServices->FreePool, 1, PhTable);
                return EFI_LOAD_ERROR;
            }
            UINT64 kseg_phys_map = (UINT64)base_phys;
            debugcon_u64("[B][KSEG_PHYS]=", (UINT64)base_phys);
            debugcon_u64("[B][KSEG_PHYS_MAP]=", kseg_phys_map);

            g_phdr_count = 0;
            g_load_base_phys = kseg_phys_map;
            g_min_vaddr = min_vaddr;

            for (UINTN i = 0; i < Eh.e_phnum; ++i) {
                if (PhTable[i].p_type != 1)
                    continue;

                UINT64 seg_off = (UINT64)PhTable[i].p_offset;
                UINT64 seg_filesz = (UINT64)PhTable[i].p_filesz;
                UINT64 seg_memsz = (UINT64)PhTable[i].p_memsz;
                UINT64 seg_vaddr = (UINT64)PhTable[i].p_vaddr;

                UINT64 seg_delta = seg_vaddr - min_vaddr;
                EFI_PHYSICAL_ADDRESS seg_phys = (EFI_PHYSICAL_ADDRESS)(kseg_phys_map + seg_delta);

                if (g_phdr_count < AYKEN_MAX_PHDR) {
                    g_phdrs[g_phdr_count].vaddr = seg_vaddr;
                    g_phdrs[g_phdr_count].memsz = seg_memsz;
                    g_phdr_count++;
                }

                Status = uefi_call_wrapper(KernelFile->SetPosition, 2, KernelFile, seg_off);
                if (EFI_ERROR(Status)) {
                    debugcon_status("[B][SEG_SEEK_FAIL]", Status);
                    uefi_call_wrapper(SystemTable->BootServices->FreePool, 1, PhTable);
                    return EFI_LOAD_ERROR;
                }

                UINTN ReadSz = (UINTN)seg_filesz;
                Status = uefi_call_wrapper(KernelFile->Read, 3, KernelFile, &ReadSz, (void*)seg_phys);
                if (EFI_ERROR(Status) || ReadSz != seg_filesz) {
                    debugcon_status("[B][SEG_READ_FAIL]", Status);
                    uefi_call_wrapper(SystemTable->BootServices->FreePool, 1, PhTable);
                    return EFI_LOAD_ERROR;
                }

                if (seg_memsz > seg_filesz) {
                    UINTN Diff = (UINTN)(seg_memsz - seg_filesz);
                    uefi_call_wrapper(SystemTable->BootServices->SetMem, 3, (void*)(seg_phys + seg_filesz), Diff, 0);
                }
            }

            if (boot_info) {
                boot_info->kernel_virt_base = min_vaddr;
                boot_info->kernel_phys_base = kseg_phys_map;
                boot_info->kernel_map_size  = total_size_aligned;
                boot_info->kernel_phys_start = kseg_phys_map;
                boot_info->kernel_phys_end   = kseg_phys_map + total_size_aligned;
            }

            debugcon_write("[B][KSEG_LOAD_OK]\n");
        } else {
            debugcon_write("[B][SEG0_NOT_FOUND]\n");
        }

        uefi_call_wrapper(SystemTable->BootServices->FreePool, 1, PhTable);
    }

    // Minimal proof only
    uefi_call_wrapper(KernelFile->Close, 1, KernelFile);
    if (RootFS) {
        uefi_call_wrapper(RootFS->Close, 1, RootFS);
    }

    return EFI_SUCCESS;
}
