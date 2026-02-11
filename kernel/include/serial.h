#pragma once
#include <stdint.h>

void serial_init_com1(void);
void serial_write_char(char c);
void serial_write(const char *s);