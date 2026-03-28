# Ring3 Transition Minimal Secure Paging Contract

**Status:** Draft (post-executable-leaf-rule hardening contract)  
**Scope:** `bootloader/efi/*`, `kernel/mm/*`, `kernel/sched/*`, `kernel/arch/x86_64/*`  
**Last Updated:** 2026-03-28

## 1. Purpose

This document defines the minimum paging and permission contract required for the
canonical Ring3 transition path after the executable user-leaf first-fetch rule
is locally closed.

It does **not** replace the active authority surfaces:

- `ci-gate-ring3-user-leaf-rule`
- `ci-gate-ring3-execution-phase10a2`
- broader `Phase10-A2` strict/global evidence
- global/freeze authority requirements

Its purpose is narrower:

1. preserve the dedicated transition-page isolation that now exists
2. prevent a false sense of safety from linker-only section separation
3. define the next hardening wall immediately after executable first-fetch rule closure
4. distinguish local rule enforcement from broader post-A2 hardening work

## 2. Current Repo Reading

The current tree already establishes several useful truths:

1. `ring3_enter_post_cr3` is isolated onto a dedicated page in canonical builds
2. probe and canonical transition lanes are source-separated in assembly
3. the kernel ELF already emits separate `PT_LOAD` segments for executable,
   read-only, and writable regions
4. the executable user-leaf first-fetch rule is locally closed through the
   authoritative chain `P10_TEXT_FRAME_WITNESS -> P10_POST_CR3_TEXT_PROBE -> P10_RING3_USER_CODE`
5. broader historical `Phase10-A2` strict/global authority remains a separate
   pending truth surface
6. scheduler preconditions already fail closed on canonical/user-range checks,
   user RIP/RSP mapping, target-root `rsp0` reachability, and target-root
   canonical transition-page presence
7. scheduler now also fails closed on canonical transition-page noncanonical VA,
   large-page coverage, and active-root vs target-root frame mismatch, while
   emitting a warning if the transition leaf remains writable

The current tree also has a concrete security gap:

1. bootloader paging currently maps all kernel `PT_LOAD` segments with
   `PTE_PRESENT | PTE_WRITABLE`
2. therefore linker/ELF separation alone does not enforce W^X at runtime

## 3. Non-Negotiable Invariants

### 3.1 Canonical Transition Isolation

For canonical builds (`AYKEN_RING3_FETCH_PROBE=0`):

1. `ring3_enter_post_cr3` MUST live on a dedicated 4KB page
2. that page MUST contain only the minimum canonical transition sequence
3. no adjacent kernel text may be required for the canonical post-`CR3` path

The current isolated page is the correct shape:

- `mov %rcx, %cr3`
- `iretq`

### 3.2 Loader Permission Fidelity

Bootloader paging MUST translate ELF `PT_LOAD` flags into real PTE permissions.

Minimum required translation:

1. `r-x` kernel segment -> supervisor, present, read-only, executable
2. `r--` kernel segment -> supervisor, present, read-only, NX
3. `rw-` kernel segment -> supervisor, present, writable, NX

Blanket writable mapping of all `PT_LOAD` segments is forbidden once this
contract lands.

This is a lifecycle rule, not a linker-only rule:

1. early boot may use temporary writable mappings while relocation is in flight
2. post-relocation must apply final segment-faithful permissions
3. steady state must preserve those permissions across identity-map teardown and
   later kernel-root cloning

### 3.3 Transition Page Contract

For the canonical transition page in both the active kernel root and the target
user CR3 root:

1. the transition VA itself MUST be canonical
2. the mapping MUST terminate at a 4KB leaf; 2MB/1GB large-page coverage is
   forbidden
3. `PRESENT=1`
4. `USER=0`
5. `NX=0` on every effective paging level in the walk (`PML4E`, `PDPTE`, `PDE`,
   and final `PTE`)
6. writable bit MUST be clear in the hardened model
7. the final physical frame MUST match between the active kernel root and the
   target root

Any mismatch of the canonical transition-page physical frame between the active
kernel root and the target CR3 root is a fatal architectural violation of the
Ring3 entry contract.

Current fail-closed requirement:

1. presence, `USER=0`, and hierarchical `NX=0` for the canonical transition
   page in both the active kernel root and the target root MUST be enforced at
   runtime
2. canonical-address and 4KB-leaf requirements for that page MUST be enforced
   at runtime
3. active-root vs target-root physical-frame parity for that page MUST be
   enforced at runtime

Post-A2 hardening requirement:

1. writable bit MUST be clear in steady state
2. writable transition-page mappings should remain warning-visible until W^X
   enforcement lands, then become fatal
3. permission drift must not rely on debug evidence alone

### 3.4 Target Root Runtime Preconditions

Before `ring3_enter_iretq` commits the transition:

1. user RIP MUST be canonical and within the user virtual-address range
2. user RSP MUST be canonical, within the user virtual-address range, and must
   not overlap kernel half
3. user RIP page MUST be `PRESENT=1`, `USER=1`, `NX=0`
4. user RSP page MUST be `PRESENT=1`, `USER=1`, `WRITABLE=1`
5. `rsp0` backing MUST be reachable as supervisor writable in the target root
6. the canonical high-half transition page MUST be reachable as supervisor
   executable in the target root

These are runtime invariants, not linker invariants.

### 3.5 Page Granularity Freeze

Until the broader `Phase10-A2` strict authority surface is closed and the secure
paging contract is fully validated:

1. the transition page MUST remain mapped through a 4KB leaf
2. large-page / huge-page coverage for the transition page is forbidden

This is a hardening and diagnosability rule. The current repo already behaves
as a 4KB paging model; this contract freezes that assumption for the transition
surface.

Why this freeze matters:

1. fault locality stays bound to a single leaf page
2. trace outputs remain stable across runs
3. data-read vs instruction-fetch mismatches stay diagnosable at one-page
   granularity

## 4. Probe Build Rules

Probe builds remain diagnostic-only.

1. canonical closure evidence MUST come from `AYKEN_RING3_FETCH_PROBE=0`
2. probe-only low-half aliasing MUST NOT leak into canonical closure claims
3. probe-only mappings may exist for diagnosis, but they are not part of the
   production transition contract

## 5. Deferred but Required Hardening

These are real security follow-ons, but they are not the active A2 blocker:
These are real security follow-ons, but they are not the executable user-leaf
rule itself:

1. boot-time or post-boot write-protect for GDT/IDT/TSS backing
2. dedicated guarded `rsp0` / `IST` stack regions instead of plain `.bss`
3. elimination of any stale-build ambiguity between canonical and probe images
4. explicit SMEP/SMAP policy and proof surface once those controls are enabled

These items should not be confused with the already-closed executable leaf rule, but they
must be addressed before a production-grade claim is credible.

This is the false-confidence breaker:

1. once post-`CR3` continuity works, the system may still be insecure if
   permission-faithful paging is not landed
2. a continuity-only pass is not a production-safe pass while blanket writable
   kernel mappings remain possible

## 6. Required Code Touch Points

Minimum code surfaces this contract affects:

1. `bootloader/efi/paging.c`
   - derive PTE flags from ELF `PT_LOAD` permissions instead of forcing
     writable mappings
2. `bootloader/efi/elf_loader.c`
   - preserve the segment metadata needed for permission-faithful mapping
3. `kernel/mm/paging.c`
   - keep 4KB transition-page semantics explicit
4. `kernel/sched/sched.c`
   - enforce fail-closed checks for canonical transition-page reachability and
     permission semantics in both the active kernel root and the target root
   - panic on active-root vs target-root physical-frame mismatch

## 7. Validation Surface

This contract is not satisfied by source review alone.

Minimum required evidence:

1. `kernel.elf` program headers show separate `r-x`, `r--`, and `rw-` loads
2. boot/runtime page-table evidence shows those permissions survive into the
   actual PTEs
3. canonical transition-page diagnostics prove:
   - same frame in active root and target root
   - supervisor executable
   - not user visible
   - not writable in the hardened model
4. Local executable-leaf rule evidence remains authoritative for the first-fetch boundary:
   - `P10_TEXT_FRAME_WITNESS`
   - `P10_POST_CR3_TEXT_PROBE`
   - `P10_RING3_USER_CODE`
5. Broader `Phase10-A2` strict/global authority may still require separate
   `ci-gate-ring3-execution-phase10a2` evidence.

## 8. Ordering Rule

Apply this work in order:

1. keep the executable user-leaf first-fetch rule closed
2. close or preserve the broader `Phase10-A2` strict surface as required by the branch
3. then land permission-faithful secure paging for the transition path
4. then harden CPU tables and fault stacks

Do not invert this order by treating W^X hardening as the current runtime
blocker. It is the next wall, not the first wall.
