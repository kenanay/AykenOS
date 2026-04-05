# Phase-14 Architecture Map

**Version:** 1.0  
**Status:** ACTIVE  
**Date:** 2026-04-03  
**Phase:** 14 — Distributed Observability Hardening  
**Authority:** ARCHITECTURE_FREEZE.md  
**Predecessor:** Phase-13 (OFFICIALLY CLOSED, tag: `phase13-official-closure-confirmed`)

---

## 1. Phase-14 Objective

```
System externalization without violating Phase-13 invariants.
```

Phase-13 answered: **"Is the system correct?"**  
Phase-14 answers: **"How is the system used?"**

---

## 2. Core Rules (Non-Negotiable)

```
service != authority
diagnostics != decision
parity != consensus
trust does not affect verdict
observability does not imply scheduling
```

These rules are inherited from Phase-13 and MUST be preserved throughout Phase-14.

---

## 3. Main Workstreams

### 3.1 Read-Only External API Stabilization (FIRST)

**Objective:** Stabilize the `/diagnostics/*` surface as a versioned, contract-bound external API.

**Scope:**
- API versioning header (`X-Ayken-API-Version`)
- Response schema stability contract
- Endpoint contract documentation
- Client-facing error codes standardization
- `GET /diagnostics/version` — API version surface

**Why first:** All other workstreams depend on a stable API contract. Without this, external consumers cannot rely on the surface.

**Non-goals:**
- Write endpoints
- Authentication/authorization
- Rate limiting

### 3.2 Replay Determinism Stability Hardening

**Objective:** Strengthen replay determinism guarantees.

**Scope:**
- Interrupt ordering nondeterminism analysis
- Replay boundary contract hardening
- Determinism incident classification improvements
- Evidence chain stability under concurrent verification

**Non-goals:**
- Automatic replay execution
- Kernel-side trust enforcement

### 3.3 proofd Query/Service Boundary Hardening

**Objective:** Ensure proofd's query surface remains strictly separated from authority semantics.

**Scope:**
- Query parameter validation hardening (bounded, deterministic)
- Response field audit (PHASE13_FORBIDDEN_FIELDS enforcement)
- Response schema stability contract for service-owned diagnostics responses
- Explicit schema coverage declarations (`none` / `root_only` / `full`) for every public diagnostics endpoint
- Unified contract-driven dispatch for public diagnostics routes (`path -> endpoint_id -> handler -> schema`)
- Service boundary documentation
- Read-only contract enforcement property tests

**Non-goals:**
- New write endpoints
- Authority resolution endpoints

### 3.4 Cross-Node Observability Graph

**Objective:** Add derived observability artifacts for cross-node verification topology.

**Scope:**
- Canonical graph contract and artifact shape
- `GET /diagnostics/runs/{run_id}/graph` — run-scoped contract-bound graph artifact
- `GET /diagnostics/graph` — partitioned derived cross-run graph surface
- `GET /diagnostics/graph/overlay` — overlay-only agreement/conflict/island diagnostics
- Node participation topology (derived, non-authoritative)
- Convergence partition visibility

**Non-goals:**
- Authority topology feedback loop
- Verifier ordering or routing hints
- Reputation scoring

### 3.5 Observability UX (Human-Readable Layer)

**Objective:** Make existing diagnostic data human-readable without changing semantics.

**Scope:**
- `GET /diagnostics/summary` — human-readable system health snapshot
- `GET /diagnostics/runs/{run_id}/summary` — human-readable run-scoped projection
- Structured text output for CLI consumers
- Incident severity labeling (display only)
- Explicit epistemic boundary:
  - `summary_origin = derived`
  - `authority_classification = non_authoritative`
  - `display_mode = human_readable`

**Non-goals:**
- Decision-making based on display labels
- Aggregated scoring
- Ranking, winner selection, or routing hints

---

## 4. Implementation Order

```
3.1 API Stabilization     ← START HERE (unblocks everything)
    ↓
3.3 proofd Boundary       ← Hardening existing surface
    ↓
3.4 Cross-Node Graph      ← New derived surface
    ↓
3.2 Replay Determinism    ← Deep hardening
    ↓
3.5 Observability UX      ← Human layer on top
```

---

## 5. Forbidden Patterns (Phase-14 Kill-Switches)

If any of these appear, Phase-14 has violated Phase-13 invariants:

| Pattern | Violation |
|---------|-----------|
| `trust_score`, `rank`, `weight` in response | Authority drift |
| `preferred_verifier`, `winning_cluster` | Consensus drift |
| POST/PUT/PATCH to `/diagnostics/*` | Service boundary violation |
| Routing decisions based on parity | Observability→control plane |
| Verdict changes based on context | Trust affecting verdict |

---

## 6. Architectural Risks

### 6.1 API Contract Drift
Adding fields without versioning creates silent breaking changes for external consumers.

### 6.2 Observability→Authority Creep
Display labels (severity, health) could be misread as authority decisions. They are not.

### 6.3 Replay Scope Creep
Replay determinism hardening can slide into replay execution if boundary is not held.

### 6.4 Service Semantic Drift
proofd must remain a wrapper over canonical artifacts, not a second semantic engine.

---

## 7. Exit Criteria

Phase-14 is complete when:

1. `/diagnostics/*` surface is versioned and contract-documented
2. Replay determinism hardening evidence committed
3. proofd query boundary property tests passing
4. Cross-node observability graph endpoint operational
5. All Phase-14 gates PASS in `ci-freeze`
6. Official closure tag minted: `phase14-official-closure`
7. Remote CI confirmation obtained
8. `CURRENT_PHASE=15` formal transition executed

---

## 8. References

- `ARCHITECTURE_FREEZE.md` — governing constraints
- `docs/roadmap/CURRENT_PHASE` — formal phase pointer (`CURRENT_PHASE=14`)
- `docs/specs/phase12-trust-layer/PHASE13_ARCHITECTURE_MAP.md` — predecessor map
- `docs/specs/phase14-distributed-observability/README.md` — phase spec
- `docs/specs/phase14-distributed-observability/PROOFD_EXTERNAL_DIAGNOSTICS_CONTRACT_v1.md` — canonical external diagnostics contract
- `docs/specs/phase14-distributed-observability/CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md` — canonical cross-node graph contract
- `reports/phase13_official_closure_candidate/` — Phase-13 closure artifacts
