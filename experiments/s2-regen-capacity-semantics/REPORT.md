# S2 Capacity Semantics — report

- Date: 2026-11-14
- Status: capacity validation enforced; oversized initial stock is rejected
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-capacity-semantics`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
initial_kwh	capacity_kwh	first_blocked_x
500	200	rejected
500	400	rejected
500	500	521
500	600	521
```

## 2. Findings

- Initial 500 kWh with capacity 200 or 400 kWh is now rejected before the run,
  with an explicit `initial electric-energy ... exceeds battery capacity` error.
- At capacity 500 kWh (exact fit) and 600 kWh the initial stock fits, and the
  forced recycling path reaches 521× before blocking.
- This closes the earlier silent-over-cap behavior: capacity is now enforced as
  an upper bound on the initial stock, not just on interval flows.

## 3. Significance

- The previous “521× regardless of capacity” result was a real semantic gap.
- The fix is fail-closed: inconsistent inputs are rejected instead of silently
  using more energy than the battery can hold.
- Regression tests cover both oversized rejection and fitted acceptance.

## 4. Limits

- Model-level units; forced path.
- Capacity is still a study parameter, not an ontology field.
