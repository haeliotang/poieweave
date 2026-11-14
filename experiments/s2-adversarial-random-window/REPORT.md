# S2 Random Adversarial Window — report

- Date: 2026-11-15
- Status: zero-day mass 13/200 vs 0/200 with delayed subsets
- Runner: `cargo run --locked -q -p poie-viability --bin poie-adversarial-random-window`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
trials	old_zero	new_zero	old_mean	new_mean
200	13	0	4.4	14.0
```

## 2. Findings

- With random subset times in days 0-10, instantaneous zero-day outcomes drop
  from 129/200 (day-0 only) to 13/200.
- Clock-advanced semantics again has no zero-day outcomes.
- Means rise to 4.4 and 14.0 days.

## 3. Significance

- Delay reduces but does not eliminate the instantaneous artifact.
- The qualitative conclusion (clock-advanced removes 0-day collapse) is robust
  to delayed failures.

## 4. Limits

- Window 0-10 days; one long-repair setting; model-level units.
