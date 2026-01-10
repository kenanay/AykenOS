#ifndef AYKEN_KEYBOARD_H
#define AYKEN_KEYBOARD_H

#include <stdint.h>

// Klavye sürücüsünü başlatır (değişkenleri sıfırlar)
void keyboard_init(void);

// Buffer'dan bir karakter okur (bloklamasız/non-blocking).
// Eğer buffer boşsa 0 döner.
char keyboard_read_char(void);

// IRQ1 Handler (Interrupt Service Routine) tarafından çağrılacak fonksiyon
void keyboard_handler(void);

#endif