# S5-V N8-H — Temporal Alignment & Trajectory Evidence Hardening

Baseline: `c038ba82bc09dc7d15c1734b7a9bcbf9ed65d284`.

## Scope

- Reproduce three false-PASS defects in the N8 comparator.
- Strict exact-time alignment with explicit `NOT_COMPARABLE`.
- Observation-status filtering (never convert UNOBSERVABLE to zero).
- Independent energy assumption decomposition and conservation checks.
- Revalidate N8-01/N8-02/N8-04 without modifying old N8 evidence.
- Exact event-time splitting for N8-02 (`failure=4500`, nominal step 1000).
- Preserve v1/v2/v3/v4 certificates and canonical runtime semantics.

## Not claimed

- N5/N6 resource lifecycle cross-model trajectory
- general trajectory-level cross-model validity
- full S2 or physical-world correctness
