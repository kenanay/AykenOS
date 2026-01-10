#ifndef AYKEN_ABDF_FORMAT_H
#define AYKEN_ABDF_FORMAT_H

#include <stdint.h>

// Rust tarafındaki (ayken-core/crates/abdf/src/header.rs)
// AbdfHeader yapısının C karşılığı.
// #[repr(C)] olduğu için memory layout birebir aynıdır.

#define ABDF_MAGIC_BYTES "ABDF"
#define ABDF_VERSION 1

typedef struct {
    uint8_t  magic[4];      // "ABDF"
    uint16_t version;       // 1
    uint16_t flags;         // 0
    uint32_t segment_count; // Segment sayısı
} abdf_header_t;

#endif