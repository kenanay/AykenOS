# Faz 2 - AI Skeleton
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

Opcode stub:
- ai.ask <prompt>: executor handler logs prompt and returns stub response; no model call

Interfaces (future-proofing):
- trait AIInterpreter { fn interpret(prompt: &str) -> Result<String, Error>; }
- Shell LLM placeholder: parse natural language -> DSL/BCIB (not implemented)
- Data/Hardware agents: reserved; may use params like type=data/hw later

Runtime hooks:
- Allow enabling/disabling AI mode in CLI; if AI mode and no backend, emit friendly stub
- Threading note: keep sync for now; plan async thread for real models later

Artifacts:
- Document opcode contract, stub output text, and integration point with executor
