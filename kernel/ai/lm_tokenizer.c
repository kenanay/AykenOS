#include "lm_tokenizer.h"

int lm_tokenize(const char *text, int *out_tokens, int max_tokens)
{
    // Basit BPE tokenizer — minimal bir sürüm
    // (gerçek uygulamada tablo yüklenir)

    int count = 0;

    while (*text && count < max_tokens) {
        out_tokens[count++] = (unsigned char)(*text);
        text++;
    }

    return count;
}
