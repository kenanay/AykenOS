#include "keyboard.h"

// Port I/O işlemleri için basit inline fonksiyon (kernel/io.h yoksa diye)
static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    __asm__ volatile ( "inb %1, %0" : "=a"(ret) : "Nd"(port) );
    return ret;
}

#define KEYBOARD_DATA_PORT 0x60
#define KEYBOARD_STATUS_PORT 0x64

// Buffer boyutu (256 karakterlik geçmiş tutar)
#define KB_BUFFER_SIZE 256

static char kb_buffer[KB_BUFFER_SIZE];
static volatile uint32_t kb_write_idx = 0;
static volatile uint32_t kb_read_idx = 0;

// Tuş durumları
static int shift_pressed = 0;
static int caps_lock = 0;

// Scancode Set 1 -> ASCII/AykenTR Haritası (TR QWERTY Layout)
// Küçük harfler ve numpad
static const char scancode_ascii_lower[128] = {
    0,  27, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\b', // 0x00 - 0x0E
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 253, 'o', 'p', 240, 252, '\n', // 0x0F - 0x1C (i->ı, [->ğ, ]->ü)
    0, 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 254, 'i', ',',          // 0x1D - 0x29 (;->ş, '->i, `->,)
    0, '<', 'z', 'x', 'c', 'v', 'b', 'n', 'm', 246, 231, '.', 0,            // 0x2A - 0x36 (\-><, ,->ö, .->ç, /->.)
    '*', 0, ' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,                              // 0x37 - 0x44
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '-', 0, 0, 0, '+', 0,                     // 0x45 - 0x53
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0                                      // 0x54 - 0x5F
};

// Büyük harfler ve Shift ile erişilen semboller
static const char scancode_ascii_upper[128] = {
    0,  27, '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\b',
    '\t', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 196, 220, '\n', // (i->I, [->Ğ, ]->Ü)
    0, 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 222, 197, ';',          // (;->Ş, '->İ, `->;)
    0, '>', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', 214, 195, ':', 0,            // (\->>, ,->Ö, .->Ç, /->:)
    '*', 0, ' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '-', 0, 0, 0, '+', 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

void keyboard_init(void) {
    kb_write_idx = 0;
    kb_read_idx = 0;
    shift_pressed = 0;
    caps_lock = 0;
    
    // NOT: Bu fonksiyon çağrıldıktan sonra IDT (Interrupt Descriptor Table)
    // üzerinde IRQ1 (INT 33) için keyboard_handler kaydedilmelidir.
}

// Buffer'a karakter ekle (Circular Buffer mantığı)
static void buffer_write(char c) {
    // Buffer dolu mu kontrol et (write+1 == read ise doludur)
    if (((kb_write_idx + 1) % KB_BUFFER_SIZE) != kb_read_idx) {
        kb_buffer[kb_write_idx] = c;
        kb_write_idx = (kb_write_idx + 1) % KB_BUFFER_SIZE;
    }
}

char keyboard_read_char(void) {
    // Buffer boş mu?
    if (kb_read_idx == kb_write_idx) {
        return 0; 
    }
    
    char c = kb_buffer[kb_read_idx];
    kb_read_idx = (kb_read_idx + 1) % KB_BUFFER_SIZE;
    return c;
}

// Interrupt Handler
void keyboard_handler(void) {
    // Port 0x60'dan scancode oku
    uint8_t scancode = inb(KEYBOARD_DATA_PORT);

    // Break code (tuş bırakıldı) kontrolü: En yüksek bit 1 ise (0x80)
    if (scancode & 0x80) {
        uint8_t released = scancode & 0x7F;
        if (released == 0x2A || released == 0x36) { // Sol veya Sağ Shift bırakıldı
            shift_pressed = 0;
        }
    } else {
        // Make code (tuş basıldı)
        if (scancode == 0x2A || scancode == 0x36) { // Sol veya Sağ Shift basıldı
            shift_pressed = 1;
        } else if (scancode == 0x3A) { // Caps Lock
            caps_lock = !caps_lock;
        } else {
            // Karakteri ASCII'ye çevir
            char c = 0;
            
            // Scancode geçerli aralıkta mı?
            if (scancode < 128) {
                // Harf mi? (Basit aralık kontrolü)
                int is_letter = (scancode >= 0x10 && scancode <= 0x19) || // q-p
                                (scancode >= 0x1E && scancode <= 0x26) || // a-l
                                (scancode >= 0x2C && scancode <= 0x32);   // z-m
                
                if (is_letter) {
                    // Harflerde Shift XOR CapsLock mantığı
                    // (Shift basılıyken CapsLock açıksa küçük yazar)
                    if (shift_pressed ^ caps_lock) {
                        c = scancode_ascii_upper[scancode];
                    } else {
                        c = scancode_ascii_lower[scancode];
                    }
                } else {
                    // Harf değilse (sayı, sembol) sadece Shift etkiler
                    // CapsLock sayıları etkilemez
                    if (shift_pressed) {
                        c = scancode_ascii_upper[scancode];
                    } else {
                        c = scancode_ascii_lower[scancode];
                    }
                }
            }

            // Geçerli bir karakterse buffer'a ekle
            if (c != 0) {
                buffer_write(c);
                
                // İsteğe bağlı: Ekrana hemen bas (Echo)
                // fb_console_put_char(c); 
            }
        }
    }
    
    // NOT: PIC (Programmable Interrupt Controller) kullanılıyorsa,
    // Assembly tarafındaki ISR wrapper'ı EOI (End of Interrupt) sinyalini
    // göndermelidir (outb(0x20, 0x20)). Bu C fonksiyonu sadece mantığı işler.
}