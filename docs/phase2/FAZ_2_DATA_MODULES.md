# Faz 2 - Data Modules

Goal: minimal handlers per ABDF type for executor.

Tabular:
- create schema, add rows, query with simple filter (single column comparisons; optional AND)
- output table formatter; basic count/avg optional

Text:
- store text blocks; query by substring match; output matched lines/snippets

Log:
- parse timestamp/level/message if present; filter by level or time window; tail N

Vector/Tensor:
- hold numeric arrays; expose metadata (shape, dtype); allow slice/read; math ops deferred

UI Scene/Widget:
- load scene + widgets via ABDF decoder; provide tree to renderer

GpuBuffer:
- placeholder struct; support create/register; no actual upload in Faz 2

Common:
- registries keyed by name/id; zero-copy reads where possible; safe bounds checks
