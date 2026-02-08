# PHASE 0 – FOUNDATIONAL OATH
## AykenOS Constitutional Foundation

**Status:** IMMUTABLE  
**Scope:** All phases, all code, all documentation  
**Overrides:** NONE  
**Waiver:** NOT PERMITTED  

---

## 0. PREAMBLE

This document defines the **non-negotiable foundational oath** of AykenOS.

Phase 0 is **not a phase in the roadmap**.  
It is the **constitutional ground** upon which all phases exist.

If any document, phase, implementation, test result, or decision
conflicts with this oath, **Phase 0 prevails without exception**.

---

## 1. MECHANISM OVER POLICY

Ring0 SHALL contain **mechanisms only**.

Policy, heuristics, preferences, intelligence, and decision logic
MUST reside in Ring3 or higher abstractions.

No exception, temporary or permanent, is permitted.

---

## 2. EVIDENCE OVER CLAIMS

No phase is considered complete without **deterministic, reproducible evidence**.

- Logs > screenshots  
- Deterministic PASS > single successful run  
- Reproducibility > narrative explanation  

A phase MAY be functionally complete and still remain **OPEN**.

---

## 3. FAILURE IS A FIRST-CLASS SIGNAL

A failing test, missing log, or inconclusive result is **not noise**.

Failure SHALL:
- Block phase closure
- Be recorded, not hidden
- Take precedence over timelines

Silencing failure is a constitutional violation.

---

## 4. RING BOUNDARIES ARE SACRED

Ring transitions are **architectural contracts**, not conveniences.

- Ring0 → mechanism only
- Ring3 → policy and intelligence
- No shortcut is allowed, even for performance or convenience

Any Ring violation invalidates the affected phase.

---

## 5. EXECUTION-CENTRIC DESIGN

AykenOS prioritizes **execution semantics**, not compatibility.

- Syscalls exist to enable execution, not APIs
- Interfaces are minimal by design
- Backward compatibility is never mandatory

Clarity of execution outweighs ecosystem expectations.

---

## 6. DATA OVER INTERPRETATION

Data structures define truth.

- ABDF / BCIB are authoritative
- Interpretation layers are replaceable
- No hidden state is acceptable

If behavior cannot be explained by data, it is invalid.

---

## 7. NO IMPLICIT MAGIC

All behavior MUST be:
- Traceable
- Observable
- Loggable

Implicit behavior is considered a design failure.

---

## 8. DETERMINISM BEFORE PERFORMANCE

Performance optimizations are invalid
if they reduce determinism or auditability.

Correctness > Determinism > Performance  
Always in this order.

---

## 9. GOVERNANCE IS PART OF THE SYSTEM

Governance is not external tooling.

- Rules are enforced by code
- CI is a constitutional actor
- Audits are continuous, not events

Manual trust is not accepted as a substitute.

---

## 10. WAIVERS HAVE LIMITS

Waivers may exist **below Phase 0 only**.

Phase 0:
- Accepts no waivers
- Expires never
- Is not subject to renewal

Any attempt to bypass Phase 0 invalidates the decision.

---

## 11. DOCUMENTATION IS A CONTRACT

Documentation SHALL reflect reality.

- Outdated docs are defects
- Historical documents MUST be marked clearly
- Ambiguity is treated as misinformation

Narrative without evidence is not documentation.

---

## 12. PHASES ARE PERMISSIONED, NOT ENTITLED

A phase may begin **only if** prior obligations are met.

Progression is earned through evidence, not intent.

Skipping a phase is equivalent to failing it.

---

## 13. AI IS A GUEST, NOT AN AUTHORITY

AI systems:
- Assist
- Analyze
- Suggest

They do NOT decide, override, or legitimize violations.

Human accountability is never delegated.

---

## 14. THIS OATH IS FINAL

Phase 0 may only be changed if:
- The entire system is explicitly re-founded
- All prior phases are invalidated
- A new oath is declared openly

Incremental modification is forbidden.

---

## DECLARATION

This oath binds:
- All contributors
- All code
- All tools
- All phases
- All future extensions

AykenOS exists **only while this oath holds**.

---

**Declared:** 6 February 2026  
**Authority:** Project Owner  
**Status:** In Force
