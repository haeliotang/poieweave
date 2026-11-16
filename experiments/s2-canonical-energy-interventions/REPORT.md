# Canonical Energy Interventions — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-energy-interventions`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
capacity/charge 100–300 kWh x 3/10/30 kW at 2.6 kW: legacy 16 d, canonical 16.25 d
recharge 20/30/40 kWh at 2.6 kW: legacy 16/60/60 d, canonical 16.25/60/60 d
```

All 12 rows agree at day granularity. Energy-limited rows shift by 0.25 d
because canonical schedules the full repair chain.

## 2. Classification

`SEMANTIC_CORRECTION_REPAIR_SCHEDULE` (day-level invariant, exact time shifted).

## 3. Consequence

`energy_interventions` can move from PARTIAL to canonical with the versioned
day-granularity scope. `redundancy` remains partial.
