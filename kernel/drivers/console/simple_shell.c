#include <stdint.h>
#include "fb_console.h"
#include "keyboard.h"
#include "../../abdf_format.h"

// Port I/O işlemleri (reboot için gerekli)
static inline void outb(uint16_t port, uint8_t val) {
    __asm__ volatile ( "outb %0, %1" : : "a"(val), "Nd"(port) );
}

static inline void outw(uint16_t port, uint16_t val) {
    __asm__ volatile ( "outw %0, %1" : : "a"(val), "Nd"(port) );
}

// Libc olmadığı için basit string karşılaştırma fonksiyonu
static int strcmp(const char *s1, const char *s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(const unsigned char*)s1 - *(const unsigned char*)s2;
}

void simple_shell_loop(void) {
    char cmd_buffer[256] = {0};
    int cmd_pos = 0;

    fb_console_print("\n========================================\n");
    fb_console_print(" AykenOS Klavye Test Kabugu (v0.1)\n");
    fb_console_print("========================================\n");
    fb_console_print("AykenOS> ");

    while (1) {
        // Non-blocking okuma yapıyoruz
        char c = keyboard_read_char();

        if (c != 0) {
            // Enter tuşu (Komut sonu)
            if (c == '\n' || c == '\r') {
                fb_console_put_char('\n');
                cmd_buffer[cmd_pos] = '\0'; // String'i sonlandır

                // Komut işleme
                if (cmd_pos > 0) {
                    if (strcmp(cmd_buffer, "exit") == 0) {
                        fb_console_print("Kabuktan cikiliyor...\n");
                        break;
                    } else if (strcmp(cmd_buffer, "help") == 0) {
                        fb_console_print("Mevcut komutlar: help, exit, clear, hello, reboot, shutdown, abdf_check\n");
                    } else if (strcmp(cmd_buffer, "clear") == 0) {
                        fb_clear();
                        // fb_clear cursor'ı başa alır, prompt'u tekrar basmaya gerek yok
                        // çünkü döngü sonunda basılacak. Ancak temiz ekran için:
                        cmd_pos = 0;
                        fb_console_print("AykenOS> ");
                        continue;
                    } else if (strcmp(cmd_buffer, "hello") == 0) {
                        fb_console_print("Merhaba! AykenOS calisiyor.\n");
                    } else if (strcmp(cmd_buffer, "reboot") == 0) {
                        fb_console_print("Sistem yeniden baslatiliyor...\n");
                        // 8042 Klavye Denetleyicisi üzerinden CPU reset (Pulse Reset Line)
                        outb(0x64, 0xFE);
                        __asm__ volatile("hlt");
                    } else if (strcmp(cmd_buffer, "shutdown") == 0) {
                        fb_console_print("Sistem kapatiliyor (QEMU)...\n");
                        // QEMU Shutdown (0x604 portuna 0x2000 yazarak)
                        outw(0x604, 0x2000);
                        // Bochs/Older QEMU Shutdown
                        outw(0xB004, 0x2000);
                        __asm__ volatile("hlt");
                    } else if (strcmp(cmd_buffer, "abdf_check") == 0) {
                        // Faz 1 Entegrasyon Testi:
                        // Rust ve C veri yapılarının boyutu eşleşiyor mu?
                        fb_console_print("ABDF Header Kontrolu:\n");
                        fb_console_print("Beklenen Boyut (Rust): 12 byte\n");
                        fb_console_print("C Struct Boyutu: ");
                        fb_print_uint(sizeof(abdf_header_t));
                        fb_console_print(" byte\n");
                        
                        if (sizeof(abdf_header_t) == 12) fb_console_print("[OK] Veri yapilari uyumlu.\n");
                        else fb_console_print("[FAIL] Hizalama hatasi var!\n");
                    } else {
                        fb_console_print("Bilinmeyen komut: ");
                        fb_console_print(cmd_buffer);
                        fb_console_print("\n");
                    }
                }

                // Yeni komut için hazırlık
                cmd_pos = 0;
                fb_console_print("AykenOS> ");
            }
            // Backspace (Silme)
            else if (c == '\b') {
                if (cmd_pos > 0) {
                    cmd_pos--;
                    fb_console_put_char('\b');
                }
            }
            // Normal karakterler
            else {
                if (cmd_pos < 255) {
                    cmd_buffer[cmd_pos++] = c;
                    fb_console_put_char(c);
                }
            }
        }

        // CPU'yu biraz dinlendir (Interrupt gelene kadar bekle)
        __asm__ volatile("hlt");
    }
}
