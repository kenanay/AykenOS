# Phase-16 BCIB / ABDF Isolation & Boundary Requirements

**Belge Türü**: Normatif  
**Faz**: Phase-16  
**Durum**: DRAFT  
**Kapsam**: BCIB execution isolation + ABDF immutability + BCIB↔ABDF boundary

---

## 0. Temel İlkeler

- **Execution ≠ Data**
- **BCIB ≠ ABDF**
- **Isolation is mandatory**
- **All violations are fail-closed**

---

## 1. BCIB Isolation Contract Requirements

### 1.1 Execution Boundary

- THE BCIB Executor **SHALL** yalnızca Ring3'te çalışmalıdır.
- THE BCIB Executor **SHALL NOT** Ring0'a policy, instruction veya execution semantics taşıyamaz.
- THE BCIB Executor **SHALL** kernel ile yalnızca `SYS_V2_SUBMIT_EXECUTION` üzerinden iletişim kurmalıdır.
- THE BCIB Executor **SHALL NOT** syscall yüzeyini genişletemez (ABI freeze).

### 1.2 Syscall & Driver Isolation

- THE BCIB **SHALL NOT** invoke arbitrary syscalls.
- THE BCIB **SHALL NOT** access DevFS directly.
- THE BCIB **SHALL NOT** invoke drivers directly.
- THE BCIB **SHALL NOT** perform MMIO, IRQ veya I/O port işlemleri.

**IF** yukarıdaki ihlallerden biri gerçekleşirse **THEN** sistem **SHALL** `BCIB_ERR_ISOLATION_VIOLATION` ile fail-closed sonlandırmalıdır.

### 1.3 Memory Isolation

- THE BCIB **SHALL NOT** access raw memory pointers.
- THE BCIB **SHALL NOT** observe kernel memory addresses.
- THE BCIB **SHALL** operate only on bounded memory regions.

### 1.4 Input / Output Contract

- THE BCIB input buffer **SHALL** be read-only.
- THE BCIB output **SHALL** be bounded and pre-declared.
- THE BCIB **SHALL NOT** allocate unbounded memory.

**IF** buffer sınırı ihlal edilirse **THEN** `MEMORY.CONTRACT.VIOLATION` üretilmelidir.

### 1.5 Execution Sandbox

- THE BCIB **SHALL** execute within a sandboxed execution runtime.
- THE BCIB **SHALL NOT** escape execution context.
- THE BCIB **SHALL NOT** access external state without declared permission.

### 1.6 Side-Effect Control

- THE BCIB **SHALL** declare all side-effects before execution.
- THE BCIB **SHALL** classify instructions:
  - `pure`
  - `data-mutating`
  - `external`
- THE BCIB **SHALL** require capability for:
  - `data-mutating`
  - `external` operations

**IF** undeclared side-effect gerçekleşirse **THEN** yürütme fail-closed sonlandırılmalıdır.

---

## 2. ABDF Immutability Contract Requirements

### 2.1 Data Model

- THE ABDF **SHALL** be the authoritative data substrate.
- THE ABDF **SHALL** own all persistent data.
- THE BCIB **SHALL NOT** define its own data storage model.

### 2.2 Immutability

- THE ABDF objects **SHALL** be immutable during BCIB execution.
- THE ABDF **SHALL NOT** allow in-place mutation.

### 2.3 Update Model

- THE ABDF mutation **SHALL** follow one of:
  - new object creation
  - append-only update
- THE ABDF **SHALL** preserve previous state (no overwrite).

### 2.4 Concurrency Model

- THE ABDF **SHALL** forbid concurrent mutable access.
- THE ABDF **SHALL** allow concurrent read-only access.

### 2.5 Ownership Model

- THE ABDF **SHALL** define:
  - `producer`
  - `reader`
  - `revoker`
- THE BCIB **SHALL NOT** become data owner.

### 2.6 Snapshot Guarantee

- THE ABDF **SHALL** provide snapshot consistency.
- THE ABDF **SHALL** guarantee deterministic read view.

### 2.7 Handle Enforcement

- THE ABDF **SHALL** expose data only via `ABDF_Handle`.
- THE ABDF **SHALL NOT** expose raw pointers.

### 2.8 Handle Lifecycle

- THE ABDF handles **SHALL** be context-bound.
- THE ABDF **SHALL** support handle revocation.

**IF** revoked handle kullanılırsa **THEN** `BCIB_ERR_ABDF_HANDLE_REVOKED` döndürülmelidir.

---

## 3. BCIB ↔ ABDF Boundary Requirements

### 3.1 Access Model

- THE BCIB **SHALL** access ABDF only via handles.
- THE BCIB **SHALL NOT** bypass ABDF interface.

### 3.2 Segment Model

ABDF segment türleri **SHALL** tanımlı olmalıdır:

- `Input`
- `Event`
- `DeviceStatus`
- `ReadResult`
- `ExecutionResult`
- `ExecutionTrace`
- `Ref`

### 3.3 Capability Enforcement

- THE BCIB **SHALL** require capability for ABDF access.
- THE ABDF **SHALL** enforce capability validation.

### 3.4 Boundary Enforcement

- THE BCIB **SHALL NOT** store data outside ABDF.
- THE BCIB **SHALL NOT** modify ABDF structure.

**IF** boundary ihlali oluşursa **THEN** `ABDF_BOUNDARY_VIOLATION` ile fail-closed yapılmalıdır.

### 3.5 Cross-Context Rules

- THE BCIB **SHALL NOT** access another context's ABDF handle.
- THE BCIB **SHALL** require explicit capability for cross-context access.

### 3.6 Stale Handle Behavior

- THE BCIB **SHALL** reject stale handles.
- THE BCIB **SHALL NOT** reuse invalid handles.

---

## 4. Failure Modes

Aşağıdaki durumlar sistemin çalışmasını engeller:

- BCIB sandbox escape
- ABDF mutability ihlali
- Raw pointer erişimi
- Capability bypass
- Driver direct access
- Syscall misuse

Bu durumlarda sistem:

👉 **fail-closed + deterministic termination**

---

## 5. Property-Based Test Requirements

Minimum testler:

- Execution isolation (context separation)
- ABDF immutability
- Handle revocation
- Boundary violation detection
- Deterministic execution
- Fail-closed enforcement

---

## 6. Enforcement & CI Gates

Aşağıdaki gate'ler zorunludur:

- `ci-gate-bcib-isolation`
- `ci-gate-abdf-immutability`
- `ci-gate-boundary-enforcement`
- `ci-gate-determinism`

---

## 7. Final Invariant

```
BCIB = sandboxed execution
ABDF = immutable data
Boundary = strictly enforced
```

---

## SONUÇ

Bu belge ile:

- ✅ BCIB kontrol altına alındı
- ✅ ABDF güvenli hale getirildi
- ✅ Sistem bütünlüğü garanti altına alındı
