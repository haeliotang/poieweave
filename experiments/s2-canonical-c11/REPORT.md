# S2 Canonical C11 Migration — report

- Date: 2026-11-16
- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c11`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
at_day  legacy_no  legacy_yes  canonical_no  canonical_yes  legacy_minimal  canonical_minimal  classification
0       0.000      0.000       0.125         0.125          filter-1        filter-1           SEMANTIC_CORRECTION_REPAIR_TIME_AT_ZERO
10      0.000      0.000       10.000        10.000         filter-1        filter-1           SEMANTIC_CORRECTION_LEGACY_IGNORED_AT_DAY
```

## 2. First divergence and ordering fix

Initial run: day-0 minima agreed at 0, but minimal sets differed
(`filter-1` vs `pump-p1`). Cause: a t=0 failure was processed while the initial
repair was running, interrupting it.

Fix: t=0 events are deferred until the initial repair completes. This defines
the order as **initial restoration, then t=0 external events**. After the fix:

- day-0 minimal set: `filter-1` in both paths
- canonical minimum is 0.125 d (3 h) because the filter failure is processed
  after the 3 h initial pump repair; legacy instant semantics reports 0
- day granularity agrees (0 = 0)

## 3. Legacy day-10 bug

The legacy runner loops over `at_day` but its `run()` function always places the
subset failures at t=0. Therefore the expected day-10 row (0 days) is not a
valid day-10 result. Canonical day-10 gives 10 days, which is the causal minimum.

## 4. Decision

- C11 is **migrated with semantic correction** at day granularity.
- Day-0 exact minimum changes 0 → 0.125 d because repair time is now causal;
  minimal set is unchanged (`filter-1`).
- Day-10 legacy row is invalid (`SEMANTIC_CORRECTION_LEGACY_IGNORED_AT_DAY`);
  canonical gives 10 d.
- A certificate for C11 must bind the day-granularity claim and the canonical
  ordering, not the old 0-day day-10 row.

## 5. Limits

- Minimal-set selection is by shortest label among equal minima; this is a
  deterministic convention, not a unique mathematical fact.
