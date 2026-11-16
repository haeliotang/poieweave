# Canonical Redundancy Restorers — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-redundancy-restorers`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
filter-redundant-only              30 -> 30
filter-redundant+filter-restorer   90 -> 90
robot-redundant-only               35 -> 35
robot-redundant+robot-restorer     90 -> 90
redundancy-filter-baseline         30 -> 30
redundancy-filter-redundant-pump   30 -> 30
redundancy-robot-baseline          30 -> 20
redundancy-robot-redundant-pump    35 -> 35
```

7/8 rows agree. The single difference is `redundancy-robot-baseline`: legacy
detects the shared repair robot failure only when the next repair is attempted
(day 30); canonical applies the robot failure at event time (day 20) and blocks
because there is no robot restorer.

## 2. Classification

`SEMANTIC_CORRECTION_REPAIR_EXECUTOR_AT_EVENT_TIME`

## 3. Consequence

`redundancy` moves from PARTIAL to canonical with this event-time correction.
