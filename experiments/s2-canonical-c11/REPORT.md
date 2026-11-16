# S2 Canonical C11 Migration — report

- Date: 2026-11-16
- Status: BLOCKED
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c11`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
at_day  legacy_no  legacy_yes  canonical_no  canonical_yes  legacy_minimal  canonical_minimal  classification
0       0          0           0             0              filter-1        pump-p1            EXPECTED_MINIMA_ONLY
10      0          0           10            10             filter-1        filter-1           SEMANTIC_CORRECTION_LEGACY_IGNORED_AT_DAY
```

## 2. First divergence

At day 0, the minima agree (0), but the minimal sets differ:

- legacy: `filter-1`
- canonical: `pump-p1`

The subset `pump-p1` yields 0 in canonical because a failure event at t=0 is
processed while the initial repair is still running, so the repair target is
re-failed and the repair is interrupted at t=0. Legacy performs initial service
restoration before event processing, so the same subset survives longer.

This is a t=0 ordering ambiguity, not yet resolved.

## 3. Legacy day-10 bug

The legacy runner loops over `at_day` but its `run()` function always places the
subset failures at t=0. Therefore the expected day-10 row (0 days) is not a
valid day-10 result. Canonical day-10 gives 10 days, which is the causal minimum.

## 4. Decision

- `BLOCKED`; C11 is not migrated.
- Day-10 legacy output is `SEMANTIC_CORRECTION` due to the legacy bug.
- Day-0 minimal-set mismatch is `UNEXPLAINED` until initial-repair/t=0 ordering is
  defined and tested.
- No certificate may be reissued for C11.

## 5. Limits

- Minimal-set selection is by shortest label among equal minima; this is a
  deterministic convention, not a unique mathematical fact.
