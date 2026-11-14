# S2 Time-Semantics Horizon — report

- Date: 2026-11-14
- Status: clock-advanced repair changes the horizon 6 vs 16 days
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-horizon`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	survived_days	blocked_by
instant-repair	6	missing resource grease: need 0.05, have 0
event-driven	16	missing resource grease: need 0.05, have 0
```

## 2. Findings

- With no sleeves, each repair consumes grease and a produced sleeve; the grease
  stock lasts only a few repairs.
- The instantaneous loop consumes the grease by day 6.
- The event-driven loop advances 5 days per repair, pushing later failures past
  the horizon; it survives to day 16 with the same grease limit.
- This is the first case where repair time changes the *horizon*, not just
  timestamps or a deadline flag.

## 3. Significance

- Time semantics is a first-class survival parameter when repairs are long
  relative to the failure interval.
- It validates the need to migrate survival results to completion-time
  semantics.

## 4. Limits

- Modified scenario; one resource limit; no scheduler.
