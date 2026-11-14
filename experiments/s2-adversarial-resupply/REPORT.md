# S2 Adversarial Subsets × Resupply — report

- Date: 2026-11-15
- Status: resupply does not fix the worst subset
- Runner: `cargo run --locked -q -p poie-viability --bin poie-adversarial-resupply`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
at_day	subsets	no_supply_min	with_supply_min	minimal_size	minimal_set
0	31	0	0	1	filter-1
10	31	0	0	1	filter-1
```

## 2. Findings

- Without resupply, the worst subset is 0 days and the minimal set is `filter-1`.
- With insert deliveries every 10 days, the worst subset is still 0 days and the
  same `filter-1` set.
- The blocker is a missing repair capability, not a consumable shortage, so
  material resupply cannot help.

## 3. Significance

- It separates two failure classes: replenishable material vs irreplaceable
  capability.
- Resupply is not a general recovery mechanism; it only addresses the former.

## 4. Limits

- One consumable; subset times 0/10 only; model-level units.
