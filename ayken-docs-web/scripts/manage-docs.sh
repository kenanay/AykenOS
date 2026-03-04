#!/bin/bash

# AykenOS Dokümantasyon Yönetim Scripti
# Bu script dokümantasyon yapısını yönetir ve yeni sayfalar oluşturur

set -e

DOCS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/docs"
TEMPLATE_FILE="$DOCS_DIR/_template.html"

# Renkler
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Yardım mesajı
show_help() {
    cat << EOF
AykenOS Dokümantasyon Yönetim Aracı

Kullanım:
    $0 [komut] [parametreler]

Komutlar:
    create <kategori> <dosya-adi> <başlık>
        Yeni bir dokümantasyon sayfası oluşturur
        Örnek: $0 create 01-baslangic hizli-baslangic "Hızlı Başlangıç"

    list [kategori]
        Tüm kategorileri veya belirli bir kategorideki sayfaları listeler
        Örnek: $0 list
        Örnek: $0 list 01-baslangic

    validate
        Tüm dokümantasyon sayfalarını doğrular (kırık linkler, eksik dosyalar)

    stats
        Dokümantasyon istatistiklerini gösterir

    help
        Bu yardım mesajını gösterir

Kategoriler:
    01-baslangic          Başlangıç ve Kurulum
    02-mimari             Sistem Mimarisi
    03-anayasal-sistem    Constitutional System
    04-gelistirme         Development Guide
    05-api-referans       API Reference
    06-felsefe            Philosophy & Principles
    07-topluluk           Community & Contributing
    08-ornekler           Examples & Tutorials
    09-sorun-giderme      Troubleshooting
    10-referans           Reference Materials

EOF
}

# Yeni sayfa oluştur
create_page() {
    local category="$1"
    local filename="$2"
    local title="$3"

    if [ -z "$category" ] || [ -z "$filename" ] || [ -z "$title" ]; then
        echo -e "${RED}Hata: Eksik parametreler${NC}"
        echo "Kullanım: $0 create <kategori> <dosya-adi> <başlık>"
        exit 1
    fi

    local category_dir="$DOCS_DIR/$category"
    local output_file="$category_dir/$filename.html"

    # Kategori dizini kontrolü
    if [ ! -d "$category_dir" ]; then
        echo -e "${RED}Hata: Kategori bulunamadı: $category${NC}"
        echo "Mevcut kategoriler:"
        ls -d "$DOCS_DIR"/*/ | xargs -n 1 basename
        exit 1
    fi

    # Dosya zaten var mı?
    if [ -f "$output_file" ]; then
        echo -e "${YELLOW}Uyarı: Dosya zaten mevcut: $output_file${NC}"
        read -p "Üzerine yazmak istiyor musunuz? (e/h) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Ee]$ ]]; then
            echo "İptal edildi."
            exit 0
        fi
    fi

    # Şablondan yeni sayfa oluştur
    if [ ! -f "$TEMPLATE_FILE" ]; then
        echo -e "${RED}Hata: Şablon dosyası bulunamadı: $TEMPLATE_FILE${NC}"
        exit 1
    fi

    # Şablonu kopyala ve değiştir
    cp "$TEMPLATE_FILE" "$output_file"
    
    # Kategori adını al
    local category_name=$(get_category_name "$category")
    
    # Placeholder'ları değiştir
    sed -i.bak "s/\[SAYFA BAŞLIĞI\]/$title/g" "$output_file"
    sed -i.bak "s/\[KATEGORİ\]/$category_name/g" "$output_file"
    sed -i.bak "s/\[TARİH\]/$(date +%Y-%m-%d)/g" "$output_file"
    
    # Backup dosyasını sil
    rm -f "$output_file.bak"

    echo -e "${GREEN}✓ Sayfa oluşturuldu: $output_file${NC}"
    echo "Düzenlemek için: code $output_file"
}

# Kategori adını al
get_category_name() {
    case "$1" in
        01-baslangic) echo "Başlangıç" ;;
        02-mimari) echo "Sistem Mimarisi" ;;
        03-anayasal-sistem) echo "Anayasal Sistem" ;;
        04-gelistirme) echo "Geliştirme" ;;
        05-api-referans) echo "API Referansı" ;;
        06-felsefe) echo "Felsefe" ;;
        07-topluluk) echo "Topluluk" ;;
        08-ornekler) echo "Örnekler" ;;
        09-sorun-giderme) echo "Sorun Giderme" ;;
        10-referans) echo "Referans" ;;
        *) echo "$1" ;;
    esac
}

# Sayfaları listele
list_pages() {
    local category="$1"

    if [ -z "$category" ]; then
        # Tüm kategorileri listele
        echo -e "${GREEN}Dokümantasyon Kategorileri:${NC}"
        echo
        for dir in "$DOCS_DIR"/*/; do
            if [ -d "$dir" ]; then
                local cat_name=$(basename "$dir")
                local cat_title=$(get_category_name "$cat_name")
                local page_count=$(find "$dir" -name "*.html" -not -name "index.html" | wc -l)
                echo -e "  ${YELLOW}$cat_name${NC} - $cat_title ($page_count sayfa)"
            fi
        done
    else
        # Belirli kategorideki sayfaları listele
        local category_dir="$DOCS_DIR/$category"
        if [ ! -d "$category_dir" ]; then
            echo -e "${RED}Hata: Kategori bulunamadı: $category${NC}"
            exit 1
        fi

        local cat_title=$(get_category_name "$category")
        echo -e "${GREEN}$cat_title ($category) Sayfaları:${NC}"
        echo

        find "$category_dir" -name "*.html" | sort | while read -r file; do
            local filename=$(basename "$file" .html)
            if [ "$filename" != "index" ] && [ "$filename" != "_template" ]; then
                echo "  - $filename.html"
            fi
        done
    fi
}

# Dokümantasyonu doğrula
validate_docs() {
    echo -e "${GREEN}Dokümantasyon doğrulanıyor...${NC}"
    echo

    local errors=0

    # Her kategoride index.html var mı?
    for dir in "$DOCS_DIR"/*/; do
        if [ -d "$dir" ]; then
            local cat_name=$(basename "$dir")
            if [ ! -f "$dir/index.html" ]; then
                echo -e "${RED}✗ Eksik index.html: $cat_name${NC}"
                ((errors++))
            else
                echo -e "${GREEN}✓ index.html mevcut: $cat_name${NC}"
            fi
        fi
    done

    echo
    if [ $errors -eq 0 ]; then
        echo -e "${GREEN}✓ Tüm kontroller başarılı${NC}"
    else
        echo -e "${RED}✗ $errors hata bulundu${NC}"
        exit 1
    fi
}

# İstatistikleri göster
show_stats() {
    echo -e "${GREEN}Dokümantasyon İstatistikleri:${NC}"
    echo

    local total_pages=0
    local total_categories=0

    for dir in "$DOCS_DIR"/*/; do
        if [ -d "$dir" ]; then
            ((total_categories++))
            local page_count=$(find "$dir" -name "*.html" -not -name "index.html" -not -name "_template.html" | wc -l)
            total_pages=$((total_pages + page_count))
        fi
    done

    echo "  Toplam Kategori: $total_categories"
    echo "  Toplam Sayfa: $total_pages"
    echo "  Ortalama Sayfa/Kategori: $((total_pages / total_categories))"
}

# Ana komut işleyici
case "${1:-help}" in
    create)
        create_page "$2" "$3" "$4"
        ;;
    list)
        list_pages "$2"
        ;;
    validate)
        validate_docs
        ;;
    stats)
        show_stats
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo -e "${RED}Bilinmeyen komut: $1${NC}"
        echo
        show_help
        exit 1
        ;;
esac
