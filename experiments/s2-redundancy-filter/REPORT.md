# S2 Redundancy vs Shared Filter — report

- Date: 2026-11-14
- Status: redundancy does not cover a shared single point of failure
- Runner: `cargo run --locked -q -p poie-viability --bin poie-redundancy-filter`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	survived_days	blocked_by
baseline	30	no restorer for filter-1
redundant-pump	30	no restorer for filter-1
```

## 2. Findings

- Adding a redundant pump has no effect once the shared filter fails.
- Both modes stop at day 30 with `no restorer for filter-1`.
- Redundancy at one layer does not remove a single point of failure at another.

## 3. Significance

- This is a counterexample to naive N+1 reasoning: the service still depends on
  a component that has no local restorer.
- It motivates failure-dependency analysis across the service chain, not just
  asset counts.

## 4. Limits

- One shared component; no filter restorer/replacement process in the model.
- Model-level units.
