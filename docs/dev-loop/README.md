# Dev Loop Documentation

**Location**: `docs/dev-loop/`  
**Spec Location**: `.kiro/specs/dev-loop-boot-monitoring/`  
**Config Location**: `.kiro/config/dev-loop-boot-monitoring.config.kiro`

---

## Purpose

This directory contains **implementation guides, integration documentation, and historical reports** for the Ayken Development Loop & Boot Monitoring System.

**This is NOT the spec.** The authoritative specification is located at:
```
.kiro/specs/dev-loop-boot-monitoring/
├── requirements.md          # Source of truth for acceptance criteria
├── design.md                # Architectural decisions and rationale
├── tasks.md                 # Implementation task breakdown
├── DEV_LOOP_CONSTITUTION.md # Immutable constitutional rules
└── GOVERNANCE.md            # Enforcement mechanisms
```

---

## Document Classification

### 📘 Implementation Guides (How-To)

**CI_INTEGRATION.md**
- GitHub Actions workflow setup
- Auto-bisect configuration
- Branch protection rules
- Artifact management
- **Audience**: DevOps, CI maintainers

**PERFORMANCE_INTEGRATION.md**
- Performance regression detection
- Baseline management
- Quick vs full mode trade-offs
- **Audience**: Performance engineers, developers

---

### 📊 Historical Reports (Transient)

**CONSISTENCY_FIX_REPORT.md**
- Spec consistency fixes from 2026-05-03
- Task renumbering rationale
- Naming policy clarification
- **Audience**: Spec maintainers, auditors
- **Status**: Historical record, not normative

---

## Navigation

### If you want to...

**Understand what the dev loop does**  
→ Read `.kiro/specs/dev-loop-boot-monitoring/requirements.md`

**Understand how it's designed**  
→ Read `.kiro/specs/dev-loop-boot-monitoring/design.md`

**Implement the dev loop**  
→ Follow `.kiro/specs/dev-loop-boot-monitoring/tasks.md`

**Set up CI integration**  
→ Read `docs/dev-loop/CI_INTEGRATION.md` (this directory)

**Integrate performance checks**  
→ Read `docs/dev-loop/PERFORMANCE_INTEGRATION.md` (this directory)

**Understand constitutional rules**  
→ Read `.kiro/specs/dev-loop-boot-monitoring/DEV_LOOP_CONSTITUTION.md`

**Understand governance enforcement**  
→ Read `.kiro/specs/dev-loop-boot-monitoring/GOVERNANCE.md`

---

## Architectural Separation

```
┌─────────────────────────────────────────────────────────────┐
│                    SPEC (Immutable Truth)                    │
│              .kiro/specs/dev-loop-boot-monitoring/          │
│                                                              │
│  • requirements.md  → WHAT must be built                    │
│  • design.md        → WHY and HOW (architecture)            │
│  • tasks.md         → Implementation breakdown              │
│  • CONSTITUTION.md  → Immutable rules                       │
│  • GOVERNANCE.md    → Enforcement mechanisms                │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                DOCS (Implementation Guides)                  │
│                    docs/dev-loop/                           │
│                                                              │
│  • CI_INTEGRATION.md         → How to set up CI            │
│  • PERFORMANCE_INTEGRATION.md → How to integrate perf      │
│  • CONSISTENCY_FIX_REPORT.md  → Historical fixes           │
└─────────────────────────────────────────────────────────────┘
```

---

## Why This Separation?

### ✅ Benefits

1. **Spec remains pure**
   - No "how-to" contamination
   - Clear acceptance criteria
   - Deterministic validation

2. **CI references are stable**
   - Spec files don't change for implementation details
   - Governance checks reference immutable rules

3. **Onboarding is faster**
   - New developers: read spec first
   - Implementers: read docs second
   - Clear hierarchy of authority

4. **Governance is stronger**
   - Spec = constitutional authority
   - Docs = interpretation and guidance

### ❌ Without Separation

- Spec becomes "explanation document"
- Tasks drift from implementation
- New contributors confused about authority
- Governance weakens over time

---

## Maintenance

### Adding New Documentation

**If it's a spec change** (requirements, design, constitutional rules):
→ Update `.kiro/specs/dev-loop-boot-monitoring/`

**If it's implementation guidance** (CI setup, integration, how-to):
→ Add to `docs/dev-loop/`

**If it's a historical report** (fixes, migrations, audits):
→ Add to `docs/dev-loop/` with clear date and status

### Updating Existing Docs

**Spec files** (requirements.md, design.md, tasks.md):
- Require architectural review
- Follow spec amendment process
- Update CONSISTENCY_FIX_REPORT.md if major changes

**Doc files** (CI_INTEGRATION.md, PERFORMANCE_INTEGRATION.md):
- Can be updated freely for clarity
- No architectural review needed
- Keep aligned with spec

---

## Cross-References

### From Spec to Docs

Spec files MAY reference docs for implementation details:
```markdown
For CI integration instructions, see docs/dev-loop/CI_INTEGRATION.md
```

### From Docs to Spec

Doc files MUST reference spec as authority:
```markdown
This guide implements requirements from .kiro/specs/dev-loop-boot-monitoring/requirements.md
```

---

## Related Documentation

- **Setup Guides**: `docs/setup/`
- **Development Guides**: `docs/development/`
- **Governance**: `docs/governance/`
- **Operations**: `docs/operations/`

---

**Last Updated**: 2026-05-03  
**Maintainer**: Kenan AY — System Architect
