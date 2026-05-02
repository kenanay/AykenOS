# Execution Marker Validation - Userspace Test Makefile
# 
# PURPOSE:
# Build and run marker validation tests independently from kernel.
# 
# SCOPE:
# - Compile with gcc (userspace)
# - Link kernel validation logic
# - Run tests

CC = gcc
CFLAGS = -Wall -Wextra -Werror -I../../kernel/include -g -O0
LDFLAGS =

# Source files
KERNEL_SRC = ../../kernel/sys/execution_marker_validation.c
TEST_SRC = execution_marker_validation_test.c

# Phase-17 injection test
PHASE17_KERNEL_SRC = ../../kernel/sys/execution_slot.c
PHASE17_TEST_SRC = phase17_marker_injection_test.c
PHASE17_TARGET = phase17_marker_injection_test

# Object files
KERNEL_OBJ = execution_marker_validation.o
TEST_OBJ = execution_marker_validation_test.o

# Target
TARGET = execution_marker_validation_test

.PHONY: all clean test test-phase17

all: $(TARGET)

$(TARGET): $(KERNEL_OBJ) $(TEST_OBJ)
	$(CC) $(LDFLAGS) -o $@ $^

$(KERNEL_OBJ): $(KERNEL_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(TEST_OBJ): $(TEST_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

test: $(TARGET)
	@echo "=== Running Marker Validation Tests ==="
	@./$(TARGET)

# Phase-17 injection runtime test (standalone)
$(PHASE17_TARGET): $(PHASE17_TEST_SRC)
	@echo "=== Building Phase-17 Injection Runtime Test ==="
	$(CC) $(CFLAGS) -o $@ $(PHASE17_TEST_SRC)

test-phase17: $(PHASE17_TARGET)
	@echo "=== Running Phase-17 Injection Runtime Tests ==="
	@./$(PHASE17_TARGET)

clean:
	rm -f $(KERNEL_OBJ) $(TEST_OBJ) $(TARGET)
	rm -f $(PHASE17_TARGET)

# Help
help:
	@echo "Execution Marker Validation Test"
	@echo ""
	@echo "Targets:"
	@echo "  all           - Build test binary"
	@echo "  test          - Build and run tests"
	@echo "  test-phase17  - Build and run Phase-17 injection runtime tests"
	@echo "  clean         - Remove build artifacts"
	@echo "  help          - Show this help"
