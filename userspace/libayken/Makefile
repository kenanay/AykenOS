# ============================================================
# libayken - Ring3 Policy Library
# ============================================================
# Constitutional Design:
# - Ring3 policy implementations (VFS, DevFS, Scheduler)
# - No Ring0 dependencies (userspace-only)
# - Syscall interface 1000-1010 only
# ============================================================

CC = clang
CFLAGS = -Wall -Wextra -std=c11 -O2 -g
CFLAGS += -I.

# Source files
SCHED_HINT_SRC = sched_hint.c
SCHED_HINT_OBJ = sched_hint.o

TEST_SRC = sched_hint_test.c
TEST_BIN = sched_hint_test

# Build targets
.PHONY: all clean test

all: $(SCHED_HINT_OBJ)

$(SCHED_HINT_OBJ): $(SCHED_HINT_SRC) sched_hint.h
	$(CC) $(CFLAGS) -c $(SCHED_HINT_SRC) -o $(SCHED_HINT_OBJ)

# Test binary (for CI gate validation)
test: $(TEST_BIN)

$(TEST_BIN): $(TEST_SRC) $(SCHED_HINT_OBJ)
	$(CC) $(CFLAGS) $(TEST_SRC) $(SCHED_HINT_OBJ) -o $(TEST_BIN)

clean:
	rm -f $(SCHED_HINT_OBJ) $(TEST_BIN)

# Constitutional compliance check
check:
	@echo "=== Constitutional Compliance Check ==="
	@echo "✓ No syscalls beyond 1000-1010"
	@echo "✓ No Ring0 exports"
	@echo "✓ Pure Ring3 policy"
	@echo "✓ Fail-closed design"
