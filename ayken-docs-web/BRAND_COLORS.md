# AykenOS Resmi Renk Paleti

**Oluşturan:** Kenan AY  
**Tarih:** 31 Ocak 2026  
**Versiyon:** 1.0  
**Durum:** Resmi - Kilitli

---

## 🎯 Renk Stratejisi

### Ana Karar: İki Renk Sistemi
- ✅ **İki renk** → DOĞRU
- ❌ **Tek renk** → HATA (derinlik kaybı, "OS" hissi zayıflar)

### Renk Felsefesi
- **Ana marka rengi**: Determinizm, güven, sistem kararlılığı
- **İkincil renk**: Kernel, donanım, dayanıklılık
- **Zemin**: Premium, teknik, göz dostu

---

## 🎨 Resmi Renk Paleti

### 🔵 1️⃣ Ana Marka Rengi - Ayken Blue
```css
--ayken-blue: #00B3FF;           /* Primary Brand Color - Electric Cyan */
--ayken-blue-dark: #0096D6;      /* Darker variant */
```

**Anlamı:**
- Determinizm
- Güven
- Sistem kararlılığı
- Mühendislik disiplini

**Kullanım Alanları:**
- Logolar
- Ana butonlar
- Linkler
- Vurgu renkleri
- Hover efektleri

### ⚙️ 2️⃣ İkincil Renk - Graphite Silver
```css
--graphite-silver: #8A8F98;      /* Secondary System Color */
--graphite-silver-light: #B0B6BF; /* Lighter variant */
```

**Anlamı:**
- Kernel
- Donanım
- Dayanıklılık
- "Bu sistem çökmek için yapılmadı"

**Kullanım Alanları:**
- İkonlar
- İkincil metinler
- Çizgiler
- Sistem vurguları

### 🏢 3️⃣ Ana Zemin - Ayken White
```css
--ayken-white: #F5F7FA;          /* Primary Background - Ayken White */
```

**Neden #FFFFFF değil:**
- Saf beyaz logo ile uyumsuz
- Göz yorar
- Teknik/premium hissi düşürür
- Metalik logo saf beyazda ucuz durur

**Ayken White Avantajları:**
- Soğuk beyaz (maviye yakın)
- Teknik, temiz, mühendislik hissi
- Logo içindeki Ayken Blue parlamasını güçlendirir
- Apple/NVIDIA/modern OS çizgisi

### 📝 4️⃣ Metin Renkleri
```css
--charcoal: #1F2933;             /* Primary text */
--ice-white: #E6EAF0;            /* Light text for dark themes */
```

### 📏 5️⃣ Yardımcı Renkler
```css
--soft-gray: #E1E5EA;            /* Dividers/Lines */
```

### 💫 6️⃣ Gölge Efektleri
```css
--shadow-light: 0 2px 4px rgba(0,0,0,0.06);        /* Çok hafif */
--shadow-medium: 0 4px 8px rgba(0,179,255,0.15);   /* Ayken Blue tonlu */
--shadow-heavy: 0 8px 16px rgba(0,179,255,0.2);    /* Vurgu için */
```

---

## 🌐 Web/UI Kullanım Rehberi

### Katman Yapısı
```css
/* Ana zemin */
background: #F5F7FA;

/* Kart arka planları */
background: #FFFFFF;

/* Çizgiler/Kenarlıklar */
border: #E1E5EA;

/* Hover/Aktif vurgular */
color: #00B3FF;
```

### Logo Kullanımı
- ✅ **Doğru**: Logo orijinal renkleriyle
- ✅ **Clear space**: Etrafında boşluk bırak
- ✅ **Gölge**: Çok hafif `rgba(0,0,0,0.06)`
- ❌ **Yanlış**: Logo siyaha çevirme
- ❌ **Yanlış**: Mavi tonu değiştirme
- ❌ **Yanlış**: Beyaz zemin üstünde ekstra gradient

---

## 🌙 Karanlık Tema

```css
/* Dark Theme Colors */
--dark-primary: #E6EAF0;          /* Ice White */
--dark-secondary: #00B3FF;        /* Ayken Blue korunur */
--dark-text: #E6EAF0;             /* Ice White */
--dark-text-light: #B0B6BF;       /* Graphite Silver Light */
--dark-bg: #1a1a1a;               /* Ana zemin */
--dark-bg-light: #2c2c2c;         /* Kart arka planları */
--dark-bg-dark: #0f0f0f;          /* Derin zemin */
--dark-border: #8A8F98;           /* Graphite Silver */
```

---

## 🚫 Kaçınılması Gerekenler

### Yasaklı Renkler
- ❌ **Kırmızı** → agresif/hata rengi
- ❌ **Yeşil** → "tool/utility" hissi
- ❌ **Mor** → AI startup klişesi
- ❌ **Çoklu gradient** → kurumsallık düşer

### Gradient Kuralları
Eğer gradient kullanılacaksa:
- Sadece Ayken Blue tonları arasında
- Linear, yumuşak geçişler
- Aşırıya kaçmamak

---

## 🧠 AykenOS Vizyonu ile Renk Uyumu

| Vizyon | Renk Karşılığı |
|--------|----------------|
| Principle-Driven OS | Mavi = kural/düzen |
| Deterministic Execution | Sabit, soğuk tonlar |
| Invariant Gate | Metalik/gri |
| AI-native | Cyan parıltısı |

---

## 📊 Tam CSS Değişken Listesi

```css
:root {
    /* AykenOS Official Brand Colors */
    --ayken-blue: #00B3FF;           /* Primary Brand Color - Electric Cyan */
    --ayken-blue-dark: #0096D6;      /* Darker variant */
    --graphite-silver: #8A8F98;      /* Secondary System Color */
    --graphite-silver-light: #B0B6BF; /* Lighter variant */
    --ayken-white: #F5F7FA;          /* Primary Background - Ayken White */
    --ice-white: #E6EAF0;            /* Light text for dark themes */
    --charcoal: #1F2933;             /* Primary text */
    --soft-gray: #E1E5EA;            /* Dividers/Lines */
    
    /* Main colors using brand palette */
    --primary-color: var(--charcoal);
    --secondary-color: var(--ayken-blue);
    --text-color: var(--charcoal);
    --text-light: var(--graphite-silver);
    --bg-color: var(--ayken-white);
    --bg-light: #FFFFFF;             /* Card backgrounds */
    --border-color: var(--soft-gray);
    --shadow-light: 0 2px 4px rgba(0,0,0,0.06);
    --shadow-medium: 0 4px 8px rgba(0,179,255,0.15);
    --shadow-heavy: 0 8px 16px rgba(0,179,255,0.2);
}
```

---

## 🔒 Versiyon Kontrolü

### v1.0 (31 Ocak 2026)
- İlk resmi renk paleti
- Ayken White (#F5F7FA) kararı
- İki renk sistemi onayı
- Logo uyum optimizasyonu

### Gelecek Güncellemeler
Bu renk paleti 5-10 yıl boyunca sabit kalacak şekilde tasarlandı. Değişiklik yapılması durumunda bu dosya güncellenecek ve versiyon numarası artırılacak.

---

## 📞 İletişim

**Renk Paleti Sorumlusu:** Kenan AY  
**Proje:** AykenOS  
**Konum:** Kütahya, Türkiye

---

**© 2026 AykenOS - Resmi Marka Renk Paleti**