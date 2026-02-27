# AykenOS Phase 10-A: Minimal Ring3 ELF Build
# - Task 1.4 output is minimal.elf
# - user.bin is kept for current kernel embedding compatibility

CC := clang
PYTHON := python3

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

LDFLAGS := -nostdlib \
           -static \
           -Wl,-T,user.ld \
           -Wl,--build-id=none \
           -Wl,-z,max-page-size=0x1000

all: minimal.elf user.bin

minimal.o: minimal.c
	$(CC) $(CFLAGS) -c -o $@ $<

minimal.elf: minimal.o user.ld
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ minimal.o

user.bin: minimal.elf minimal.o user.ld
	$(CC) $(CFLAGS) -nostdlib -static -Wl,-T,user.ld -Wl,--oformat=binary -Wl,-z,max-page-size=0x1000 -o $@ minimal.o

test: minimal.elf
	$(PYTHON) ./test_minimal_elf.py ./minimal.elf

clean:
	rm -f minimal.o minimal.elf user.bin

.PHONY: all clean test
