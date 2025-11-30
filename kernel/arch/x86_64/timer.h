#pragma once
#include <stdint.h>

void timer_init(uint32_t frequency_hz);
uint64_t timer_ticks(void);
