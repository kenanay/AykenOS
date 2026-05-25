# AykenOS Phase 10-A: Minimal Ring3 ELF Build
# - Task 3: Assembly stub for syscall roundtrip test
# - user.bin is kept for current kernel embedding compatibility

CC := clang
AS := clang
PYTHON := python3
MINIMAL_MODE ?= phase10a2
MODE_STAMP := .build_mode.stamp

ifeq ($(MINIMAL_MODE),phase10a2)
MINIMAL_SRC := minimal.S
else ifeq ($(MINIMAL_MODE),phase10a2-entry-bp)
MINIMAL_SRC := minimal_phase10a2_entry_bp.S
else ifeq ($(MINIMAL_MODE),phase10a2-entry-nobp)
MINIMAL_SRC := minimal_phase10a2_entry_nobp.S
else ifeq ($(MINIMAL_MODE),phase10a2-canary-witness)
MINIMAL_SRC := minimal_phase10a2_canary_witness.S
else ifeq ($(MINIMAL_MODE),phase10a2-text-witness)
MINIMAL_SRC := minimal_phase10a2_text_witness.S
else ifeq ($(MINIMAL_MODE),phase10a2-text-witness-bp)
MINIMAL_SRC := minimal_phase10a2_text_witness_bp.S
else ifeq ($(MINIMAL_MODE),phase10a2-mailbox-bp)
MINIMAL_SRC := minimal_phase10a2_mailbox_bp.S
else ifeq ($(MINIMAL_MODE),syscall-v2-runtime)
MINIMAL_SRC := minimal_syscall_v2_runtime.S
else ifeq ($(MINIMAL_MODE),bcib-public-e2e)
MINIMAL_SRC := minimal_bcib_public_e2e.S
else ifeq ($(MINIMAL_MODE),bcib-worker-completion)
MINIMAL_SRC := minimal_bcib_worker_completion.S
else ifeq ($(MINIMAL_MODE),bcib-timeout-race)
MINIMAL_SRC := minimal_bcib_timeout_race.S
else
$(error Invalid MINIMAL_MODE='$(MINIMAL_MODE)'. Use phase10a2, phase10a2-entry-bp, phase10a2-entry-nobp, phase10a2-canary-witness, phase10a2-text-witness, phase10a2-text-witness-bp, phase10a2-mailbox-bp, syscall-v2-runtime, bcib-public-e2e, bcib-worker-completion, or bcib-timeout-race)
endif

CFLAGS := -target x86_64-elf \
          -ffreestanding \
          -fno-stack-protector \
          -fno-pic \
          -mno-red-zone \
          -nostdlib \
          -static \
          -Wall \
          -Wextra \
          -O2

ASFLAGS := -target x86_64-elf \
           -ffreestanding \
           -fno-pic \
           -mno-red-zone \
           -nostdlib

LDFLAGS := -nostdlib \
           -static \
           -Wl,-T,user.ld \
           -Wl,--build-id=none \
           -Wl,-z,max-page-size=0x1000

all: minimal.elf user.bin

$(MODE_STAMP): FORCE
	@if [ ! -f $(MODE_STAMP) ] || [ "$$(cat $(MODE_STAMP) 2>/dev/null)" != "$(MINIMAL_MODE)" ]; then \
		echo "$(MINIMAL_MODE)" > $(MODE_STAMP); \
	fi

minimal.o: $(MINIMAL_SRC) $(MODE_STAMP)
	@echo "[PHASE10] userspace mode: $(MINIMAL_MODE) ($(MINIMAL_SRC))"
	$(AS) $(ASFLAGS) -c -o $@ $(MINIMAL_SRC)

minimal.elf: minimal.o user.ld
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ minimal.o

user.bin: minimal.elf minimal.o user.ld
	$(CC) $(CFLAGS) -nostdlib -static -Wl,-T,user.ld -Wl,--oformat=binary -Wl,-z,max-page-size=0x1000 -o $@ minimal.o

test: minimal.elf
	$(PYTHON) ./test_minimal_elf.py ./minimal.elf

clean:
	rm -f minimal.o minimal.elf user.bin $(MODE_STAMP)

.PHONY: all clean test FORCE
FORCE:
