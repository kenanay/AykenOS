# AykenOS Veri-Merkezli Mimari Spesifikasyonu

**Oluşturan:** Kenan AY  
**Proje:** AykenOS - Veri-Merkezli AI-Native İşletim Sistemi  
**Tarih:** 9 Ocak 2026  
**Amaç:** Veri-odaklı paradigmanın teknik implementasyonu

## 1. Veri-Merkezli Felsefe

### 1.1 Paradigma Değişimi

```
Geleneksel OS:  Dosya → İşlem → Sonuç
AykenOS:       Veri Nesnesi → Niyet → AI Destekli Dönüşüm
```

**Temel İlkeler:**
- **Veri Birincildir**: Her bilgi yapılandırılmış, anlamlı veri nesnesidir
- **Şema Farkındalığı**: Sistem veri yapısını anlayarak işlem yapar
- **Bağlamsal İşleme**: Verinin anlamı ve ilişkileri sistem tarafından bilinir
- **AI Entegrasyonu**: Veri işleme doğal olarak AI desteklidir

### 1.2 Veri Nesnesi Kavramı

```c
// userspace/libayken/data_object.h
// Oluşturan: Kenan AY

typedef struct data_object {
    char name[256];              // Nesne adı (örn: "users", "reports")
    char type[64];               // Veri türü (örn: "tabular", "text", "graph")
    void *data_ptr;              // Gerçek veri pointer'ı
    container_meta_t *metadata;  // Meta-veri bilgileri
    
    // Veri işleme operasyonları
    struct data_operations *ops;
    
    // AI bağlamı
    char ai_context[512];        // AI için bağlamsal bilgi
    char *ai_summary;            // AI tarafından üretilen özet
    
    // İlerde: Versioning, caching, indexing
    // İlerde: Distributed data support
    // İlerde: Real-time synchronization
} data_object_t;
```

## 2. Meta-Veri Sistemi

### 2.1 Meta-Veri Deposu Mimarisi

```c
// userspace/libayken/metadata/meta_database.c
// Oluşturan: Kenan AY

typedef struct meta_database {
    char db_path[512];           // Veritabanı dosya yolu
    void *db_handle;             // Veritabanı handle (SQLite/JSON)
    
    // İndeks yapıları
    hash_table_t *name_index;   // Ad bazlı hızlı erişim
    hash_table_t *type_index;   // Tür bazlı gruplandırma
    
    // Cache sistemi
    lru_cache_t *meta_cache;    // Sık kullanılan meta-veriler
    
    // İlerde: Distributed metadata
    // İlerde: Metadata versioning
    // İlerde: Automatic schema evolution
} meta_database_t;

// Meta-veri CRUD operasyonları
int meta_db_init(meta_database_t *db, const char *db_path);
int meta_db_create_container(meta_database_t *db, container_meta_t *meta);
container_meta_t* meta_db_get_container(meta_database_t *db, const char *name);
int meta_db_update_container(meta_database_t *db, const char *name, container_meta_t *meta);
int meta_db_delete_container(meta_database_t *db, const char *name);

// Gelişmiş sorgulama
container_meta_t** meta_db_query_by_type(meta_database_t *db, const char *type);
container_meta_t** meta_db_query_by_schema(meta_database_t *db, const char *schema_pattern);

// İlerde: Full-text search on metadata
// İlerde: Relationship discovery between containers
// İlerde: Automatic metadata inference
```

### 2.2 Şema Yönetimi

```c
// userspace/libayken/metadata/schema_manager.c
// Oluşturan: Kenan AY

typedef struct schema_definition {
    char name[128];              // Şema adı
    char version[32];            // Şema versiyonu
    char json_schema[2048];      // JSON Schema tanımı
    
    // Validasyon kuralları
    validation_rule_t *rules;
    
    // Dönüşüm kuralları (schema evolution)
    transformation_rule_t *transforms;
    
    // İlerde: Schema inheritance
    // İlerde: Cross-schema relationships
    // İlerde: Automatic schema generation from data
} schema_definition_t;

// Şema operasyonları
int schema_register(schema_definition_t *schema);
int schema_validate_data(const char *schema_name, void *data);
int schema_transform_data(const char *from_schema, const char *to_schema, void *data);

// Şema evrimi (schema evolution)
int schema_evolve(const char *schema_name, const char *new_version);

// İlerde: AI-powered schema suggestion
// İlerde: Automatic data migration
// İlerde: Schema compatibility checking
```

### 2.3 Veri İlişkileri

```c
// userspace/libayken/metadata/data_relationships.c
// Oluşturan: Kenan AY

typedef enum relationship_type {
    REL_ONE_TO_ONE,
    REL_ONE_TO_MANY,
    REL_MANY_TO_MANY,
    REL_HIERARCHICAL,
    REL_TEMPORAL,               // Zaman bazlı ilişki
    REL_SEMANTIC,               // Anlamsal ilişki (AI tarafından keşfedilen)
    // İlerde: Graph relationships, dependency chains
} relationship_type_t;

typedef struct data_relationship {
    char source_container[256];
    char target_container[256];
    relationship_type_t type;
    char relationship_key[128]; // İlişki anahtarı (örn: "user_id")
    
    // İlişki meta-verisi
    char description[512];
    float confidence_score;     // AI tarafından keşfedilen ilişkiler için
    
    // İlerde: Relationship constraints
    // İlerde: Cascade operations
    // İlerde: Relationship validation
} data_relationship_t;

// İlişki yönetimi
int relationship_create(data_relationship_t *rel);
data_relationship_t** relationship_find_by_container(const char *container_name);
int relationship_validate_integrity(const char *container_name);

// AI destekli ilişki keşfi
data_relationship_t** ai_discover_relationships(const char *container_name);

// İlerde: Automatic relationship maintenance
// İlerde: Relationship-based query optimization
// İlerde: Cross-container transactions
```

## 3. Veri Türü Sistemi

### 3.1 Tabular Veri Türü

```c
// userspace/libayken/data_types/tabular.c
// Oluşturan: Kenan AY

typedef struct column_definition {
    char name[128];
    char type[32];               // "int", "string", "float", "datetime", "json"
    bool nullable;
    bool indexed;
    char constraints[256];       // "UNIQUE", "CHECK(...)", etc.
    
    // İstatistiksel bilgiler
    uint64_t distinct_count;
    void *min_value, *max_value;
    
    // İlerde: Column-level encryption
    // İlerde: Computed columns
    // İlerde: Column compression
} column_definition_t;

typedef struct tabular_data {
    column_definition_t *columns;
    size_t column_count;
    
    // Veri depolama
    void ***rows;                // 2D array of typed data
    size_t row_count;
    size_t capacity;
    
    // İndeksler
    btree_index_t **indexes;     // B-tree indeksler
    hash_index_t **hash_indexes; // Hash indeksler
    
    // İstatistikler
    table_statistics_t *stats;
    
    // İlerde: Partitioning support
    // İlerde: Columnar storage option
    // İlerde: Compression algorithms
} tabular_data_t;

// Temel operasyonlar
int tabular_create(tabular_data_t *table, column_definition_t *columns, size_t col_count);
int tabular_add_row(tabular_data_t *table, void **row_data);
int tabular_update_row(tabular_data_t *table, size_t row_index, void **new_data);
int tabular_delete_row(tabular_data_t *table, size_t row_index);

// Sorgulama
query_result_t* tabular_query(tabular_data_t *table, const char *filter);
query_result_t* tabular_join(tabular_data_t *left, tabular_data_t *right, const char *join_condition);
query_result_t* tabular_aggregate(tabular_data_t *table, const char *group_by, const char *aggregates);

// İndeksleme
int tabular_create_index(tabular_data_t *table, const char *column_name, index_type_t type);
int tabular_drop_index(tabular_data_t *table, const char *column_name);

// İlerde: SQL-like query engine
// İlerde: Distributed query processing
// İlerde: AI-powered query optimization
```

### 3.2 Text Veri Türü

```c
// userspace/libayken/data_types/text.c
// Oluşturan: Kenan AY

typedef struct text_metadata {
    char encoding[32];           // "utf-8", "ascii", "utf-16"
    char language[16];           // "tr", "en", "auto"
    char content_type[64];       // "plain", "markdown", "html", "code"
    
    // Metin istatistikleri
    size_t word_count;
    size_t sentence_count;
    size_t paragraph_count;
    
    // AI analiz sonuçları
    float sentiment_score;       // Duygu analizi (-1.0 to 1.0)
    char *keywords[32];          // Anahtar kelimeler
    char *summary;               // AI özeti
    
    // İlerde: Named entity recognition
    // İlerde: Topic modeling
    // İlerde: Semantic embeddings
} text_metadata_t;

typedef struct text_data {
    char *content;
    size_t length;
    size_t capacity;
    
    text_metadata_t *metadata;
    
    // Metin indeksleri
    inverted_index_t *word_index;    // Kelime bazlı arama
    ngram_index_t *ngram_index;      // N-gram indeksi
    
    // Versiyonlama
    text_version_t *versions;        // Metin versiyonları
    
    // İlerde: Real-time collaborative editing
    // İlerde: Automatic translation
    // İlerde: Plagiarism detection
} text_data_t;

// Temel operasyonlar
int text_create(text_data_t *text, const char *initial_content);
int text_append(text_data_t *text, const char *new_content);
int text_insert(text_data_t *text, size_t position, const char *content);
int text_delete(text_data_t *text, size_t start, size_t length);
int text_replace(text_data_t *text, const char *pattern, const char *replacement);

// Arama ve analiz
search_result_t* text_search(text_data_t *text, const char *query);
search_result_t* text_regex_search(text_data_t *text, const char *regex);
char* text_extract(text_data_t *text, size_t start, size_t end);

// AI destekli operasyonlar
char* text_ai_summarize(text_data_t *text, float compression_ratio);
float text_ai_sentiment_analysis(text_data_t *text);
char** text_ai_extract_keywords(text_data_t *text, int max_keywords);

// İlerde: Semantic search
// İlerde: Automatic categorization
// İlerde: Content generation assistance
```

### 3.3 Graph Veri Türü

```c
// userspace/libayken/data_types/graph.c
// Oluşturan: Kenan AY

typedef struct graph_node {
    uint64_t id;
    char label[128];
    void *properties;            // JSON properties
    
    // Bağlantılar
    struct graph_edge **outgoing_edges;
    struct graph_edge **incoming_edges;
    size_t out_degree, in_degree;
    
    // İlerde: Node clustering, centrality measures
} graph_node_t;

typedef struct graph_edge {
    uint64_t id;
    graph_node_t *source;
    graph_node_t *target;
    char label[128];
    void *properties;            // JSON properties
    float weight;
    
    // İlerde: Edge types, temporal edges
} graph_edge_t;

typedef struct graph_data {
    graph_node_t **nodes;
    graph_edge_t **edges;
    size_t node_count, edge_count;
    
    // İndeksler
    hash_table_t *node_index;   // ID bazlı node erişimi
    hash_table_t *edge_index;   // ID bazlı edge erişimi
    
    // Graph algoritmaları için cache
    adjacency_matrix_t *adj_matrix;
    
    // İlerde: Graph partitioning
    // İlerde: Distributed graph processing
    // İlerde: Graph neural networks
} graph_data_t;

// Temel operasyonlar
int graph_create(graph_data_t *graph);
int graph_add_node(graph_data_t *graph, graph_node_t *node);
int graph_add_edge(graph_data_t *graph, graph_edge_t *edge);
int graph_remove_node(graph_data_t *graph, uint64_t node_id);
int graph_remove_edge(graph_data_t *graph, uint64_t edge_id);

// Graph algoritmaları
path_result_t* graph_shortest_path(graph_data_t *graph, uint64_t source, uint64_t target);
component_result_t* graph_connected_components(graph_data_t *graph);
centrality_result_t* graph_centrality_analysis(graph_data_t *graph);

// Graph sorguları
query_result_t* graph_pattern_match(graph_data_t *graph, const char *pattern);
subgraph_t* graph_subgraph_extract(graph_data_t *graph, const char *criteria);

// İlerde: Graph machine learning
// İlerde: Dynamic graph analysis
// İlerde: Graph visualization generation
```

## 4. Veri İşleme Motoru

### 4.1 Query Engine

```c
// userspace/libayken/query/query_engine.c
// Oluşturan: Kenan AY

typedef struct query_context {
    char query_string[2048];
    data_object_t *target_objects[16];  // Sorgu hedef nesneleri
    size_t object_count;
    
    // Query planlama
    query_plan_t *execution_plan;
    
    // Performans metrikleri
    uint64_t execution_time_ms;
    uint64_t rows_processed;
    
    // İlerde: Distributed query coordination
    // İlerde: Query caching and optimization
} query_context_t;

typedef struct query_plan {
    query_operation_t *operations;      // İşlem dizisi
    size_t operation_count;
    
    // Optimizasyon bilgileri
    cost_estimate_t estimated_cost;
    index_usage_t *index_usage;
    
    // İlerde: Parallel execution plan
    // İlerde: Adaptive query optimization
} query_plan_t;

// Query planlama ve çalıştırma
query_plan_t* query_plan_create(const char *query_string, data_object_t **objects);
query_result_t* query_execute(query_context_t *ctx);
int query_optimize_plan(query_plan_t *plan);

// Çoklu veri türü sorguları
query_result_t* query_cross_type(data_object_t **objects, const char *join_conditions);

// İlerde: Machine learning for query optimization
// İlerde: Automatic index recommendation
// İlerde: Query result caching
```

### 4.2 Veri Dönüşüm Motoru

```c
// userspace/libayken/transform/data_transformer.c
// Oluşturan: Kenan AY

typedef struct transformation_rule {
    char name[128];
    char source_type[64];
    char target_type[64];
    
    // Dönüşüm fonksiyonu
    int (*transform_func)(void *source_data, void **target_data);
    
    // Dönüşüm parametreleri
    void *parameters;
    
    // İlerde: Reversible transformations
    // İlerde: Lossy transformation warnings
} transformation_rule_t;

typedef struct data_pipeline {
    char name[256];
    transformation_rule_t **rules;
    size_t rule_count;
    
    // Pipeline durumu
    pipeline_state_t state;
    
    // İlerde: Real-time pipeline processing
    // İlerde: Pipeline monitoring and alerting
} data_pipeline_t;

// Veri dönüşüm operasyonları
int transform_register_rule(transformation_rule_t *rule);
int transform_data(const char *rule_name, void *source, void **target);
int transform_pipeline_execute(data_pipeline_t *pipeline, data_object_t *input);

// Otomatik dönüşüm keşfi
transformation_rule_t** transform_discover_rules(data_object_t *source, data_object_t *target);

// İlerde: AI-powered transformation suggestion
// İlerde: Data quality assessment during transformation
// İlerde: Incremental transformation for large datasets
```

## 5. Shell Entegrasyonu

### 5.1 Veri-Odaklı Shell Komutları

```c
// userspace/ayken-shell/data_commands.c
// Oluşturan: Kenan AY

typedef struct shell_data_context {
    char current_container[256];     // Aktif veri konteyneri
    data_object_t *active_object;    // Aktif veri nesnesi
    query_context_t *query_ctx;      // Sorgu bağlamı
    
    // Komut geçmişi
    char command_history[32][512];
    size_t history_count;
    
    // İlerde: Multi-container context
    // İlerde: Transaction support
    // İlerde: Undo/redo operations
} shell_data_context_t;

// Temel veri komutları
int cmd_data_create(shell_data_context_t *ctx, const char *args);
int cmd_data_select(shell_data_context_t *ctx, const char *container_name);
int cmd_data_add(shell_data_context_t *ctx, const char *data_json);
int cmd_data_query(shell_data_context_t *ctx, const char *filter);
int cmd_data_update(shell_data_context_t *ctx, const char *update_spec);
int cmd_data_delete(shell_data_context_t *ctx, const char *delete_spec);

// Gelişmiş komutlar
int cmd_data_join(shell_data_context_t *ctx, const char *join_spec);
int cmd_data_aggregate(shell_data_context_t *ctx, const char *agg_spec);
int cmd_data_transform(shell_data_context_t *ctx, const char *transform_spec);

// Meta-veri komutları
int cmd_data_describe(shell_data_context_t *ctx, const char *container_name);
int cmd_data_schema(shell_data_context_t *ctx, const char *schema_spec);
int cmd_data_relationships(shell_data_context_t *ctx, const char *container_name);

// İlerde: Natural language command interface
// İlerde: Command auto-completion based on data schema
// İlerde: Interactive data exploration
```

### 5.2 DSL (Domain Specific Language) Parser

```c
// userspace/ayken-shell/dsl_parser.c
// Oluşturan: Kenan AY

typedef enum dsl_command_type {
    DSL_CONTEXT_SELECT,          // > data.users
    DSL_CONTEXT_OPERATION,       // >> add {...}
    DSL_BATCH_OPERATION,         // >[ ] cmd1 | cmd2
    DSL_AI_QUERY,               // > ? "natural language query"
    DSL_PIPELINE,               // >| transform | filter | output
    // İlerde: Complex nested operations
} dsl_command_type_t;

typedef struct dsl_command {
    dsl_command_type_t type;
    char command_text[1024];
    char *parameters[16];
    size_t param_count;
    
    // Komut meta-verisi
    char context[256];
    bool requires_ai;
    
    // İlerde: Command validation, optimization hints
} dsl_command_t;

// DSL parsing
dsl_command_t* dsl_parse_command(const char *input);
int dsl_validate_command(dsl_command_t *cmd, shell_data_context_t *ctx);
int dsl_execute_command(dsl_command_t *cmd, shell_data_context_t *ctx);

// Hiyerarşik komut işleme
int dsl_process_context_select(const char *context, shell_data_context_t *ctx);
int dsl_process_context_operation(const char *operation, shell_data_context_t *ctx);
int dsl_process_batch_operations(const char *batch_spec, shell_data_context_t *ctx);

// İlerde: DSL syntax highlighting
// İlerde: Interactive DSL builder
// İlerde: DSL macro system
```

## 6. POSIX Uyumluluk Katmanı

### 6.1 Çift Görünüm Sistemi

```c
// userspace/libayken/posix/dual_view_manager.c
// Oluşturan: Kenan AY

typedef struct dual_view_mapping {
    char posix_path[512];            // POSIX dosya yolu
    char container_name[256];        // AykenOS veri konteyneri
    char view_type[64];              // "file", "directory", "symlink"
    
    // Senkronizasyon durumu
    uint64_t last_sync_time;
    bool sync_required;
    
    // Çakışma çözümü
    conflict_resolution_t conflict_policy;
    
    // İlerde: Bidirectional sync optimization
    // İlerde: Real-time change notification
} dual_view_mapping_t;

typedef struct posix_compatibility_layer {
    dual_view_mapping_t **mappings;
    size_t mapping_count;
    
    // POSIX emülasyon
    posix_file_ops_t *file_ops;
    posix_dir_ops_t *dir_ops;
    
    // Senkronizasyon motoru
    sync_engine_t *sync_engine;
    
    // İlerde: Performance optimization cache
    // İlerde: Conflict resolution strategies
} posix_compatibility_layer_t;

// Çift görünüm yönetimi
int dual_view_create_mapping(const char *posix_path, const char *container_name);
int dual_view_sync(dual_view_mapping_t *mapping);
int dual_view_resolve_conflict(dual_view_mapping_t *mapping, conflict_type_t conflict);

// POSIX emülasyon
int posix_open(const char *path, int flags);
ssize_t posix_read(int fd, void *buf, size_t count);
ssize_t posix_write(int fd, const void *buf, size_t count);
int posix_close(int fd);

// Dizin operasyonları
DIR* posix_opendir(const char *path);
struct dirent* posix_readdir(DIR *dirp);
int posix_closedir(DIR *dirp);

// İlerde: Advanced POSIX compatibility
// İlerde: Performance optimization for POSIX tools
// İlerde: Automatic mapping discovery
```

### 6.2 Veri Serialization/Deserialization

```c
// userspace/libayken/serialization/data_serializer.c
// Oluşturan: Kenan AY

typedef enum serialization_format {
    SERIAL_JSON,
    SERIAL_CSV,
    SERIAL_XML,
    SERIAL_BINARY,
    SERIAL_ABDF,                 // AykenOS native format
    // İlerde: Protocol Buffers, MessagePack, Avro
} serialization_format_t;

typedef struct serialization_context {
    serialization_format_t format;
    void *format_options;        // Format-specific options
    
    // Performans ayarları
    bool use_compression;
    compression_type_t compression;
    
    // Şema bilgisi
    schema_definition_t *schema;
    
    // İlerde: Streaming serialization
    // İlerde: Incremental serialization
} serialization_context_t;

// Serialization operasyonları
int serialize_data_object(data_object_t *obj, serialization_context_t *ctx, 
                         void **output, size_t *output_size);
int deserialize_data_object(void *input, size_t input_size, 
                           serialization_context_t *ctx, data_object_t **obj);

// Format dönüşümleri
int convert_format(void *input, serialization_format_t from_format,
                  serialization_format_t to_format, void **output);

// POSIX dosya formatı emülasyonu
int export_as_posix_file(data_object_t *obj, const char *file_path, 
                        serialization_format_t format);
int import_from_posix_file(const char *file_path, data_object_t **obj);

// İlerde: Schema-aware serialization optimization
// İlerde: Automatic format detection
// İlerde: Streaming large dataset serialization
```

## 7. Performans ve Optimizasyon

### 7.1 Veri İndeksleme Sistemi

```c
// userspace/libayken/indexing/index_manager.c
// Oluşturan: Kenan AY

typedef enum index_type {
    INDEX_BTREE,                 // B-tree indeks (sıralı erişim)
    INDEX_HASH,                  // Hash indeks (eşitlik sorguları)
    INDEX_BITMAP,                // Bitmap indeks (düşük kardinalite)
    INDEX_FULLTEXT,              // Full-text arama indeksi
    INDEX_SPATIAL,               // Spatial/geometric indeks
    INDEX_AI_EMBEDDING,          // AI embedding vektör indeksi
    // İlerde: LSM-tree, R-tree, inverted index
} index_type_t;

typedef struct index_definition {
    char name[128];
    char container_name[256];
    char column_names[16][128];  // İndekslenecek kolonlar
    size_t column_count;
    index_type_t type;
    
    // İndeks parametreleri
    void *type_specific_params;
    
    // İstatistikler
    uint64_t size_bytes;
    uint64_t last_update_time;
    
    // İlerde: Partial indexes, expression indexes
} index_definition_t;

// İndeks yönetimi
int index_create(index_definition_t *def);
int index_drop(const char *index_name);
int index_rebuild(const char *index_name);
int index_update(const char *index_name, void *old_data, void *new_data);

// İndeks kullanımı
query_result_t* index_search(const char *index_name, void *search_key);
query_result_t* index_range_search(const char *index_name, void *start_key, void *end_key);

// Otomatik indeks önerisi
index_recommendation_t** index_analyze_workload(const char *container_name);

// İlerde: Adaptive indexing based on query patterns
// İlerde: Distributed indexing
// İlerde: Machine learning for index optimization
```

### 7.2 Caching Sistemi

```c
// userspace/libayken/caching/data_cache.c
// Oluşturan: Kenan AY

typedef struct cache_policy {
    cache_algorithm_t algorithm; // LRU, LFU, ARC, etc.
    size_t max_size_bytes;
    size_t max_entries;
    uint32_t ttl_seconds;        // Time-to-live
    
    // Cache davranışı
    bool write_through;
    bool write_back;
    
    // İlerde: Adaptive cache sizing
    // İlerde: Multi-level caching
} cache_policy_t;

typedef struct data_cache {
    hash_table_t *cache_table;
    lru_list_t *lru_list;        // LRU için
    
    // Cache istatistikleri
    uint64_t hit_count;
    uint64_t miss_count;
    uint64_t eviction_count;
    
    cache_policy_t *policy;
    
    // İlerde: Cache warming strategies
    // İlerde: Distributed cache coordination
} data_cache_t;

// Cache operasyonları
int cache_init(data_cache_t *cache, cache_policy_t *policy);
int cache_put(data_cache_t *cache, const char *key, void *data, size_t size);
void* cache_get(data_cache_t *cache, const char *key);
int cache_invalidate(data_cache_t *cache, const char *key);
int cache_clear(data_cache_t *cache);

// Cache istatistikleri
cache_stats_t cache_get_stats(data_cache_t *cache);
float cache_hit_ratio(data_cache_t *cache);

// İlerde: Predictive caching using AI
// İlerde: Cache coherence in distributed systems
// İlerde: Automatic cache tuning
```

## 8. Güvenlik ve Erişim Kontrolü

### 8.1 Veri Seviyesi Güvenlik

```c
// userspace/libayken/security/data_security.c
// Oluşturan: Kenan AY

typedef struct data_access_policy {
    char policy_name[128];
    char container_pattern[256]; // Hangi konteynerlere uygulanır
    
    // Erişim kuralları
    access_rule_t *rules;
    size_t rule_count;
    
    // Şifreleme gereksinimleri
    encryption_requirement_t encryption;
    
    // Audit gereksinimleri
    bool audit_required;
    audit_level_t audit_level;
    
    // İlerde: Dynamic access control
    // İlerde: Attribute-based access control (ABAC)
} data_access_policy_t;

typedef struct access_rule {
    char principal[256];         // Kullanıcı/grup/rol
    permission_set_t permissions; // READ, WRITE, DELETE, ADMIN
    
    // Koşullu erişim
    char conditions[512];        // "time > 09:00 AND location = 'office'"
    
    // Veri seviyesi kısıtlamalar
    char row_filter[256];        // Satır seviyesi filtreleme
    char column_mask[256];       // Kolon seviyesi maskeleme
    
    // İlerde: Dynamic permissions based on data content
    // İlerde: Machine learning for anomaly detection
} access_rule_t;

// Güvenlik kontrolü
int security_check_access(const char *principal, const char *container_name, 
                         permission_t requested_permission);
int security_apply_row_filter(const char *principal, query_result_t *result);
int security_apply_column_mask(const char *principal, query_result_t *result);

// Audit logging
int audit_log_access(const char *principal, const char *container_name, 
                    const char *operation, const char *details);

// İlerde: Real-time security monitoring
// İlerde: Automated threat response
// İlerde: Privacy-preserving analytics
```

### 8.2 Veri Şifreleme

```c
// userspace/libayken/security/data_encryption.c
// Oluşturan: Kenan AY

typedef enum encryption_type {
    ENCRYPT_NONE,
    ENCRYPT_AES256,
    ENCRYPT_CHACHA20,
    ENCRYPT_COLUMN_LEVEL,        // Kolon seviyesi şifreleme
    ENCRYPT_FIELD_LEVEL,         // Alan seviyesi şifreleme
    ENCRYPT_HOMOMORPHIC,         // Homomorfik şifreleme (hesaplama yapılabilir)
    // İlerde: Quantum-resistant encryption
} encryption_type_t;

typedef struct encryption_context {
    encryption_type_t type;
    uint8_t key[64];             // Şifreleme anahtarı
    uint8_t iv[32];              // Initialization vector
    
    // Anahtar yönetimi
    char key_id[128];            // Anahtar tanımlayıcısı
    key_rotation_policy_t rotation_policy;
    
    // İlerde: Hardware security module (HSM) integration
    // İlerde: Key escrow and recovery
} encryption_context_t;

// Şifreleme operasyonları
int encrypt_data_object(data_object_t *obj, encryption_context_t *ctx);
int decrypt_data_object(data_object_t *obj, encryption_context_t *ctx);

// Kolon seviyesi şifreleme
int encrypt_column(tabular_data_t *table, const char *column_name, encryption_context_t *ctx);
int decrypt_column(tabular_data_t *table, const char *column_name, encryption_context_t *ctx);

// Anahtar yönetimi
int key_generate(encryption_type_t type, encryption_context_t *ctx);
int key_rotate(const char *key_id, encryption_context_t *new_ctx);
int key_derive(const char *master_key, const char *context, encryption_context_t *ctx);

// İlerde: Searchable encryption
// İlerde: Zero-knowledge proofs
// İlerde: Secure multi-party computation
```

## 9. Sonuç

Bu veri-merkezli mimari, AykenOS'un **özgün değer önerisini** teknik olarak gerçekleştiren kapsamlı bir sistemdir:

### 9.1 Temel Başarılar
- **Veri Birincilliği**: Dosya sistemi yerine veri nesnesi paradigması
- **Şema Farkındalığı**: Sistem veri yapısını anlayarak işlem yapar
- **AI Entegrasyonu**: Veri işleme doğal olarak AI desteklidir
- **POSIX Uyumluluğu**: Mevcut araçlarla çalışabilirlik korunur

### 9.2 Yenilikçi Özellikler
- **Çift Görünüm Sistemi**: POSIX ve veri-odaklı görünümler
- **Meta-Veri Odaklı İşleme**: Veri ilişkileri ve şema evrimi
- **Hiyerarşik DSL**: Bağlam odaklı komut sistemi
- **AI Destekli Veri Keşfi**: Otomatik ilişki ve pattern keşfi

Bu mimari, AykenOS'u **geleneksel işletim sistemlerinden** kesin olarak ayıran ve **yeni bir kategori** tanımlayan teknik temeli sağlar.

---

**Oluşturan:** Kenan AY  
**AykenOS Veri-Merkezli Mimari Spesifikasyonu**  
**© 2026 AykenOS Project**