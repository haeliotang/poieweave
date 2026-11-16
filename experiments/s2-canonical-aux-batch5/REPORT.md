# Canonical Auxiliary Batch 5 — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch5`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
failure-envelope  legacy 5/10  canonical 9/10   SEMANTIC_CORRECTION_COVERAGE_DEFINITION
failure-pairs     legacy 11/46 canonical 33/46  SEMANTIC_CORRECTION_COVERAGE_DEFINITION
regen-threshold   max_safe 60   first_blocked 61  EXPECTED_AGREEMENT
```

## 2. Findings

- Coverage counts change because legacy counts asset restorability and canonical
  counts whether the service continues. The same non-service assets as C71
  (`bat-1`, `bus-1`, `charger-1`, `robot-m1`) explain the single-failure gap.
- The regeneration threshold remains exactly 60 safe / 61 blocked.

## 3. Classification

- coverage: `SEMANTIC_CORRECTION_COVERAGE_DEFINITION`
- threshold: `EXPECTED_AGREEMENT`

## 4. Limits

Sample only; 46 auxiliary experiments remain uncanonicalised.
