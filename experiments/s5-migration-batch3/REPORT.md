# S5 Migration Batch 3 — report

- Date: 2026-11-15
- Status: load/resupply/battery boundaries checked
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-batch3`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
scenario	canonical_horizon_h	legacy_horizon_h	canonical_energy_kwh	classification
load-repair	5.0	5.0	4.5	load-integrates
resupply-before-arrival	0.0	0.0	40.0	blocked:[MissingResource { resource: "grease", needed: 0.05, available: 0.0 }]
initial-overcapacity	0.0	0.0	10.0	canonical_reject=true,legacy_reject=true
```

## 2. Findings

- 3B load: both paths reach 5 h; canonical final energy is 4.5 kWh after 5 kWh
  load and 0.5 kWh repair consumption.
- 3A resupply: both block at t=0 with missing grease, so a later delivery is not
  usable retroactively.
- 3C battery: both reject initial energy above capacity.

## 3. Classification

- No unexplained divergence in these three cases.
- Joint repair + resupply + load + generation ledger (3D) is not yet covered.

## 4. Limits

- Three scenarios; no 3D joint ledger; no discrete resupply overcapacity trace.
