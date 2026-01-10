# AykenOS Framebuffer Console - Kullanım Kılavuzu

## ✨ Özellikler

### ✅ Tamamlanan Özellikler

1. **fb_console_put_char()** - Tek karakter yazdırma (UTF-8 destekli)
2. **fb_console_print()** - String yazdırma (UTF-8 destekli)
3. **Mini-terminal** - Splash ekran altında şık log penceresi
4. **Türkçe karakter desteği** - Ç, Ğ, İ, Ö, Ş, Ü ve küçük harfleri
5. **Renklendirme + Opacity** - 16 renk paleti + RGB + şeffaflık

## 📝 Temel Kullanım

### Başlatma

```c
#include "drivers/console/fb_console.h"

void kernel_main(ayken_boot_info_t *boot_info) {
    // Framebuffer console'u başlat
    fb_console_init(boot_info);
    
    // Splash ekranı çiz (logo + progress bar + mini-terminal)
    fb_draw_splash_screen();
    
    // Artık yazdırabilirsiniz!
    fb_console_print("AykenOS başlatılıyor...\n");
}
```

### Basit Yazdırma

```c
// Tek karakter
fb_console_put_char('A');
fb_console_put_char('\n');

// String yazdırma
fb_console_print("Merhaba Dünya!\n");

// Türkçe karakterler
fb_console_print("Türkçe: ÇçĞğİıÖöŞşÜü\n");

// Sayı yazdırma
fb_print_int(-12345);
fb_console_print("\n");

fb_print_uint(67890);
fb_console_print("\n");

fb_print_hex(0xDEADBEEF);
fb_console_print("\n");
```

## 🎨 Renklendirme

### Renk Paleti Kullanımı

```c
// Renkleri ayarla (ön plan, arka plan)
fb_set_color(FB_COLOR_BRIGHT_GREEN, FB_COLOR_BLACK);
fb_console_print("Yeşil yazı!\n");

// Sadece bir string için renk
fb_print_colored("Kırmızı uyarı!\n", FB_COLOR_BRIGHT_RED);

// Renkleri sıfırla
fb_reset_colors();
```

### RGB Renkleri

```c
// Özel RGB renkleri (0xRRGGBB formatında)
fb_set_color_rgb(0xFF5500, 0x000000); // Turuncu yazı
fb_console_print("Özel renk!\n");
```

### Opacity (Şeffaflık)

```c
// Şeffaflık ayarla (0-255)
fb_set_opacity(128); // %50 şeffaf
fb_console_print("Yarı saydam yazı\n");

fb_set_opacity(255); // Tam opak
```

## 🖼️ Splash Ekran ve Progress Bar

### Splash Ekran

```c
// Tam splash ekranı çiz
fb_draw_splash_screen();

// Bu fonksiyon şunları yapar:
// - Gradient arka plan
// - Logo için yer ayırır (ortada)
// - Başlık yazısı
// - Progress bar
// - Mini-terminal (sağ altta)
```

### Progress Bar Güncelleme

```c
// Boot aşamalarında progress bar'ı güncelle
fb_update_progress(0);   // %0
fb_console_print("[BOOT] Kernel yükleniyor...\n");

fb_update_progress(25);  // %25
fb_console_print("[BOOT] Bellek başlatılıyor...\n");

fb_update_progress(50);  // %50
fb_console_print("[BOOT] Sürücüler yükleniyor...\n");

fb_update_progress(75);  // %75
fb_console_print("[BOOT] Sistem servisleri...\n");

fb_update_progress(100); // %100
fb_console_print("[BOOT] Hazır!\n");
```

## 🎯 Mini-Terminal

Mini-terminal otomatik olarak `fb_draw_splash_screen()` ile oluşturulur.

### Özellikler:
- Sağ alt köşede konumlanır
- Yarı saydam arka plan
- Otomatik scroll
- 50 sütun x 8 satır
- Başlık çubuğu: "AykenOS Boot Log"

### Manuel Ayarlama

```c
// Farklı boyutta text bölgesi
fb_set_text_region(80, 25); // 80x25 karakterlik alan
```

## 🌈 Renk Paleti

```
FB_COLOR_BLACK           - Siyah
FB_COLOR_RED             - Kırmızı
FB_COLOR_GREEN           - Yeşil
FB_COLOR_YELLOW          - Sarı
FB_COLOR_BLUE            - Mavi
FB_COLOR_MAGENTA         - Magenta
FB_COLOR_CYAN            - Cyan
FB_COLOR_WHITE           - Beyaz
FB_COLOR_BRIGHT_BLACK    - Gri
FB_COLOR_BRIGHT_RED      - Parlak Kırmızı
FB_COLOR_BRIGHT_GREEN    - Parlak Yeşil
FB_COLOR_BRIGHT_YELLOW   - Parlak Sarı
FB_COLOR_BRIGHT_BLUE     - Parlak Mavi
FB_COLOR_BRIGHT_MAGENTA  - Parlak Magenta
FB_COLOR_BRIGHT_CYAN     - Parlak Cyan
FB_COLOR_BRIGHT_WHITE    - Parlak Beyaz
```

## 💡 Örnek Boot Sequence

```c
void boot_sequence(ayken_boot_info_t *boot_info) {
    // 1. Console'u başlat
    fb_console_init(boot_info);
    
    // 2. Splash ekranı göster
    fb_draw_splash_screen();
    
    // 3. Boot mesajları
    fb_set_color(FB_COLOR_BRIGHT_CYAN, FB_COLOR_BLACK);
    fb_console_print("[OK] ");
    fb_reset_colors();
    fb_console_print("Framebuffer başlatıldı\n");
    
    fb_update_progress(10);
    
    // 4. Bellek kontrolü
    fb_set_color(FB_COLOR_BRIGHT_CYAN, FB_COLOR_BLACK);
    fb_console_print("[OK] ");
    fb_reset_colors();
    fb_console_print("Bellek: ");
    fb_print_uint(boot_info->mem_size / 1024 / 1024);
    fb_console_print(" MB\n");
    
    fb_update_progress(25);
    
    // 5. CPU bilgisi
    fb_set_color(FB_COLOR_BRIGHT_CYAN, FB_COLOR_BLACK);
    fb_console_print("[OK] ");
    fb_reset_colors();
    fb_console_print("CPU başlatıldı\n");
    
    fb_update_progress(40);
    
    // 6. Türkçe mesaj
    fb_set_color(FB_COLOR_BRIGHT_GREEN, FB_COLOR_BLACK);
    fb_console_print("Hoş geldiniz! ");
    fb_reset_colors();
    fb_console_print("AykenOS çalışıyor 🚀\n");
    
    fb_update_progress(100);
}
```

## 🔧 Gelişmiş Özellikler

### Özel Terminal Penceresi

```c
// Ekranın farklı bir yerinde terminal
uint32_t x = 100;
uint32_t y = 100;
uint32_t cols = 60;
uint32_t rows = 10;

fb_draw_mini_terminal(x, y, cols, rows);
```

### Piksel Seviyesi Çizim

```c
// Logo animator veya özel grafikler için
fb_put_pixel(x, y, 0xFFFF0000); // Kırmızı piksel
```

### Ekranı Temizleme

```c
fb_clear(); // Tüm ekranı temizle
```

## 📊 Performans Notları

- UTF-8 decode sadece Türkçe karakterler için optimize edilmiş
- Scroll işlemi sadece text bölgesini etkiler
- Opacity hesaplamaları inline optimize edilmiş
- Gradient ve blend işlemleri donanım hızlandırması olmadan çalışır

## 🐛 Bilinen Sınırlamalar

1. UTF-8 desteği sadece Türkçe karakterler için (2-byte)
2. 3-byte ve 4-byte UTF-8 karakterler desteklenmiyor
3. Font boyutu sabit: 8x16 piksel
4. Maksimum çözünürlük: Framebuffer boyutuna bağlı

## 🎓 İpuçları

1. **Renkli log seviyeleri kullanın:**
   - INFO: Cyan
   - WARNING: Yellow
   - ERROR: Red
   - SUCCESS: Green

2. **Progress bar'ı düzenli güncelleyin:**
   - Her önemli boot aşamasında
   - Kullanıcı deneyimi için önemli

3. **Mini-terminal boyutunu ayarlayın:**
   - Çözünürlüğe göre optimize edin
   - Çok küçük: okunaksız
   - Çok büyük: splash'i kapatır

4. **Opacity'yi dikkatli kullanın:**
   - Performans etkisi var
   - Okunabilirliği düşürebilir
