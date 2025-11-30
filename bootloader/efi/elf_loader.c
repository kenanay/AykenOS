// bootloader/efi/elf_loader.c
#include "elf_loader.h"
#include "boot_info.h"

EFI_STATUS elf_load_kernel(EFI_HANDLE ImageHandle,
                           EFI_SYSTEM_TABLE *SystemTable,
                           CHAR16 *kernel_path,
                           ayken_boot_info_t *boot_info,
                           UINT64 *kernel_entry_out)
{
    EFI_STATUS Status;
    EFI_FILE_IO_INTERFACE *FileIO;
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *Volume;
    EFI_FILE_HANDLE RootFS;
    EFI_FILE_HANDLE KernelFile;

    // 1) Dosya sistemine eriş
    Status = SystemTable->BootServices->HandleProtocol(
        ImageHandle,
        &gEfiSimpleFileSystemProtocolGuid,
        (void**)&Volume
    );
    if (EFI_ERROR(Status)) return Status;

    Status = Volume->OpenVolume(Volume, &RootFS);
    if (EFI_ERROR(Status)) return Status;

    // 2) kernel.elf dosyasını aç
    Status = RootFS->Open(
        RootFS,
        &KernelFile,
        kernel_path,
        EFI_FILE_MODE_READ,
        0
    );
    if (EFI_ERROR(Status)) return Status;

    // 3) ELF header’ı oku
    Elf64_Ehdr Eh;
    UINTN Size = sizeof(Eh);
    Status = KernelFile->Read(KernelFile, &Size, &Eh);
    if (EFI_ERROR(Status) || Size != sizeof(Eh)) {
        KernelFile->Close(KernelFile);
        return EFI_LOAD_ERROR;
    }

    // ELF sihirli bayt kontrolü: 0x7F 'E' 'L' 'F'
    if (Eh.e_ident[0] != 0x7F || Eh.e_ident[1] != 'E' ||
        Eh.e_ident[2] != 'L'  || Eh.e_ident[3] != 'F') {
        KernelFile->Close(KernelFile);
        return EFI_LOAD_ERROR;
    }

    // 4) Program header’ları oku
    Elf64_Phdr *PhTable;
    UINTN PhTableSize = Eh.e_phentsize * Eh.e_phnum;

    Status = SystemTable->BootServices->AllocatePool(
        EfiLoaderData, PhTableSize, (void**)&PhTable
    );
    if (EFI_ERROR(Status)) {
        KernelFile->Close(KernelFile);
        return Status;
    }

    Status = KernelFile->SetPosition(KernelFile, Eh.e_phoff);
    if (EFI_ERROR(Status)) goto ERR;

    Size = PhTableSize;
    Status = KernelFile->Read(KernelFile, &Size, PhTable);
    if (EFI_ERROR(Status) || Size != PhTableSize) goto ERR;

    // 5) PT_LOAD segmentlerini fiziksel belleğe kopyala
    UINT64 kernel_phys_start = (UINT64)-1;
    UINT64 kernel_phys_end   = 0;

    for (int i = 0; i < Eh.e_phnum; ++i) {
        Elf64_Phdr *Ph = &PhTable[i];

        if (Ph->p_type != PT_LOAD)
            continue;

        UINT64 seg_phys = (UINT64)Ph->p_paddr;
        UINT64 seg_size_mem = (UINT64)Ph->p_memsz;
        UINT64 seg_size_file = (UINT64)Ph->p_filesz;
        UINT64 seg_offset = (UINT64)Ph->p_offset;

        if (seg_phys < kernel_phys_start)
            kernel_phys_start = seg_phys;
        if (seg_phys + seg_size_mem > kernel_phys_end)
            kernel_phys_end = seg_phys + seg_size_mem;

        // Segmenti yükle
        // Not: UEFI AllocatePages ile fiziksel adres isteyebiliriz.
        EFI_PHYSICAL_ADDRESS Dest = seg_phys;
        UINTN Pages = (seg_size_mem + 0xFFF) / 0x1000;

        Status = SystemTable->BootServices->AllocatePages(
            AllocateAddress, EfiLoaderData, Pages, &Dest
        );
        if (EFI_ERROR(Status)) goto ERR;

        // Dosyadan içeriği oku
        Status = KernelFile->SetPosition(KernelFile, seg_offset);
        if (EFI_ERROR(Status)) goto ERR;

        UINTN ReadSize = seg_size_file;
        Status = KernelFile->Read(KernelFile, &ReadSize, (void*)Dest);
        if (EFI_ERROR(Status)) goto ERR;

        // Dosya boyutundan büyük memsz kısmını sıfırla (bss)
        if (seg_size_mem > seg_size_file) {
            UINT64 diff = seg_size_mem - seg_size_file;
            UINT8 *zero_start = (UINT8*)(Dest + seg_size_file);
            for (UINT64 j = 0; j < diff; ++j)
                zero_start[j] = 0;
        }
    }

    // 6) boot_info içini doldur
    boot_info->kernel_phys_start = kernel_phys_start;
    boot_info->kernel_phys_end   = kernel_phys_end;

    // kernel entry (ELF entry adresi)
    *kernel_entry_out = (UINT64)Eh.e_entry;

    SystemTable->BootServices->FreePool(PhTable);
    KernelFile->Close(KernelFile);
    return EFI_SUCCESS;

ERR:
    SystemTable->BootServices->FreePool(PhTable);
    KernelFile->Close(KernelFile);
    return EFI_LOAD_ERROR;
}
