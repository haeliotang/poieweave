# S2 Adversarial Subsets × Time Semantics — report

- Date: 2026-11-14
- Status: clock-advanced repair removes the 0-day adversarial minimum
- Runner: `cargo run --locked -q -p poie-viability --bin poie-adversarial-time`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
at_day	subsets	instant_min	event_min	minimal_size	minimal_set
0	31	0	5	1	pump-p1
10	31	0	5	1	pump-p1
```

## 2. Findings

- Under instantaneous repair, at least one subset collapses to 0 days.
- Under clock-advanced repair, the worst subset survives 5 days; the minimum is
  `pump-p1` alone.
- Moving the subset to day 10 does not change the qualitative result.

## 3. Significance

- Time semantics changes not only the median/horizon but also the adversarial
  worst case.
- The 0-day collapse is an artifact of treating repairs as instantaneous in the
  dense-failure regime.

## 4. Limits

- One long-repair setting; subset times 0/10 only.
