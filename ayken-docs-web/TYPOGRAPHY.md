# AykenOS Tipografi ve Font Sistemi

**Oluşturan:** Kenan AY  
**Tarih:** 31 Ocak 2026  
**Versiyon:** 1.0  
**Durum:** Resmi - Kilitli

---

## 🎯 Tipografi Stratejisi

### AykenOS Font Felsefesi
- **Teknik ve profesyonel** görünüm
- **Okunabilirlik** her şeyden önce
- **Cross-platform uyumluluk**
- **Modern ve temiz** tasarım
- **Kod ve dokümantasyon** ayrımı

### Hedef Algı
- Mühendislik disiplini
- Sistem güvenilirliği
- Enterprise-grade kalite
- Developer-friendly

---

## 📝 Ana Font Sistemi

### 🏢 Primary Font Stack - Interface
```css
--font-primary: 'Segoe UI', system-ui, -apple-system, 'Helvetica Neue', Arial, sans-serif;
```

**Seçim Gerekçesi:**
- **Segoe UI**: Windows'ta mükemmel rendering
- **system-ui**: Her platformda native font
- **-apple-system**: macOS/iOS optimizasyonu
- **Helvetica Neue**: macOS fallback
- **Arial**: Universal fallback

**Kullanım Alanları:**
- Web sitesi ana metinleri
- UI elementleri
- Butonlar ve navigasyon
- Genel arayüz

### 💻 Monospace Font Stack - Code
```css
--font-mono: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', 'Consolas', 'Courier New', monospace;
```

**Seçim Gerekçesi:**
- **SF Mono**: Apple'ın developer font'u
- **Monaco**: macOS klasik monospace
- **Inconsolata**: Açık kaynak, okunabilir
- **Roboto Mono**: Google'ın modern monospace
- **Consolas**: Windows developer standartı
- **Courier New**: Universal fallback

**Kullanım Alanları:**
- Kod blokları
- Terminal çıktıları
- API referansları
- Teknik dokümantasyon
- Inline kod

---

## 📏 Font Boyutları

### Responsive Font Scale
```css
/* Base font size */
html { font-size: 16px; }

/* Font size variables */
--font-size-xs: 0.75rem;    /* 12px */
--font-size-sm: 0.875rem;   /* 14px */
--font-size-base: 1rem;     /* 16px */
--font-size-lg: 1.125rem;   /* 18px */
--font-size-xl: 1.25rem;    /* 20px */
--font-size-2xl: 1.5rem;    /* 24px */
--font-size-3xl: 1.875rem;  /* 30px */
--font-size-4xl: 2.25rem;   /* 36px */
--font-size-5xl: 3rem;      /* 48px */
```

### Kullanım Rehberi

| Boyut | Kullanım Alanı | Örnek |
|-------|----------------|-------|
| `xs` | Küçük notlar, metadata | Copyright, timestamps |
| `sm` | İkincil metinler | Açıklamalar, alt başlıklar |
| `base` | Ana metin | Paragraflar, liste öğeleri |
| `lg` | Vurgu metinleri | Önemli açıklamalar |
| `xl` | Küçük başlıklar | H4, H5 |
| `2xl` | Orta başlıklar | H3 |
| `3xl` | Büyük başlıklar | H2 |
| `4xl` | Ana başlıklar | H1 |
| `5xl` | Hero başlıklar | Landing page |

---

## ⚖️ Font Ağırlıkları

### Font Weight Scale
```css
--font-weight-light: 300;
--font-weight-normal: 400;
--font-weight-medium: 500;
--font-weight-semibold: 600;
--font-weight-bold: 700;
--font-weight-extrabold: 800;
```

### Kullanım Kuralları

| Ağırlık | Kullanım | Etki |
|---------|----------|------|
| `300` | İkincil metinler | Hafif, zarif |
| `400` | Ana metin | Standart okunabilirlik |
| `500` | Vurgu metinleri | Orta vurgu |
| `600` | Butonlar, linkler | Güçlü vurgu |
| `700` | Başlıklar | Güçlü hiyerarşi |
| `800` | Hero başlıklar | Maksimum etki |

---

## 📐 Satır Yüksekliği (Line Height)

### Line Height Scale
```css
--line-height-tight: 1.25;    /* Başlıklar için */
--line-height-snug: 1.375;    /* Kısa metinler */
--line-height-normal: 1.5;    /* Standart */
--line-height-relaxed: 1.625; /* Rahat okuma */
--line-height-loose: 2;       /* Çok rahat */
```

### Kullanım Rehberi
- **Başlıklar**: `1.25` - Kompakt görünüm
- **UI Elementleri**: `1.375` - Dengeli
- **Ana Metin**: `1.5` - Optimal okunabilirlik
- **Uzun Metinler**: `1.625` - Göz yorgunluğu önleme
- **Özel Durumlar**: `2` - Maksimum rahatlık

---

## 🎨 Tipografi Renkleri

### Metin Renk Hiyerarşisi
```css
/* Light Theme */
--text-primary: #1F2933;      /* Ana metin - Charcoal */
--text-secondary: #8A8F98;    /* İkincil metin - Graphite Silver */
--text-muted: #B0B6BF;        /* Soluk metin */
--text-inverse: #FFFFFF;      /* Ters metin (koyu zemin) */
--text-brand: #00B3FF;        /* Marka rengi - Ayken Blue */

/* Dark Theme */
--text-primary-dark: #E6EAF0; /* Ana metin - Ice White */
--text-secondary-dark: #B0B6BF; /* İkincil metin */
--text-muted-dark: #8A8F98;   /* Soluk metin */
--text-inverse-dark: #1F2933; /* Ters metin (açık zemin) */
--text-brand-dark: #00B3FF;   /* Marka rengi korunur */
```

---

## 📱 Responsive Tipografi

### Breakpoint Bazlı Font Boyutları

```css
/* Mobile First Approach */
.hero-title {
    font-size: 2rem;        /* 32px - Mobile */
    line-height: 1.25;
}

@media (min-width: 768px) {
    .hero-title {
        font-size: 2.5rem;  /* 40px - Tablet */
    }
}

@media (min-width: 1024px) {
    .hero-title {
        font-size: 3rem;    /* 48px - Desktop */
    }
}

@media (min-width: 1280px) {
    .hero-title {
        font-size: 3.5rem;  /* 56px - Large Desktop */
    }
}
```

### Responsive Kuralları
- **Mobile**: Daha küçük fontlar, daha sıkı spacing
- **Tablet**: Orta boyutlar, dengeli spacing
- **Desktop**: Büyük fontlar, geniş spacing
- **Large Desktop**: Maksimum boyutlar, premium spacing

---

## 🔤 Tipografi Bileşenleri

### Başlık Hiyerarşisi
```css
h1 {
    font-size: var(--font-size-4xl);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-tight);
    color: var(--text-primary);
    margin-bottom: 1.5rem;
}

h2 {
    font-size: var(--font-size-3xl);
    font-weight: var(--font-weight-semibold);
    line-height: var(--line-height-tight);
    color: var(--text-primary);
    margin-bottom: 1.25rem;
}

h3 {
    font-size: var(--font-size-2xl);
    font-weight: var(--font-weight-semibold);
    line-height: var(--line-height-snug);
    color: var(--text-primary);
    margin-bottom: 1rem;
}

h4 {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-medium);
    line-height: var(--line-height-snug);
    color: var(--text-primary);
    margin-bottom: 0.75rem;
}
```

### Paragraf Stilleri
```css
p {
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-normal);
    line-height: var(--line-height-relaxed);
    color: var(--text-primary);
    margin-bottom: 1rem;
}

.lead {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-normal);
    line-height: var(--line-height-relaxed);
    color: var(--text-secondary);
}

.small {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
}
```

### Kod Stilleri
```css
code {
    font-family: var(--font-mono);
    font-size: 0.875em;
    background: var(--bg-light);
    padding: 0.125rem 0.25rem;
    border-radius: 0.25rem;
    color: var(--text-brand);
}

pre {
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    line-height: var(--line-height-normal);
    background: var(--bg-dark);
    padding: 1rem;
    border-radius: 0.5rem;
    overflow-x: auto;
    color: var(--text-primary);
}
```

---

## 🎯 Özel AykenOS Tipografi Sınıfları

### Marka Tipografisi
```css
.ayken-title {
    font-family: var(--font-primary);
    font-size: var(--font-size-4xl);
    font-weight: var(--font-weight-extrabold);
    line-height: var(--line-height-tight);
    color: var(--text-primary);
    letter-spacing: -0.025em;
}

.ayken-subtitle {
    font-family: var(--font-primary);
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-medium);
    line-height: var(--line-height-snug);
    color: var(--text-secondary);
}

.ayken-tagline {
    font-family: var(--font-primary);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    line-height: var(--line-height-normal);
    color: var(--text-brand);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}
```

### Teknik Dokümantasyon
```css
.tech-heading {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    border-bottom: 2px solid var(--text-brand);
    padding-bottom: 0.5rem;
}

.api-method {
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    background: var(--bg-light);
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    color: var(--text-brand);
}

.syscall-id {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    background: var(--text-brand);
    color: white;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
}
```

---

## 📊 Tipografi Performansı

### Font Loading Stratejisi
```css
/* Font Display Optimization */
@font-face {
    font-family: 'CustomFont';
    src: url('font.woff2') format('woff2');
    font-display: swap; /* FOIT önleme */
}

/* Preload Critical Fonts */
<link rel="preload" href="font.woff2" as="font" type="font/woff2" crossorigin>
```

### Performance Best Practices
- **Font-display: swap** kullan
- **Critical fonts** preload et
- **WOFF2** formatını tercih et
- **Font subsetting** uygula
- **System fonts** fallback olarak kullan

---

## ♿ Erişilebilirlik

### WCAG Uyumluluk
```css
/* Minimum kontrast oranları */
/* Normal metin: 4.5:1 */
/* Büyük metin: 3:1 */
/* AA seviyesi uyumluluk */

.accessible-text {
    font-size: 16px; /* Minimum okuma boyutu */
    line-height: 1.5; /* Optimal satır aralığı */
    color: #1F2933;   /* Yüksek kontrast */
}

/* Focus states */
.focusable:focus {
    outline: 2px solid var(--text-brand);
    outline-offset: 2px;
}
```

### Erişilebilirlik Kuralları
- Minimum 16px font boyutu
- 4.5:1 kontrast oranı (normal metin)
- 3:1 kontrast oranı (büyük metin)
- Focus indicator'ları
- Screen reader uyumluluğu

---

## 🌍 Çoklu Dil Desteği

### Dil Bazlı Font Ayarları
```css
/* Türkçe */
:lang(tr) {
    font-family: var(--font-primary);
    /* Türkçe karakterler için optimizasyon */
}

/* İngilizce */
:lang(en) {
    font-family: var(--font-primary);
    /* İngilizce için standart */
}

/* Kod dilleri */
.language-javascript,
.language-rust,
.language-c {
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
}
```

---

## 🔧 Implementasyon Rehberi

### CSS Custom Properties
```css
:root {
    /* Font Families */
    --font-primary: 'Segoe UI', system-ui, -apple-system, 'Helvetica Neue', Arial, sans-serif;
    --font-mono: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', 'Consolas', 'Courier New', monospace;
    
    /* Font Sizes */
    --font-size-xs: 0.75rem;
    --font-size-sm: 0.875rem;
    --font-size-base: 1rem;
    --font-size-lg: 1.125rem;
    --font-size-xl: 1.25rem;
    --font-size-2xl: 1.5rem;
    --font-size-3xl: 1.875rem;
    --font-size-4xl: 2.25rem;
    --font-size-5xl: 3rem;
    
    /* Font Weights */
    --font-weight-light: 300;
    --font-weight-normal: 400;
    --font-weight-medium: 500;
    --font-weight-semibold: 600;
    --font-weight-bold: 700;
    --font-weight-extrabold: 800;
    
    /* Line Heights */
    --line-height-tight: 1.25;
    --line-height-snug: 1.375;
    --line-height-normal: 1.5;
    --line-height-relaxed: 1.625;
    --line-height-loose: 2;
}
```

### Utility Classes
```css
/* Font Size Utilities */
.text-xs { font-size: var(--font-size-xs); }
.text-sm { font-size: var(--font-size-sm); }
.text-base { font-size: var(--font-size-base); }
.text-lg { font-size: var(--font-size-lg); }
.text-xl { font-size: var(--font-size-xl); }
.text-2xl { font-size: var(--font-size-2xl); }
.text-3xl { font-size: var(--font-size-3xl); }
.text-4xl { font-size: var(--font-size-4xl); }
.text-5xl { font-size: var(--font-size-5xl); }

/* Font Weight Utilities */
.font-light { font-weight: var(--font-weight-light); }
.font-normal { font-weight: var(--font-weight-normal); }
.font-medium { font-weight: var(--font-weight-medium); }
.font-semibold { font-weight: var(--font-weight-semibold); }
.font-bold { font-weight: var(--font-weight-bold); }
.font-extrabold { font-weight: var(--font-weight-extrabold); }

/* Font Family Utilities */
.font-primary { font-family: var(--font-primary); }
.font-mono { font-family: var(--font-mono); }
```

---

## 🔒 Versiyon Kontrolü

### v1.0 (31 Ocak 2026)
- İlk resmi tipografi sistemi
- Font stack belirlendi
- Responsive scale oluşturuldu
- Erişilebilirlik kuralları eklendi

### Gelecek Güncellemeler
Bu tipografi sistemi uzun vadeli kullanım için tasarlandı. Değişiklik yapılması durumunda bu dosya güncellenecek ve versiyon numarası artırılacak.

---

## 📞 İletişim

**Tipografi Sorumlusu:** Kenan AY  
**Proje:** AykenOS  
**Konum:** Kütahya, Türkiye

---

**© 2026 AykenOS - Resmi Tipografi Sistemi**