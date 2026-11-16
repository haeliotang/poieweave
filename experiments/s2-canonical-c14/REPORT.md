# S2 Canonical C14 Migration — report

- Date: 2026-11-16
- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c14`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode            instant  event  canonical  energy  termination                         classification
baseline        12       31     2          160.0   interrupted at day 2             SEMANTIC_CORRECTION_IN_REPAIR_FAILURE
redundant-pump  90       90     90         200.0   completed                        EXPECTED_AGREEMENT
```

## 2. Findings

- Redundant pump: 90 days in all three semantics, final energy 200 kWh.
- Baseline: canonical blocks at day 2 because a pump failure occurs during the
  5-day repair; legacy event-driven defers failures and reaches 31 days.
- The baseline 31-day value is superseded by canonical semantics.

## 3. Classification

- `redundant-pump`: `EXPECTED_AGREEMENT`.
- `baseline`: `SEMANTIC_CORRECTION_IN_REPAIR_FAILURE`.

## 4. Limits

- One redundancy layout; model-level units.
