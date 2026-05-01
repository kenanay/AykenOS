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

# Object files
KERNEL_OBJ = execution_marker_validation.o
TEST_OBJ = execution_marker_validation_test.o

# Target
TARGET = execution_marker_validation_test

.PHONY: all clean test

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

clean:
	rm -f $(KERNEL_OBJ) $(TEST_OBJ) $(TARGET)

# Help
help:
	@echo "Execution Marker Validation Test"
	@echo ""
	@echo "Targets:"
	@echo "  all    - Build test binary"
	@echo "  test   - Build and run tests"
	@echo "  clean  - Remove build artifacts"
	@echo "  help   - Show this help"
