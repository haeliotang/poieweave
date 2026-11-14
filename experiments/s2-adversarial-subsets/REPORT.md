# S2 Adversarial Subsets — report

- Date: 2026-11-14
- Status: all 31 subsets searched; minimal 0-day set is filter-1
- Runner: `cargo run --locked -q -p poie-viability --bin poie-adversarial-subsets`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
subsets	min_days	minimal_size	minimal_set
31	0	1	filter-1
```

## 2. Findings

- All 31 non-empty subsets were simulated at t=0.
- The minimum horizon is 0 days.
- The smallest set reaching it is `filter-1` alone, size 1.
- Larger sets may also reach 0, but none is smaller.

## 3. Significance

- This exhausts the local failure-set space over five assets, not just pairs.
- It confirms the pair result: the shared filter's missing restorer dominates the
  other combinations.

## 4. Limits

- Five assets, t=0 only.
- Model-level units.
