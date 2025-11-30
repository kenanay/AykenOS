# ============================================================
# AykenOS Build System
#  - x86_64 Kernel (ELF, higher-half)
#  - UEFI Bootloader (BOOTX64.EFI)
#  - EFI.img + QEMU run
# ============================================================

# ------------------------------------------------------------
# 1) Kernel toolchain
# ------------------------------------------------------------

KERNEL_CC = x86_64-elf-gcc
KERNEL_LD = x86_64-elf-ld

KERNEL_CFLAGS = -ffreestanding -m64 -O2 -Wall -Wextra -Ikernel/include
KERNEL_CFLAGS += -mcmodel=large -fno-pic -fno-omit-frame-pointer -fno-stack-protector
KERNEL_CFLAGS += -mno-red-zone

KERNEL_LDFLAGS = -nostdlib -z max-page-size=0x1000

KERNEL_ELF = kernel.elf

KERNEL_DIR = kernel
ARCH_DIR   = kernel/arch/x86_64

# Kernel kaynak dosyaları
KERNEL_C_SOURCES   = $(shell find $(KERNEL_DIR) -type f -name "*.c")
KERNEL_ASM_SOURCES = $(shell find $(ARCH_DIR) -type f -name "*.asm")

KERNEL_OBJS = $(KERNEL_C_SOURCES:.c=.o) $(KERNEL_ASM_SOURCES:.asm=.o)


# ------------------------------------------------------------
# 2) UEFI Bootloader toolchain
# ------------------------------------------------------------
# Burayı ortamına göre ayarlayacaksın:
# - Eğer clang kullanıyorsan: EFI_CC = clang
# - Eğer mingw-w64 kullanıyorsan: EFI_CC = x86_64-w64-mingw32-gcc
#
# Windows + WSL senaryosunda genelde clang daha rahat.

EFI_CC = clang

# gnu-efi veya EDK2 header dizinlerin varsa buraya ekle:
# Örn (Linux, gnu-efi için):
# EFI_INC = /usr/include/efi
# EFI_INC_ARCH = /usr/include/efi/x86_64
EFI_INC      =
EFI_INC_ARCH =

EFI_CFLAGS = -ffreestanding -fshort-wchar -mno-red-zone -Wall -Wextra
EFI_CFLAGS += -I$(EFI_INC) -I$(EFI_INC_ARCH) -Ikernel/include
EFI_CFLAGS += -target x86_64-pc-win32-coff

EFI_LDFLAGS = -nostdlib \
  -Wl,--subsystem,efi \
  -Wl,--image-base,0x100000 \
  -Wl,--entry,efi_main

BOOTLOADER_DIR = bootloader/efi

EFI_SRC = \
  $(BOOTLOADER_DIR)/efi_main.c \
  $(BOOTLOADER_DIR)/ayken_boot.c \
  $(BOOTLOADER_DIR)/elf_loader.c \
  $(BOOTLOADER_DIR)/paging.c

EFI_OBJS = $(EFI_SRC:.c=.efi.o)

BOOT_EFI = $(BOOTLOADER_DIR)/BOOTX64.EFI


# ------------------------------------------------------------
# 3) Top-level hedefler
# ------------------------------------------------------------

all: $(KERNEL_ELF) $(BOOT_EFI)

kernel: $(KERNEL_ELF)
bootloader: $(BOOT_EFI)


# ------------------------------------------------------------
# 4) Kernel build
# ------------------------------------------------------------

$(KERNEL_ELF): $(KERNEL_OBJS) linker.ld
	$(KERNEL_LD) -T linker.ld $(KERNEL_LDFLAGS) -o $@ $(KERNEL_OBJS)

# C -> .o
%.o: %.c
	$(KERNEL_CC) $(KERNEL_CFLAGS) -c $< -o $@

# asm -> .o (kernel/arch/x86_64/*.asm)
%.o: %.asm
	nasm -f elf64 $< -o $@


# ------------------------------------------------------------
# 5) UEFI Bootloader build (BOOTX64.EFI)
# ------------------------------------------------------------

$(BOOT_EFI): $(EFI_OBJS)
	$(EFI_CC) $(EFI_LDFLAGS) -o $@ $(EFI_OBJS)

$(BOOTLOADER_DIR)/%.efi.o: $(BOOTLOADER_DIR)/%.c
	$(EFI_CC) $(EFI_CFLAGS) -c $< -o $@


# ------------------------------------------------------------
# 6) EFI disk image + QEMU
# ------------------------------------------------------------

EFI_IMG = EFI.img

efi-img: $(KERNEL_ELF) $(BOOT_EFI)
	./make_efi_img.sh

run: efi-img
	qemu-system-x86_64 -drive format=raw,file=$(EFI_IMG)

clean:
	rm -f $(KERNEL_OBJS) $(KERNEL_ELF) $(EFI_OBJS) $(BOOT_EFI) $(EFI_IMG)

.PHONY: all clean run efi-img kernel bootloader
