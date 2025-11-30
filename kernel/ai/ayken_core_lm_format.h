#ifndef AYKEN_CORE_LM_FORMAT_H
#define AYKEN_CORE_LM_FORMAT_H

#include <stdint.h>

#define AYKEN_LM_MAGIC  "AYKLM01"   // 7 byte + '\0'

// ------------------------------
//  1) Quant türleri
// ------------------------------
typedef enum {
    AYKEN_QUANT_Q4_0 = 1,
    AYKEN_QUANT_Q2_0 = 2
} ayken_quant_type_t;

// ------------------------------
//  2) Model header
// ------------------------------
typedef struct __attribute__((packed)) {
    char     magic[8];        // "AYKLM01"
    uint32_t version;         // 1
    uint32_t quant_type;      // ayken_quant_type_t
    uint32_t vocab_size;
    uint32_t hidden_size;
    uint32_t n_layers;
    uint32_t n_heads;

    uint32_t tensor_count;    // tensor descriptor sayısı
    uint32_t reserved;        // hizalama

    uint64_t tensor_table_offset; // header → tensor table
    uint64_t weights_offset;      // weights bloğu başlangıcı
    uint64_t weights_size;        // weights toplam boyut (bytes)
} ayken_lm_file_header_t;

// ------------------------------
//  3) Tensor türleri
// ------------------------------
typedef enum {
    AYKEN_TENSOR_EMBED = 1,

    AYKEN_TENSOR_W_Q,
    AYKEN_TENSOR_W_K,
    AYKEN_TENSOR_W_V,
    AYKEN_TENSOR_W_O,

    AYKEN_TENSOR_W_FF1,
    AYKEN_TENSOR_W_FF2,

    AYKEN_TENSOR_LN1_GAIN,
    AYKEN_TENSOR_LN1_BIAS,
    AYKEN_TENSOR_LN2_GAIN,
    AYKEN_TENSOR_LN2_BIAS,

    AYKEN_TENSOR_W_LOGITS
} ayken_tensor_type_t;

// ------------------------------
//  4) Tensor descriptor tablosu
// ------------------------------
typedef struct __attribute__((packed)) {
    uint32_t type;       // ayken_tensor_type_t
    uint32_t layer_idx;  // 0..n_layers-1 veya 0xFFFFFFFF
    uint64_t offset;     // weights blob içindeki başlangıç
    uint32_t rows;       // matris satır
    uint32_t cols;       // matris sütun
} ayken_lm_tensor_desc_t;

#endif
