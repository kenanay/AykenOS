# AykenOS Teknik Spesifikasyon v1.0

**Oluşturan:** Kenan AY  
**Proje:** AykenOS - Veri-Merkezli AI-Native İşletim Sistemi  
**Tarih:** 9 Ocak 2026  
**Sürüm:** 1.0  

## 1. Proje Vizyonu ve Felsefe

### 1.1 Temel Felsefe

AykenOS, geleneksel işletim sistemi paradigmalarını **tamamen yeniden tanımlayan** özgün bir yaklaşımdır:

- **Dosya kavramı ikincildir** → Veri nesneleri birincildir
- **Komut kavramı niyete dönüşür** → Doğal dil ile sistem etkileşimi
- **Yapay zeka eklenti değil, birinci sınıf sistem bileşenidir**
- **Uyumluluk değil, yeni kategori tanımlamak hedeflenir**

### 1.2 Ayırt Edici Özellikler

```
Geleneksel OS:     Dosya → Komut → Çıktı
AykenOS:          Veri → Niyet → AI Destekli Sonuç
```

**Temel İlkeler:**
1. **Veri Birincildir**: Her bilgi yapılandırılmış, anlamlı veri nesnesidir
2. **AI-Native**: Yapay zeka sistem çekirdeğinde, eklenti değil
3. **Bağlam Odaklı**: Komut değil, çalışma bağlamı ile etkileşim
4. **Kendi Kuralları**: POSIX uyumluluğu ikincil, özgün paradigma birincil

## 2. Sistem Mimarisi

### 2.1 Katmanlı Mimari

```
┌─────────────────────────────────────────┐
│           Kullanıcı Etkileşim           │ ← Faz 4-6
│    (Shell, UI, Görsel Sahneler)        │
├─────────────────────────────────────────┤
│         AI Ajanları ve Yorumlama       │ ← Faz 3
│   (Shell LLM, HW Agent, Veri AI)       │
├─────────────────────────────────────────┤
│      Veri Katmanı ve Meta Sistem       │ ← Faz 2
│  (ABDF, Meta-DB, Veri Konteynerleri)   │
├─────────────────────────────────────────┤
│         Ring3 Runtime Katmanı          │ ← Faz 2
│    (VFS, Scheduler, DevFS Proxy)       │
├─────────────────────────────────────────┤
│           Ring0 Çekirdek                │ ← Faz 1
│  (Syscalls, Memory, Context Switch)    │
├─────────────────────────────────────────┤
│            Donanım Katmanı             │
│     (x86_64, ARM64, RISC-V)           │
└─────────────────────────────────────────┘
```

### 2.2 Çekirdek Mimarisi (Ring0)

**Temel Bileşenler:**
- **Execution-Centric Syscalls**: 10 syscall ile minimal Ring0 yüzeyi
- **Capability System**: Token tabanlı güvenlik modeli
- **Memory Management**: Sayfa tabanlı bellek yönetimi
- **Context Switching**: Ring0↔Ring3 geçiş mekanizması

**Tasarım İlkesi:** Ring0'da sadece **mekanizma**, Ring3'te **politika**

## 3. Faz Bazlı Geliştirme Planı

### 3.1 Faz 1: Çekirdek Temeli (ZORUNLU TAMAMLAMA)

#### 3.1.1 Kullanıcı Modu Geçişi
```c
// kernel/arch/x86_64/user_mode.c
// Oluşturan: Kenan AY

// Ring3 geçiş için GDT/TSS yapılandırması
typedef struct {
    uint64_t rsp0;          // Kernel stack pointer
    uint64_t user_cs;       // User code segment (0x23)
    uint64_t user_ss;       // User data segment (0x1B)
} user_context_t;

// İlerde: ARM64 ve RISC-V için benzer yapılar eklenecek
// İlerde: Güvenlik politikaları ve capability entegrasyonu
int switch_to_user_mode(user_context_t *ctx);
```

#### 3.1.2 Sistem Çağrıları v1 (Geçiş Dönemi)
```c
// kernel/sys/syscall_v1.c
// Oluşturan: Kenan AY

// Geçiş dönemi için POSIX-benzeri syscall'lar (0-99 range)
#define SYS_READ    0
#define SYS_WRITE   1
#define SYS_OPEN    2
#define SYS_CLOSE   3
#define SYS_EXIT    60

// İlerde: Bu syscall'lar Phase 2.5'te kaldırılacak
// İlerde: Sadece execution-centric syscall'lar kalacak
uint64_t syscall_v1_handler(uint64_t num, uint64_t arg1, ...);
```

#### 3.1.3 DevFS Altyapısı
```c
// kernel/fs/devfs.c
// Oluşturan: Kenan AY

typedef struct device_node {
    char name[64];
    int (*read)(void *buf, size_t count);
    int (*write)(const void *buf, size_t count);
    // İlerde: ioctl, mmap, capability kontrolü eklenecek
} device_node_t;

// Zorunlu cihazlar: /dev/null, /dev/zero, /dev/console
// İlerde: /dev/keyboard, /dev/serial, /dev/gpu eklenecek
int devfs_register_device(const char *name, device_node_t *dev);
```

#### 3.1.4 Çoklu Süreç Testi
```c
// kernel/test/multiprocess_test.c
// Oluşturan: Kenan AY

// PID1: Init süreci
// PID2: Test süreci
// Preemptive context switching doğrulaması
void test_pid1_pid2_scenario(void);

// İlerde: Daha karmaşık süreç senaryoları
// İlerde: Process lifecycle management
// İlerde: Inter-process communication
```

**Faz 1 Tamamlanma Kriterleri:**
- [ ] Ring3 geçişi çalışıyor
- [ ] INT 0x80 syscall mekanizması aktif
- [ ] PID1+PID2 preemptive test başarılı
- [ ] DevFS temel cihazları çalışıyor
- [ ] Bellek leak/stress testleri geçiyor

### 3.2 Faz 2: Veri-Merkezli Sistem (KRİTİK DÖNÜŞÜM)

#### 3.2.1 Meta-Veri Deposu
```c
// userspace/libayken/meta_database.c
// Oluşturan: Kenan AY

typedef struct data_container_meta {
    char name[256];
    char type[64];           // "tabular", "text", "graph", "ui_scene"
    char schema[1024];       // JSON schema tanımı
    uint64_t creation_time;
    uint32_t permissions;
    char ai_context[512];    // AI için bağlamsal bilgi
    // İlerde: Versioning, indexing, caching bilgileri
} container_meta_t;

// Veri konteynerlerinin yaşam döngüsü yönetimi
int meta_db_create_container(const char *name, const char *type, const char *schema);
container_meta_t* meta_db_get_container(const char *name);
int meta_db_update_container(const char *name, container_meta_t *meta);

// İlerde: Distributed meta-database
// İlerde: AI-powered schema inference
// İlerde: Automatic data relationship discovery
```

#### 3.2.2 Veri Türü Implementasyonları
```c
// userspace/libayken/data_types/tabular.c
// Oluşturan: Kenan AY

typedef struct tabular_data {
    char **column_names;
    char **column_types;     // "int", "string", "float", "datetime"
    void ***rows;            // 2D array of typed data
    size_t row_count;
    size_t column_count;
    // İlerde: Indexing, partitioning, compression
} tabular_data_t;

// Temel veri işlemleri
int tabular_add_row(tabular_data_t *table, void **row_data);
void** tabular_query(tabular_data_t *table, const char *filter);
int tabular_create_index(tabular_data_t *table, const char *column);

// İlerde: SQL-benzeri query engine
// İlerde: Distributed data processing
// İlerde: AI-powered data analysis
```

```c
// userspace/libayken/data_types/text.c
// Oluşturan: Kenan AY

typedef struct text_data {
    char *content;
    size_t length;
    char encoding[32];       // "utf-8", "ascii"
    char language[16];       // "tr", "en", "auto"
    // İlerde: Semantic embeddings, NLP metadata
} text_data_t;

// Metin işleme operasyonları
int text_append(text_data_t *text, const char *new_content);
char* text_search(text_data_t *text, const char *pattern);
char* text_extract(text_data_t *text, size_t start, size_t end);

// İlerde: AI-powered text analysis
// İlerde: Automatic language detection
// İlerde: Semantic search capabilities
```

#### 3.2.3 Shell-VFS-DSL Köprüsü
```c
// userspace/ayken-shell/shell_vfs_bridge.c
// Oluşturan: Kenan AY

typedef struct shell_context {
    char current_container[256];
    container_meta_t *active_meta;
    void *active_data;       // Aktif veri nesnesi
    // İlerde: Multi-container context, transaction support
} shell_context_t;

// DSL komutlarını veri işlemlerine çeviren köprü
int shell_execute_data_command(shell_context_t *ctx, const char *command);

// Örnek: "> data.users" → users konteynerini aktif et
// Örnek: ">> add {...}" → aktif konteynere veri ekle
// Örnek: ">> query 'role==admin'" → filtreleme yap

// İlerde: Complex query optimization
// İlerde: Multi-step data pipelines
// İlerde: AI-assisted command completion
```

#### 3.2.4 POSIX-Veri Çift Görünümü
```c
// userspace/libayken/posix_compatibility.c
// Oluşturan: Kenan AY

// Aynı veri için iki farklı görünüm sağlayan katman
typedef struct dual_view_manager {
    // POSIX görünümü: düz dosya sistemi
    char posix_path[512];
    
    // AykenOS görünümü: veri nesnesi
    char container_name[256];
    container_meta_t *meta;
    
    // İlerde: Real-time synchronization
    // İlerde: Conflict resolution strategies
} dual_view_t;

// POSIX araçları için dosya sistemi emülasyonu
int posix_view_create(const char *path, dual_view_t *view);
int posix_view_sync(dual_view_t *view);

// İlerde: Bidirectional sync mechanisms
// İlerde: POSIX tool integration layer
// İlerde: Performance optimization for large datasets
```

**Faz 2 Tamamlanma Kriterleri:**
- [ ] Meta-veri deposu çalışıyor
- [ ] Tabular ve text veri türleri functional
- [ ] Shell DSL komutları veri nesnelerine bağlanıyor
- [ ] POSIX-veri çift görünümü çalışıyor
- [ ] `data.create`, `data.add`, `data.query` komutları çalışıyor

### 3.3 Faz 3: AI-Native Sistem (ZEKA ENTEGRASYONu)

#### 3.3.1 AI Çekirdek Modülleri
```rust
// userspace/ai-runtime/src/ayken_core_lm.rs
// Oluşturan: Kenan AY

pub struct AykenCoreLM {
    model_weights: Vec<f32>,
    tokenizer: Tokenizer,
    context_window: usize,
    // İlerde: Multi-model support, model switching
}

impl AykenCoreLM {
    // Doğal dil komutlarını sistem komutlarına çevirme
    pub fn translate_natural_to_system(&self, input: &str) -> Result<String, AIError> {
        // İlerde: Context-aware translation
        // İlerde: User preference learning
        // İlerde: Multi-language support
    }
    
    // Veri özetleme ve analiz
    pub fn analyze_data(&self, data: &DataContainer) -> DataSummary {
        // İlerde: Advanced statistical analysis
        // İlerde: Pattern recognition
        // İlerde: Predictive insights
    }
}
```

#### 3.3.2 Shell LLM Entegrasyonu
```c
// userspace/ayken-shell/ai_integration.c
// Oluşturan: Kenan AY

typedef struct ai_shell_context {
    shell_context_t *shell_ctx;
    // AI model handle (Rust FFI)
    void *ai_model;
    char conversation_history[4096];
    // İlerde: Personalization, learning from user patterns
} ai_shell_context_t;

// Doğal dil sorguları için AI desteği
// Örnek: "> ? 'admin kullanıcıları listele'"
int ai_process_natural_query(ai_shell_context_t *ctx, const char *query);

// AI önerilerini güvenli komutlara dönüştürme
char* ai_suggest_commands(ai_shell_context_t *ctx, const char *intent);

// İlerde: Conversational AI interface
// İlerde: Context-aware suggestions
// İlerde: Multi-turn dialogue support
```

#### 3.3.3 Donanım AI Ajanı
```c
// userspace/hw-agent/hardware_ai.c
// Oluşturan: Kenan AY

typedef struct hw_telemetry {
    float cpu_usage;
    float memory_usage;
    float disk_io;
    float network_io;
    float temperature;
    // İlerde: GPU metrics, power consumption, detailed sensors
} hw_telemetry_t;

// Donanım durumunu AI ile yorumlama
char* ai_explain_system_status(hw_telemetry_t *metrics, const char *user_question);

// Örnek: "Neden yavaş?" → "CPU kullanımı %95, bellek sıkışmış"
// İlerde: Predictive maintenance
// İlerde: Automatic optimization suggestions
// İlerde: Anomaly detection
```

**Faz 3 Tamamlanma Kriterleri:**
- [ ] TinyLLM modeli AykenOS'ta çalışıyor
- [ ] Doğal dil sorguları sistem komutlarına çevriliyor
- [ ] AI önerileri güvenli sınırlar içinde uygulanıyor
- [ ] Donanım AI ajanı sistem durumunu yorumluyor
- [ ] AI servisleri izole çalışıyor (güvenlik)

### 3.4 Faz 4: Görsel ve Etkileşim Katmanı

#### 3.4.1 Görsel Sahne Sistemi
```c
// userspace/ayken-ui/scene_manager.c
// Oluşturan: Kenan AY

typedef struct ui_scene {
    char name[64];
    uint32_t width, height;
    struct widget *widgets;
    // İlerde: 3D scenes, VR/AR support, interactive elements
} ui_scene_t;

typedef struct widget {
    char type[32];           // "graph", "table", "text", "gauge"
    char data_source[256];   // Veri kaynağı referansı
    uint32_t x, y, w, h;     // Pozisyon ve boyut
    // İlerde: Animation, interaction callbacks, styling
} widget_t;

// Komut satırından görsel sahne oluşturma
// Örnek: "> ui.scene 'sysdash'" → yeni sahne
// Örnek: ">> widget 'cpu' from:sys.hw.metrics['cpu']"
int scene_create(const char *name);
int scene_add_widget(const char *scene_name, widget_t *widget);
int scene_render(const char *scene_name);

// İlerde: Real-time data binding
// İlerde: Interactive dashboard elements
// İlerde: Multi-monitor support
```

#### 3.4.2 AI Destekli Görselleştirme
```c
// userspace/ayken-ui/ai_visualization.c
// Oluşturan: Kenan AY

// AI'nın veri için en uygun görselleştirmeyi önermesi
typedef struct viz_recommendation {
    char chart_type[32];     // "line", "bar", "scatter", "heatmap"
    char layout[64];         // "grid:2x2", "single", "dashboard"
    char color_scheme[32];   // "dark", "light", "colorblind"
    // İlerde: Interactive elements, animation suggestions
} viz_recommendation_t;

viz_recommendation_t ai_recommend_visualization(const DataContainer *data, const char *user_intent);

// Örnek: "Bu metrikleri en anlamlı nasıl göstereyim?"
// İlerde: Automatic chart generation
// İlerde: Interactive data exploration
// İlerde: Storytelling with data
```

### 3.5 Faz 5: Çoklu Platform ve Yaygınlaştırma

#### 3.5.1 Mimari Soyutlama
```c
// kernel/arch/arch_abstraction.h
// Oluşturan: Kenan AY

// Platform bağımsız arayüzler
typedef struct arch_ops {
    int (*context_switch)(void *old_ctx, void *new_ctx);
    int (*setup_user_mode)(void *user_ctx);
    int (*handle_interrupt)(int irq);
    // İlerde: NUMA support, heterogeneous computing
} arch_ops_t;

// x86_64, ARM64, RISC-V için ayrı implementasyonlar
extern arch_ops_t x86_64_ops;
extern arch_ops_t arm64_ops;
extern arch_ops_t riscv_ops;

// İlerde: Dynamic architecture detection
// İlerde: Hybrid architecture support
// İlerde: Microcontroller variants
```

#### 3.5.2 Performans Optimizasyonu
```c
// kernel/perf/optimization.c
// Oluşturan: Kenan AY

typedef struct perf_profile {
    uint64_t syscall_latency;
    uint64_t context_switch_time;
    uint64_t memory_bandwidth;
    uint64_t ai_inference_time;
    // İlerde: Detailed profiling, bottleneck analysis
} perf_profile_t;

// Platform özel optimizasyonlar
int optimize_for_platform(const char *platform);

// İlerde: Adaptive optimization
// İlerde: Machine learning for performance tuning
// İlerde: Real-time performance monitoring
```

### 3.6 Faz 6: Gelecek Vizyonu

#### 3.6.1 Ağ ve Dağıtık Sistem
```c
// userspace/network/ayken_net.c
// Oluşturan: Kenan AY

// AykenOS'a özgü ağ protokolü
typedef struct ayken_net_packet {
    uint32_t type;           // DATA_CONTAINER, AI_REQUEST, SYSTEM_SYNC
    char source_node[64];
    char dest_node[64];
    void *payload;
    // İlerde: Encryption, compression, routing
} ayken_packet_t;

// Dağıtık veri konteynerleri
int net_sync_container(const char *container_name, const char *remote_node);
int net_distribute_ai_task(const char *task, const char *target_nodes[]);

// İlerde: Blockchain integration for data integrity
// İlerde: Federated learning across nodes
// İlerde: Automatic load balancing
```

## 4. Teknik Gereksinimler

### 4.1 Donanım Gereksinimleri

**Minimum:**
- x86_64 işlemci (Intel/AMD)
- 512 MB RAM
- 1 GB disk alanı
- UEFI firmware

**Önerilen:**
- Multi-core işlemci
- 2 GB+ RAM (AI modelleri için)
- SSD depolama
- GPU (gelecekteki AI hızlandırma için)

**Desteklenen Mimariler:**
- x86_64 (birincil)
- ARM64 (Raspberry Pi, mobil)
- RISC-V (gelecek)

### 4.2 Yazılım Bağımlılıkları

**Geliştirme Ortamı:**
- Rust 1.70+ (userspace bileşenleri)
- Clang/LLVM 15+ (kernel)
- NASM (assembly)
- QEMU (test)

**Runtime Bağımlılıkları:**
- Yok (self-contained sistem)

## 5. Güvenlik Modeli

### 5.1 Capability-Based Security
```c
// kernel/include/capability.h
// Oluşturan: Kenan AY

typedef struct capability_token {
    uint64_t id;
    uint32_t permissions;    // READ, WRITE, EXECUTE, AI_ACCESS
    uint32_t resource_type;  // MEMORY, DEVICE, DATA_CONTAINER, AI_MODEL
    uint64_t expiration;
    // İlerde: Delegation chains, revocation lists
} capability_token_t;

// Güvenlik politikaları
int security_check_ai_operation(capability_token_t *token, const char *operation);
int security_validate_data_access(capability_token_t *token, const char *container);

// İlerde: ML-based anomaly detection
// İlerde: Behavioral analysis for threat detection
// İlerde: Automatic policy adaptation
```

### 5.2 AI Güvenlik Sınırları
```c
// userspace/ai-runtime/src/ai_security.rs
// Oluşturan: Kenan AY

pub struct AISafetyLimits {
    max_inference_time: Duration,
    allowed_operations: Vec<String>,
    forbidden_patterns: Vec<String>,
    // İlerde: Dynamic risk assessment, content filtering
}

impl AISafetyLimits {
    // AI çıktılarını güvenlik açısından doğrulama
    pub fn validate_ai_output(&self, output: &str) -> Result<String, SecurityError> {
        // İlerde: Advanced content analysis
        // İlerde: Context-aware validation
        // İlerde: User-specific safety profiles
    }
}
```

## 6. Test ve Doğrulama

### 6.1 Otomatik Test Altyapısı
```bash
#!/bin/bash
# tools/test/automated_testing.sh
# Oluşturan: Kenan AY

# Faz bazlı test senaryoları
test_phase1_kernel() {
    # Ring3 geçiş testleri
    # Syscall round-trip testleri
    # Çoklu süreç testleri
    # İlerde: Stress testing, fuzzing, security testing
}

test_phase2_data() {
    # Meta-veri deposu testleri
    # Veri türü işlem testleri
    # Shell-VFS entegrasyon testleri
    # İlerde: Performance benchmarks, scalability tests
}

test_phase3_ai() {
    # AI model yükleme testleri
    # Doğal dil çeviri testleri
    # Güvenlik sınır testleri
    # İlerde: AI accuracy metrics, bias detection
}
```

### 6.2 Performans Metrikleri
```c
// tools/benchmark/performance_metrics.c
// Oluşturan: Kenan AY

typedef struct ayken_metrics {
    uint64_t boot_time_ms;
    uint64_t syscall_latency_ns;
    uint64_t ai_inference_ms;
    uint64_t data_query_ms;
    // İlerde: Detailed profiling, regression detection
} ayken_metrics_t;

// Sürekli performans izleme
int benchmark_system_performance(ayken_metrics_t *metrics);

// İlerde: Automated performance regression detection
// İlerde: Comparative analysis with other systems
// İlerde: Real-world workload simulation
```

## 7. Lisanslama ve Dağıtım

### 7.1 İkili Lisans Modeli

**Topluluk Sürümü (ASAL):**
- Açık kaynak geliştirme
- Eğitim ve araştırma kullanımı
- Topluluk katkıları

**Ticari Sürüm (ACL):**
- Ticari kullanım hakları
- Gelişmiş AI modelleri
- Profesyonel destek

### 7.2 Dağıtım Stratejisi
```
Faz 1-2: Geliştirici önizlemesi
Faz 3:   Alpha sürüm (topluluk)
Faz 4:   Beta sürüm (erken kullanıcılar)
Faz 5:   1.0 sürüm (genel kullanım)
Faz 6:   Enterprise sürüm
```

## 8. Sonuç

AykenOS, **veri-merkezli, AI-native** paradigmasıyla işletim sistemi kategorisinde yeni bir sınıf tanımlamaktadır. Bu spesifikasyon:

1. **Teknik gerçeklenebilirlik** sağlar
2. **Faz bazlı ilerleme** planı sunar  
3. **Test edilebilir hedefler** koyar
4. **Gelecek vizyonu** çizer

**Kritik Başarı Faktörleri:**
- Faz geçişlerinde %100 tamamlanma zorunluluğu
- Veri-odaklı paradigmanın tutarlı uygulanması
- AI güvenlik sınırlarının korunması
- Performans ve kararlılığın sürekli doğrulanması

Bu spesifikasyon, AykenOS'un **özgün değerini** koruyarak **teknik mükemmelliği** hedefleyen bir yol haritası sunmaktadır.

---

**Oluşturan:** Kenan AY  
**AykenOS Projesi - Teknik Spesifikasyon v1.0**  
**© 2026 AykenOS Project**