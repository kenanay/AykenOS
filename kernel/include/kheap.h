#ifndef AYKEN_KHEAP_H
#define AYKEN_KHEAP_H

#include <stdint.h>

void kheap_init(void);
void *kmalloc(uint64_t size);
void kfree(void *ptr);

#endif // AYKEN_KHEAP_H
