# Canonical C68 Monte Carlo — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c68`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
series     min p10 median mean p90 max survived_90
legacy     20  28  72     64.8 90  90  622
canonical  20  28  68     64.0 90  90  611
```

Buckets shift by at most 31 runs (`[0,40)`: 403 -> 434; survived 90:
622 -> 611). Min, p10, p90 and max are unchanged.

## 2. Classification

`SEMANTIC_CORRECTION_IN_REPAIR_FAILURE`

The same failure schedules are evaluated at canonical event time; in-repair
failures shorten some runs. The distribution is shifted, not collapsed.

## 3. Consequence

C68 is no longer legacy-only. The historical distribution remains recorded;
the canonical distribution supersedes it for S5-T evidence.
