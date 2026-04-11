# AykenOS Driver + DevFS + ABDF/BCIB Teknik Planı

**Tarih:** 11 Nisan 2026  
**Kapsam:** Execution closure, device discovery, driver omurgası, DevFS görünürlük katmanı, ABDF device segmentleri, BCIB runtime bridge, semantic/user yüzeyi  
**Mimari eksen:** Ring0 = mekanizma, Ring3 = politika

---

## 1. Amaç ve hüküm

Bu planın amacı, AykenOS'un mevcut execution-centric mimarisini bozmeden gerçek donanım hattını sisteme bağlamaktır. Hedef doğrudan “driver yazmak” değil; önce production-blocker execution closure'ı kapatmak, ardından donanımı sistematik olarak tanıyan ve gerçek device yüzeylerini BCIB/ABDF zincirine kontrollü bağlayan bir omurga kurmaktır.

Bu planın ana hükmü şudur:

- **BCIB yeniden tasarlanmayacak, izolasyon sınırı korunacak.**
- **ABDF driver içine gömülmeyecek, typed runtime representation olarak kullanılacak.**
- **Driver katmanı Ring0 mekanizma olarak kalacak.**
- **Semantic yorum, AI, policy ve kullanıcı niyeti Ring3'te kalacak.**
- **DevFS son hedef değil, kontrollü görünürlük ve bridge yüzeyi olacak.**

---

## 2. Mevcut proje gerçekliği

### 2.1 Güçlü çekirdek

AykenOS'un bugünkü durumu zayıf değildir. Mevcut raporlara göre:

- Phase 15 resmi olarak kapalıdır ve BCIB Execution Engine v3 kapanışı doğrulanmıştır.
- Üç katmanlı yapı (`BcibVerifierPlanner`, `BcibExecutionRuntime`, `SchedulerSubmitBridge`) gönderilmiştir.
- ABDF ve BCIB mevcut substrate olarak kabul edilmiştir; Phase 16 bunları yeniden tanımlayamaz, ancak orkestre edebilir.
- Freeze aktif ve bağlayıcıdır; Ring0/Ring3 ayrımı, syscall ABI ve anayasal CI kapıları değişmez alanlardır.
- Faz 1 raporları çekirdek boot, bellek, scheduler, interrupt ve `/dev` çatısının temelinin mevcut olduğunu göstermektedir.

### 2.2 Kritik eksikler

Aynı kaynaklara göre kritik eksikler nettir:

- Phase 16 Faz B kapanmamıştır.
- Gerçek Ring3 BCIB execution worker hattı eksiktir.
- Gerçek `submit_execution / wait_result` kernel zinciri eksiktir.
- Kernel determinism proof kapanmamıştır.
- PCI/device discovery yoktur.
- Gerçek device registry / auto bind yoktur.
- Gerçek keyboard/serial/disk driver'ları eksiktir.
- DevFS iskeleti vardır ama gerçek device surface eksiktir.

### 2.3 Sonuç

Bu tablo şu anlama gelir:

- Sistem **driver-ready** görünüyor, ama henüz **driver-capable** değil.
- BCIB ve ABDF doğru yöndedir, ama hâlâ gerçek device hattına bağlanmamıştır.
- Production-grade device → BCIB zinciri için önce Phase 16 Faz B kapanmalıdır.

---

## 3. Mimari sınırlar

### 3.1 Ring0 tarafı

Ring0'da yalnızca aşağıdakiler kalmalıdır:

- PCI scan / hardware discovery
- port I/O / MMIO / IRQ bağlama
- driver probe/init/read/write/poll mekanizması
- DevFS publish ve node dispatch mekanizması
- syscall v2 execution submission/wait mekanizması

Ring0 **şunları yapmamalıdır**:

- AI inference
- policy kararı
- semantic yorum
- scheduler policy logic
- kullanıcı niyeti çözümleme

### 3.2 Ring3 tarafı

Ring3'te aşağıdakiler kalmalıdır:

- BCIB yorumlama
- runtime bridge
- semantic CLI
- AI runtime
- device verisinin anlamlandırılması
- policy ve capability kararı

### 3.3 BCIB sınırı

BCIB şu sınırlar içinde kalmalıdır:

- syscall çağırmaz
- driver çağırmaz
- raw IRQ, MMIO, I/O port bilmez
- kernel pointer veya device pointer taşımaz
- yalnızca ABDF segmentleri ve runtime bridge üzerinden dış dünyayla temas eder

### 3.4 ABDF sınırı

ABDF şu rolde kalmalıdır:

- driver iç veri yapısı değil
- typed runtime state/result surface
- replay ve determinism için authoritative typed representation

---

## 4. Öncelik sırası

Bu planın doğru teknik sırası aşağıdaki gibidir:

1. **Execution closure**
2. **Hardware discovery (PCI/device modeli)**
3. **Driver registry + auto bind**
4. **DevFS publish ve gerçek device surface**
5. **İlk gerçek driver'lar (keyboard, serial, disk read-only)**
6. **ABDF device segmentleri**
7. **BCIB device bridge**
8. **Device → ABDF → BCIB mini pipeline**
9. **Semantic/user surface**
10. **Security hardening ve AI/device policy**

Bu sırayı bozmak teknik borç üretir.

---

## 5. Faz 0 — Execution closure (blocker)

### Amaç

Host runtime'da kanıtlanmış execution zincirini gerçek QEMU/kernel sonucu ile kapatmak.

### Yapılması gerekenler

- Ring3 BCIB execution worker payload
- gerçek `SYS_V2_SUBMIT_EXECUTION` path
- gerçek `SYS_V2_WAIT_RESULT` path
- kernel result fingerprint üretimi
- host runtime sonucu ile kernel sonucu karşılaştırma yüzeyi
- QEMU/kernel determinism proof
- evidence ve gate yüzeylerinin güncellenmesi

### Neden önce bu var?

Çünkü Phase 16 Faz B kapanmadan şu iddia yapılamaz:

> "Aynı BCIB → aynı kernel sonucu"

Şu an kanıtlanan yüzey sadece host runtime determinism'dir; kernel determinism closure henüz tamamlanmamıştır.

### Kabul kriterleri

- Aynı BCIB girdisi aynı kernel fingerprint'i üretir.
- `submit_execution` ve `wait_result` gerçek kernel yolundan geçer.
- QEMU evidence ile doğrulama alınır.
- Execution closure truth surfaces güncellenir.
- Bu eksen production-blocker olmaktan çıkar.

### Uyarılar

- Bu kapanmadan device pipeline “tamamlandı” denmeyecek.
- En fazla “mimari iskelet entegre edildi” denebilir.
- Device → BCIB production-grade iddiası bu aşama kapanmadan yapılamaz.

---

## 6. Faz 1 — Hardware discovery omurgası

### Amaç

OS'un donanımı sistematik olarak tanımasını sağlamak.

### Yapılması gerekenler

#### 6.1 PCI enumeration

x86_64 için klasik PCI config-space discovery:

- bus/dev/function tarama
- vendor_id / device_id okuma
- class_code / subclass okuma
- header type okuma
- BAR okuma
- IRQ line bilgisi alma

#### 6.2 Unified device modeli

Tek ortak `device` yapısı oluşturulmalı:

- bus
- dev
- func
- vendor_id
- device_id
- class_code
- subclass
- `bar[6]`
- irq
- capability_token
- devfs_name
- driver_data

#### 6.3 Device registry

Gerekli yüzeyler:

- `device_register`
- `device_find_by_id`
- `device_find_by_class`
- `device_iter`
- `device_publish_state`

### Kabul kriterleri

- QEMU altında enumerate edilen PCI cihazları listelenebilmeli.
- Tek tip `Device` modeli registry'ye kaydolmalı.
- BAR ve IRQ yüzeyi en azından metadata düzeyinde görülebilmeli.

### Uyarılar

- PCI scan evrenseldir; AykenOS'a özgü olan bunun capability ve execution modeline bağlanış biçimidir.
- Discovery katmanı driver yazmak değildir.
- Bu aşama olmadan driver kodu kör yazılmış olur.

---

## 7. Faz 2 — Driver registry ve auto bind

### Amaç

Bulunan device'ları uygun driver'larla kontrollü biçimde eşleştirmek.

### Yapılması gerekenler

#### 7.1 Driver trait / vtable

Her driver için ortak yüzey:

- `matches(device)`
- `probe(device)`
- `init(device)`
- `read(...)`
- `write(...)`
- opsiyonel `poll(...)`
- opsiyonel `irq(...)`
- opsiyonel `read_event(...)`

#### 7.2 Auto bind zinciri

Boot veya init sırasında:

- tüm devices registry'ye girer
- tüm driver'lar registry'ye kaydedilir
- `matches + probe + init` akışıyla bind olur
- başarılı init sonrası driver attachment device'e yazılır

#### 7.3 Capability mapping

Her bind edilen device için capability üretimi:

- DevFS erişimi
- runtime/device bridge erişimi
- privileged operation yüzeyleri

### Kabul kriterleri

- Driver'lar cihazlara otomatik bağlanır.
- Aynı cihaza çakışmalı bind yapılmaz.
- Probe başarısızlığı sistem stabilitesini bozmaz.
- Capability yüzeyi gerçek enforcement için kullanılabilir olur.

### Uyarılar

- Capability yüzeyi boş bırakılmayacak.
- Sonradan eklenirse bütün driver surface yeniden kırılır.
- Driver modeli Linux kopyası olmayacak; execution-centric üst katmanı besleyen mekanizma olacak.

---

## 8. Faz 3 — DevFS publish ve gerçek device surface

### Amaç

Bind edilen aygıtları userspace/runtime için sabit ve kontrollü görünür yüzeye dönüştürmek.

### Yapılması gerekenler

- publish hook
- node metadata
- capability-aware open/read/write/poll/read_event surface
- adlandırma politikası:
  - `/dev/kbd`
  - `/dev/ttyS0`
  - `/dev/sda`
- driver context ile node ilişkisi

### Kabul kriterleri

- Driver bind sonrası node oluşmalı.
- Node yoksa userspace erişimi olmamalı.
- Capability yoksa erişim reddedilmeli.
- DevFS node inspect edilebilmeli.

### Uyarılar

- DevFS son hedef değildir.
- DevFS, execution-centric BCIB bridge için görünürlük ve dispatch köprüsüdür.
- Device verisinin semantik yorumu DevFS'de yapılmayacak.

---

## 9. Faz 4 — İlk gerçek driver seti

Bu aşamada öncelik sırası korunmalıdır.

### 9.1 Birinci driver: PS/2 keyboard

#### Neden ilk?

- IRQ tabanlı
- test etmesi kolay
- kullanıcı görünürlüğü yüksek
- ABDF `InputEvent` segmenti için doğal kaynak

#### Yapılması gerekenler

- IRQ1 bridge
- scan code alma
- minimal translate layer
- ring buffer
- `read_event` ve/veya `read`
- mock hw_ops
- real hw_ops
- DevFS publish

#### Kabul kriterleri

- Mock testte aynı scancode aynı buffer sonucunu verir.
- QEMU altında klavye girdisi alınır.
- Overflow kontrollüdür.
- `/dev/kbd` yüzeyi çalışır.

#### Uyarı

- Keyboard driver semantik yorum yapmayacak.
- BCIB veya ABDF bilmeyecek.
- Sadece ham event üretecek.

### 9.2 İkinci driver: serial 16550

#### Neden ikinci?

- debug için kritiktir
- deterministic log surface sağlar
- QEMU ile iyi çalışır

#### Yapılması gerekenler

- RX/TX init
- FIFO kullanımı
- polling ve gerekirse IRQ
- `/dev/ttyS0`

#### Kabul kriterleri

- QEMU serial akışı görünür.
- RX/TX testleri geçer.
- Panic/deadlock olmaz.

### 9.3 Üçüncü driver: disk read-only

#### Neden read-only?

- block I/O surface'i açar
- veri bozmadan test imkânı verir
- erken write path riski yaratmaz

#### Yapılması gerekenler

- block read API
- status metadata
- `/dev/sda` placeholder
- write reddi

#### Kabul kriterleri

- Belirli block read veri döndürür.
- Write reddedilir.
- Device status okunabilir.

### Uyarılar

- Network bu aşamada alınmayacak.
- IRQ + DMA + concurrency + security yükünü erken açmak yanlış olur.

---

## 10. Driver geliştirme yöntemi

### Zorunlu yaklaşım

Her driver iki backend ile geliştirilmelidir:

- `real_hw_ops`
- `mock_hw_ops`

### Test katmanları

1. unit test / mock
2. QEMU emülasyonu
3. fiziksel donanım

### Doğrulama ölçütleri

- aynı input → aynı state
- IRQ sonrası buffer tutarlılığı
- overflow kontrolü
- kernel panic yok
- memory corruption yok
- capability ihlali engelleniyor
- DevFS publish sonrası surface görünür

### Uyarılar

- İlk test ortamı fiziksel makine olmayacak.
- Mock yazmadan driver yazmak kabul edilmeyecek.
- Logging / trace / evidence olmadan driver doğru kabul edilmeyecek.

---

## 11. Faz 5 — ABDF entegrasyonunun doğru yeri

### Ana ilke

ABDF driver'ın içine gömülmeyecek. Driver'ın ürettiği ham veri, runtime tarafından typed ABDF surface'e dönüştürülecek.

### 11.1 Keyboard → ABDF

Driver üretir:

- scancode
- ascii
- timestamp
- source device
- flags

ABDF surface:

- `INPUT_EVENTS`
- append-only veya ring-backed event segment

### 11.2 Serial → ABDF

Driver üretir:

- RX bytes
- line events
- overflow/error

ABDF surface:

- `STREAM_INPUT`
- `SERIAL_EVENTS`
- gerekirse `ReadResult`

### 11.3 Disk → ABDF

Driver üretir:

- block read result
- device status
- I/O durum bilgisi

ABDF surface:

- `READ_RESULT`
- `DEVICE_STATUS`
- gerekirse `BLOCK_VIEW`

### 11.4 Telemetry → ABDF

Device katmanı üretir:

- IRQ count
- overrun
- ready/error

ABDF surface:

- `DEVICE_METRICS`
- `DEVICE_HEALTH`

### İlk zorunlu segmentler

- `InputEvent`
- `DeviceStatus`
- `ReadResult`

### Kabul kriterleri

- Keyboard event typed ABDF yüzeyine yazılabiliyor.
- Serial ve disk sonucu typed ABDF yüzeyi üretebiliyor.
- ABDF header/segment/meta bütünlüğü korunuyor.
- Pointer-free ve little-endian kontratı korunuyor.

### Uyarılar

- ABDF ilk aşamada her yere yayılmayacak.
- Önce input/status/result yüzeyleri yeterli.
- ABDF authoritative typed representation olacak; driver storage formatı olmayacak.

---

## 12. Faz 6 — BCIB entegrasyonunun doğru yeri

### Ana ilke

BCIB driver çağırmaz. BCIB, runtime/device bridge üzerinden device surface ister.

### Gerekli minimal opcode ailesi

- `OP_DEVICE_OPEN`
- `OP_DEVICE_READ`
- `OP_DEVICE_WRITE`
- `OP_INPUT_FETCH`
- `OP_EVENT_POLL`
- `OP_RETURN`

### İlk anlamlı akışlar

#### Keyboard
- `OP_INPUT_FETCH keyboard`
- runtime `keyboard` logical source'unu çözer
- DevFS/device bridge üzerinden event çeker
- typed `InputEvent` ABDF segmentine yazar
- BCIB handle alır

#### Serial
- `OP_EVENT_POLL serial`
- runtime `/dev/ttyS0` eşlemesini çözer
- byte/event verisini alır
- ABDF stream/result segmentine bağlar

#### Disk
- `OP_DEVICE_READ disk0`
- runtime block read yapar
- `ReadResult` + payload üretir
- ABDF segmentine bağlar

### Kabul kriterleri

- BCIB pointer-free kalır.
- BCIB kernel/driver bilgisi taşımaz.
- Runtime bridge logical source çözümleme yapar.
- Mock ve QEMU altında `OP_INPUT_FETCH` ve `OP_DEVICE_READ` çalışır.

### Uyarılar

- Bu opcode'lar yeni syscall ailesi gibi tasarlanmayacak.
- Önce runtime bridge olarak açılacak.
- Syscall ABI frozen olduğu için bu sınır aşılmayacak.
- BCIB core'u büyütme değil, kontrollü bridge ekleme yapılacak.

---

## 13. Faz 7 — Device → ABDF → BCIB mini pipeline

### Amaç

Gerçek I/O'dan typed runtime execution'a ilk tam zinciri doğrulamak.

### Hedef zincir

`keyboard driver → DevFS → runtime bridge → ABDF InputEvent → BCIB OP_INPUT_FETCH`

### Yapılması gerekenler

- Keyboard raw event üretimi
- Event normalize
- ABDF segmentine yazım
- BCIB device opcode ile event fetch
- result handle üzerinden sonraki instruction kullanımı

### Kabul kriterleri

- Aynı input → aynı ABDF event
- Aynı ABDF event → aynı BCIB sonucu
- Mock ve QEMU tutarlı
- Panic, race, bozuk segment yok

### Uyarılar

- Bu aşama ilk gerçek execution-centric device tüketimidir.
- Klasik `read("/dev/kbd")` modeli değil, execution-driven device usage modeli kurulacak.

---

## 14. Faz 8 — User / semantic yüzey

### Amaç

Device verisini kullanıcı ve shell tarafında güvenli biçimde tüketmek.

### Yapılması gerekenler

- semantic CLI input/device komutları
- device inspect yüzeyi
- event readback
- safe user commands
- privileged operation guardrails

### Kabul kriterleri

- kullanıcı device listesini görebilir
- input event okuyabilir
- privileged device surface policy/capability ile korunur

### Uyarılar

- Bu faz erken açılmayacak.
- Discovery/bind/driver ve BCIB bridge bitmeden user semantics açmak sahte ilerleme üretir.

---

## 15. Faz 9 — Security hardening

### Amaç

Device erişimini capability ve rol bazlı güvenceye almak.

### Yapılması gerekenler

- admin/user ayrımı
- device capability scopes
- privileged operation flags
- AI/device guardrail policy
- approval-required surfaces

### Kabul kriterleri

- capability olmadan device read/write yok
- admin gerektiren yüzeyler ayrışmış
- AI runtime device eylemleri doğrudan değil policy üzerinden alıyor

### Uyarılar

- Security sonradan yamalanmayacak.
- İlk günden minimal capability yüzeyi olacak, sert enforcement sonra güçlenecek.

---

## 16. Dil ve katman seçimi

### C/ASM kullanılacak yerler

- port I/O
- IRQ entry
- low-level arch bridge
- driver low-level register erişimi
- DevFS node ops yüzeyi

### Rust kullanılacak yerler

- device modeli
- registry
- auto bind
- capability mapping
- runtime bridge
- raw → ABDF normalize
- BCIB device opcode dispatch
- userspace device-runtime ve semantic layer

### Hüküm

Doğru model hibrittir:

- **low-level dar yüzey = C/ASM**
- **lifecycle/orchestration/runtime = Rust**

---

## 17. Dosya ve modül planı

### Kernel

- `kernel/arch/x86_64/io.asm`
- `kernel/arch/x86_64/io.h`
- `kernel/arch/x86_64/irq_keyboard.c`
- `kernel/bus/pci.c`
- `kernel/bus/pci.h`
- `kernel/bus/pci_caps.c`
- `kernel/bus/pci_ids.h`
- `kernel/device/device_bridge.c`
- `kernel/device/device_bridge.h`
- `kernel/device/devfs_publish.c`
- `kernel/device/devfs_publish.h`
- `kernel/drivers/ps2_keyboard.c`
- `kernel/drivers/ps2_keyboard.h`
- `kernel/drivers/serial_16550.c`
- `kernel/drivers/serial_16550.h`
- `kernel/drivers/ata_stub.c`
- `kernel/drivers/ata_stub.h`
- `kernel/fs/devfs.c`

### Rust / userspace

- `userspace/device-runtime/src/lib.rs`
- `userspace/device-runtime/src/ffi.rs`
- `userspace/device-runtime/src/device.rs`
- `userspace/device-runtime/src/registry.rs`
- `userspace/device-runtime/src/driver.rs`
- `userspace/device-runtime/src/binder.rs`
- `userspace/device-runtime/src/capability.rs`
- `userspace/device-runtime/src/class.rs`
- `userspace/bcib-runtime/src/device_ops.rs`
- `userspace/bcib-runtime/src/devfs_bridge.rs`

### ABDF / BCIB

- `ayken-core/crates/abdf/src/input_segment.rs`
- `ayken-core/crates/abdf/src/device_status.rs`
- `ayken-core/crates/abdf/src/read_result.rs`
- `ayken-core/crates/bcib/src/opcode_device.rs`
- ilgili `lib.rs`, `segment.rs`, `types.rs` güncellemeleri

### Execution closure doğrulama

- execution runtime/worker dosyaları
- syscall execution slot dosyaları
- proof/evidence araçları
- gate scriptleri
- truth surface markdown güncellemeleri

---

## 18. Kaçınılması gereken hatalar

1. Execution closure kapanmadan “tamamlandı” demek
2. PCI/device discovery olmadan driver'ı nihai model gibi kurgulamak
3. DevFS'yi son hedef sanmak
4. ABDF'yi driver içine gömmek
5. BCIB opcode'larını doğrudan syscall genişletmesine çevirmek
6. Network'ü erkene almak
7. Freeze altında mainline'a mimari genişleme zorlamak
8. BCIB'e syscall, driver pointer, raw IRQ veya port bilgisi eklemek

---

## 19. Nihai mimari hedef

Bu plan doğru uygulanırsa AykenOS şu zincire ulaşır:

`Execution closure proven → hardware discovered → devices registered → drivers auto-bound → DevFS nodes published → real keyboard/serial I/O active → ABDF typed device surfaces active → BCIB device bridge active → semantic layer device-aware → security enforcement hardened`

Bu durumda:

- driver = mekanizma
- DevFS = görünürlük köprüsü
- ABDF = typed state/data
- BCIB = yürütme çekirdeği
- AI = plan üretici

Bu zincir tamamlanmadan AykenOS güçlü bir execution substrate olarak kalır; bu zincir tamamlandığında gerçek execution-centric OS kimliğine yaklaşır.

---

## 20. Son hüküm

Bu planın resmi teknik kararı şudur:

> **AykenOS'ta bir sonraki büyük iş paketi driver omurgasıdır; ancak bu omurga, Phase 16 Faz B execution closure kapatılmadan production-grade sayılmayacak, PCI/device discovery ve driver binding omurgası kurulmadan gerçek driver geliştirme ana akışa alınmayacak, ABDF ve BCIB entegrasyonu ise driver/discovery omurgasından sonra input/status/result yüzeylerinde başlayıp execution closure sonrası production-grade düzeyde kapanacaktır.**
