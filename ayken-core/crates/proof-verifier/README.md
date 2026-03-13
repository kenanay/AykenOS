# proof-verifier

Deterministic, userspace/offline proof verification engine for AykenOS Phase-12.

Current milestone:
- P12-07 crate skeleton
- library-first verification pipeline
- portable core and trust overlay boundaries
- fail-closed scaffold for later cryptographic hardening

This crate does not implement networking, service supervision, or Ring0 integration.

Planned module boundaries:
- `canonical/`
- `bundle/`
- `portable_core/`
- `overlay/`
- `registry/`
- `policy/`
- `verdict/`
- `receipt/`

