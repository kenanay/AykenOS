# ============================================================
# AykenOS Build System
#  - x86_64 Kernel (ELF, higher-half)
#  - UEFI Bootloader (BOOTX64.EFI)
#  - EFI.img + QEMU run
# ============================================================

# ------------------------------------------------------------
# 1) Kernel toolchain
# ------------------------------------------------------------

KERNEL_CC = clang --target=x86_64-elf
KERNEL_LD = ld.lld

KERNEL_CFLAGS = -ffreestanding -m64 -O2 -Wall -Wextra -Ikernel/include -Iuserspace/libayken
KERNEL_CFLAGS += -mcmodel=large -fno-pic -fno-omit-frame-pointer -fno-stack-protector
KERNEL_CFLAGS += -mno-red-zone
# For gdt_idt.c force kernel code model to avoid 32-bit relocations in higher half
KERNEL_CFLAGS_GDT := $(filter-out -mcmodel=large,$(KERNEL_CFLAGS)) -mcmodel=kernel

KERNEL_LDFLAGS = -nostdlib -z max-page-size=0x1000

KERNEL_ELF = kernel.elf

KERNEL_DIR = kernel
ARCH_DIR   = kernel/arch/x86_64

# Kernel kaynak dosyaları
ifeq ($(OS),Windows_NT)
define find_files
$(shell powershell -NoProfile -Command "Get-ChildItem -Path '$(1)' -Recurse -Filter $(2) -File | ForEach-Object { $$_.FullName }")
endef
else
define find_files
$(shell find $(1) -type f -name "$(2)")
endef
endif

KERNEL_C_SOURCES   = $(call find_files,$(KERNEL_DIR),*.c)
KERNEL_ASM_SOURCES = $(call find_files,$(ARCH_DIR),*.asm)
KERNEL_S_SOURCES   = $(call find_files,$(ARCH_DIR),*.S)

# Userspace library sources (Ring3 VFS implementation)
USERSPACE_DIR = userspace/libayken
USERSPACE_C_SOURCES = $(call find_files,$(USERSPACE_DIR),*.c)

KERNEL_OBJS = $(KERNEL_C_SOURCES:.c=.o) $(KERNEL_ASM_SOURCES:.asm=.o) $(KERNEL_S_SOURCES:.S=.o) \
	userspace/libayken/scheduler_stubs.o \
	userspace/libayken/devfs.o \
	userspace/libayken/vfs.o \
	userspace/libayken/vfs_test.o

# Rust workspace (userspace runtime/dispatcher)
USERSPACE_RUST_DIR = userspace
USERSPACE_RUNTIME_BIN = $(USERSPACE_RUST_DIR)/target/debug/dispatcher.exe


# ------------------------------------------------------------
# 2) UEFI Bootloader toolchain
# ------------------------------------------------------------
# Burayı ortamına göre ayarlayacaksın:
# - Eğer clang kullanıyorsan: EFI_CC = clang
# - Eğer mingw-w64 kullanıyorsan: EFI_CC = x86_64-w64-mingw32-gcc
#
# Windows + WSL senaryosunda genelde clang daha rahat.

EFI_CC = clang
EFI_LD = lld-link

# gnu-efi veya EDK2 header dizinlerin varsa buraya ekle:
# Örn (Linux, gnu-efi için):
# EFI_INC = /usr/include/efi
# EFI_INC_ARCH = /usr/include/efi/x86_64
EFI_INC      = tools/gnu-efi/inc
EFI_INC_ARCH = tools/gnu-efi/inc/x86_64

EFI_CFLAGS = -ffreestanding -fshort-wchar -mno-red-zone -Wall -Wextra
EFI_CFLAGS += -I$(EFI_INC) -I$(EFI_INC_ARCH) -Ikernel/include
EFI_CFLAGS += -target x86_64-pc-win32-coff

EFI_LDFLAGS = /nodefaultlib /subsystem:efi_application /entry:efi_main /base:0x100000

BOOTLOADER_DIR = bootloader/efi

EFI_SRC = \
  $(BOOTLOADER_DIR)/efi_main.c \
  $(BOOTLOADER_DIR)/ayken_boot.c \
  $(BOOTLOADER_DIR)/elf_loader.c \
  $(BOOTLOADER_DIR)/paging.c \
  $(BOOTLOADER_DIR)/efistubs.c

EFI_ASM_SRC = \
  $(BOOTLOADER_DIR)/boot.S \
  $(BOOTLOADER_DIR)/boot_idt.S
EFI_ASM_OBJS = $(EFI_ASM_SRC:.S=.efi.o)

EFI_OBJS = $(EFI_SRC:.c=.efi.o) $(EFI_ASM_OBJS)

BOOT_EFI = $(BOOTLOADER_DIR)/BOOTX64.EFI


# ------------------------------------------------------------
# 3) Top-level hedefler
# ------------------------------------------------------------

all: check-deps $(KERNEL_ELF) $(BOOT_EFI)

kernel: check-deps $(KERNEL_ELF)
bootloader: check-deps $(BOOT_EFI)
userspace-runtime:
	@cd $(USERSPACE_RUST_DIR) && cargo build -p bcib-runtime --bin dispatcher


# ------------------------------------------------------------
# 4) Kernel build
# ------------------------------------------------------------

$(KERNEL_ELF): $(KERNEL_OBJS) linker.ld
	$(KERNEL_LD) -T linker.ld $(KERNEL_LDFLAGS) -o $@ $(KERNEL_OBJS)

# C -> .o
kernel/arch/x86_64/gdt_idt.o: KERNEL_CFLAGS := $(KERNEL_CFLAGS_GDT)
%.o: %.c
	$(KERNEL_CC) $(KERNEL_CFLAGS) -c $< -o $@

# asm -> .o (kernel/arch/x86_64/*.asm)
%.o: %.asm
	nasm -f elf64 $< -o $@

# S -> .o (kernel/arch/x86_64/*.S) - GNU assembler
%.o: %.S
	$(KERNEL_CC) $(KERNEL_CFLAGS) -c $< -o $@


# ------------------------------------------------------------
# 5) UEFI Bootloader build (BOOTX64.EFI)
# ------------------------------------------------------------

$(BOOT_EFI): $(EFI_OBJS)
	$(EFI_LD) $(EFI_LDFLAGS) /out:$@ $(EFI_OBJS)

$(BOOTLOADER_DIR)/%.efi.o: $(BOOTLOADER_DIR)/%.c
	$(EFI_CC) $(EFI_CFLAGS) -c $< -o $@


# ------------------------------------------------------------
# 6) EFI disk image + QEMU
# ------------------------------------------------------------

EFI_IMG = EFI.img
OVMF_CODE = firmware/ovmf/OVMF_CODE.fd
OVMF_VARS_CLEAN = OVMF_VARS.clean.fd
OVMF_VARS_RUN = ovmf_vars.fd

efi-img: $(KERNEL_ELF) $(BOOT_EFI)
	@if [ "$(OS)" = "Windows_NT" ]; then \
		powershell -ExecutionPolicy Bypass -File tools/build/make_efi_img.ps1; \
	else \
		./tools/build/make_efi_img.sh; \
	fi

run: efi-img
	@# CRITICAL: Use clean NVRAM to avoid BootOrder corruption
	cp -f $(OVMF_VARS_CLEAN) $(OVMF_VARS_RUN)
	qemu-system-x86_64 \
		-machine q35 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive if=pflash,format=raw,file=$(OVMF_VARS_RUN) \
		-drive format=raw,file=$(EFI_IMG) \
		-boot order=c \
		-debugcon file:debug_run.log \
		-global isa-debugcon.iobase=0xe9 \
		-nographic

run-preempt: efi-img
	@# Phase 4.5 deterministic preempt validation runner
	QEMU_TIMEOUT=12 ./run_preempt_test.sh

clean:
	rm -f $(KERNEL_OBJS) $(KERNEL_ELF) $(EFI_OBJS) $(BOOT_EFI) $(EFI_IMG)

clean-noimg:
	rm -f $(KERNEL_OBJS) $(KERNEL_ELF) $(EFI_OBJS) $(BOOT_EFI)

.PHONY: all clean run run-preempt efi-img kernel bootloader

# ------------------------------------------------------------
# 7) Validation and dependency checking targets
# ------------------------------------------------------------

# Check for required tools before building
check-deps:
	@echo "Checking build dependencies..."
	@missing_tools=""; \
	if ! command -v $(KERNEL_CC) >/dev/null 2>&1; then \
		missing_tools="$$missing_tools $(KERNEL_CC)"; \
	fi; \
	if ! command -v $(KERNEL_LD) >/dev/null 2>&1; then \
		missing_tools="$$missing_tools $(KERNEL_LD)"; \
	fi; \
	if ! command -v $(EFI_CC) >/dev/null 2>&1; then \
		missing_tools="$$missing_tools $(EFI_CC)"; \
	fi; \
	if ! command -v nasm >/dev/null 2>&1; then \
		missing_tools="$$missing_tools nasm"; \
	fi; \
	if [ -n "$$missing_tools" ]; then \
		echo "ERROR: Missing required tools:$$missing_tools"; \
		echo "Run 'make setup' or 'make install-deps' to install dependencies"; \
		exit 1; \
	else \
		echo "All required build tools found"; \
	fi

# Install dependencies automatically
install-deps:
	@echo "Installing build dependencies..."
	@if command -v powershell >/dev/null 2>&1; then \
		echo "Using PowerShell for Windows dependency installation..."; \
		powershell -ExecutionPolicy Bypass -File tools/setup/setup_and_validate.ps1; \
	elif command -v apt >/dev/null 2>&1; then \
		echo "Using apt package manager..."; \
		sudo apt update && sudo apt install -y gcc-multilib nasm clang make qemu-system-x86; \
	elif command -v yum >/dev/null 2>&1; then \
		echo "Using yum package manager..."; \
		sudo yum install -y gcc nasm clang make qemu-system-x86; \
	elif command -v pacman >/dev/null 2>&1; then \
		echo "Using pacman package manager..."; \
		sudo pacman -S --noconfirm gcc nasm clang make qemu; \
	else \
		echo "No supported package manager found."; \
		echo "Please install dependencies manually or use WSL2 with Ubuntu."; \
		echo "See docs/development/BUILD_FIXES_COMPLETE.md for detailed instructions."; \
		exit 1; \
	fi

# Comprehensive validation
validate: validate-toolchain validate-build validate-qemu

validate-toolchain:
	@echo "Running toolchain validation..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File tools/validation/validate_toolchain.ps1; \
	else \
		./tools/validation/validate_toolchain.sh --skip-qemu; \
	fi

validate-build: check-deps
	@echo "Validating build system..."
	@echo "Testing clean build process..."
	@$(MAKE) clean >/dev/null 2>&1 || true
	@if $(MAKE) all >/dev/null 2>&1; then \
		echo "Build validation: PASS"; \
		if [ -f "$(KERNEL_ELF)" ] && [ -f "$(BOOT_EFI)" ]; then \
			echo "Build artifacts validation: PASS"; \
		else \
			echo "Build artifacts validation: FAIL - Missing output files"; \
			exit 1; \
		fi; \
	else \
		echo "Build validation: FAIL"; \
		exit 1; \
	fi

validate-qemu: efi-img
	@echo "Running QEMU validation..."
	@if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then \
		echo "QEMU not found - skipping boot validation"; \
		echo "Install QEMU with: make install-deps"; \
	else \
		if command -v powershell >/dev/null 2>&1; then \
			powershell -ExecutionPolicy Bypass -File tools/qemu/qemu_test_runner.ps1; \
		else \
			./tools/qemu/qemu_test_runner.sh; \
		fi; \
	fi

# QEMU environment validation (Task 1.5.1.3)
validate-qemu-env:
	@echo "Running QEMU environment validation..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File tools/qemu/qemu_environment_validator.ps1; \
	else \
		bash tools/qemu/qemu_environment_validator.sh; \
	fi

# Comprehensive QEMU integration tests
validate-qemu-integration:
	@echo "Running QEMU integration tests..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File tools/qemu/qemu_integration_tests.ps1; \
	else \
		bash tools/qemu/qemu_integration_tests.sh; \
	fi

validate-full: clean validate-build validate-qemu
	@echo "Full validation completed successfully!"
	@echo "System is ready for development!"

# Development environment setup
setup:
	@echo "Setting up AykenOS development environment..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File tools/setup/setup_and_validate.ps1; \
	else \
		./tools/setup/setup_and_validate.sh; \
	fi

# Quick development workflow
dev: clean all validate-qemu
	@echo "Development build and test completed!"

# Continuous integration target
ci: check-deps validate-full
	@echo "CI validation completed successfully!"

# Stability and safety targets
freeze-stable:
	@echo "Creating stable checkpoint..."
	@git tag -a "phase2-stable-$(shell date +%Y%m%d)" -m "Phase 2 stable checkpoint"
	@git branch stable-phase2-backup 2>/dev/null || true
	@echo "Stable checkpoint created. Use 'make restore-stable' to rollback if needed."

restore-stable:
	@echo "Restoring to last stable checkpoint..."
	@git checkout stable-phase2-backup
	@echo "Restored to stable state."

validate-stability: validate-build validate-qemu
	@echo "Running stability validation..."
	@echo "Boot sequence: Testing..."
	@$(MAKE) run > stability_test.log 2>&1 &
	@sleep 5
	@pkill qemu-system-x86_64 2>/dev/null || true
	@if grep -q "Kernel loaded" stability_test.log; then \
		echo "Stability validation: PASS - Boot sequence stable"; \
	else \
		echo "Stability validation: FAIL - Boot issues detected"; \
		exit 1; \
	fi

# Help target
help:
	@echo "AykenOS Build System - Available targets:"
	@echo ""
	@echo "Build targets:"
	@echo "  all          - Build kernel and bootloader"
	@echo "  kernel       - Build kernel only"
	@echo "  bootloader   - Build UEFI bootloader only"
	@echo "  userspace-runtime - Build userspace dispatcher (bcib-runtime/bin/dispatcher)"
	@echo "  efi-img      - Create EFI disk image"
	@echo "  clean        - Clean build artifacts"
	@echo ""
	@echo "Development targets:"
	@echo "  dev          - Quick build and test cycle"
	@echo "  run          - Build and run in QEMU"
	@echo "  setup        - Set up development environment"
	@echo ""
	@echo "Validation targets:"
	@echo "  validate               - Run all validations"
	@echo "  validate-toolchain     - Check required tools"
	@echo "  validate-build         - Test build system"
	@echo "  validate-qemu          - Test QEMU boot"
	@echo "  validate-qemu-env      - QEMU environment validation (Task 1.5.1.3)"
	@echo "  validate-qemu-integration - Comprehensive QEMU integration tests"
	@echo "  validate-full          - Complete validation suite"
	@echo ""
	@echo "Dependency targets:"
	@echo "  check-deps   - Check for required tools"
	@echo "  install-deps - Install missing dependencies"
	@echo ""
	@echo "CI/CD targets:"
	@echo "  ci           - Continuous integration validation"
	@echo "  help         - Show this help message"

.PHONY: check-deps install-deps validate validate-toolchain validate-build validate-qemu validate-qemu-env validate-qemu-integration validate-full setup dev ci help

# UEFI bootloader assembly sources (.S)
$(BOOTLOADER_DIR)/%.efi.o: $(BOOTLOADER_DIR)/%.S
	$(EFI_CC) $(EFI_CFLAGS) -c $< -o $@
