# S2 Time-Semantics Boundary — report

- Date: 2026-11-14
- Status: the grease boundary shifts from 4/6/8 to 10/16/21 days
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-boundary`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
grease_kg	instant_days	event_driven_days
0.10	4	10
0.15	6	16
0.20	8	21
0.30	8	21
0.50	8	21
```

## 2. Findings

- Instantaneous: 4/6/8 days, saturating at 8 from 0.20 kg upward.
- Clock-advanced: 10/16/21 days, saturating at 21 from 0.20 kg upward.
- The boundary is not just shifted; the saturation point is different because
  long repairs push later failures past the 30-day horizon.

## 3. Significance

- This is a direct demonstration that time semantics changes a survival
  boundary curve, not only individual timestamps.
- It gives a concrete dataset for migrating S2 boundaries to completion-time
  semantics.

## 4. Limits

- Modified scenario; one resource limit; no scheduler.
