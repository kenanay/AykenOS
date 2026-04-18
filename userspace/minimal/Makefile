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
else ifeq ($(MINIMAL_MODE),runtime-bridge-test)
MINIMAL_SRC := minimal_runtime_bridge_test.S
else ifeq ($(MINIMAL_MODE),runtime-bridge-ping)
MINIMAL_SRC := minimal_runtime_bridge_ping.S
else ifeq ($(MINIMAL_MODE),runtime-bridge-forbidden)
MINIMAL_SRC := minimal_runtime_bridge_forbidden.S
else ifeq ($(MINIMAL_MODE),execution-delivery-proof)
MINIMAL_SRC := minimal_execution_delivery_proof.S
else ifeq ($(MINIMAL_MODE),execution-marker-only)
MINIMAL_SRC := minimal_execution_marker_only.S
else ifeq ($(MINIMAL_MODE),entry-proof)
MINIMAL_SRC := minimal_entry_proof.S
else ifeq ($(MINIMAL_MODE),bcib-forbidden)
MINIMAL_SRC := minimal_bcib_forbidden.S
else ifeq ($(MINIMAL_MODE),bcib-worker-bootstrap)
MINIMAL_SRC := minimal_bcib_worker.S
else ifeq ($(MINIMAL_MODE),dual-worker-pipeline)
# Dual-payload mode: build both BCIB and USER workers
MINIMAL_SRC := minimal_bcib_worker.S
DUAL_PAYLOAD_MODE := 1
else ifeq ($(MINIMAL_MODE),second-syscall-proof)
MINIMAL_SRC := minimal_second_syscall_proof.S
else ifeq ($(MINIMAL_MODE),ring3-spin-probe)
MINIMAL_SRC := minimal_ring3_spin_probe.S
else
$(error Invalid MINIMAL_MODE='$(MINIMAL_MODE)'. Use phase10a2, phase10a2-entry-bp, phase10a2-entry-nobp, phase10a2-canary-witness, phase10a2-text-witness, phase10a2-text-witness-bp, phase10a2-mailbox-bp, syscall-v2-runtime, runtime-bridge-test, entry-proof, bcib-forbidden, bcib-worker-bootstrap, dual-worker-pipeline, or second-syscall-proof)
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

ifdef DUAL_PAYLOAD_MODE
# Dual-payload mode: build both BCIB and USER workers
all: bcib_worker.elf user_worker.elf user.bin

bcib_worker.o: minimal_bcib_worker.S $(MODE_STAMP)
	@echo "[DUAL-PAYLOAD] Building BCIB worker payload"
	$(AS) $(ASFLAGS) -c -o $@ minimal_bcib_worker.S

bcib_worker.elf: bcib_worker.o user.ld
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ bcib_worker.o

user_worker.o: minimal_user_worker.S $(MODE_STAMP)
	@echo "[DUAL-PAYLOAD] Building USER worker payload"
	$(AS) $(ASFLAGS) -c -o $@ minimal_user_worker.S

user_worker.elf: user_worker.o user.ld
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ user_worker.o

# Legacy user.bin for compatibility (uses BCIB worker)
user.bin: bcib_worker.elf bcib_worker.o user.ld
	$(CC) $(CFLAGS) -nostdlib -static -Wl,-T,user.ld -Wl,--oformat=binary -Wl,-z,max-page-size=0x1000 -o $@ bcib_worker.o

else
# Single-payload mode (legacy)
all: minimal.elf user.bin

minimal.o: $(MINIMAL_SRC) $(MODE_STAMP)
	@echo "[PHASE10] userspace mode: $(MINIMAL_MODE) ($(MINIMAL_SRC))"
	$(AS) $(ASFLAGS) -c -o $@ $(MINIMAL_SRC)

minimal.elf: minimal.o user.ld
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ minimal.o

user.bin: minimal.elf minimal.o user.ld
	$(CC) $(CFLAGS) -nostdlib -static -Wl,-T,user.ld -Wl,--oformat=binary -Wl,-z,max-page-size=0x1000 -o $@ minimal.o
endif

$(MODE_STAMP): FORCE
	@if [ ! -f $(MODE_STAMP) ] || [ "$$(cat $(MODE_STAMP) 2>/dev/null)" != "$(MINIMAL_MODE)" ]; then \
		echo "$(MINIMAL_MODE)" > $(MODE_STAMP); \
	fi

test: minimal.elf
	$(PYTHON) ./test_minimal_elf.py ./minimal.elf

clean:
	rm -f minimal.o minimal.elf user.bin bcib_worker.o bcib_worker.elf user_worker.o user_worker.elf $(MODE_STAMP)

.PHONY: all clean test FORCE
FORCE:
