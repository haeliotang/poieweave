# S2 Random Adversarial × Time Semantics — report

- Date: 2026-11-15
- Status: 129/200 zero-day outcomes instant vs 0/200 clock-advanced
- Runner: `cargo run --locked -q -p poie-viability --bin poie-adversarial-random-time`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
trials	old_zero	new_zero	old_mean	new_mean
200	129	0	1.4	9.9
```

## 2. Findings

- Instantaneous semantics: 129 of 200 random subsets collapse to 0 days, mean
  1.4 days.
- Clock-advanced semantics: no subset collapses to 0, mean 9.9 days.
- The zero-day mass is a modeling artifact of instantaneous repairs under dense
  failures, not a property of the failure set.

## 3. Significance

- It generalizes the 31-subset result to a random sample.
- It shows the time-semantics effect is large in distribution, not only at the
  minimum.

## 4. Limits

- Subsets at day 0; one long-repair setting; model-level units.
