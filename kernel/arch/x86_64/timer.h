#pragma once
#include <stdint.h>

void timer_init(uint32_t frequency_hz);
uint64_t timer_ticks(void);
uint32_t timer_frequency_hz(void);
uint64_t timer_ticks_to_ms(uint64_t ticks);
uint64_t timer_ms_to_ticks_ceil(uint64_t ms);
