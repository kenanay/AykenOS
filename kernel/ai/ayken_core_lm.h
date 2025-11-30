#ifndef AYKEN_CORE_LM_H
#define AYKEN_CORE_LM_H

#include <stdint.h>
#include "ayken_core_lm_format.h"

typedef struct {
    uint32_t vocab_size;
    uint32_t hidden_size;
    uint32_t n_layers;
    uint32_t n_heads;
    ayken_quant_type_t quant_type;

    void    *weights;       // sanal adres (mapped)
    uint64_t weights_size;  // byte
} lm_model_t;

void        ayken_core_lm_init(void);
lm_model_t* ayken_core_lm_get(void);

// *** YENİ: tensor tablosu API'si ***
const ayken_lm_tensor_desc_t*
ayken_core_lm_find_tensor(ayken_tensor_type_t type, int layer_idx);

const void*
ayken_core_lm_get_tensor_data(const ayken_lm_tensor_desc_t *desc);

#endif
