#ifndef AYKEN_FB_CONSOLE_H
#define AYKEN_FB_CONSOLE_H

#include <stdint.h>
#include "../../include/boot_info.h"

// Renk paleti
typedef enum {
    FB_COLOR_BLACK = 0,
    FB_COLOR_RED,
    FB_COLOR_GREEN,
    FB_COLOR_YELLOW,
    FB_COLOR_BLUE,
    FB_COLOR_MAGENTA,
    FB_COLOR_CYAN,
    FB_COLOR_WHITE,
    FB_COLOR_BRIGHT_BLACK,
    FB_COLOR_BRIGHT_RED,
    FB_COLOR_BRIGHT_GREEN,
    FB_COLOR_BRIGHT_YELLOW,
    FB_COLOR_BRIGHT_BLUE,
    FB_COLOR_BRIGHT_MAGENTA,
    FB_COLOR_BRIGHT_CYAN,
    FB_COLOR_BRIGHT_WHITE
} fb_color_t;

// Temel fonksiyonlar
void fb_console_init(ayken_boot_info_t *boot);
void fb_set_text_region(uint32_t cols, uint32_t rows);
void fb_clear(void);

// Piksel çizim fonksiyonu (logo_animator için gerekli)
void fb_put_pixel(uint32_t x, uint32_t y, uint32_t color);

// Karakter ve string yazdırma (UTF-8 destekli)
void fb_console_put_char(char c);
void fb_console_print(const char *s);

// Eski API uyumluluğu
void fb_putc(char c);
void fb_print(const char *s);

// Sayı yazdırma
void fb_print_int(int64_t value);
void fb_print_uint(uint64_t value);
void fb_print_hex(uint64_t v);
void fb_print_hex32(uint32_t v);

// Renk kontrolü
void fb_set_color(fb_color_t fg, fb_color_t bg);
void fb_set_color_rgb(uint32_t fg_rgb, uint32_t bg_rgb);
void fb_set_opacity(uint8_t opacity);
void fb_reset_colors(void);
void fb_print_colored(const char *s, fb_color_t color);

// Splash ekranı ve mini-terminal
void fb_draw_splash_screen(void);
void fb_draw_mini_terminal(uint32_t x, uint32_t y, uint32_t cols, uint32_t rows);
void fb_update_progress(uint8_t percent);

#endif
