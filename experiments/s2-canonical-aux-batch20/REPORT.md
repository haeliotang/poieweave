# Canonical Auxiliary Batch 20 — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch20`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
recycle-residue bar/residue 1.2/0, 1.0/0.2, 0.9/0.3, 0.6/0.6 -> 60/60/60/50
adversarial-rand 500 trials, seed 0x9e3779b97f4a7c15
  legacy    zero=2 under30=103 under60=215 mean=60.3 min=0.0
  canonical zero=0 under30=109 under60=233 mean=57.7 min=0.1
```

## 2. Finding

Recycle residue agrees exactly. The random distribution changes because t=0
events are deferred until the initial repair completes: no zero-day outcome
remains, and the lower tail is replaced by 0.125-day horizons.

## 3. Classification

- recycle-residue: `EXPECTED_AGREEMENT`
- adversarial-rand: `SEMANTIC_CORRECTION_TIME_DISTRIBUTION`

## 4. Remaining

Only `metrics` (definitional) and `order-sensitivity` (negative) remain; they are
not numeric survival migration targets.
