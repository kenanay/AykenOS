# Phase-21 First Bounded Receipt Schema

This document describes the static shape of a possible later receipt for the
Phase-21 first bounded implementation skeleton.

The schema is documentation only. It is not evidence acceptance, proof,
package acceptance, source acceptance, runtime authority, trust authority,
registry authority, capability authority, or source merge authority.

## Static Shape

A future receipt may include:

1. `subject_sha`: exact SHA under review.
2. `changed_files`: exact changed-file list.
3. `category_mapping`: file-to-category mapping.
4. `non_executing_boundary`: recorded non-execution statement.
5. `denied_authorities`: denied authority list.
6. `evidence_runs`: exact CI run identifiers, if later accepted.
7. `result`: later review result, if separately authorized.

## Denials

Receipt schema presence does not:

1. Accept evidence.
2. Issue proof.
3. Accept a package.
4. Accept source.
5. Authorize code execution.
6. Authorize package loading.
7. Assign trust.
8. Publish registry entries.
9. Grant source merge authority.

Unknown authority readings fail closed.
