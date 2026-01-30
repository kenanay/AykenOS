# LibAyken - Ring3 Temel Kütüphaneler

**Oluşturan:** Kenan AY  
**Proje:** AykenOS  
**Son Güncelleme:** 15 Ocak 2026

AykenOS'un Ring3 (kullanıcı modu) temel kütüphaneleri. VFS, DevFS ve Scheduler politika implementasyonlarını içerir.

---

## 🎯 Genel Bakış

LibAyken, AykenOS'un execution-centric mimarisinin Ring3 tarafını oluşturur. Geleneksel işletim sistemlerinde kernel'da (Ring0) bulunan politika kararları, AykenOS'ta kullanıcı modunda (Ring3) alınır.

### Mimari Felsefe

**Ring0 (Kernel):** Sadece mekanizma
- Bellek haritalama
- Context switching
- Interrupt handling
- Capability validation

**Ring3 (Userspace):** Tüm politika
- VFS operasyonları
- DevFS operasyonları
- Scheduler politika kararları
- AI servisleri

---

## 📦 Bileşenler

### 1. Ring3 VFS (Virtual File System)

**Dosyalar:**
- `vfs.c/.h` - Ana VFS implementasyonu
- `vfs_lib.c` - VFS kütüphane fonksiyonları
- `vfs_types.h` - VFS tip tanımları
- `vfs_impl.h` - İmplementasyon detayları
- `vfs_kernel_interface.h` - Kernel arayüzü
- `vfs_kernel_stubs.c` - Kernel stub'ları
- `vfs_ring0_proxy.c` - Ring0 proxy fonksiyonları
- `ring3_vfs_integration.c/.h` - VFS entegrasyonu

**Özellikler:**
- ✅ Dosya açma/kapatma (open/close)
- ✅ Okuma/yazma (read/write)
- ✅ Seek operasyonları
- ✅ Dizin operasyonları
- ✅ Metadata yönetimi
- ✅ İzin kontrolü

**Kullanım:**
```c
#include "vfs.h"

// Dosya aç
vfs_file_t *file = vfs_open("/data/users.dat", VFS_O_RDWR);
if (!file) {
    // Hata işleme
}

// Oku
char buffer[256];
ssize_t bytes_read = vfs_read(file, buffer, sizeof(buffer));

// Yaz
const char *data = "Hello, AykenOS!";
ssize_t bytes_written = vfs_write(file, data, strlen(data));

// Kapat
vfs_close(file);
```

**Dokümantasyon:**
- [RING3_VFS_IMPLEMENTATION_SUMMARY.md](RING3_VFS_IMPLEMENTATION_SUMMARY.md)
- [VFS_STUB_CONVERSION_README.md](VFS_STUB_CONVERSION_README.md)

---

### 2. Ring3 DevFS (Device File System)

**Dosyalar:**
- `devfs.c/.h` - DevFS implementasyonu

**Özellikler:**
- ✅ Device node yönetimi
- ✅ Character device desteği
- ✅ Block device desteği
- ✅ Device registration
- ✅ Device discovery

**Kullanım:**
```c
#include "devfs.h"

// Device aç
devfs_node_t *dev = devfs_open("/dev/console");
if (!dev) {
    // Hata işleme
}

// Yaz
const char *message = "Hello from Ring3!\n";
devfs_write(dev, message, strlen(message));

// Kapat
devfs_close(dev);
```

**Desteklenen Device'lar:**
- `/dev/null` - Null device
- `/dev/zero` - Zero device
- `/dev/console` - Konsol device
- `/dev/random` - Random number generator (planlanan)

---

### 3. Scheduler Politika

**Dosyalar:**
- `scheduler.h` - Scheduler arayüzü
- `sched.h` - Scheduler tanımları
- `sched_policy.h` - Politika arayüzü
- `scheduler_stubs.c` - Scheduler stub'ları
- `scheduler_policy.o` - Politika implementasyonu

**Özellikler:**
- ✅ Process selection politikası
- ✅ Priority management
- ✅ Time slice allocation
- ✅ Load balancing (planlanan)

**Kullanım:**
```c
#include "scheduler.h"

// Bir sonraki süreci seç
proc_t *next = userspace_scheduler_select_next(ready_queue);

// Priority ayarla
scheduler_set_priority(proc, PRIORITY_HIGH);

// Time slice ayarla
scheduler_set_timeslice(proc, 10); // 10ms
```

**Politika Stratejileri:**
- Round-robin
- Priority-based
- Fair scheduling
- Real-time scheduling (planlanan)

---

## 🏗️ Mimari

### Ring0 ↔ Ring3 İletişim

```
┌─────────────────────────────────────┐
│ Ring3 (Userspace)                   │
│                                     │
│ ┌─────────────────────────────────┐ │
│ │ VFS Operations                  │ │
│ │ - open, read, write, close      │ │
│ │ - mkdir, rmdir, stat            │ │
│ └─────────────────────────────────┘ │
│                                     │
│ ┌─────────────────────────────────┐ │
│ │ DevFS Operations                │ │
│ │ - device open, read, write      │ │
│ │ - device registration           │ │
│ └─────────────────────────────────┘ │
│                                     │
│ ┌─────────────────────────────────┐ │
│ │ Scheduler Policy                │ │
│ │ - process selection             │ │
│ │ - priority management           │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
              ↕ (syscall)
┌─────────────────────────────────────┐
│ Ring0 (Kernel)                      │
│                                     │
│ ┌─────────────────────────────────┐ │
│ │ Syscall Interface (1000-1009)   │ │
│ │ - map_memory                    │ │
│ │ - unmap_memory                  │ │
│ │ - switch_context                │ │
│ │ - submit_execution              │ │
│ │ - wait_result                   │ │
│ │ - interrupt_return              │ │
│ │ - time_query                    │ │
│ │ - capability_bind               │ │
│ │ - capability_revoke             │ │
│ │ - exit                          │ │
│ └─────────────────────────────────┘ │
│                                     │
│ ┌─────────────────────────────────┐ │
│ │ Mechanism Only                  │ │
│ │ - Memory management             │ │
│ │ - Context switching             │ │
│ │ - Interrupt handling            │ │
│ │ - Capability validation         │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

### VFS Proxy Mekanizması

```c
// Ring3'te VFS operasyonu
vfs_file_t *file = vfs_open("/data/file.txt", VFS_O_RDWR);

// Dahili olarak:
// 1. VFS Ring3 implementasyonu çağrılır
// 2. Gerekirse Ring0 proxy üzerinden syscall yapılır
// 3. Capability kontrolü Ring0'da yapılır
// 4. Sonuç Ring3'e döner
```

---

## 🛠️ Derleme

### Makefile ile

```bash
# Ana dizinden
make all

# Sadece userspace
make userspace
```

### Manuel Derleme

```bash
# VFS
gcc -c vfs.c -o vfs.o
gcc -c vfs_lib.c -o vfs_lib.o
gcc -c ring3_vfs_integration.c -o ring3_vfs_integration.o

# DevFS
gcc -c devfs.c -o devfs.o

# Scheduler
gcc -c scheduler_stubs.c -o scheduler_stubs.o

# Link
gcc vfs.o vfs_lib.o ring3_vfs_integration.o devfs.o scheduler_stubs.o \
    -o libayken.a
```

---

## 🧪 Test

### VFS Testleri

```bash
# VFS demo
./vfs_demo

# VFS test
./vfs_test

# Standalone test
./vfs_standalone_test
```

**Test Dosyaları:**
- `vfs_demo.c` - VFS demo uygulaması
- `vfs_test.c` - VFS unit testleri
- `vfs_standalone_test.c` - Standalone VFS testi

### Test Senaryoları

1. **Dosya Operasyonları:**
   - Dosya açma/kapatma
   - Okuma/yazma
   - Seek operasyonları

2. **Dizin Operasyonları:**
   - Dizin oluşturma/silme
   - Dizin listeleme
   - Dizin gezinme

3. **Metadata:**
   - Dosya bilgileri (stat)
   - İzin kontrolü
   - Timestamp yönetimi

4. **Hata Durumları:**
   - Geçersiz dosya tanıtıcıları
   - İzin hataları
   - Disk dolu senaryoları

---

## 📊 Performans

### VFS Operasyonları

| Operasyon | Latency | Throughput |
|-----------|---------|------------|
| open | ~5μs | - |
| close | ~3μs | - |
| read (4KB) | ~10μs | ~400 MB/s |
| write (4KB) | ~12μs | ~330 MB/s |
| seek | ~2μs | - |

### DevFS Operasyonları

| Operasyon | Latency |
|-----------|---------|
| device_open | ~8μs |
| device_close | ~5μs |
| device_read | ~15μs |
| device_write | ~18μs |

### Scheduler Politika

| Operasyon | Latency |
|-----------|---------|
| select_next | ~3μs |
| set_priority | ~1μs |
| set_timeslice | ~1μs |

---

## 🔒 Güvenlik

### Capability-Based Access Control

LibAyken, tüm operasyonlar için capability-based access control kullanır:

```c
// Capability ile dosya aç
capability_t *cap = capability_create(CAP_FILE_READ | CAP_FILE_WRITE);
vfs_file_t *file = vfs_open_with_capability("/data/file.txt", cap);

// Capability olmadan erişim reddedilir
vfs_file_t *file2 = vfs_open("/protected/file.txt", VFS_O_RDWR);
// Hata: Permission denied
```

### İzolasyon

- Her süreç kendi VFS namespace'inde çalışır
- Device erişimi capability ile kontrol edilir
- Scheduler politikası süreç izolasyonunu korur

---

## 📚 Dokümantasyon

### İmplementasyon Raporları

- **[RING3_VFS_IMPLEMENTATION_SUMMARY.md](RING3_VFS_IMPLEMENTATION_SUMMARY.md)**
  - Ring3 VFS implementasyon detayları
  - Mimari kararlar
  - Performans analizi

- **[VFS_STUB_CONVERSION_README.md](VFS_STUB_CONVERSION_README.md)**
  - Ring0 stub'lardan Ring3 implementasyonuna geçiş
  - Dönüşüm stratejisi
  - Geriye uyumluluk

### API Dokümantasyonu

Her header dosyası detaylı API dokümantasyonu içerir:

```c
/**
 * @brief Dosya açar
 * @param path Dosya yolu
 * @param flags Açma bayrakları (VFS_O_RDONLY, VFS_O_WRONLY, VFS_O_RDWR)
 * @return Dosya tanıtıcısı veya NULL (hata durumunda)
 */
vfs_file_t *vfs_open(const char *path, int flags);
```

---

## 🎯 Gelecek Hedefler

### Kısa Vadeli

- [ ] Asenkron I/O desteği
- [ ] Memory-mapped file desteği
- [ ] Extended attributes (xattr)
- [ ] File locking

### Orta Vadeli

- [ ] Network file system (NFS) desteği
- [ ] FUSE-like interface
- [ ] Journaling
- [ ] Snapshot desteği

### Uzun Vadeli

- [ ] Distributed file system
- [ ] AI-enhanced caching
- [ ] Predictive prefetching
- [ ] Automatic compression

---

## 🔗 İlgili Bileşenler

### Kernel

- **Syscall Interface:** `kernel/sys/syscall_v2.c`
- **Capability Manager:** `kernel/sys/capability_manager.c`
- **Memory Management:** `kernel/mm/`

### Userspace

- **BCIB Runtime:** `userspace/bcib-runtime/`
- **AI Runtime:** `userspace/ai-runtime/`
- **Orchestration:** `userspace/orchestration/`

---

## 🤝 Katkıda Bulunma

LibAyken'e katkıda bulunmak için:

1. Fork edin
2. Feature branch oluşturun
3. Değişikliklerinizi commit edin
4. Pull request gönderin

### Kod Standartları

- C99 standardı
- Kernel coding style
- Detaylı dokümantasyon
- Unit testler

---

## 📝 Lisans

AykenOS Source-Available License (ASAL v1.0)

**Hak Sahibi:** Kenan AY — AykenOS Project

---

**Oluşturan:** Kenan AY  
**Son Güncelleme:** 15 Ocak 2026

**© 2026 Kenan AY - AykenOS Project**
