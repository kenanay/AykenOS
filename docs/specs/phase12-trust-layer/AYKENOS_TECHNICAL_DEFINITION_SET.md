# AykenOS Technical Definition Set

**Version:** 1.0
**Status:** Informational definition set
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative reference artifact
**Related Spec:** `README.md`, `AYKENOS_ARCHITECTURE_ONE_PAGE.md`, `AYKENOS_RESEARCH_POSITIONING.md`, `AYKENOS_UNIQUE_ARCHITECTURAL_DECISIONS.md`, `DISTRIBUTED_VERIFICATION_SYSTEMS_PAPER_OUTLINE.md`, `DISTRIBUTED_VERIFICATION_SYSTEMS_PAPER.md`

---

## 1. Purpose

This document defines the canonical short, medium, and full technical descriptions of AykenOS.

It exists to keep one rule explicit:

`different communication lengths != different system definition`

The goal is to let README, architecture notes, research positioning, paper drafts, comparison tables, and presentations describe the same system without terminology drift.

---

## 2. One-Sentence Definition

AykenOS is a deterministic verification architecture that separates kernel execution, verification semantics, artifact-based truth surfaces, and distributed diagnostics without introducing consensus or authority arbitration.

Recommended uses:

- paper abstract
- conference slide
- system comparison table
- README introduction

---

## 3. Three-Sentence Definition

AykenOS is a deterministic verification architecture built around explicit separation between execution, verification, evidence artifacts, and distributed diagnostics. The kernel provides execution mechanisms while userspace verification services produce artifact-bound receipts and verification verdicts. Distributed observability surfaces such as parity, topology, and incident graphs expose cross-node verification behavior without elevating diagnostics into authority, consensus, or truth election.

Recommended uses:

- paper introduction
- research positioning
- system overview section

---

## 4. Canonical Paragraph Definition

AykenOS is a deterministic verification architecture that separates kernel execution, verification semantics, evidence artifacts, and distributed diagnostics into explicit layers. The kernel provides mechanism, userspace verification services produce artifact-bound verdicts and receipts, and parity/topology surfaces expose cross-node observability without elevating diagnostics into authority or consensus. In this model, artifacts are the canonical truth interface, services wrap canonical artifacts, and distributed verification scales through evidence-first observability rather than truth election or replicated-state consensus.

Recommended uses:

- research paper
- architecture documents
- system specification
- formal positioning notes

---

## 5. Usage Guidance

Use the one-sentence form when space is constrained.

Use the three-sentence form when a compact technical overview is needed.

Use the canonical paragraph when AykenOS is being defined normatively for architecture, research, or positioning purposes.

Synchronization rule:

- the canonical paragraph is the primary reference definition
- the one-sentence and three-sentence forms must remain semantically aligned with it
- if the canonical paragraph changes, all three forms must be reviewed together

---

## 6. Why This Set Exists

This set gives AykenOS three stable communication layers:

- rapid system definition
- compact technical overview
- full canonical architectural definition

The intended result is:

`AykenOS architecture language = stable across README, architecture, research, and paper surfaces`
