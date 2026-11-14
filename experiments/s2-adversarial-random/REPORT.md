# S2 Random Adversarial Sample — report

- Date: 2026-11-14
- Status: 2/500 random draws reach 0 days
- Runner: `cargo run --locked -q -p poie-viability --bin poie-adversarial-random`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
trials	zero	under30	under60	mean_days	min_days	worst_set	at_day
500	2	103	215	60.3	0	filter-1+pv-1	0
```

## 2. Findings

- Only 2 of 500 random subset/time draws reach 0 days.
- 103/500 are under 30 days and 215/500 under 60 days.
- Mean horizon is 60.3 days; the observed worst is filter-1+pv-1 at day 0.

## 3. Significance

- Exhaustive subsets show a 0-day set exists; random sampling shows it is rare
  under this draw distribution.
- This separates "smallest failure set" from "how often a random adverse draw
  hits it".

## 4. Limits

- Seeded sample of 500, not exhaustive.
- Five assets, one scenario.
