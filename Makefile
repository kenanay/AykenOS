# ============================================================
# AykenOS Build System
#  - x86_64 Kernel (ELF, higher-half)
#  - UEFI Bootloader (BOOTX64.EFI)
#  - EFI.img + QEMU run
# ============================================================

# ------------------------------------------------------------
# 1) Kernel toolchain
# ------------------------------------------------------------

KERNEL_CC_BIN = clang
KERNEL_CC = $(KERNEL_CC_BIN) --target=x86_64-elf
KERNEL_LD_BIN = ld.lld
KERNEL_LD = $(KERNEL_LD_BIN)

KERNEL_CFLAGS = -ffreestanding -m64 -Wall -Wextra -Ikernel/include
KERNEL_CFLAGS += -mcmodel=large -fno-pic -fno-omit-frame-pointer -fno-stack-protector
KERNEL_CFLAGS += -mno-red-zone
KERNEL_CFLAGS += -MMD -MP
KERNEL_ASMFLAGS = -Ikernel/include/generated/ -Ikernel/include/

# ------------------------------------------------------------
# Kernel profile and debug feature flags
# ------------------------------------------------------------
# Kernel profile (release | validation)
KERNEL_PROFILE ?= release
PROFILE_RELEASE_FLAGS :=
PROFILE_VALIDATION_FLAGS :=

ifneq ($(filter $(KERNEL_PROFILE),release validation),$(KERNEL_PROFILE))
$(error Invalid KERNEL_PROFILE='$(KERNEL_PROFILE)'. Use release or validation)
endif

VALIDATION_WERROR ?= 0
AYKEN_SCHED_FALLBACK ?= 0
AYKEN_INTENTIONAL_PERF_REGRESSION_MS ?= 0
AYKEN_MB_SELFTEST ?= 1
AYKEN_GATE4_POLICY_TEST ?= 0
AYKEN_GATE45_PROOF ?= 0
KERNEL_EXPORT_POLICY ?= 1
AYKEN_CR3_PCID ?= 0
# Phase10-C1 default: strict mailbox-owner bootstrap (no transitional policy bridge).
AYKEN_SCHED_BOOTSTRAP_POLICY ?= 0

ifneq ($(filter $(AYKEN_SCHED_FALLBACK),0 1),$(AYKEN_SCHED_FALLBACK))
$(error Invalid AYKEN_SCHED_FALLBACK='$(AYKEN_SCHED_FALLBACK)'. Use 0 or 1)
endif

ifneq ($(filter $(AYKEN_MB_SELFTEST),0 1),$(AYKEN_MB_SELFTEST))
$(error Invalid AYKEN_MB_SELFTEST='$(AYKEN_MB_SELFTEST)'. Use 0 or 1)
endif

ifneq ($(filter $(AYKEN_GATE4_POLICY_TEST),0 1),$(AYKEN_GATE4_POLICY_TEST))
$(error Invalid AYKEN_GATE4_POLICY_TEST='$(AYKEN_GATE4_POLICY_TEST)'. Use 0 or 1)
endif

ifneq ($(filter $(AYKEN_GATE45_PROOF),0 1),$(AYKEN_GATE45_PROOF))
$(error Invalid AYKEN_GATE45_PROOF='$(AYKEN_GATE45_PROOF)'. Use 0 or 1)
endif

ifneq ($(filter $(KERNEL_EXPORT_POLICY),0 1),$(KERNEL_EXPORT_POLICY))
$(error Invalid KERNEL_EXPORT_POLICY='$(KERNEL_EXPORT_POLICY)'. Use 0 or 1)
endif

ifneq ($(filter $(AYKEN_CR3_PCID),0 1),$(AYKEN_CR3_PCID))
$(error Invalid AYKEN_CR3_PCID='$(AYKEN_CR3_PCID)'. Use 0 or 1)
endif

ifneq ($(filter $(AYKEN_SCHED_BOOTSTRAP_POLICY),0 1),$(AYKEN_SCHED_BOOTSTRAP_POLICY))
$(error Invalid AYKEN_SCHED_BOOTSTRAP_POLICY='$(AYKEN_SCHED_BOOTSTRAP_POLICY)'. Use 0 or 1)
endif

ifeq ($(AYKEN_SCHED_BOOTSTRAP_POLICY),0)
ifeq ($(AYKEN_SCHED_FALLBACK),1)
$(error AYKEN_SCHED_FALLBACK=1 is forbidden when AYKEN_SCHED_BOOTSTRAP_POLICY=0)
endif
endif

ifeq ($(AYKEN_SCHED_FALLBACK),1)
ifneq ($(KERNEL_PROFILE),validation)
$(error AYKEN_SCHED_FALLBACK=1 is only allowed with KERNEL_PROFILE=validation)
endif
endif

# Debug flags: set defaults based on profile, allow override via env
ifeq ($(KERNEL_PROFILE),validation)
PROFILE_VALIDATION_FLAGS := 1
AYKEN_DEBUG_IRQ ?= 1
AYKEN_DEBUG_SCHED ?= 1
KERNEL_CFLAGS += -O0 -g3 -DAYKEN_VALIDATION=1
ifeq ($(VALIDATION_WERROR),1)
KERNEL_CFLAGS += -Werror
endif
else
PROFILE_RELEASE_FLAGS := 1
AYKEN_DEBUG_IRQ ?= 0
AYKEN_DEBUG_SCHED ?= 0
KERNEL_CFLAGS += -O2 -g1
endif

ifeq ($(AYKEN_DEBUG_IRQ),1)
KERNEL_CFLAGS += -DAYKEN_DEBUG_IRQ=1
endif

ifeq ($(AYKEN_DEBUG_SCHED),1)
KERNEL_CFLAGS += -DAYKEN_DEBUG_SCHED=1
KERNEL_ASMFLAGS += -DAYKEN_DEBUG_SCHED=1
endif
KERNEL_CFLAGS += -DAYKEN_SCHED_FALLBACK=$(AYKEN_SCHED_FALLBACK)
KERNEL_CFLAGS += -DAYKEN_INTENTIONAL_PERF_REGRESSION_MS=$(AYKEN_INTENTIONAL_PERF_REGRESSION_MS)
KERNEL_CFLAGS += -DAYKEN_MB_SELFTEST=$(AYKEN_MB_SELFTEST)
KERNEL_CFLAGS += -DAYKEN_GATE4_POLICY_TEST=$(AYKEN_GATE4_POLICY_TEST)
KERNEL_CFLAGS += -DAYKEN_GATE45_PROOF=$(AYKEN_GATE45_PROOF)
KERNEL_CFLAGS += -DAYKEN_CR3_PCID=$(AYKEN_CR3_PCID)
KERNEL_CFLAGS += -DAYKEN_SCHED_BOOTSTRAP_POLICY=$(AYKEN_SCHED_BOOTSTRAP_POLICY)
KERNEL_ASMFLAGS += -DAYKEN_CR3_PCID=$(AYKEN_CR3_PCID)
# For gdt_idt.c force kernel code model to avoid 32-bit relocations in higher half
KERNEL_CFLAGS_GDT := $(filter-out -mcmodel=large,$(KERNEL_CFLAGS)) -mcmodel=kernel

KERNEL_LDFLAGS = -nostdlib -z max-page-size=0x1000
KERNEL_MAP ?=

KERNEL_ELF = kernel.elf
CTX_SWITCH_ASM = kernel/arch/x86_64/context_switch.asm
PROFILE_STAMP = .build_profile.stamp
ABI_H = kernel/include/ayken_abi.h
ABI_INC = kernel/include/generated/ayken_abi.inc
RING0_SYMBOL_WHITELIST = scripts/ci/constitutional-ring0-symbol-whitelist.regex
RING0_EXPORT_MAP = kernel/include/generated/ring0.exports.map
KERNEL_LINK_EXTRA_FLAGS =
KERNEL_LINK_EXTRA_DEPS =

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

KERNEL_C_SOURCES_ALL = $(call find_files,$(KERNEL_DIR),*.c)
KERNEL_C_TEST_SOURCES = $(filter %_test.c,$(KERNEL_C_SOURCES_ALL))
# Ring0 topology enforces mechanism/policy separation; no default source exclusion hacks.
KERNEL_C_EXCLUDE_SOURCES =
KERNEL_C_SOURCES = $(filter-out $(KERNEL_C_TEST_SOURCES) $(KERNEL_C_EXCLUDE_SOURCES),$(KERNEL_C_SOURCES_ALL))
KERNEL_ASM_SOURCES = $(call find_files,$(ARCH_DIR),*.asm)
KERNEL_S_SOURCES   = $(call find_files,$(ARCH_DIR),*.S)

# Phase 10: User binary embedding
USER_MINIMAL_DIR = userspace/minimal
USER_MINIMAL_ELF = $(USER_MINIMAL_DIR)/minimal.elf
USER_MINIMAL_BIN = $(USER_MINIMAL_DIR)/user.bin
USER_MINIMAL_SOURCES = $(wildcard $(USER_MINIMAL_DIR)/*.c) \
                       $(wildcard $(USER_MINIMAL_DIR)/*.S) \
                       $(USER_MINIMAL_DIR)/user.ld \
                       $(USER_MINIMAL_DIR)/Makefile
EMBED_ELF_TOOL = tools/embed_elf.py
EMBEDDED_ELF_HEADER = kernel/include/embedded_elf.h

# Kernel image contains Ring0 code only.
# Ring3 userspace components are built via separate userspace targets.
KERNEL_OBJS = $(KERNEL_C_SOURCES:.c=.o) $(KERNEL_ASM_SOURCES:.asm=.o) $(KERNEL_S_SOURCES:.S=.o)
KERNEL_DEPS = $(KERNEL_OBJS:.o=.d)

ifeq ($(KERNEL_EXPORT_POLICY),1)
KERNEL_LINK_EXTRA_FLAGS += --version-script=$(RING0_EXPORT_MAP)
KERNEL_LINK_EXTRA_DEPS += $(RING0_EXPORT_MAP)
endif

# Rust workspace (userspace runtime/dispatcher)
USERSPACE_RUST_DIR = userspace
USERSPACE_RUNTIME_BIN = $(USERSPACE_RUST_DIR)/target/debug/dispatcher.exe

# CI evidence and boundary gate defaults
EVIDENCE_ROOT ?= evidence
RUN_ID_DEFAULT := $(shell date -u +"%Y%m%dT%H%M%SZ")-$(shell git rev-parse --short HEAD 2>/dev/null || echo nogit)
RUN_ID ?= $(RUN_ID_DEFAULT)
# Command-line RUN_ID= (empty) must not collapse evidence path to evidence/run-.
ifeq ($(strip $(RUN_ID)),)
override RUN_ID := $(RUN_ID_DEFAULT)
endif
RUN_ID := $(RUN_ID)
EVIDENCE_RUN_DIR := $(EVIDENCE_ROOT)/run-$(RUN_ID)
CI_TARGETS ?= kernel.elf
ABI_INIT_BASELINE ?= 0
ABI_DIFF_RANGE ?=
CONSTITUTIONAL_STRICT ?= 1
CONSTITUTIONAL_STRICT_FLAG = $(if $(filter 1,$(CONSTITUTIONAL_STRICT)),--strict,--no-strict)
GOVERNANCE_POLICY_KERNEL_PROFILE ?= validation
RUNTIME_MARKER_CONTRACT_ENFORCE ?= 1
BEHAVIORAL_SUITE_PHASE ?= 5
WORKSPACE_STRICT ?= 1
WORKSPACE_STRICT_FLAG = $(if $(filter 1,$(WORKSPACE_STRICT)),--strict,--no-strict)
PERF_INIT_BASELINE ?= 0
PERF_ENV_MISMATCH_POLICY ?= fail
PERF_QEMU_TIMEOUT ?= 30
PERF_KERNEL_PROFILE ?= validation
PERF_BASELINE_FILE ?= scripts/ci/perf-baseline.lock.json
PERF_AUTHORITY_ENV_FILE ?= scripts/ci/perf_authority.env
PERF_BASELINE_AUTHORITY_DEFAULT := $(shell sed -n 's/^PERF_BASELINE_AUTHORITY=//p' $(PERF_AUTHORITY_ENV_FILE) 2>/dev/null | head -n1)
PERF_BASELINE_AUTHORITY ?= $(if $(PERF_BASELINE_AUTHORITY_DEFAULT),$(PERF_BASELINE_AUTHORITY_DEFAULT),github-hosted-ubuntu-24.04-x64)
PERF_REQUIRE_CI_FOR_BASELINE_INIT ?= 1
PERF_CI_IMAGE_DIGEST ?= unknown
SYSCALL_V2_RUNTIME_KERNEL_PROFILE ?= validation
SYSCALL_V2_RUNTIME_WARMUP ?= 1
ifeq ($(PERF_BASELINE_MODE),provisional)
SYSCALL_V2_RUNTIME_RUNS ?= 3
SYSCALL_V2_RUNTIME_TIMEOUT ?= 40
SYSCALL_V2_RUNTIME_REQUIRED_SUCCESS_RATE ?= 60
else
SYSCALL_V2_RUNTIME_RUNS ?= 5
SYSCALL_V2_RUNTIME_TIMEOUT ?= 20
SYSCALL_V2_RUNTIME_REQUIRED_SUCCESS_RATE ?= 100
endif
RING0_EXPORT_MAX ?= 165
PERF_VARIANCE_RUNS ?= 5
PERF_VARIANCE_WARMUP ?= 1
PERF_VARIANCE_QEMU_TIMEOUT ?= 12
PERF_VARIANCE_STRICT_MARKERS ?= 1
PERF_VARIANCE_FORCE_EFI_REBUILD ?= 0
RING3_QEMU_TIMEOUT ?= 35
PHASE10B_MODE ?= negative
PHASE10B_A2_EVIDENCE_DIR ?= $(EVIDENCE_RUN_DIR)/gates/ring3-execution-phase10a2
PHASE10C_REQUIRE_METADATA ?= 1
PHASE10C_A2_EVIDENCE_DIR ?= $(EVIDENCE_RUN_DIR)/gates/ring3-execution-phase10a2
PHASE10C_ENFORCE ?= 0
PHASE10C_FREEZE_GATE = $(if $(filter 1,$(PHASE10C_ENFORCE)),ci-gate-scheduler-mailbox-phase10c,)
GATE45_QEMU_TIMEOUT ?= 20
GATE45_BOOTSTRAP_POLICY ?= 1
GATE45_MB_SELFTEST ?= 0


# ------------------------------------------------------------
# 2) UEFI Bootloader toolchain
# ------------------------------------------------------------
# Burayı ortamına göre ayarlayacaksın:
# - Eğer clang kullanıyorsan: EFI_CC = clang
# - Eğer mingw-w64 kullanıyorsan: EFI_CC = x86_64-w64-mingw32-gcc
#
# Windows + WSL senaryosunda genelde clang daha rahat.

EFI_CC_BIN = clang
EFI_CC = $(EFI_CC_BIN)
EFI_LD_BIN = lld-link
EFI_LD = $(EFI_LD_BIN)

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

all: check-deps guard-context-offsets $(USER_MINIMAL_BIN) $(EMBEDDED_ELF_HEADER) $(KERNEL_ELF) $(BOOT_EFI)

kernel: check-deps guard-context-offsets $(USER_MINIMAL_BIN) $(EMBEDDED_ELF_HEADER) $(KERNEL_ELF)
bootloader: check-deps $(BOOT_EFI)
user-minimal: $(USER_MINIMAL_BIN)
userspace-runtime:
	@cd $(USERSPACE_RUST_DIR) && cargo build -p bcib-runtime --bin dispatcher
release:
	@$(MAKE) KERNEL_PROFILE=release all
validation:
	@$(MAKE) KERNEL_PROFILE=validation all
validation-strict:
	@$(MAKE) KERNEL_PROFILE=validation VALIDATION_WERROR=1 all


# ------------------------------------------------------------
# 4) Kernel build
# ------------------------------------------------------------

$(PROFILE_STAMP): FORCE
	@if [ ! -f $(PROFILE_STAMP) ] || [ "$$(cat $(PROFILE_STAMP) 2>/dev/null)" != "$(KERNEL_PROFILE)" ]; then \
		echo "$(KERNEL_PROFILE)" > $(PROFILE_STAMP); \
	fi

$(KERNEL_OBJS): $(PROFILE_STAMP) $(EMBEDDED_ELF_HEADER)

$(KERNEL_ELF): $(KERNEL_OBJS) linker.ld $(PROFILE_STAMP) $(KERNEL_LINK_EXTRA_DEPS)
	$(KERNEL_LD) -T linker.ld $(KERNEL_LDFLAGS) $(KERNEL_LINK_EXTRA_FLAGS) $(if $(strip $(KERNEL_MAP)),-Map=$(KERNEL_MAP),) -o $@ $(KERNEL_OBJS)

# Generate NASM ABI include from single C ABI source.
.PHONY: generate-abi
generate-abi: $(ABI_INC)
	@echo "ABI include generated: $(ABI_INC)"

$(ABI_INC): $(ABI_H)
	@mkdir -p $(dir $@)
	@echo "; AUTO-GENERATED. DO NOT EDIT." > $@
	@echo "; Source: $(ABI_H)" >> $@
	@awk '\
		$$1 == "#define" && $$2 == "AYKEN_ABI_VERSION" { \
			val = $$3; gsub(/[uU]/, "", val); \
			printf("%%define AYKEN_ABI_VERSION %s\n", val); \
			next; \
		} \
		$$1 == "#define" && $$2 ~ /^(CTX_|IRQF_)/ { \
			val = $$3; gsub(/[uU]/, "", val); \
			printf("%%define %s %s\n", $$2, val); \
		}' $(ABI_H) >> $@

$(RING0_EXPORT_MAP): $(KERNEL_OBJS) $(RING0_SYMBOL_WHITELIST) scripts/ci/generate_ring0_export_map.py
	@mkdir -p $(dir $@)
	@python3 scripts/ci/generate_ring0_export_map.py \
		--whitelist "$(RING0_SYMBOL_WHITELIST)" \
		--output "$@" \
		--nm "nm" \
		--objects $(KERNEL_OBJS)

$(KERNEL_ASM_SOURCES:.asm=.o): $(ABI_INC)

# C -> .o
kernel/arch/x86_64/gdt_idt.o: KERNEL_CFLAGS := $(KERNEL_CFLAGS_GDT)
%.o: %.c
	$(KERNEL_CC) $(KERNEL_CFLAGS) -c $< -o $@

# asm -> .o (kernel/arch/x86_64/*.asm)
%.o: %.asm
	nasm -f elf64 $(KERNEL_ASMFLAGS) $< -o $@

# S -> .o (kernel/arch/x86_64/*.S) - GNU assembler
%.o: %.S
	$(KERNEL_CC) $(KERNEL_CFLAGS) -c $< -o $@

# Phase 10: User binary build and embedding
$(USER_MINIMAL_ELF): $(USER_MINIMAL_SOURCES)
	@echo "[PHASE10] Building minimal user ELF..."
	@$(MAKE) -C $(USER_MINIMAL_DIR) minimal.elf

$(USER_MINIMAL_BIN): $(USER_MINIMAL_ELF)
	@echo "[PHASE10] Building minimal user binary..."
	@$(MAKE) -C $(USER_MINIMAL_DIR) user.bin

$(EMBEDDED_ELF_HEADER): $(USER_MINIMAL_ELF) $(EMBED_ELF_TOOL)
	@echo "[PHASE10] Generating embedded ELF header..."
	@python3 $(EMBED_ELF_TOOL) --input $(USER_MINIMAL_ELF) --output $(EMBEDDED_ELF_HEADER)

-include $(KERNEL_DEPS)


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
		bash ./tools/build/make_efi_img.sh; \
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

run-preempt:
	@# Phase 4.5 deterministic preempt validation runner
	@$(MAKE) KERNEL_PROFILE=validation guard-context-offsets efi-img
	QEMU_TIMEOUT=12 ./run_preempt_test.sh

run-preempt-strict:
	@# Strict marker-mode preempt validation (no AB fallback)
	@$(MAKE) KERNEL_PROFILE=validation guard-context-offsets kernel bootloader
	QEMU_TIMEOUT=12 STRICT_MARKERS=1 FORCE_EFI_REBUILD=1 ./run_preempt_test.sh

clean:
	rm -f $(KERNEL_OBJS) $(KERNEL_DEPS) $(KERNEL_ELF) $(EFI_OBJS) $(BOOT_EFI) $(EFI_IMG) .build_profile.stamp $(ABI_INC) $(RING0_EXPORT_MAP) $(EMBEDDED_ELF_HEADER)

clean-noimg:
	rm -f $(KERNEL_OBJS) $(KERNEL_DEPS) $(KERNEL_ELF) $(EFI_OBJS) $(BOOT_EFI) .build_profile.stamp $(ABI_INC) $(RING0_EXPORT_MAP) $(EMBEDDED_ELF_HEADER)

.PHONY: all clean run run-preempt run-preempt-strict efi-img kernel bootloader guard-context-offsets release validation validation-strict FORCE
FORCE:

# Enforce context ABI discipline: raw numeric offsets are forbidden in context_switch.asm.
guard-context-offsets:
	@echo "Checking context offset discipline..."
	@if grep -nE '^[[:space:]]*[^;].*\[(rdi|rsi)[^]]*\+[[:space:]]*[0-9]+' $(CTX_SWITCH_ASM) >/dev/null; then \
		echo "ERROR: Raw numeric context offsets found in $(CTX_SWITCH_ASM). Use CTX_* constants only."; \
		grep -nE '^[[:space:]]*[^;].*\[(rdi|rsi)[^]]*\+[[:space:]]*[0-9]+' $(CTX_SWITCH_ASM); \
		exit 1; \
	fi
	@if grep -nE '^[[:space:]]*[^;].*\[(rdi|rsi)[^]]*\+[[:space:]]*CTX_[^]]*[[:space:]]*[\+\-\*]' $(CTX_SWITCH_ASM) >/dev/null; then \
		echo "ERROR: Context offsets must be exact [rdi|rsi + CTX_*] (no arithmetic on CTX_*)."; \
		grep -nE '^[[:space:]]*[^;].*\[(rdi|rsi)[^]]*\+[[:space:]]*CTX_[^]]*[[:space:]]*[\+\-\*]' $(CTX_SWITCH_ASM); \
		exit 1; \
	fi
	@if awk '\
		/^[[:space:]]*;/ { next } \
		{ \
			line = $$0; \
			while (match(line, /\[(rdi|rsi)[^]]*\+[[:space:]]*[A-Za-z_][A-Za-z0-9_]*/)) { \
				expr = substr(line, RSTART, RLENGTH); \
				token = expr; \
				sub(/^.*\+[[:space:]]*/, "", token); \
				if (token !~ /^CTX_[A-Za-z0-9_]*$$/) { \
					print NR ":" $$0; \
					bad = 1; \
					break; \
				} \
				line = substr(line, RSTART + RLENGTH); \
			} \
		} \
		END { exit bad }' $(CTX_SWITCH_ASM); then :; else \
		echo "ERROR: Non-CTX_* context offset alias used in $(CTX_SWITCH_ASM)."; \
		exit 1; \
	fi
	@echo "Context offset guard: PASS"

# ------------------------------------------------------------
# 7) Validation and dependency checking targets
# ------------------------------------------------------------

# Check for required tools before building
check-deps:
	@echo "Checking build dependencies..."
	@missing_tools=""; \
	if ! command -v $(KERNEL_CC_BIN) >/dev/null 2>&1; then \
		missing_tools="$$missing_tools $(KERNEL_CC_BIN)"; \
	fi; \
	if ! command -v $(KERNEL_LD_BIN) >/dev/null 2>&1; then \
		missing_tools="$$missing_tools $(KERNEL_LD_BIN)"; \
	fi; \
	if ! command -v $(EFI_CC_BIN) >/dev/null 2>&1; then \
		missing_tools="$$missing_tools $(EFI_CC_BIN)"; \
	fi; \
	if ! command -v $(EFI_LD_BIN) >/dev/null 2>&1; then \
		missing_tools="$$missing_tools $(EFI_LD_BIN)"; \
	fi; \
	if ! command -v nasm >/dev/null 2>&1; then \
		missing_tools="$$missing_tools nasm"; \
	fi; \
	if ! command -v nm >/dev/null 2>&1; then \
		missing_tools="$$missing_tools nm"; \
	fi; \
	if ! command -v python3 >/dev/null 2>&1; then \
		missing_tools="$$missing_tools python3"; \
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
		sudo apt update && sudo apt install -y gcc-multilib nasm clang lld make qemu-system-x86; \
	elif command -v yum >/dev/null 2>&1; then \
		echo "Using yum package manager..."; \
		sudo yum install -y gcc nasm clang lld make qemu-system-x86; \
	elif command -v pacman >/dev/null 2>&1; then \
		echo "Using pacman package manager..."; \
		sudo pacman -S --noconfirm gcc nasm clang lld make qemu; \
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

# Continuous integration target (currently enforced gates)
ci: check-deps ci-gate-boundary ci-gate-hygiene validate-full
	@echo "CI validation completed successfully!"

# ================================================
# Pre-CI Discipline (Local Advisory Layer)
# ================================================
# Local fail-closed discipline check before CI.
# Does NOT replace CI. CI remains mandatory.
#
# Gates: 4 core discipline gates (~30-60s)
#   - ABI Stability
#   - Boundary Enforcement
#   - Hygiene
#   - Constitutional Compliance
#
# Runtime gates (Ring0 Exports, Workspace, Syscall v2, Sched Bridge,
# Policy Accept) run in CI only.

.PHONY: pre-ci
pre-ci:
	@bash scripts/ci/pre_ci_discipline.sh

# ================================================
# Freeze Suite (Constitutional Enforcement)
# ================================================
# Freeze suite target (strict): calls all declared freeze gates.
ci-freeze-guard:
	@if [ "$(AYKEN_SCHED_FALLBACK)" != "0" ]; then \
		echo "ERROR: ci-freeze requires AYKEN_SCHED_FALLBACK=0 (current=$(AYKEN_SCHED_FALLBACK))"; \
		exit 2; \
	fi
	@if [ "$(AYKEN_CR3_PCID)" != "0" ]; then \
		echo "ERROR: ci-freeze requires AYKEN_CR3_PCID=0 (current=$(AYKEN_CR3_PCID))"; \
		exit 2; \
	fi
	@if [ "$(PHASE10C_ENFORCE)" = "1" ] && [ "$(AYKEN_SCHED_BOOTSTRAP_POLICY)" != "0" ]; then \
		echo "ERROR: ci-freeze with PHASE10C_ENFORCE=1 requires AYKEN_SCHED_BOOTSTRAP_POLICY=0 (current=$(AYKEN_SCHED_BOOTSTRAP_POLICY))"; \
		exit 2; \
	fi

ci-freeze: ci-freeze-guard ci-gate-abi ci-gate-boundary ci-gate-ring0-exports ci-gate-hygiene ci-gate-tooling-isolation ci-gate-constitutional ci-gate-governance-policy ci-gate-drift-activation ci-gate-structural-abi ci-gate-runtime-marker-contract ci-gate-ring3-execution-phase10a2 ci-gate-syscall-semantics-phase10b $(PHASE10C_FREEZE_GATE) ci-gate-workspace ci-gate-syscall-v2-runtime ci-gate-sched-bridge-runtime ci-gate-behavioral-suite ci-gate-policy-accept ci-gate-performance
	@echo "Freeze CI suite completed successfully!"

# Local freeze (skip performance and tooling-isolation gates for development)
ci-freeze-local: ci-freeze-guard ci-gate-abi ci-gate-boundary ci-gate-ring0-exports ci-gate-hygiene ci-gate-constitutional ci-gate-governance-policy ci-gate-drift-activation ci-gate-structural-abi ci-gate-runtime-marker-contract ci-gate-ring3-execution-phase10a2 ci-gate-syscall-semantics-phase10b ci-gate-scheduler-mailbox-phase10c ci-gate-workspace ci-gate-syscall-v2-runtime ci-gate-sched-bridge-runtime ci-gate-behavioral-suite ci-gate-policy-accept
	@echo "Local freeze suite completed successfully (performance & tooling-isolation gates skipped)!"

# CI boundary gate with evidence collection
ci-evidence-dir:
	@mkdir -p "$(EVIDENCE_RUN_DIR)/meta"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/artifacts"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/reports"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/abi"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/symbol-scan"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/ring0-exports"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/hygiene"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/tooling-isolation"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/constitutional"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/governance-policy"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/drift-activation"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/structural-abi"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/runtime-marker-contract"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/behavioral-suite"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/ring3-execution-phase10a2"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/syscall-semantics-phase10b"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/scheduler-mailbox-phase10c"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/workspace"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/syscall-v2-runtime"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/policy-accept"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/decision-switch-phase45"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/performance"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/logs"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/reports"
	@printf '{\n  "run_id": "%s",\n  "time_utc": "%s"\n}\n' \
		"$(RUN_ID)" "$$(date -u +"%Y-%m-%dT%H:%M:%SZ")" > "$(EVIDENCE_RUN_DIR)/meta/run.json"
	@git rev-parse HEAD > "$(EVIDENCE_RUN_DIR)/meta/git.txt" 2>/dev/null || true
	@{ \
		echo "clang: $$(clang --version 2>/dev/null | head -n1 || echo NA)"; \
		echo "ld.lld: $$(ld.lld --version 2>/dev/null | head -n1 || echo NA)"; \
		echo "nasm: $$(nasm -v 2>/dev/null || echo NA)"; \
	} > "$(EVIDENCE_RUN_DIR)/meta/toolchain.txt"

ci-gate-boundary: ci-evidence-dir
	@echo "== CI GATE BOUNDARY =="
	@echo "run_id: $(RUN_ID)"
	@echo "targets: $(CI_TARGETS)"
	@rm -f "$(KERNEL_ELF)" "$(EVIDENCE_RUN_DIR)/artifacts/kernel.map"
	@if echo "$(MAKEFLAGS)" | grep -Eq '(^|[[:space:]])n($$|[[:space:]])|--just-print|--dry-run|--recon'; then \
		echo "DRY-RUN: skipping boundary kernel build invocation"; \
	else \
		mkdir -p "$(EVIDENCE_RUN_DIR)/logs"; \
		$(MAKE) KERNEL_PROFILE=validation KERNEL_MAP="$(EVIDENCE_RUN_DIR)/artifacts/kernel.map" guard-context-offsets kernel > "$(EVIDENCE_RUN_DIR)/logs/build.log" 2>&1; \
	fi
	@printf '{\n  "run_id": "%s",\n  "time_utc": "%s"\n}\n' \
		"$(RUN_ID)" "$$(date -u +"%Y-%m-%dT%H:%M:%SZ")" > "$(EVIDENCE_RUN_DIR)/meta/run.json"
	@git rev-parse HEAD > "$(EVIDENCE_RUN_DIR)/meta/git.txt" 2>/dev/null || true
	@{ \
		echo "clang: $$(clang --version 2>/dev/null | head -n1 || echo NA)"; \
		echo "ld.lld: $$(ld.lld --version 2>/dev/null | head -n1 || echo NA)"; \
		echo "nasm: $$(nasm -v 2>/dev/null || echo NA)"; \
	} > "$(EVIDENCE_RUN_DIR)/meta/toolchain.txt"
	@for target in $(CI_TARGETS); do \
		if [ -f "$$target" ]; then \
			base="$$(basename "$$target")"; \
			cp -f "$$target" "$(EVIDENCE_RUN_DIR)/artifacts/$$base"; \
			if command -v sha256sum >/dev/null 2>&1; then \
				sha256sum "$(EVIDENCE_RUN_DIR)/artifacts/$$base" | awk '{print $$1}' > "$(EVIDENCE_RUN_DIR)/artifacts/$$base.sha256"; \
			elif command -v shasum >/dev/null 2>&1; then \
				shasum -a 256 "$(EVIDENCE_RUN_DIR)/artifacts/$$base" | awk '{print $$1}' > "$(EVIDENCE_RUN_DIR)/artifacts/$$base.sha256"; \
			fi; \
		fi; \
	done
	@./tools/ci/symbol-scan.sh \
		--targets "$(CI_TARGETS)" \
		--deny tools/ci/deny.symbols \
		--allow tools/ci/allow.symbols \
		--evidence-dir "$(EVIDENCE_RUN_DIR)/gates/symbol-scan"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/symbol-scan/report.json" "$(EVIDENCE_RUN_DIR)/reports/symbol-scan.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-ring0-exports: ci-evidence-dir
	@echo "== CI GATE RING0 EXPORTS =="
	@echo "run_id: $(RUN_ID)"
	@echo "ring0_export_max: $(RING0_EXPORT_MAX)"
	@./scripts/ci/check_ring0_exports.sh \
		--evidence-dir "$(EVIDENCE_RUN_DIR)/gates/ring0-exports" \
		--kernel-profile "validation" \
		--max-exports "$(RING0_EXPORT_MAX)"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/ring0-exports/report.json" "$(EVIDENCE_RUN_DIR)/reports/ring0-exports.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: ring0-exports evidence at $(EVIDENCE_RUN_DIR)"

# Standalone summary verdict gate for existing run directory.
ci-summarize:
	@./tools/ci/summarize.sh --run-dir "$(EVIDENCE_RUN_DIR)"
	@python3 -c 'import json,sys; p=sys.argv[1]; v=json.load(open(p, encoding="utf-8")).get("verdict"); acceptable=("PASS","SKIP","WARN"); print(f"ERROR: summary verdict is {v} ({p})") if v not in acceptable else None; sys.exit(0 if v in acceptable else 2)' "$(EVIDENCE_RUN_DIR)/reports/summary.json"

# ABI gate (implemented): deterministic generation + baseline lock compare.
ci-gate-abi: ci-evidence-dir
	@echo "== CI GATE ABI =="
	@echo "run_id: $(RUN_ID)"
	@echo "abi_diff_range: $(if $(strip $(ABI_DIFF_RANGE)),$(ABI_DIFF_RANGE),auto)"
	@if [ "$(ABI_INIT_BASELINE)" = "1" ]; then \
		ABI_DIFF_RANGE="$(ABI_DIFF_RANGE)" ./scripts/ci/gate_abi.sh --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/abi" --init-baseline; \
	else \
		ABI_DIFF_RANGE="$(ABI_DIFF_RANGE)" ./scripts/ci/gate_abi.sh --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/abi"; \
	fi
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/abi/report.json" "$(EVIDENCE_RUN_DIR)/reports/abi.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: ABI evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-workspace: ci-evidence-dir
	@echo "== CI GATE WORKSPACE =="
	@echo "run_id: $(RUN_ID)"
	@./scripts/ci/gate_workspace.sh $(WORKSPACE_STRICT_FLAG) --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/workspace"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/workspace/report.json" "$(EVIDENCE_RUN_DIR)/reports/workspace.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: workspace evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-hygiene: ci-evidence-dir
	@echo "== CI GATE HYGIENE =="
	@echo "run_id: $(RUN_ID)"
	@./scripts/ci/gate_hygiene_simple.sh "$(EVIDENCE_RUN_DIR)/gates/hygiene"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/hygiene/report.json" "$(EVIDENCE_RUN_DIR)/reports/hygiene.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: hygiene evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-tooling-isolation: ci-evidence-dir
	@echo "== CI GATE TOOLING ISOLATION =="
	@echo "run_id: $(RUN_ID)"
ifeq ($(PERF_BASELINE_MODE),provisional)
	@echo "tooling-isolation: SKIPPED (provisional mode - mixed tooling/kernel changes allowed)"
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/tooling-isolation"
	@echo '{"gate":"tooling-isolation","verdict":"SKIP","reason":"provisional mode","violations_count":0}' > "$(EVIDENCE_RUN_DIR)/gates/tooling-isolation/report.json"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/tooling-isolation/report.json" "$(EVIDENCE_RUN_DIR)/reports/tooling-isolation.json"
else
	@./scripts/ci/gate_tooling_isolation.sh --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/tooling-isolation"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/tooling-isolation/report.json" "$(EVIDENCE_RUN_DIR)/reports/tooling-isolation.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: tooling-isolation evidence at $(EVIDENCE_RUN_DIR)"
endif

ci-gate-constitutional: ci-evidence-dir
	@echo "== CI GATE CONSTITUTIONAL =="
	@echo "run_id: $(RUN_ID)"
	@echo "ayken_sched_fallback: $(AYKEN_SCHED_FALLBACK)"
	@AYKEN_SCHED_FALLBACK="$(AYKEN_SCHED_FALLBACK)" ./scripts/ci/gate_constitutional.sh $(CONSTITUTIONAL_STRICT_FLAG) --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/constitutional"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/constitutional/report.json" "$(EVIDENCE_RUN_DIR)/reports/constitutional.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: constitutional evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-governance-policy: ci-evidence-dir
	@echo "== CI GATE GOVERNANCE POLICY =="
	@echo "run_id: $(RUN_ID)"
	@echo "kernel_profile: $(GOVERNANCE_POLICY_KERNEL_PROFILE)"
	@KERNEL_PROFILE="$(GOVERNANCE_POLICY_KERNEL_PROFILE)" ./scripts/ci/gate_governance_policy.sh --kernel-profile "$(GOVERNANCE_POLICY_KERNEL_PROFILE)" --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/governance-policy"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/governance-policy/report.json" "$(EVIDENCE_RUN_DIR)/reports/governance-policy.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: governance-policy evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-drift-activation: ci-evidence-dir
	@echo "== CI GATE DRIFT ACTIVATION =="
	@echo "run_id: $(RUN_ID)"
	@./scripts/ci/gate_drift_activation.sh --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/drift-activation"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/drift-activation/report.json" "$(EVIDENCE_RUN_DIR)/reports/drift-activation.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: drift-activation evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-structural-abi: ci-evidence-dir
	@echo "== CI GATE STRUCTURAL ABI =="
	@echo "run_id: $(RUN_ID)"
	@echo "abi_diff_range: $(if $(strip $(ABI_DIFF_RANGE)),$(ABI_DIFF_RANGE),auto)"
	@ABI_DIFF_RANGE="$(ABI_DIFF_RANGE)" ./scripts/ci/gate_structural_abi.sh --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/structural-abi"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/structural-abi/report.json" "$(EVIDENCE_RUN_DIR)/reports/structural-abi.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: structural ABI evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-runtime-marker-contract: ci-evidence-dir
	@echo "== CI GATE RUNTIME MARKER CONTRACT =="
	@echo "run_id: $(RUN_ID)"
	@echo "enforced: $(RUNTIME_MARKER_CONTRACT_ENFORCE)"
	@echo "abi_diff_range: $(if $(strip $(ABI_DIFF_RANGE)),$(ABI_DIFF_RANGE),auto)"
	@RUNTIME_MARKER_CONTRACT_ENFORCE="$(RUNTIME_MARKER_CONTRACT_ENFORCE)" ABI_DIFF_RANGE="$(ABI_DIFF_RANGE)" ./scripts/ci/gate_runtime_marker_contract.sh --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/runtime-marker-contract"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/runtime-marker-contract/report.json" "$(EVIDENCE_RUN_DIR)/reports/runtime-marker-contract.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: runtime marker contract evidence at $(EVIDENCE_RUN_DIR)"

# Backward-compatible composite alias.
ci-gate-structural-constitution: ci-gate-structural-abi ci-gate-runtime-marker-contract
	@echo "OK: structural constitution composite gate passed"

ci-gate-syscall-v2-runtime: ci-evidence-dir
	@echo "== CI GATE SYSCALL V2 RUNTIME =="
	@echo "run_id: $(RUN_ID)"
	@echo "kernel_profile: $(SYSCALL_V2_RUNTIME_KERNEL_PROFILE)"
	@echo "warmup_runs: $(SYSCALL_V2_RUNTIME_WARMUP)"
	@echo "measurement_runs: $(SYSCALL_V2_RUNTIME_RUNS)"
	@echo "timeout_seconds: $(SYSCALL_V2_RUNTIME_TIMEOUT)"
	@echo "required_success_rate: $(SYSCALL_V2_RUNTIME_REQUIRED_SUCCESS_RATE)"
	@./scripts/ci/gate_syscall_v2_runtime.sh \
		--evidence-dir "$(EVIDENCE_RUN_DIR)/gates/syscall-v2-runtime" \
		--kernel-profile "$(SYSCALL_V2_RUNTIME_KERNEL_PROFILE)" \
		--warmup-runs "$(SYSCALL_V2_RUNTIME_WARMUP)" \
		--measurement-runs "$(SYSCALL_V2_RUNTIME_RUNS)" \
		--timeout-seconds "$(SYSCALL_V2_RUNTIME_TIMEOUT)" \
		--required-success-rate "$(SYSCALL_V2_RUNTIME_REQUIRED_SUCCESS_RATE)"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/syscall-v2-runtime/report.json" "$(EVIDENCE_RUN_DIR)/reports/syscall-v2-runtime.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: syscall-v2-runtime evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-sched-bridge-runtime: ci-evidence-dir
	@echo "== CI GATE SCHED BRIDGE RUNTIME =="
	@echo "run_id: $(RUN_ID)"
	@echo "kernel_profile: validation (enforced)"
	@echo "runtime_marker_contract_enforce: $(RUNTIME_MARKER_CONTRACT_ENFORCE)"
	@RUN_ID=$(RUN_ID) KERNEL_PROFILE=validation RUNTIME_MARKER_CONTRACT_ENFORCE="$(RUNTIME_MARKER_CONTRACT_ENFORCE)" bash scripts/ci/gate_sched_bridge_runtime.sh
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: sched-bridge-runtime evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-behavioral-suite: ci-evidence-dir
	@echo "== CI GATE BEHAVIORAL SUITE =="
	@echo "run_id: $(RUN_ID)"
	@echo "kernel_profile: validation (enforced)"
	@echo "behavioral_suite_phase: $(BEHAVIORAL_SUITE_PHASE)"
	@RUN_ID=$(RUN_ID) KERNEL_PROFILE=validation BEHAVIORAL_SUITE_PHASE="$(BEHAVIORAL_SUITE_PHASE)" bash scripts/ci/gate_behavioral_suite.sh --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/behavioral-suite" --phase "$(BEHAVIORAL_SUITE_PHASE)"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/behavioral-suite/report.json" "$(EVIDENCE_RUN_DIR)/reports/behavioral-suite.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: behavioral-suite evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-ring3-execution-phase10a2: ci-evidence-dir
	@echo "== CI GATE RING3 EXECUTION PHASE10-A2 =="
	@echo "run_id: $(RUN_ID)"
	@echo "kernel_profile: validation (enforced)"
	@echo "ayken_cr3_pcid: 0 (enforced)"
	@echo "qemu_timeout_seconds: $(RING3_QEMU_TIMEOUT)"
	@RUN_ID=$(RUN_ID) KERNEL_PROFILE=validation AYKEN_CR3_PCID=0 bash scripts/ci/gate_ring3_execution_phase10a2.sh --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/ring3-execution-phase10a2" --qemu-timeout "$(RING3_QEMU_TIMEOUT)"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/ring3-execution-phase10a2/report.json" "$(EVIDENCE_RUN_DIR)/reports/ring3-execution-phase10a2.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: ring3-execution-phase10a2 evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-syscall-semantics-phase10b: ci-gate-ring3-execution-phase10a2
	@echo "== CI GATE SYSCALL SEMANTICS PHASE10-B =="
	@echo "run_id: $(RUN_ID)"
	@echo "phase10b_mode: $(PHASE10B_MODE)"
	@echo "phase10b_a2_evidence: $(PHASE10B_A2_EVIDENCE_DIR)"
	@RUN_ID=$(RUN_ID) PHASE10B_MODE="$(PHASE10B_MODE)" bash scripts/ci/gate_syscall_semantics_phase10b.sh \
		--evidence-dir "$(EVIDENCE_RUN_DIR)/gates/syscall-semantics-phase10b" \
		--phase10a2-evidence "$(PHASE10B_A2_EVIDENCE_DIR)" \
		--mode "$(PHASE10B_MODE)"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/syscall-semantics-phase10b/report.json" "$(EVIDENCE_RUN_DIR)/reports/syscall-semantics-phase10b.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: syscall-semantics-phase10b evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-scheduler-mailbox-phase10c: ci-gate-ring3-execution-phase10a2
	@echo "== CI GATE SCHEDULER MAILBOX PHASE10-C =="
	@echo "run_id: $(RUN_ID)"
	@echo "phase10c_require_metadata: $(PHASE10C_REQUIRE_METADATA)"
	@echo "phase10c_a2_evidence: $(PHASE10C_A2_EVIDENCE_DIR)"
	@RUN_ID=$(RUN_ID) PHASE10C_REQUIRE_METADATA="$(PHASE10C_REQUIRE_METADATA)" bash scripts/ci/gate_scheduler_mailbox_phase10c.sh \
		--evidence-dir "$(EVIDENCE_RUN_DIR)/gates/scheduler-mailbox-phase10c" \
		--phase10a2-evidence "$(PHASE10C_A2_EVIDENCE_DIR)" \
		--require-metadata "$(PHASE10C_REQUIRE_METADATA)"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/scheduler-mailbox-phase10c/report.json" "$(EVIDENCE_RUN_DIR)/reports/scheduler-mailbox-phase10c.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: scheduler-mailbox-phase10c evidence at $(EVIDENCE_RUN_DIR)"

ci-gate-policy-accept: ci-evidence-dir
	@echo "== CI GATE POLICY ACCEPT =="
	@echo "run_id: $(RUN_ID)"
	@echo "kernel_profile: validation (enforced)"
	@RUN_ID=$(RUN_ID) KERNEL_PROFILE=validation bash scripts/ci/gate_4_policy_accept.sh
	@mkdir -p "$(EVIDENCE_RUN_DIR)/gates/policy-accept"
	@cp -f "evidence/gate-4-policy-accept/$(RUN_ID)/report.json" "$(EVIDENCE_RUN_DIR)/gates/policy-accept/report.json"
	@cp -f "evidence/gate-4-policy-accept/$(RUN_ID)/violations.txt" "$(EVIDENCE_RUN_DIR)/gates/policy-accept/violations.txt"
	@cp -f "evidence/gate-4-policy-accept/$(RUN_ID)/report.json" "$(EVIDENCE_RUN_DIR)/reports/policy-accept.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: policy-accept evidence at evidence/gate-4-policy-accept/$(RUN_ID)"

ci-gate-decision-switch-phase45: ci-evidence-dir
	@echo "== CI GATE DECISION SWITCH PHASE4.5 =="
	@echo "run_id: $(RUN_ID)"
	@echo "kernel_profile: validation (enforced)"
	@echo "qemu_timeout_seconds: $(GATE45_QEMU_TIMEOUT)"
	@echo "gate4_bootstrap_policy: $(GATE45_BOOTSTRAP_POLICY)"
	@echo "gate4_mb_selftest: $(GATE45_MB_SELFTEST)"
	@RUN_ID=$(RUN_ID) KERNEL_PROFILE=validation QEMU_TIMEOUT="$(GATE45_QEMU_TIMEOUT)" GATE4_BOOTSTRAP_POLICY="$(GATE45_BOOTSTRAP_POLICY)" GATE4_MB_SELFTEST="$(GATE45_MB_SELFTEST)" bash scripts/ci/gate_4_5_decision_switch_proof.sh
	@cp -f "evidence/gate-4.5-decision-switch-proof/$(RUN_ID)/report.json" "$(EVIDENCE_RUN_DIR)/gates/decision-switch-phase45/report.json"
	@cp -f "evidence/gate-4.5-decision-switch-proof/$(RUN_ID)/violations.txt" "$(EVIDENCE_RUN_DIR)/gates/decision-switch-phase45/violations.txt"
	@cp -f "evidence/gate-4.5-decision-switch-proof/$(RUN_ID)/report.json" "$(EVIDENCE_RUN_DIR)/reports/decision-switch-phase45.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: decision-switch-phase45 evidence at evidence/gate-4.5-decision-switch-proof/$(RUN_ID)"

ci-gate-policy-proof-regression: ci-gate-policy-accept ci-gate-decision-switch-phase45
	@echo "OK: policy-proof regression suite passed (Gate-4 + Gate-4.5)"

ci-gate-performance: ci-evidence-dir
	@echo "== CI GATE PERFORMANCE =="
	@echo "run_id: $(RUN_ID)"
	@echo "perf_baseline_authority: $(PERF_BASELINE_AUTHORITY)"
	@echo "perf_ci_image_digest: $(PERF_CI_IMAGE_DIGEST)"
	@echo "perf_require_ci_for_baseline_init: $(PERF_REQUIRE_CI_FOR_BASELINE_INIT)"
	@echo "ayken_sched_fallback: $(AYKEN_SCHED_FALLBACK)"
	@if [ "$(PERF_INIT_BASELINE)" = "1" ]; then \
		AYKEN_SCHED_FALLBACK="$(AYKEN_SCHED_FALLBACK)" \
		PERF_BASELINE_AUTHORITY="$(PERF_BASELINE_AUTHORITY)" \
		PERF_REQUIRE_CI_FOR_BASELINE_INIT="$(PERF_REQUIRE_CI_FOR_BASELINE_INIT)" \
		PERF_CI_IMAGE_DIGEST="$(PERF_CI_IMAGE_DIGEST)" \
		./scripts/ci/gate_performance.sh \
			--evidence-dir "$(EVIDENCE_RUN_DIR)/gates/performance" \
			--kernel-profile "$(PERF_KERNEL_PROFILE)" \
			--qemu-timeout "$(PERF_QEMU_TIMEOUT)" \
			--env-mismatch-policy "$(PERF_ENV_MISMATCH_POLICY)" \
				--baseline-file "$(PERF_BASELINE_FILE)" \
				--init-baseline; \
	else \
		AYKEN_SCHED_FALLBACK="$(AYKEN_SCHED_FALLBACK)" \
		PERF_BASELINE_AUTHORITY="$(PERF_BASELINE_AUTHORITY)" \
		PERF_REQUIRE_CI_FOR_BASELINE_INIT="$(PERF_REQUIRE_CI_FOR_BASELINE_INIT)" \
		PERF_CI_IMAGE_DIGEST="$(PERF_CI_IMAGE_DIGEST)" \
		./scripts/ci/gate_performance.sh \
			--evidence-dir "$(EVIDENCE_RUN_DIR)/gates/performance" \
			--kernel-profile "$(PERF_KERNEL_PROFILE)" \
			--qemu-timeout "$(PERF_QEMU_TIMEOUT)" \
			--env-mismatch-policy "$(PERF_ENV_MISMATCH_POLICY)" \
			--baseline-file "$(PERF_BASELINE_FILE)"; \
	fi
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/performance/report.json" "$(EVIDENCE_RUN_DIR)/reports/performance.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: performance evidence at $(EVIDENCE_RUN_DIR)"

perf-preempt-variance-local:
	@echo "== LOCAL PREEMPT VARIANCE =="
	@./scripts/ci/local_preempt_variance.sh \
		--runs "$(PERF_VARIANCE_RUNS)" \
		--warmup "$(PERF_VARIANCE_WARMUP)" \
		--qemu-timeout "$(PERF_VARIANCE_QEMU_TIMEOUT)" \
		--kernel-profile "$(PERF_KERNEL_PROFILE)" \
		--strict-markers "$(PERF_VARIANCE_STRICT_MARKERS)" \
		--force-efi-rebuild "$(PERF_VARIANCE_FORCE_EFI_REBUILD)"

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
	@echo "  release      - Build all with KERNEL_PROFILE=release"
	@echo "  validation   - Build all with KERNEL_PROFILE=validation"
	@echo "  validation-strict - Validation build with -Werror"
	@echo "  userspace-runtime - Build userspace dispatcher (bcib-runtime/bin/dispatcher)"
	@echo "  efi-img      - Create EFI disk image"
	@echo "  clean        - Clean build artifacts"
	@echo ""
	@echo "Development targets:"
	@echo "  dev          - Quick build and test cycle"
	@echo "  run          - Build and run in QEMU"
	@echo "  run-preempt  - Validation profile preempt test runner"
	@echo "  run-preempt-strict - Validation preempt runner with STRICT_MARKERS=1"
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
	@echo "  pre-ci       - Local discipline check (4 gates, ~30-60s)"
	@echo "                 Core: ABI, Boundary, Hygiene, Constitutional"
	@echo "                 Advisory only. CI remains mandatory."
	@echo "  ci           - Current CI chain (boundary + hygiene + validate-full)"
	@echo "  ci-freeze    - Strict freeze suite (all implemented gates)"
	@echo "    (hard guard: AYKEN_SCHED_FALLBACK must be 0)"
	@echo "  ci-gate-boundary - Boundary symbol scan gate with evidence output"
	@echo "  ci-gate-ring0-exports - Link-time Ring0 export surface gate (nm + whitelist + max count)"
	@echo "  ci-gate-hygiene - Repo hygiene gate with evidence output"
	@echo "  ci-gate-tooling-isolation - Fail-closed guard: perf/preempt tooling PRs cannot touch kernel/"
	@echo "  ci-gate-constitutional - Constitutional freeze gate (ABI/boundary/export/contracts hard-lock)"
	@echo "  ci-gate-governance-policy - Policy gate (source deny + AHS thresholds + waiver audit)"
	@echo "    (profile selector: GOVERNANCE_POLICY_KERNEL_PROFILE=validation)"
	@echo "  ci-gate-drift-activation - Phase-9 drift blocking activation requirement enforcement"
	@echo "  ci-gate-structural-abi - Gate-5A permanent ABI constitution lock (layout + semver policy)"
	@echo "  ci-gate-runtime-marker-contract - Gate-5B phase-scoped marker contract lock (format + anchors + semver)"
	@echo "    (toggle: RUNTIME_MARKER_CONTRACT_ENFORCE=0 to disable phase-scoped marker lock)"
	@echo "  ci-gate-structural-constitution - Composite alias: structural-abi + runtime-marker-contract"
	@echo "    (override strict locally: CONSTITUTIONAL_STRICT=0)"
	@echo "  ci-gate-behavioral-suite - Gate-6 behavioral proof suite (phase-driven)"
	@echo "    (phase selector: BEHAVIORAL_SUITE_PHASE=5 by default)"
	@echo "  ci-gate-ring3-execution-phase10a2 - Strict Phase10 scheduler+syscall+Ring3 marker-order gate"
	@echo "    (controls: RING3_QEMU_TIMEOUT, enforced: AYKEN_CR3_PCID=0)"
	@echo "    (bootstrap mode: AYKEN_SCHED_BOOTSTRAP_POLICY=0 strict default, 1 transitional override)"
	@echo "  ci-gate-syscall-semantics-phase10b - Phase10-B syscall boundary semantic state-machine gate"
	@echo "    (controls: PHASE10B_MODE=negative|positive)"
	@echo "    (A2 evidence override: PHASE10B_A2_EVIDENCE_DIR=<path>)"
	@echo "    (note: positive mode requires a CAP-free runtime scenario)"
	@echo "  ci-gate-scheduler-mailbox-phase10c - Phase10-C scheduler mailbox policy/mechanism gate (draft)"
	@echo "    (controls: PHASE10C_REQUIRE_METADATA=0|1)"
	@echo "    (A2 evidence override: PHASE10C_A2_EVIDENCE_DIR=<path>)"
	@echo "    (local freeze: enforced; ci-freeze toggle: PHASE10C_ENFORCE=0|1)"
	@echo "  ci-gate-workspace - Workspace determinism/repro/linkset gate (override: WORKSPACE_STRICT=0)"
	@echo "  ci-gate-syscall-v2-runtime - Runtime syscall v2 contract gate (Ring3 -> int80 -> Ring0)"
	@echo "    (controls: SYSCALL_V2_RUNTIME_* vars)"
	@echo "  ci-gate-policy-accept - Gate-4 isolated policy accept proof gate"
	@echo "  ci-gate-decision-switch-phase45 - Gate-4.5 decision->switch proof gate"
	@echo "    (controls: GATE45_QEMU_TIMEOUT, GATE45_BOOTSTRAP_POLICY, GATE45_MB_SELFTEST)"
	@echo "  ci-gate-policy-proof-regression - Composite regression suite: Gate-4 then Gate-4.5"
	@echo "  ci-summarize - Summarize discovered gate reports and enforce PASS"
	@echo "  ci-gate-abi - ABI drift gate (use ABI_INIT_BASELINE=1 for explicit first baseline write)"
	@echo "  ci-gate-performance - Performance baseline/env hash gate"
	@echo "    (use PERF_INIT_BASELINE=1 for first baseline write)"
	@echo "    (authority/digest: PERF_BASELINE_AUTHORITY, PERF_CI_IMAGE_DIGEST)"
	@echo "    (intentional regression test only: AYKEN_INTENTIONAL_PERF_REGRESSION_MS=<ms>)"
	@echo "    (scheduler fallback policy: AYKEN_SCHED_FALLBACK=0 for freeze)"
	@echo "  Linker export policy: KERNEL_EXPORT_POLICY=1 (default, constitutional mode)"
	@echo "  perf-preempt-variance-local - Local preempt determinism harness (mean/stdev/cv)"
	@echo "    (overrides: PERF_VARIANCE_* vars, PERF_KERNEL_PROFILE)"
	@echo "  help         - Show this help message"

.PHONY: check-deps install-deps validate validate-toolchain validate-build validate-qemu validate-qemu-env validate-qemu-integration validate-full setup dev ci ci-freeze ci-freeze-guard ci-evidence-dir ci-gate-boundary ci-gate-ring0-exports ci-summarize ci-gate-abi ci-gate-workspace ci-gate-hygiene ci-gate-tooling-isolation ci-gate-constitutional ci-gate-governance-policy ci-gate-drift-activation ci-gate-structural-abi ci-gate-runtime-marker-contract ci-gate-structural-constitution ci-gate-syscall-v2-runtime ci-gate-sched-bridge-runtime ci-gate-behavioral-suite ci-gate-ring3-execution-phase10a2 ci-gate-syscall-semantics-phase10b ci-gate-scheduler-mailbox-phase10c ci-gate-policy-accept ci-gate-decision-switch-phase45 ci-gate-policy-proof-regression ci-gate-performance perf-preempt-variance-local generate-abi help

# UEFI bootloader assembly sources (.S)
$(BOOTLOADER_DIR)/%.efi.o: $(BOOTLOADER_DIR)/%.S
	$(EFI_CC) $(EFI_CFLAGS) -c $< -o $@
