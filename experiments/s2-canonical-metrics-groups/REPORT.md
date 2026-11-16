# Canonical Metrics Groups — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-metrics-groups`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
coverage                     legacy=5 canonical=9 total=10
regenerative_capacity_days   legacy=20 canonical=20.0
yield 1.0/0.8/0.5            legacy 60/60/50, canonical 60/60/50
recharge average 2.50 kW      legacy threshold 2.6, canonical threshold 2.6
```

## 2. Classification

- regenerative capacity: `EXPECTED_AGREEMENT` (20 days)
- yield sensitivity: `EXPECTED_AGREEMENT` (60/60/50)
- recharge boundary: `EXPECTED_AGREEMENT` (2.6 kW)
- recovery coverage: `SEMANTIC_CORRECTION` — legacy 5/10 counts asset
  restorability; canonical 9/10 counts whether the service continues. The four
  differing cases are non-service assets `bat-1`, `bus-1`, `charger-1`,
  `robot-m1`, which cannot be repaired but also do not stop potable water.
  These are different metric definitions, not a numerical error.

## 3. Consequence

The aggregate metrics certificate remains BLOCKED because its `recovery_coverage`
field still means the old 5/10 definition. A canonical metrics certificate must
version the definition; the other three groups are ready.
