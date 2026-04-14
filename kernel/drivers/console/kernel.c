#include <boot_info.h>
#include "fb_console.h"
#include "keyboard.h"

// simple_shell.c içinde tanımlı fonksiyon
void simple_shell_loop(void);

// GDT ve IDT kurulum fonksiyonları (henüz oluşturulmadıysa yorum satırı yapın)
// void gdt_init(void);
// void idt_init(void);

void kernel_main(ayken_boot_info_t *boot_info) {
    // EARLIEST KERNEL ENTRY MARKER - Boot observability proof
    __asm__ volatile("outb %b0, %w1" : : "a"((uint8_t)'K'), "Nd"((uint16_t)0xE9));
    __asm__ volatile("outb %b0, %w1" : : "a"((uint8_t)'0'), "Nd"((uint16_t)0xE9));
    __asm__ volatile("outb %b0, %w1" : : "a"((uint8_t)'\n'), "Nd"((uint16_t)0xE9));
    
    // Serial output for redundancy
    __asm__ volatile("outb %b0, %w1" : : "a"((uint8_t)'K'), "Nd"((uint16_t)0x3F8));
    __asm__ volatile("outb %b0, %w1" : : "a"((uint8_t)'0'), "Nd"((uint16_t)0x3F8));
    __asm__ volatile("outb %b0, %w1" : : "a"((uint8_t)'\n'), "Nd"((uint16_t)0x3F8));
    
    // 1. Framebuffer konsolu başlat
    fb_console_init(boot_info);

    // 2. Splash ekranı ve görsel arayüz
    fb_draw_splash_screen();
    
    fb_console_print("\n[KERNEL] AykenOS v0.1 baslatiliyor...\n");
    fb_update_progress(10);

    // 3. Temel sistem bileşenlerini başlat
    // (GDT ve IDT burada başlatılmalı)
    // gdt_init();
    // idt_init();
    // fb_console_print("[KERNEL] GDT ve IDT yuklendi.\n");
    
    // 4. Klavye sürücüsünü başlat
    keyboard_init();
    fb_console_print("[KERNEL] Klavye surucusu baslatildi.\n");
    fb_update_progress(30);

    // Kesmeleri aktif et (Klavye için gerekli)
    // __asm__ volatile("sti");
    // fb_console_print("[KERNEL] Kesmeler aktif edildi.\n");

    fb_update_progress(100);
    fb_console_print("[KERNEL] Shell baslatiliyor...\n");

    // 5. Shell döngüsüne gir
    simple_shell_loop();

    // Shell'den çıkılırsa sistemi durdur
    fb_console_print("\n[KERNEL] Sistem durduruldu.\n");
    while (1) {
        __asm__ volatile("hlt");
    }
}
