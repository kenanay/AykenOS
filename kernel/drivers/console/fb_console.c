// kernel/drivers/console/fb_console.c
#include "fb_console.h"
#include "font8x16.h"
#include <stddef.h>

// Framebuffer bilgileri
static uint8_t  *fb = NULL;
static uint32_t fb_width  = 0;
static uint32_t fb_height = 0;
static uint32_t fb_pitch  = 0;
static uint32_t fb_bpp    = 0;

// Font
#define FONT_W 8
#define FONT_H 16

// Metin bölgesi (mini-log için)
static uint32_t text_cols = 0;
static uint32_t text_rows = 0;
static uint32_t text_origin_x = 0; // karakter cinsinden
static uint32_t text_origin_y = 0; // karakter cinsinden

static uint32_t cursor_x = 0; // text bölgesi içinde
static uint32_t cursor_y = 0;

// Renkler (ARGB)
static const uint32_t COL_BG = 0x00000000; // siyah
static const uint32_t COL_FG = 0x00FFFFFF; // beyaz
static const uint32_t COL_SPLASH_BG = 0x00101020; // koyu lacivertimsı
static const uint32_t COL_BAR_FRAME = 0x00FFFFFF;
static const uint32_t COL_BAR_FILL  = 0x0040A0FF;

// Renk paleti (ANSI benzeri)
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

static const uint32_t color_palette[16] = {
    0x00000000, // Black
    0x00AA0000, // Red
    0x0000AA00, // Green
    0x00AAAA00, // Yellow
    0x000000AA, // Blue
    0x00AA00AA, // Magenta
    0x0000AAAA, // Cyan
    0x00AAAAAA, // White
    0x00555555, // Bright Black (Gray)
    0x00FF5555, // Bright Red
    0x0055FF55, // Bright Green
    0x00FFFF55, // Bright Yellow
    0x005555FF, // Bright Blue
    0x00FF55FF, // Bright Magenta
    0x0055FFFF, // Bright Cyan
    0x00FFFFFF  // Bright White
};

// Aktif renkler
static uint32_t current_fg = 0x00FFFFFF;
static uint32_t current_bg = 0x00000000;
static uint8_t  current_opacity = 255; // 0-255

// -----------------------------------------------------
// Yardımcı: fiziki piksel çiz
// -----------------------------------------------------
void fb_put_pixel(uint32_t x, uint32_t y, uint32_t color)
{
    if (!fb) return;
    if (x >= fb_width || y >= fb_height) return;

    uint32_t *row = (uint32_t*)(fb + (uint64_t)y * fb_pitch);
    row[x] = color;
}

// Dikdörtgen doldur
static void fill_rect(uint32_t x, uint32_t y,
                      uint32_t w, uint32_t h,
                      uint32_t color)
{
    for (uint32_t iy = 0; iy < h; ++iy) {
        if (y + iy >= fb_height) break;
        uint32_t *row = (uint32_t*)(fb + (uint64_t)(y + iy) * fb_pitch);
        for (uint32_t ix = 0; ix < w; ++ix) {
            if (x + ix >= fb_width) break;
            row[x + ix] = color;
        }
    }
}

// Basit çerçeve
static void draw_rect_frame(uint32_t x, uint32_t y,
                            uint32_t w, uint32_t h,
                            uint32_t color)
{
    for (uint32_t ix = 0; ix < w; ++ix) {
        if (x + ix >= fb_width) break;
        if (y < fb_height) fb_put_pixel(x + ix, y, color);
        if (y + h - 1 < fb_height) fb_put_pixel(x + ix, y + h - 1, color);
    }
    for (uint32_t iy = 0; iy < h; ++iy) {
        if (y + iy >= fb_height) break;
        if (x < fb_width) fb_put_pixel(x, y + iy, color);
        if (x + w - 1 < fb_width) fb_put_pixel(x + w - 1, y + iy, color);
    }
}

// -----------------------------------------------------
// Renk yardımcıları
// -----------------------------------------------------
static inline uint32_t apply_opacity(uint32_t color, uint8_t opacity)
{
    if (opacity == 255) return color;
    
    uint8_t r = (color >> 16) & 0xFF;
    uint8_t g = (color >> 8) & 0xFF;
    uint8_t b = color & 0xFF;
    
    r = (r * opacity) / 255;
    g = (g * opacity) / 255;
    b = (b * opacity) / 255;
    
    return (color & 0xFF000000) | (r << 16) | (g << 8) | b;
}

static inline uint32_t blend_colors(uint32_t fg, uint32_t bg, uint8_t alpha)
{
    if (alpha == 255) return fg;
    if (alpha == 0) return bg;
    
    uint8_t fg_r = (fg >> 16) & 0xFF;
    uint8_t fg_g = (fg >> 8) & 0xFF;
    uint8_t fg_b = fg & 0xFF;
    
    uint8_t bg_r = (bg >> 16) & 0xFF;
    uint8_t bg_g = (bg >> 8) & 0xFF;
    uint8_t bg_b = bg & 0xFF;
    
    uint8_t r = ((fg_r * alpha) + (bg_r * (255 - alpha))) / 255;
    uint8_t g = ((fg_g * alpha) + (bg_g * (255 - alpha))) / 255;
    uint8_t b = ((fg_b * alpha) + (bg_b * (255 - alpha))) / 255;
    
    return 0xFF000000 | (r << 16) | (g << 8) | b;
}

// -----------------------------------------------------
// 8x16 karakter çizimi (gerçek font ile + opacity)
// -----------------------------------------------------
static void draw_char_pix(uint32_t px, uint32_t py, char c,
                          uint32_t fg, uint32_t bg)
{
    const uint8_t *glyph = ayken_font8x16[(unsigned char)c];
    
    // Opacity uygula
    fg = apply_opacity(fg, current_opacity);
    bg = apply_opacity(bg, current_opacity);

    for (uint32_t row = 0; row < FONT_H; ++row) {
        uint8_t bits = glyph[row];
        for (uint32_t col = 0; col < FONT_W; ++col) {
            uint32_t mask = 1 << (7 - col);
            uint32_t color = (bits & mask) ? fg : bg;
            fb_put_pixel(px + col, py + row, color);
        }
    }
}

// Text bölgesi içindeki hücreye karakter bas
static void draw_cell(uint32_t cx, uint32_t cy, char c)
{
    uint32_t px = (text_origin_x + cx) * FONT_W;
    uint32_t py = (text_origin_y + cy) * FONT_H;

    draw_char_pix(px, py, c, current_fg, current_bg);
}

// UTF-8 basit decode (sadece Türkçe karakterler için)
static int decode_utf8_char(const char *s, uint32_t *out_char)
{
    unsigned char c = (unsigned char)s[0];
    
    // ASCII
    if (c < 0x80) {
        *out_char = c;
        return 1;
    }
    
    // 2-byte UTF-8 (Türkçe karakterler)
    if ((c & 0xE0) == 0xC0) {
        if (s[1] == 0) return 1; // Hatalı
        
        unsigned char c1 = (unsigned char)s[0];
        unsigned char c2 = (unsigned char)s[1];
        
        // Türkçe karakterler için basit mapping
        // Ç: C3 87 → 195
        if (c1 == 0xC3 && c2 == 0x87) { *out_char = 195; return 2; }
        // ç: C3 A7 → 231
        if (c1 == 0xC3 && c2 == 0xA7) { *out_char = 231; return 2; }
        // Ğ: C4 9E → 196
        if (c1 == 0xC4 && c2 == 0x9E) { *out_char = 196; return 2; }
        // ğ: C4 9F → 240
        if (c1 == 0xC4 && c2 == 0x9F) { *out_char = 240; return 2; }
        // İ: C4 B0 → 197
        if (c1 == 0xC4 && c2 == 0xB0) { *out_char = 197; return 2; }
        // ı: C4 B1 → 253
        if (c1 == 0xC4 && c2 == 0xB1) { *out_char = 253; return 2; }
        // Ö: C3 96 → 214
        if (c1 == 0xC3 && c2 == 0x96) { *out_char = 214; return 2; }
        // ö: C3 B6 → 246
        if (c1 == 0xC3 && c2 == 0xB6) { *out_char = 246; return 2; }
        // Ş: C5 9E → 222
        if (c1 == 0xC5 && c2 == 0x9E) { *out_char = 222; return 2; }
        // ş: C5 9F → 254
        if (c1 == 0xC5 && c2 == 0x9F) { *out_char = 254; return 2; }
        // Ü: C3 9C → 220
        if (c1 == 0xC3 && c2 == 0x9C) { *out_char = 220; return 2; }
        // ü: C3 BC → 252
        if (c1 == 0xC3 && c2 == 0xBC) { *out_char = 252; return 2; }
        
        // Bilinmeyen UTF-8, '?' göster
        *out_char = '?';
        return 2;
    }
    
    // Desteklenmeyen, skip
    *out_char = '?';
    return 1;
}

// -----------------------------------------------------
// Scroll ve text işlemleri
// -----------------------------------------------------
static void scroll_if_needed(void)
{
    if (cursor_y < text_rows)
        return;

    // Text bölgesini bir satır yukarı kaydır (sadece ilgili bölge)
    uint32_t region_px_y = text_origin_y * FONT_H;
    uint32_t region_h_px = text_rows * FONT_H;
    uint32_t line_px_h   = FONT_H;

    // Bölge başlangıcı
    uint8_t *base = fb + (uint64_t)region_px_y * fb_pitch;
    uint64_t bytes_per_line = (uint64_t)fb_pitch * line_px_h;
    uint64_t total_bytes    = (uint64_t)fb_pitch * region_h_px;

    // Yukarı kopyala
    for (uint64_t i = 0; i < total_bytes - bytes_per_line; ++i) {
        base[i] = base[i + bytes_per_line];
    }

    // Son satırı temizle
    uint8_t *last = base + (total_bytes - bytes_per_line);
    for (uint64_t i = 0; i < bytes_per_line; ++i) {
        last[i] = 0;
    }

    if (cursor_y > 0)
        cursor_y--;
}

void fb_clear(void)
{
    if (!fb) return;

    fill_rect(0, 0, fb_width, fb_height, COL_BG);

    cursor_x = cursor_y = 0;
}

// -----------------------------------------------------
// Konsol başlatma
// -----------------------------------------------------
void fb_console_init(ayken_boot_info_t *boot)
{
    // Fiziksel framebuffer adresini higher-half'a map ettiğini varsayıyorum.
    // Daha güzel hal: phys_to_virt(boot->fb_phys_addr)
    fb = (uint8_t*)(boot->fb_phys_addr + 0xFFFFFFFF80000000ULL);

    fb_width  = boot->fb_width;
    fb_height = boot->fb_height;
    fb_pitch  = boot->fb_pitch;
    fb_bpp    = boot->fb_bpp;

    // Tüm ekranı temizle
    fill_rect(0, 0, fb_width, fb_height, COL_BG);

    // Varsayılan text bölgesi: tüm ekran
    text_origin_x = 0;
    text_origin_y = 0;
    text_cols = fb_width  / FONT_W;
    text_rows = fb_height / FONT_H;

    cursor_x = cursor_y = 0;
}

// Mini-log bölgesi için text alanını sağ alt köşeye taşı
void fb_set_text_region(uint32_t cols, uint32_t rows)
{
    if (cols == 0 || rows == 0) return;

    text_cols = cols;
    text_rows = rows;

    // Sağ altta konumlandır (karakter cinsinden)
    text_origin_x = (fb_width  / FONT_W) - cols - 1;
    text_origin_y = (fb_height / FONT_H) - rows - 1;

    cursor_x = cursor_y = 0;

    // Bölgeyi temizle
    uint32_t px = text_origin_x * FONT_W;
    uint32_t py = text_origin_y * FONT_H;
    uint32_t w  = cols * FONT_W;
    uint32_t h  = rows * FONT_H;
    fill_rect(px, py, w, h, COL_BG);
}

// -----------------------------------------------------
// Print fonksiyonları (UTF-8 destekli)
// -----------------------------------------------------
void fb_console_put_char(char c)
{
    if (!fb) return;

    if (c == '\n') {
        cursor_x = 0;
        cursor_y++;
        scroll_if_needed();
        return;
    }
    if (c == '\r') {
        cursor_x = 0;
        return;
    }
    if (c == '\t') {
        // Tab: 4 boşluk
        for (int i = 0; i < 4; i++) {
            draw_cell(cursor_x, cursor_y, ' ');
            cursor_x++;
            if (cursor_x >= text_cols) {
                cursor_x = 0;
                cursor_y++;
                scroll_if_needed();
            }
        }
        return;
    }

    draw_cell(cursor_x, cursor_y, c);

    cursor_x++;
    if (cursor_x >= text_cols) {
        cursor_x = 0;
        cursor_y++;
        scroll_if_needed();
    }
}

void fb_console_print(const char *s)
{
    if (!s || !fb) return;
    
    while (*s) {
        uint32_t ch;
        int bytes = decode_utf8_char(s, &ch);
        
        if (ch == '\n') {
            cursor_x = 0;
            cursor_y++;
            scroll_if_needed();
        } else if (ch == '\r') {
            cursor_x = 0;
        } else if (ch == '\t') {
            for (int i = 0; i < 4; i++) {
                draw_cell(cursor_x, cursor_y, ' ');
                cursor_x++;
                if (cursor_x >= text_cols) {
                    cursor_x = 0;
                    cursor_y++;
                    scroll_if_needed();
                }
            }
        } else {
            draw_cell(cursor_x, cursor_y, (char)ch);
            cursor_x++;
            if (cursor_x >= text_cols) {
                cursor_x = 0;
                cursor_y++;
                scroll_if_needed();
            }
        }
        
        s += bytes;
    }
}

// Eski API uyumluluğu için
void fb_putc(char c)
{
    fb_console_put_char(c);
}

void fb_print(const char *s)
{
    fb_console_print(s);
}

void fb_print_hex(uint64_t v)
{
    fb_print("0x");
    int shift = 60;
    for (; shift >= 0; shift -= 4) {
        uint8_t nibble = (v >> shift) & 0xF;
        char c = (nibble < 10) ? ('0' + nibble) : ('A' + (nibble - 10));
        fb_putc(c);
    }
}

void fb_print_hex32(uint32_t v)
{
    fb_print("0x");
    int shift = 28;
    for (; shift >= 0; shift -= 4) {
        uint8_t nibble = (v >> shift) & 0xF;
        char c = (nibble < 10) ? ('0' + nibble) : ('A' + (nibble - 10));
        fb_putc(c);
    }
}

// -----------------------------------------------------
// Renk kontrol fonksiyonları
// -----------------------------------------------------
void fb_set_color(fb_color_t fg, fb_color_t bg)
{
    if (fg < 16) current_fg = color_palette[fg];
    if (bg < 16) current_bg = color_palette[bg];
}

void fb_set_color_rgb(uint32_t fg_rgb, uint32_t bg_rgb)
{
    current_fg = 0xFF000000 | (fg_rgb & 0x00FFFFFF);
    current_bg = 0xFF000000 | (bg_rgb & 0x00FFFFFF);
}

void fb_set_opacity(uint8_t opacity)
{
    current_opacity = opacity;
}

void fb_reset_colors(void)
{
    current_fg = COL_FG;
    current_bg = COL_BG;
    current_opacity = 255;
}

// -----------------------------------------------------
// Gelişmiş print fonksiyonları
// -----------------------------------------------------
void fb_print_colored(const char *s, fb_color_t color)
{
    uint32_t old_fg = current_fg;
    current_fg = color_palette[color];
    fb_console_print(s);
    current_fg = old_fg;
}

void fb_print_int(int64_t value)
{
    if (value == 0) {
        fb_putc('0');
        return;
    }
    
    if (value < 0) {
        fb_putc('-');
        value = -value;
    }
    
    char buf[32];
    int i = 0;
    
    while (value > 0) {
        buf[i++] = '0' + (value % 10);
        value /= 10;
    }
    
    // Ters çevir
    for (int j = i - 1; j >= 0; j--) {
        fb_putc(buf[j]);
    }
}

void fb_print_uint(uint64_t value)
{
    if (value == 0) {
        fb_putc('0');
        return;
    }
    
    char buf[32];
    int i = 0;
    
    while (value > 0) {
        buf[i++] = '0' + (value % 10);
        value /= 10;
    }
    
    for (int j = i - 1; j >= 0; j--) {
        fb_putc(buf[j]);
    }
}

// -----------------------------------------------------
// Mini-terminal çizimi (splash ekran altında)
// -----------------------------------------------------
void fb_draw_mini_terminal(uint32_t x, uint32_t y, uint32_t cols, uint32_t rows)
{
    uint32_t w = cols * FONT_W + 8;
    uint32_t h = rows * FONT_H + 8;
    
    // Yarı saydam arka plan
    uint32_t term_bg = 0xE0000000; // Yarı saydam siyah
    
    for (uint32_t iy = 0; iy < h; iy++) {
        for (uint32_t ix = 0; ix < w; ix++) {
            if (x + ix >= fb_width || y + iy >= fb_height) continue;
            
            // Mevcut pikseli oku
            uint32_t *pixel = (uint32_t*)(fb + (uint64_t)(y + iy) * fb_pitch) + (x + ix);
            uint32_t bg = *pixel;
            
            // Blend yap
            *pixel = blend_colors(term_bg, bg, 224);
        }
    }
    
    // Çerçeve çiz
    draw_rect_frame(x, y, w, h, 0xFF40A0FF);
    
    // Başlık çubuğu
    fill_rect(x + 1, y + 1, w - 2, FONT_H + 2, 0xFF1A1A2E);
    
    const char *title = " AykenOS Boot Log ";
    uint32_t title_x = x + 4;
    uint32_t title_y = y + 3;
    
    for (uint32_t i = 0; title[i]; i++) {
        draw_char_pix(title_x + i * FONT_W, title_y, title[i],
                     0xFF40A0FF, 0xFF1A1A2E);
    }
}

// -----------------------------------------------------
// Splash ekran + progress bar
// -----------------------------------------------------
void fb_draw_splash_screen(void)
{
    if (!fb) return;

    // Gradient arka plan (üstten alta koyudan açığa)
    for (uint32_t y = 0; y < fb_height; y++) {
        uint8_t intensity = (y * 32) / fb_height;
        uint32_t color = 0xFF000000 | (intensity << 16) | (intensity << 9) | (intensity << 2);
        
        uint32_t *row = (uint32_t*)(fb + (uint64_t)y * fb_pitch);
        for (uint32_t x = 0; x < fb_width; x++) {
            row[x] = color;
        }
    }

    // Logo için yer ayır (ortada, yukarıda)
    // Logo animator burayı kullanacak

    // Başlık yazısı
    const char *title = "AykenOS 0.1-dev";
    uint32_t title_len = 0;
    while (title[title_len]) title_len++;

    uint32_t title_w_px = title_len * FONT_W;
    uint32_t title_x = (fb_width  - title_w_px) / 2;
    uint32_t title_y = fb_height / 3 + 80; // Logo altında

    // Başlık arka planı
    fill_rect(title_x - 12, title_y - 6,
              title_w_px + 24, FONT_H + 12,
              0xC0101828);

    // Başlık çerçevesi
    draw_rect_frame(title_x - 12, title_y - 6,
                   title_w_px + 24, FONT_H + 12,
                   0xFF40A0FF);

    // Başlık yazısı
    for (uint32_t i = 0; i < title_len; ++i) {
        draw_char_pix(title_x + i*FONT_W, title_y, title[i],
                      0xFFFFFFFF, 0xC0101828);
    }

    // Alt yazı
    const char *subtitle = "64-bit Kernel Booting...";
    uint32_t sub_len = 0;
    while (subtitle[sub_len]) sub_len++;
    
    uint32_t sub_x = (fb_width - sub_len * FONT_W) / 2;
    uint32_t sub_y = title_y + FONT_H + 16;
    
    for (uint32_t i = 0; i < sub_len; ++i) {
        draw_char_pix(sub_x + i*FONT_W, sub_y, subtitle[i],
                     0xFFA0C0FF, 0x00000000);
    }

    // Progress bar
    uint32_t bar_w = fb_width / 2;
    uint32_t bar_h = 12;
    uint32_t bar_x = (fb_width  - bar_w) / 2;
    uint32_t bar_y = (fb_height * 2) / 3;

    // Progress bar arka plan
    fill_rect(bar_x - 2, bar_y - 2, bar_w + 4, bar_h + 4, 0xFF1A1A2E);
    
    // Progress bar çerçeve
    draw_rect_frame(bar_x, bar_y, bar_w, bar_h, 0xFF40A0FF);

    // İlk doldurma: %0
    // (Daha sonra update fonksiyonu ile güncellenecek)

    // Mini-terminal bölgesi (sağ altta)
    uint32_t term_cols = 50;
    uint32_t term_rows = 8;
    uint32_t term_w = term_cols * FONT_W + 8;
    uint32_t term_h = term_rows * FONT_H + 8 + FONT_H + 4; // Başlık için ekstra
    uint32_t term_x = fb_width - term_w - 20;
    uint32_t term_y = fb_height - term_h - 20;

    // Mini-terminal çiz
    fb_draw_mini_terminal(term_x, term_y, term_cols, term_rows);

    // Text bölgesini mini-terminal içine ayarla
    text_origin_x = (term_x + 4) / FONT_W;
    text_origin_y = (term_y + FONT_H + 6) / FONT_H;
    text_cols = term_cols;
    text_rows = term_rows;
    cursor_x = cursor_y = 0;
}

// Progress bar güncelleme
void fb_update_progress(uint8_t percent)
{
    if (!fb || percent > 100) return;
    
    uint32_t bar_w = fb_width / 2;
    uint32_t bar_h = 12;
    uint32_t bar_x = (fb_width  - bar_w) / 2;
    uint32_t bar_y = (fb_height * 2) / 3;
    
    // İç kısmı temizle
    fill_rect(bar_x + 1, bar_y + 1, bar_w - 2, bar_h - 2, 0xFF000000);
    
    // Yeni dolguyu çiz
    uint32_t fill_w = ((bar_w - 2) * percent) / 100;
    
    // Gradient fill
    for (uint32_t x = 0; x < fill_w; x++) {
        uint8_t intensity = 64 + ((x * 191) / fill_w);
        uint32_t color = 0xFF000000 | (intensity / 4) | (intensity << 7) | (intensity << 16);
        
        for (uint32_t y = 0; y < bar_h - 2; y++) {
            fb_put_pixel(bar_x + 1 + x, bar_y + 1 + y, color);
        }
    }
    
    // Yüzde yazısı
    char percent_str[8];
    percent_str[0] = '0' + (percent / 10);
    percent_str[1] = '0' + (percent % 10);
    percent_str[2] = '%';
    percent_str[3] = '\0';
    
    uint32_t pct_x = bar_x + bar_w / 2 - 12;
    uint32_t pct_y = bar_y + bar_h + 8;
    
    for (int i = 0; percent_str[i]; i++) {
        draw_char_pix(pct_x + i * FONT_W, pct_y, percent_str[i],
                     0xFFFFFFFF, 0x00000000);
    }
}
