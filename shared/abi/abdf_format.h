#ifndef AYKEN_ABDF_FORMAT_H
#define AYKEN_ABDF_FORMAT_H

#include <stdint.h>

#define ABDF_MAGIC_BYTES "ABDF"
#define ABDF_VERSION 1

typedef struct {
    uint8_t  magic[4];
    uint16_t version;
    uint16_t flags;
    uint32_t segment_count;
} abdf_header_t;

#endif
