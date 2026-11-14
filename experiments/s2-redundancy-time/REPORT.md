# S2 Redundancy × Time Semantics — report

- Date: 2026-11-15
- Status: redundancy dominates; both semantics reach 90 days
- Runner: `cargo run --locked -q -p poie-viability --bin poie-redundancy-time`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	load_kw	instant_days	event_driven_days
baseline	0.0	12	31
redundant-pump	0.0	90	90
```

## 2. Findings

- Baseline: 12 days instantaneously vs 31 days with clock-advanced repair.
- Redundant pump: 90 days under both semantics.
- The redundancy effect (31/12 -> 90) is much larger than the time-semantics
  effect (12 -> 31).

## 3. Significance

- Redundancy can remove the failure mode that time semantics only postpones.
- Effect ordering matters: repair-time modeling cannot substitute for redundancy.

## 4. Limits

- One redundancy layout; 5-day repair; model-level units.
