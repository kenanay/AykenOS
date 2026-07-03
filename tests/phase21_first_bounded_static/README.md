# Phase-21 First Bounded Static Tests

This directory contains non-runtime static tests for the Phase-21 first
bounded implementation skeleton.

The tests are not package acceptance, implementation acceptance, runtime
execution, process start authority, runtime state authority, package loading
authority, source acceptance, or source merge authority.

## Boundary

Tests in this directory must not:

1. Boot runtime behavior.
2. Start a process.
3. Create runtime state.
4. Install, load, or execute packages.
5. Load modules.
6. Mount workspaces.
7. Instantiate plugins.
8. Issue capabilities.
9. Publish registry entries.
10. Assign trust.

Any runtime test reading fails closed.
