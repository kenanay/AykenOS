# Faz 2 - ABDF/BCIB v0.2 Plan
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

Goals:
- Stabilize ABDF and BCIB to v0.2, add UI/GPU types, expand opcode set, lock arg/encoding rules.

ABDF updates:
- Types: UiScene (id, size, bg_color, root_widget_ref), UiWidget (type, layout, style, children indices), GpuBuffer (usage, size, format).
- Keep no-pointer rule; use indices/offsets; honor alignment rules set in Faz1 (little-endian fields).
- String pool: reuse for widget text and names; document max sizes and alignment.

BCIB updates:
- Header: magic BCIB, version=0.2, lengths for code and pools.
- Opcode set (planned):
  - data.create, data.add, data.query, data.update?, data.delete? (update/delete optional)
  - ui.render (real), ui.event (stub), ctx/select helpers if needed
  - ai.ask (stub), sys.info (optional)
  - control: end/halt marker
- Args: fixed-size prefix where possible; strings by pool offset; numeric args little-endian; variable arg count encoded with length byte/word.

Validation rules:
- Reject unknown opcode, version mismatch, or malformed offsets.
- Alignment: instructions packed; multi-byte args aligned to natural size or explicit padding rule (document chosen rule; default packed LE).

Outputs:
- Spec markdown for ABDF/BCIB v0.2
- Updated opcode enum list + argument table
- Example encoded sequences for data.query and ui.render
