# Phase-17 Minimal Execution Path

**Authority:** Kenan AY - Architectural Steward  
**Status:** CANONICAL  
**Phase:** 17  
**Effective Date:** 2026-05-01  
**Purpose:** EN KÜÇÜK çalışan doğru pipeline tanımı

---

## 1. Minimal Akış (Canonical)

```
submit_execution()
    ↓
scheduler pickup
    ↓
execution_slot.execute()
    ↓
execution_slot.write_output()
    ↓
execution_slot.verify()
    ↓
execution_slot.commit()
    ↓
wait_result()
```

**Kural:**
> Bu akış değiştirilemez (optimization sonrası bile)

---

## 2. Kernel İç Akış (Detaylı)

### 2.1 EXECUTING State

```c
// BCIB decode
bcib_decode(slot->bcib_buffer, &instructions);

// Instruction execution
for (i = 0; i < instructions.count; i++) {
    execute_instruction(&instructions.ops[i], slot);
}

// Transition
slot->state = SLOT_STATE_WRITE_OUTPUT;
```

**Marker:**
```
[EXEC_START]
```

---

### 2.2 WRITE_OUTPUT State

```c
// Raw output buffer'a yazılır
write_raw_output(slot->result_buffer, output_data, output_size);

// Buffer seal
slot->result_buffer->flags |= BUFFER_SEALED;

// Transition
slot->state = SLOT_STATE_VERIFYING;
```

**Marker:**
```
[EXEC_OUTPUT_WRITTEN]
[EXEC_COMPLETE_OK]
```

---

### 2.3 VERIFYING State

```c
// 1. raw_output_hash hesaplanır
compute_raw_output_hash(slot->result_buffer, slot->raw_output_hash);

// 2. execution_context_snapshot_hash hesaplanır
compute_context_snapshot_hash(slot, slot->context_snapshot_hash);

// 3. fingerprint hesaplanır
generate_execution_fingerprint(slot, slot->fingerprint);

// 4. verification checks
if (!verify_execution_contract(slot)) {
    slot->state = SLOT_STATE_FAILED;
    return -1;
}

// Transition
slot->state = SLOT_STATE_VERIFIED;
```

**Marker:**
```
[VERIFY_START]
[VERIFY_PASS]
```

---

### 2.4 VERIFIED State

```c
// Buffer immutable
slot->result_buffer->flags |= BUFFER_IMMUTABLE;

// Receipt generation
generate_execution_receipt(slot, &slot->receipt);

// Transition
slot->state = SLOT_STATE_COMMITTED;
```

**Marker:**
```
[RESULT_OK]
```

---

### 2.5 COMMITTED State

```c
// Result userspace'e açılır
publish_result_to_userspace(slot);

// Scheduler notification
notify_scheduler_completion(slot);
```

**Marker:**
```
[WAIT_OK]
```

---

## 3. Marker Sırası (Zorunlu)

**Doğru Sıra:**
```
[EXEC_START]
[EXEC_OUTPUT_WRITTEN]
[EXEC_COMPLETE_OK]
[VERIFY_START]
[VERIFY_PASS]
[RESULT_OK]
[WAIT_OK]
```

**Kural:**
> Bu sıra değiştirilemez (determinism kanıtı için)

---

## 4. Determinism Tanımı

**PASS Condition:**
```
run1.raw_output_hash == run2.raw_output_hash
```

**FAIL Condition:**
```
herhangi bir byte farkı
```

**Test:**
```bash
# Run 1
make ci-gate-bcib-determinism RUN=1
sha256sum evidence/run-1/raw_output.bin > hash1.txt

# Run 2
make ci-gate-bcib-determinism RUN=2
sha256sum evidence/run-2/raw_output.bin > hash2.txt

# Compare
diff hash1.txt hash2.txt
# (boş olmalı)
```

---

## 5. Minimum Gereksinimler

**Execution Environment:**
- ✅ Tek thread execution
- ✅ Sabit input (BCIB)
- ✅ Sabit memory mapping
- ✅ Sabit scheduler davranışı (aynı context için)

**Kernel State:**
- ✅ execution_slot isolated
- ✅ No global state mutation
- ✅ No system time dependency

**AI Runtime (Phase-17):**
- ✅ THREADS=1
- ✅ SEED=FIXED
- ✅ MODEL_HASH=FIXED

---

## 6. Yapılmayacaklar (Kapsam Dışı)

**Bu minimal path içinde YOK:**

❌ AI semantic processing  
❌ Multi-thread execution  
❌ Distributed scheduling  
❌ Dynamic memory behavior  
❌ Adaptive optimization  
❌ Performance tuning  
❌ Caching  
❌ Prefetching

**Neden?**
> Bu özellikler determinism'i bozar veya Phase-17 kapsamı dışındadır

---

## 7. Amaç

**Bu path:**
- ✅ Determinism kanıtı üretir
- ✅ CI gate çalıştırır
- ✅ Phase-17'nin çekirdeğidir

**Kural:**
> Bu path kırılırsa → tüm Phase-17 başarısız sayılır

---

## 8. Syscall Integration

**Submit:**
```c
// userspace
int64_t execution_id = sys_v2_submit_execution(bcib_buffer, bcib_size);
```

**Wait:**
```c
// userspace
int64_t result = sys_v2_wait_result(execution_id, result_buffer, buffer_size);
```

**Kernel:**
```c
// kernel/sys/syscall_v2.c
int64_t sys_v2_submit_execution(void *bcib, size_t size) {
    execution_slot_t *slot = allocate_execution_slot();
    copy_bcib_to_slot(slot, bcib, size);
    scheduler_enqueue(slot);
    return slot->id;
}

int64_t sys_v2_wait_result(int64_t exec_id, void *buffer, size_t size) {
    execution_slot_t *slot = find_execution_slot(exec_id);
    
    // Wait until COMMITTED
    while (slot->state != SLOT_STATE_COMMITTED) {
        scheduler_yield();
    }
    
    // Copy result
    copy_result_to_userspace(slot, buffer, size);
    return slot->result_size;
}
```

---

## 9. Error Handling

**Execution Error:**
```c
if (execution_failed) {
    slot->state = SLOT_STATE_FAILED;
    slot->error_code = ERROR_EXECUTION_FAILED;
    return -1;
}
```

**Verification Error:**
```c
if (verification_failed) {
    slot->state = SLOT_STATE_FAILED;
    slot->error_code = ERROR_VERIFICATION_FAILED;
    return -1;
}
```

**Terminal State:**
```c
if (slot->state == SLOT_STATE_FAILED) {
    // No retry
    // Evidence logged
    // Userspace notified
}
```

---

## 10. Evidence Generation

**Per Execution:**
```
evidence/run-<RUN_ID>/execution-<EXEC_ID>/
├── bcib_input.bin
├── raw_output.bin
├── raw_output.sha256
├── execution_fingerprint.bin
├── execution_context_snapshot.bin
├── receipt.json
└── markers.log
```

**Kural:**
- Evidence immutable
- Evidence reproducible
- Evidence CI-safe

---

## 11. Performance Baseline

**Minimal Path Metrics:**
```
Execution latency:    < 100ms (target)
Verification latency: < 10ms (STRICT mode)
Verification latency: < 1ms (RELAXED mode)
```

**Kural:**
> Optimization minimal path'i kıramaz

---

## 12. Implementation Checklist

**Phase-17 Başlangıç:**
- [ ] execution_slot.c skeleton
- [ ] State machine implemented
- [ ] Verification mode (STRICT/RELAXED)
- [ ] Commit = publish to userspace
- [ ] Marker logging
- [ ] Evidence generation
- [ ] ci-gate-bcib-determinism v2 (kernel output)

**Kural:**
> Bu checklist tamamlanmadan Phase-17 başlamış sayılmaz

---

## 13. Validation

**Minimal Path Validation:**
```bash
# 1. Build
make KERNEL_PROFILE=validation

# 2. Run
make ci-gate-bcib-determinism

# 3. Verify markers
grep -E '\[EXEC_START\]|\[RESULT_OK\]|\[WAIT_OK\]' evidence/run-*/markers.log

# 4. Verify determinism
diff evidence/run-1/raw_output.sha256 evidence/run-2/raw_output.sha256
```

**Expected:**
- All markers present
- Correct order
- Hash parity

---

## 14. Final Rule

**Minimal Path = Phase-17 Foundation**

> Eğer bu path çalışmıyorsa:
> - Optimization yapma
> - Feature ekleme
> - Performance tuning yapma

**Önce:**
> Minimal path çalışır hale getir

**Sonra:**
> Optimize et

---

**Hazırlayan:** Kenan AY - Architectural Steward  
**Tarih:** 01 Mayıs 2026  
**Versiyon:** 1.0  
**Durum:** CANONICAL

**© 2026 Kenan AY - AykenOS Project**
