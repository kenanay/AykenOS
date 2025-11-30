// kernel/ai/lm_runtime.c

#include <stdint.h>
#include <stddef.h>

#include "lm_runtime.h"
#include "ayken_core_lm.h"
#include "ayken_core_lm_format.h"
#include "lm_tokenizer.h"
#include "../drivers/console/fb_console.h"

// =========================
// Sabitler / sınırlar
// =========================

#define LM_MAX_TOKENS    256
#define LM_MAX_CTX       256
#define LM_MAX_HIDDEN    512   // tiny model için güvenlik sınırı

// Çalışma buffer'ları
static int   g_tokens[LM_MAX_TOKENS];
static float g_hidden[LM_MAX_HIDDEN];
static float g_logits[LM_MAX_HIDDEN];



// =========================
// Basit yardımcılar
// =========================

static int lm_min(int a, int b) {
    return (a < b) ? a : b;
}

static int lm_argmax(const float *v, int n) {
    if (n <= 0) return 0;
    float best = v[0];
    int idx = 0;
    for (int i = 1; i < n; i++) {
        if (v[i] > best) {
            best = v[i];
            idx = i;
        }
    }
    return idx;
}

// Geçici: token -> char mapping
// Gerçek sistemde vocab tablosu kullanacaksın.
static char lm_token_to_char(int token) {
    if (token < 0)   token = 0;
    if (token > 255) token = 255;
    return (char)token;
}

// =========================
// Q4/Q2 blok formatları
// =========================

// Q4_0: 32 weight = 16 byte packed + scale
typedef struct {
    int8_t  scale;
    uint8_t data[16];     // 2 ağırlık / byte (4-bit)
} q4_block_t;

// Q2_0: 32 weight = 8 byte packed + scale
typedef struct {
    int8_t  scale;
    uint8_t data[8];      // 4 ağırlık / byte (2-bit)
} q2_block_t;

// =========================
// Q4/Q2 dequant işlemleri
// =========================

// Q4: her byte -> 2 ağırlık (4-bit)
static inline float dq_q4(uint8_t byte, int hi)
{
    int v;
    if (hi)
        v = (byte >> 4) & 0xF;
    else
        v = byte & 0xF;

    // 0..15 -> -8..+7
    return (float)(v - 8);
}

static inline void dequant_q4_block(const q4_block_t *blk, float *out)
{
    float s = (float)blk->scale;
    for (int i = 0; i < 32; i++) {
        uint8_t b = blk->data[i >> 1];
        float d   = dq_q4(b, (i & 1));
        out[i] = d * s;
    }
}

// Q2: her byte -> 4 ağırlık (2-bit)
static inline float dq_q2(uint8_t byte, int shift)
{
    int v = (byte >> shift) & 0x3;
    // 0..3 -> -2..+1
    return (float)(v - 2);
}

static inline void dequant_q2_block(const q2_block_t *blk, float *out)
{
    float s = (float)blk->scale;
    for (int i = 0; i < 32; i++) {
        uint8_t b = blk->data[i >> 2];
        int shift = (i & 3) * 2;
        float d   = dq_q2(b, shift);
        out[i] = d * s;
    }
}

// =========================
// Kuantize matmul iskeleti
// =========================
//
// weights:
//  - rows: out_size
//  - cols: in_size
//  - her satır blocks_per_row adet Q4/Q2 bloktan oluşur
//

static void lm_matmul_qx(const lm_model_t *m,
                         const void *weights,
                         const float *input,
                         int in_size,
                         int out_size,
                         float *output)
{
    // 1) Çıkış vektörünü sıfırla
    for (int i = 0; i < out_size; i++) {
        output[i] = 0.0f;
    }

    if (!weights) {
        // Henüz gerçek ağırlık layout'unu bağlamadıysan NO-OP
        return;
    }

    int blocks_per_row = (in_size + 31) / 32;

    int block_size = (m->quant_type == AYKEN_QUANT_Q4_0)
                     ? (int)sizeof(q4_block_t)
                     : (int)sizeof(q2_block_t);

    for (int o = 0; o < out_size; o++) {

        const uint8_t *row_ptr = (const uint8_t*)weights
                               + (size_t)o * (size_t)blocks_per_row * (size_t)block_size;

        for (int b = 0; b < blocks_per_row; b++) {

            float tmp[32];

            if (m->quant_type == AYKEN_QUANT_Q4_0) {

                const q4_block_t *blk =
                    (const q4_block_t*)(row_ptr + (size_t)b * sizeof(q4_block_t));
                dequant_q4_block(blk, tmp);

            } else if (m->quant_type == AYKEN_QUANT_Q2_0) {

                const q2_block_t *blk =
                    (const q2_block_t*)(row_ptr + (size_t)b * sizeof(q2_block_t));
                dequant_q2_block(blk, tmp);
            }

            // 3) Bu bloktaki 32 ağırlığı input’la çarp
            for (int i = 0; i < 32; i++) {
                int idx = b * 32 + i;
                if (idx < in_size) {
                    output[o] += tmp[i] * input[idx];
                }
            }
        }
    }
}

// =========================
// Embedding & layer iskeleti
// =========================

// 1) Embedding: token -> hidden vektör
static void lm_embed_token(const lm_model_t *m, int token, float *hidden_out)
{
    // TODO:
    //  - core_model.weights içinde embedding tablosu ayrılmış olacak
    //  - tensor tablosundan AYKEN_TENSOR_EMBED descriptor'ını bulup,
    //    ilgili satırı direkt okuyacak.
    //
    // Şimdilik basit bir pseudo-embedding:
    int h = (int)m->hidden_size;
    if (h > LM_MAX_HIDDEN) h = LM_MAX_HIDDEN;

    float base = (float)(token % 31) / 31.0f;
    for (int i = 0; i < h; i++) {
        hidden_out[i] = base;
    }
}

// 2) Tek transformer katmanı (iskelet)
static void lm_run_layer(const lm_model_t *m, float *hidden, int layer_idx)
{
    int h = (int)m->hidden_size;
    if (h > LM_MAX_HIDDEN) h = LM_MAX_HIDDEN;

    // --- Tensör descriptor'larını bul ---
    const ayken_lm_tensor_desc_t *td_wq =
        ayken_core_lm_find_tensor(AYKEN_TENSOR_W_Q, layer_idx);
    const ayken_lm_tensor_desc_t *td_wk =
        ayken_core_lm_find_tensor(AYKEN_TENSOR_W_K, layer_idx);
    const ayken_lm_tensor_desc_t *td_wv =
        ayken_core_lm_find_tensor(AYKEN_TENSOR_W_V, layer_idx);
    const ayken_lm_tensor_desc_t *td_wo =
        ayken_core_lm_find_tensor(AYKEN_TENSOR_W_O, layer_idx);

    const ayken_lm_tensor_desc_t *td_ff1 =
        ayken_core_lm_find_tensor(AYKEN_TENSOR_W_FF1, layer_idx);
    const ayken_lm_tensor_desc_t *td_ff2 =
        ayken_core_lm_find_tensor(AYKEN_TENSOR_W_FF2, layer_idx);

    const void *W_q  = ayken_core_lm_get_tensor_data(td_wq);
    const void *W_k  = ayken_core_lm_get_tensor_data(td_wk);
    const void *W_v  = ayken_core_lm_get_tensor_data(td_wv);
    const void *W_o  = ayken_core_lm_get_tensor_data(td_wo);
    const void *W_f1 = ayken_core_lm_get_tensor_data(td_ff1);
    const void *W_f2 = ayken_core_lm_get_tensor_data(td_ff2);

    // --- Çalışma buffer'ları ---
    float q[LM_MAX_HIDDEN];
    float k[LM_MAX_HIDDEN];
    float v[LM_MAX_HIDDEN];
    float att_out[LM_MAX_HIDDEN];
    float ff_tmp[LM_MAX_HIDDEN];

    // 1) Q/K/V projeksiyonları
    lm_matmul_qx(m, W_q, hidden, h, h, q);
    lm_matmul_qx(m, W_k, hidden, h, h, k);
    lm_matmul_qx(m, W_v, hidden, h, h, v);

    // 2) Attention (burada sadece iskelet; gerçek attention yok)
    // TODO:
    //  - q, k, v -> scaled dot-product attention
    //  - çok başlı (multi-head) tasarım
    // Şimdilik: v çıktısını olduğu gibi att_out’a kopyalıyoruz.
    for (int i = 0; i < h; i++) {
        att_out[i] = v[i];
    }

    // 3) Attention çıkışını W_o ile projekte et
    lm_matmul_qx(m, W_o, att_out, h, h, hidden);

    // 4) FFN (iki matmul + aktivasyon)
    lm_matmul_qx(m, W_f1, hidden, h, h, ff_tmp);

    // Basit aktivasyon (ReLU gibi davranan klon)
    for (int i = 0; i < h; i++) {
        if (ff_tmp[i] < 0.0f) ff_tmp[i] = 0.0f;
    }

    lm_matmul_qx(m, W_f2, ff_tmp, h, h, hidden);

    // TODO:
    //  - Residual bağlantı (x + f(x))
    //  - LayerNorm
    // Şimdilik bu kısım atlanıyor (hidden doğrudan güncellendi).
}

// 3) Tüm katmanları çalıştır
static void lm_run_transformer(const lm_model_t *m, float *hidden)
{
    uint32_t L = m->n_layers;
    for (uint32_t i = 0; i < L; i++) {
        lm_run_layer(m, hidden, (int)i);
    }
}

// 4) Hidden -> logits dönüşümü
static void lm_hidden_to_logits(const lm_model_t *m,
                                const float *hidden,
                                float *logits_out)
{
    int vocab = (int)m->vocab_size;
    int h     = (int)m->hidden_size;

    int n = lm_min(vocab, h);
    if (n > LM_MAX_HIDDEN) n = LM_MAX_HIDDEN;

    // TODO:
    //  - Gerçek projeksiyon: W_LOGITS * hidden
    // Şimdilik: ilk n hidden değerini logits olarak kullanıyoruz.
    for (int i = 0; i < n; i++) {
        logits_out[i] = hidden[i];
    }

    // Vocab hidden'dan büyükse geri kalan logits çok küçük olsun
    for (int i = n; i < vocab && i < LM_MAX_HIDDEN; i++) {
        logits_out[i] = -1e9f;
    }
}

// =========================
// Ana Inference API
// =========================
//
// prompt  : giriş metni
// out     : çıktı buffer'ı
// max_out : en fazla kaç karakter yazılacak
//
// Şimdilik: tek adım, tek karakter üretir.
//

int lm_infer(const char *prompt, char *out, int max_out)
{
    lm_model_t *m = ayken_core_lm_get();
    if (!m || m->weights == NULL || m->hidden_size == 0) {
        fb_print("[AykenCoreLM][runtime] Model not initialized.\n");
        return -1;
    }

    // 1) Prompt'u tokenize et
    int n_tokens = lm_tokenize(prompt, g_tokens, LM_MAX_TOKENS);
    if (n_tokens <= 0) {
        fb_print("[AykenCoreLM][runtime] Tokenization failed.\n");
        return -1;
    }

    // 2) Basit: sadece son token üzerinden tahmin
    int last_token = g_tokens[n_tokens - 1];

    int h = (int)m->hidden_size;
    if (h > LM_MAX_HIDDEN) h = LM_MAX_HIDDEN;

    // 3) Embedding
    lm_embed_token(m, last_token, g_hidden);

    // 4) Transformer katmanları
    lm_run_transformer(m, g_hidden);

    // 5) Hidden -> logits
    lm_hidden_to_logits(m, g_hidden, g_logits);

    // 6) Argmax ile bir sonraki token seç
    int vocab = (int)m->vocab_size;
    if (vocab > LM_MAX_HIDDEN) {
        vocab = LM_MAX_HIDDEN;
    }

    int next_token = lm_argmax(g_logits, vocab);

    // 7) Şimdilik 1 token -> 1 char üret
    if (max_out <= 0) return 0;

    char c = lm_token_to_char(next_token);
    out[0] = c;

    if (max_out > 1) {
        out[1] = '\0';
    }

    return 1;   // 1 karakter ürettik
}
