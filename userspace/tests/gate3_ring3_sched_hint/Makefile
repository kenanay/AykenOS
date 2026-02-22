# userspace/tests/gate3_ring3_sched_hint/Makefile
# Gate-3: Ring3 Runtime Validation Test

CC = clang
LD = ld.lld
CFLAGS = --target=x86_64-elf -ffreestanding -nostdlib -fno-pic -fno-stack-protector -mno-red-zone -O0 -g
LDFLAGS = -nostdlib -static

TARGET = gate3_ring3_test.elf

all: $(TARGET)

$(TARGET): main.o
	$(LD) $(LDFLAGS) -Ttext=0x400000 -o $@ $<

main.o: main.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f *.o $(TARGET)

.PHONY: all clean
