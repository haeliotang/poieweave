# S5 Migration Batch 3 — report

- Date: 2026-11-15
- Status: load/resupply/battery boundaries checked
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-batch3`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
scenario	canonical_horizon_h	legacy_horizon_h	canonical_energy_kwh	legacy_energy_kwh	classification
load-repair	5.0	5.0	4.5	4.5	load-integrates
resupply-before-arrival	0.0	0.0	40.0	40.0	blocked:[MissingResource { resource: "grease", needed: 0.05, available: 0.0 }]
initial-overcapacity	0.0	0.0	10.0	10.0	canonical_reject=true,legacy_reject=true
```

## 2. Findings

- 3B load: both paths reach 5 h and end at 4.5 kWh after 5 kWh load and
  0.5 kWh repair consumption.
- 3A resupply: both block at t=0 with missing grease, so a later delivery is not
  usable retroactively.
- 3C battery: both reject initial energy above capacity.

## 3. Classification

- No unexplained divergence in these three cases.
- Joint repair + resupply + load + generation ledger (3D) is not yet covered.

## 4. 3D joint ledger

The engine trace now records generated, resupplied, load, repair and curtailed
energy per event. A dedicated test builds the 6 kWh / 2.5 kW PV / 1 kW load /
2 kWh resupply / 3 h repair case and verifies:

```text
initial 6.0
+ PV 7.5 + resupply 2.0
- load 3.0 - repair 0.5 - curtailed 2.5
= final 9.5 kWh
```

A second test fixes the boundary rule that a resupply exactly at the horizon is
not applied.

## 5. Limits

- Three migration scenarios plus the 3D unit ledger; the 3D case is a unit test,
  not a separate migration artifact.
