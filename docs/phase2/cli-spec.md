# Ayken Sistem Spesifikasyonları v0.1
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

Bu doküman, Ayken CLI (Semantic CLI) ve Ayken Semantic Data System (FS) için hazırlanan yeni nesil etkileşim ve veri saklama paradigmalarını içerir.

## 1. Semantic CLI Spec v0.1 (Ayken CLI)

### 1.1. Amaç
Bu CLI:
*   Klasik "dosya/klasör + komut" mantığını bırakır.
*   Veri varlıkları (DataObject), sistem bağlamı (sys.hw, sys.proc) ve arayüz sahneleri (ui.scene) ile konuşur.
*   TinyLLM’leri doğrudan komut yürütmek için değil, sorgu planlamak, özet çıkarmak, ilişki kurmak için kullanır.
*   Hiyerarşik, branch’li, reaktif bir DSL sağlar.

**Özet:** Kullanıcı dosya sistemiyle değil, veri ve sistem modeliyle konuşur. AI, plan yapar; runtime uygular.

### 1.2. Temel Kavramlar
*   **Context:** Şu an hangi varlıkla/sistemle konuştuğun (örn: `data.users`, `sys.hw`, `ui.scene.main`).
*   **Command:** Bağlam içinde yapılan iş (örn: `create`, `add`, `query`, `ai.summarize`).
*   **Level:** Komutun hiyerarşik seviyesi (`>`, `>>`, `3*>`).
*   **Branch:** Aynı anda veya ardışık yürütülen komut grubu (`>[ ... ]`).
*   **Macro:** Tek seferde tanımlanıp tekrar çağrılabilen komut şablonu (`>#`).

### 1.3. Prompt & Level Sistemi

#### 1.3.1. Semboller
| Sembol | Tanım |
| :--- | :--- |
| `>` | Root / Global seviye (üst basamak) |
| `>>` | Mevcut context içinde alt aksiyon |
| `N*>` | Derinlik / Önem seviyesi (örn: `5*>`) |
| `>[ ` | Branch/Pipeline başlangıcı |
| `]` | Branch sonu |
| `>#` | Macro/Şablon tanımı |

**Örnek:**
```bash
> data.users           # context: data.users
>> create              # users koleksiyonunu oluştur
>> add {id:1,name:"Ali"}
>> query id==1

> sys.hw[
  >> status
  >> ai.explain "neden yavaş?"
]
```

### 1.4. Lexical Elemanlar
*   **Identifier:** `[a-zA-Z_][a-zA-Z0-9_]*` (Örn: `data`, `users`, `ai`, `sys`, `hw`)
*   **Path:** `identifier("." identifier)*` (Örn: `data.users`, `sys.hw`, `ui.scene.sysdash`)
*   **Arguments:**
    *   **Scalar:** `42`, `"Ali"`, `true`, `false`
    *   **Object:** JSON benzeri → `{id:1,name:"Ali"}`
    *   **Key-value:** `role:"admin"` veya `by:"age"`
*   **String:** Çift tırnak `"..."` içinde

### 1.5. Grammar (EBNF Taslağı)
```ebnf
Input       = { Statement , Newline } ;

Statement   = Level , Command ;

Level       = RootLevel
            | DepthLevel
            | BranchLevel
            ;

RootLevel   = ">" , { ">" } ;
DepthLevel  = Digit , "*>" ;
BranchLevel = ( RootLevel | DepthLevel ) , "[" ;

Command     = Path , { " " , Argument } , [ BranchBody ] ;

BranchBody  = Newline , { InnerStatement } , "]" ;

InnerStatement = ">>" , Command ;

Path        = Ident , { "." , Ident } ;

Argument    = KeyedArg | ObjectArg | RawArg ;

KeyedArg    = Ident , ":" , Value ;
ObjectArg   = "{" , ... , "}" ;    (* JSON benzeri ayrıştırma *)
RawArg      = String | Number | Ident ;

Value       = String | Number | Boolean | ObjectArg | List ;

List        = "[" , [ Value , { "," , Value } ] , "]" ;
```
*Not: Bu taslak. Gerçek parser’da pest veya nom ile daha detaylı yazılır.*

### 1.6. Namespace’ler
Baz ana alanlar:
*   `data.*` → Semantic FS varlıkları
*   `sys.hw.*` → Donanım ve telemetri
*   `sys.proc.*` → Process/servis yönetimi (gelecek faz)
*   `ui.*` → Dashboard ve sahne yönetimi
*   `ai.*` → Global AI görevleri (özet, arama vs.)

**Örnek:**
```bash
> data.logs
>> query level=="error"
>> ai.summarize

> ui.scene "sysdash"
>> bind "cpu"  from:sys.hw.metrics['cpu_usage']
>> render
```

### 1.7. Komut Yapıları

#### 1.7.1. Data Komutları (data.*)
*   `create`
*   `add { ... }`
*   `query <expr>`
*   `delete <filter>`
*   `ai.summarize`, `ai.cluster`, `ai.explain` (FS + AI entegrasyonu)

**Örnek:**
```bash
> data.users
>> create kind:"tabular" schema:{id:"int",name:"string",role:"string"}
>> add {id:1,name:"Ali",role:"admin"}
>> add {id:2,name:"Veli",role:"user"}
>> query role=="admin"
>> ai.summarize
```

#### 1.7.2. Sistem Komutları (sys.hw.*)
```bash
> sys.hw
>> status
>> metrics
>> ai.explain "neden yavaş"
```

#### 1.7.3. UI Komutları (ui.*)
```bash
> ui.scene "sysdash"
>> layout grid:"2x2"
>> widget "cpu"   from:sys.hw.metrics['cpu_usage']
>> widget "mem"   from:sys.hw.metrics['mem_used']
>> render
```

### 1.8. Çalışma Modeli
1.  Kullanıcı bir satır girer.
2.  **Parser:** Level + Command + Args → AST.
3.  **Dispatcher:**
    *   `data.*` → `DataCommandHandler`
    *   `sys.hw.*` → `HwCommandHandler`
    *   `ui.*` → `UiCommandHandler`
4.  **Handler:** Ayken FS, HW agent, UI engine ile konuşur.
5.  **Çıktı:** Text, DataResult, UiAction.

### 1.9. Hata Modeli (MVP)
*   **Parse error:** Syntax hatası → “ParseError: invalid syntax near ...”
*   **Unknown path:** “Unknown namespace: foo.bar”
*   **Type mismatch:** “Expected tabular, got text”
*   **Permission error:** Policy engine’den (ileride)

---

## 2. Yeni Paradigmaya Göre FS Tasarımı (Ayken Semantic Data System v0.1)

Bu FS, klasik "dosya/klasör" yerine; `DataObject`, `Semantic Category`, `Typed Storage`, `Metadata + AI Profile`, `Query Engine`, `Embedding/Index` üzerine kuruludur.

### 2.1. Amaç
*   Veri, anlamıyla birlikte saklansın.
*   Her veri varlığının türü, şeması, kategorisi, politikası ve AI profili olsun.
*   CLI doğrudan bu varlıklarla konuşsun, path ile dosya kovalamak zorunda kalmasın.

### 2.2. Temel Kavramlar

#### 2.2.1. DataObject
Mantıksal veri varlığı. (Örn: `data.users`, `data.logs.app`, `data.metrics.cpu`)

**Rust Modeli:**
```rust
pub struct DataObject {
    pub id: String,             // "data.users"
    pub kind: DataKind,         // Tabular/Text/Log/...
    pub storage: StorageDescriptor,
    pub meta: MetaDescriptor,
    pub ai: AiProfile,
}
```

#### 2.2.2. DataKind
```rust
pub enum DataKind {
    Tabular(TabularSchema),
    Text(TextMeta),
    Log(LogMeta),
    Graph(GraphMeta),
    Blob(BlobMeta),
}
```
*   **Tabular:** Satır/sütun veri (kullanıcılar, ürünler)
*   **Text:** Serbest metin koleksiyonu (döküman, açıklama)
*   **Log:** Timestamp + level + message
*   **Graph:** Node/edge yapıları
*   **Blob:** Binary veri (image, model weights vs.)

#### 2.2.3. StorageDescriptor
Verinin fiziksel konum ve formatı:
```rust
pub struct StorageDescriptor {
    pub path: String,          // "/var/ayken/data/users.main.jsonl"
    pub format: StorageFormat, // JsonLines, ColumnarV1, BinLog, vs.
    pub backend: BackendKind,  // LocalFs, Memory, Remote, ObjectStore
}
```
*İlk aşamada: LocalFs + JsonLines yeterli.*

#### 2.2.4. MetaDescriptor
Ek bilgiler:
```rust
pub struct MetaDescriptor {
    pub category: String,       // "identity", "logs", "metrics", "config"
    pub tags: Vec<String>,      // ["critical", "auth", "pii"]
    pub created_at: u64,
    pub updated_at: u64,
    pub owner: String,          // "root", "ayken-ai"
}
```

#### 2.2.5. AiProfile
Veri + AI ilişkisini anlatır:
```rust
pub struct AiProfile {
    pub enabled: bool,
    pub tiny_model: Option<String>,        // "ayken-mini-64m"
    pub embedding_index_path: Option<String>,
    pub policies: AiPolicies,
}

pub struct AiPolicies {
    pub can_generate: bool,
    pub can_modify_predicate: String, // örn: "role == admin"
}
```
Bu sayede FS, nerede embedding var ve hangi modelle hangi veri çalıştırılacak bilir.

### 2.3. İşlemler (FS API)

#### 2.3.1. create_object
**CLI:**
```bash
> data.users
>> create kind:"tabular" schema:{id:"int",name:"string",role:"string"}
```
**FS API:**
```rust
fn create_object(id: &str, kind: DataKind, meta: MetaDescriptor) -> Result<DataObject, FsError>;
```

#### 2.3.2. add / insert
**CLI:**
```bash
> data.users
>> add {id:1,name:"Ali",role:"admin"}
```
**FS:**
```rust
fn insert(&self, obj: &DataObject, row: serde_json::Value) -> Result<(), FsError>;
```

#### 2.3.3. query
**CLI:**
```bash
> data.users
>> query role=="admin"
```
**FS:**
```rust
fn query(
    &self,
    obj: &DataObject,
    expr: QueryExpr
) -> Result<QueryResult, FsError>;
```

#### 2.3.4. metadata yönetimi
`data.users.meta` üzerinden:
```bash
> data.users
>> meta.show
>> meta.tag add:"critical"
```

### 2.4. Sorgu İfadesi (QueryExpr)
MVP'de sadece basit filtreler: `==`, `!=`, `>`, `<`.

**Örnek:**
```bash
> data.users.query role=="admin" and age>18
```

**AST:**
```rust
pub enum QueryExpr {
    Eq(String, Value),
    Ne(String, Value),
    Gt(String, Value),
    Lt(String, Value),
    And(Box<QueryExpr>, Box<QueryExpr>),
    Or(Box<QueryExpr>, Box<QueryExpr>),
}
```

### 2.5. Semantic FS ve CLI İlişkisi
CLI tarafında:
```bash
> data.users
>> create ...
>> add ...
>> query ...
>> ai.summarize
```
Bu komutlar `DataCommandHandler` içinde `ayken-fs` API'sini çağırır ve `ayken-ai-core` API'si ile birlikte çalışır.
*   CLI, FS ile doğrudan gürültülü konuşmaz.
*   CLI → Handler → FS / AI / Runtime zinciri oluşur.

### 2.6. Klasik FS ile İlişki
Altta hâlâ klasik dosya sistemi var, ama kullanıcı onu görmez. Sadece `StorageDescriptor.path` olarak kullanılır. "mkdir", "rm" gibi komutlar CLI'nin resmi parçası değildir. Kök soyutlama bir dosya değil, **semantic data object**'tir.

### 2.7. Basit Örnek: users akışı
1.  **Kullanıcı:**
    ```bash
    > data.users
    >> create kind:"tabular" schema:{id:"int",name:"string",role:"string"}
    ```
2.  **FS:**
    *   `/var/ayken/data/users.main.jsonl` dosyasını oluşturur.
    *   `data.users` için metadata kaydeder.
3.  **Kullanıcı:**
    ```bash
    >> add {id:1,name:"Ali",role:"admin"}
    >> add {id:2,name:"Veli",role:"user"}
    >> query role=="admin"
    ```
4.  **FS:**
    *   JSON satırlarını okur.
    *   Expression'a göre filtreler.
    *   Sonucu CLI'ye iletir.
5.  **Output:**
    ```json
    [{id:1,name:"Ali",role:"admin"}]
    ```

### 2.8. FS’nin Yeni Paradigmadaki Rolü
*   Klasör/dosya mantığını "içeride" kullanır.
*   Kullanıcıya sadece veri varlıkları ve anlamı gösterilir.
*   AI ile embedding index'leri, semantic arama, "benzer kayıtları getir", "özet çıkar" gibi işler yapar.
*   Bu da klasik FS'i **AI destekli bir veri işletim sistemine** dönüştürür.

---

**Kapanış**
*   **Semantic CLI Spec:** CLI'yi dosya ve klasörlerden kurtarıp, veri ve sistem modeliyle konuşturuyor.
*   **Yeni Paradigmaya Göre FS:** FS'i pasif bir "disk kayıtçısı" olmaktan çıkarıp, semantik, tipli, AI-aware bir veri sistemi haline getiriyor.

Bu ikisi birlikte AykenOS / AykenRuntime için tamamen yeni bir bilgisayar etkileşim modeli tanımlar.
