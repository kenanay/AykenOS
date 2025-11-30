#ifndef AYKEN_LOGO_ANIMATOR_H
#define AYKEN_LOGO_ANIMATOR_H

#include <stdint.h>

// Ekran boyutuna göre logo seçimi + merkezde konumlandırma
void logo_animator_init(uint32_t fb_width, uint32_t fb_height);

// Her timer interrupt'ta bir kez çağrılacak (zaman sayaç)
void logo_animator_tick(void);

// Her frame'de (ya timer IRQ içinde ya da kernel loop'unda) çağrılacak
// → swirl + glow efektli logoyu framebuffer'a çizer
void logo_animator_draw(void);

#endif // AYKEN_LOGO_ANIMATOR_H
