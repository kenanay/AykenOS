// kernel/drivers/ui/logo_animator.c
//
// AykenOS Boot Logo Animatörü
// - 128x128 ve 256x256 logo arasında seçim
// - Swirl + glow (dairesel enerji akışı) animasyonu
// - Gerçek rotation yok; ışık dağılımı kaydırılarak
//   dönüyormuş illüzyonu verilir.

#include <stdint.h>
#include "logo_animator.h"

#include "ayken_logo_128.h"
#include "ayken_logo_256.h"
#include "../console/fb_console.h"

// ---------------------------------------------------------------------------
// Sinus LUT (0..255 index → -127..127 arası yaklaşık sinüs değeri)
// Dışarıdan libm (sinf, atan2f vs.) kullanmamak için statik tablo.
// ---------------------------------------------------------------------------
static const int8_t sin_lut[256] = {
      0,   3,   6,   9,  12,  16,  19,  22,
     25,  28,  31,  34,  37,  40,  43,  46,
     49,  52,  55,  58,  61,  64,  67,  70,
     73,  76,  79,  82,  85,  88,  91,  94,
     96,  99, 102, 105, 107, 110, 113, 115,
    118, 121, 123, 126, 128, 131, 133, 136,
    138, 140, 143, 145, 147, 150, 152, 154,
    156, 158, 160, 162, 164, 166, 168, 170,
    172, 173, 175, 177, 178, 180, 181, 183,
    184, 185, 187, 188, 189, 190, 191, 192,
    193, 194, 195, 196, 197, 197, 198, 199,
    199, 200, 200, 201, 201, 201, 202, 202,
    202, 202, 202, 202, 202, 202, 202, 201,
    201, 201, 200, 200, 199, 199, 198, 197,
    197, 196, 195, 194, 193, 192, 191, 190,
    189, 188, 187, 185, 184, 183, 181, 180,
    178, 177, 175, 173, 172, 170, 168, 166,
    164, 162, 160, 158, 156, 154, 152, 150,
    147, 145, 143, 140, 138, 136, 133, 131,
    128, 126, 123, 121, 118, 115, 113, 110,
    107, 105, 102,  99,  96,  94,  91,  88,
     85,  82,  79,  76,  73,  70,  67,  64,
     61,  58,  55,  52,  49,  46,  43,  40,
     37,  34,  31,  28,  25,  22,  19,  16,
     12,   9,   6,   3,   0,  -3,  -6,  -9,
    -12, -16, -19, -22, -25, -28, -31, -34,
    -37, -40, -43, -46, -49, -52, -55, -58,
    -61, -64, -67, -70, -73, -76, -79, -82,
    -85, -88, -91, -94, -96, -99,-102,-105,
   -107,-110,-113,-115,-118,-121,-123,-126,
   -128,-131,-133,-136,-138,-140,-143,-145,
   -147,-150,-152,-154,-156,-158,-160,-162,
   -164,-166,-168,-170,-172,-173,-175,-177,
   -178,-180,-181,-183,-184,-185,-187,-188,
   -189,-190,-191,-192,-193,-194,-195,-196,
   -197,-197,-198,-199,-199,-200,-200,-201,
   -201,-201,-202,-202,-202,-202,-202,-202
};

// ---------------------------------------------------------------------------
// Seçilen logo'ya işaretçi (128 veya 256)
// ---------------------------------------------------------------------------

static const unsigned int (*g_logo)[256] = NULL; // en büyük boyuta göre
static uint32_t g_logo_w = 0;
static uint32_t g_logo_h = 0;

// Ekrandaki konumu (sol üst piksel)
static uint32_t g_pos_x = 0;
static uint32_t g_pos_y = 0;

// Zaman sayacı (timer tick ile artacak)
static uint32_t g_time = 0;

// Swirl parametreleri
static const uint8_t  SWIRL_FREQ   = 5;   // açısal frekans benzeri
static const uint8_t  ROT_SPEED    = 3;   // zaman ilerleme hızı
static const uint16_t BASE_BRIGHT  = 180; // taban parlaklık (0..255)
static const uint16_t BRIGHT_RANGE = 70;  // sin dalgası genliği

// ---------------------------------------------------------------------------
// Yardımcı: parlaklık uygulama (0..255 faktör)
// ---------------------------------------------------------------------------
static inline uint32_t apply_brightness(uint32_t color, uint8_t factor)
{
    uint8_t a = (color >> 24) & 0xFF;
    uint8_t r = (color >> 16) & 0xFF;
    uint8_t g = (color >>  8) & 0xFF;
    uint8_t b =  color        & 0xFF;

    // factor 0..255
    r = (uint8_t)((r * factor) / 255);
    g = (uint8_t)((g * factor) / 255);
    b = (uint8_t)((b * factor) / 255);

    return (a << 24) | (r << 16) | (g << 8) | b;
}

// ---------------------------------------------------------------------------
// Logo seçim + konumlandırma
// ---------------------------------------------------------------------------
void logo_animator_init(uint32_t fb_width, uint32_t fb_height)
{
    // 1) Çözünürlüğe göre logo boyutunu seç
    if (fb_width >= 1280 && fb_height >= 720) {
        // Yüksek çözünürlük → 256x256
        g_logo    = ayken_logo256;
        g_logo_w  = 256;
        g_logo_h  = 256;
    } else {
        // Daha düşük çözünürlük → 128x128
        g_logo    = ayken_logo128;
        g_logo_w  = 128;
        g_logo_h  = 128;
    }

    // 2) Ekran üzerinde logoyu merkezle (yukarı doğru biraz offset'li)
    uint32_t center_x = fb_width  / 2;
    uint32_t center_y = fb_height / 3;

    if (center_x >= g_logo_w / 2)
        g_pos_x = center_x - (g_logo_w / 2);
    else
        g_pos_x = 0;

    if (center_y >= g_logo_h / 2)
        g_pos_y = center_y - (g_logo_h / 2);
    else
        g_pos_y = 0;

    g_time = 0;
}

// ---------------------------------------------------------------------------
// Her timer IRQ'da bir kez çağrılacak
// ---------------------------------------------------------------------------
void logo_animator_tick(void)
{
    g_time++;
}

// ---------------------------------------------------------------------------
// Swirl + glow efektli logoyu çiz
// ---------------------------------------------------------------------------
void logo_animator_draw(void)
{
    if (!g_logo || g_logo_w == 0 || g_logo_h == 0)
        return;

    // Logo merkezini kendi koordinat sisteminde al
    int32_t cx = (int32_t)g_logo_w  / 2;
    int32_t cy = (int32_t)g_logo_h / 2;

    // Zaman parametreleri
    uint8_t t_rot   = (uint8_t)(g_time * ROT_SPEED);
    uint8_t t_glow  = (uint8_t)(g_time); // farklı bir faz da kullanılabilir

    for (uint32_t y = 0; y < g_logo_h; ++y) {
        for (uint32_t x = 0; x < g_logo_w; ++x) {
            uint32_t base = g_logo[y][x];

            // Arka plan ise uğraşma (tam siyah piksel)
            if ((base & 0x00FFFFFF) == 0)
                continue;

            int32_t dx = (int32_t)x - cx;
            int32_t dy = (int32_t)y - cy;

            // Swirl için pseudo-açı → dx, dy ve zamanla karıştır
            // (Gerçek atan2 yok; sin_lut ile ilüzyon veriyoruz)
            uint8_t angle_idx  = (uint8_t)((dx * SWIRL_FREQ + dy * 3 + t_rot) & 0xFF);
            uint8_t glow_idx   = (uint8_t)((dy * 2 + t_glow) & 0xFF);

            int16_t swirl_val  = sin_lut[angle_idx];  // -127..127
            int16_t glow_val   = sin_lut[glow_idx];   // -127..127

            // Taban parlaklık + swirl + hafif breathing
            int16_t bright = (int16_t)BASE_BRIGHT
                           + (swirl_val * (int16_t)BRIGHT_RANGE) / 127
                           + (glow_val  * (int16_t)(BRIGHT_RANGE / 2)) / 127;

            if (bright < 0)   bright = 0;
            if (bright > 255) bright = 255;

            uint32_t color = apply_brightness(base, (uint8_t)bright);

            fb_put_pixel(g_pos_x + x, g_pos_y + y, color);
        }
    }
}
