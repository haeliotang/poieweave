# S2 Cutting-Insert Boundary — report

- Date: 2026-11-14
- Status: insert stock is binding below 4 units
- Runner: `cargo run --locked -q -p poie-viability --bin poie-insert-bound`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
insert_count	survived_days	blocked_by
2	40	missing resource cutting-insert: need 1, have 0
3	50	missing resource cutting-insert: need 1, have 0
4	60	cyclic process dependency at turn-sleeve-p1
6	60	cyclic process dependency at turn-sleeve-p1
8	60	cyclic process dependency at turn-sleeve-p1
12	60	cyclic process dependency at turn-sleeve-p1
```

## 2. Findings

- With 2 or 3 inserts the run stops on missing `cutting-insert`, at 40/50 days.
- Each additional insert adds about 10 days in this range.
- With 4+ inserts the blocker moves to the cyclic `turn-sleeve-p1` dependency and
  the horizon stays 60 days.
- This identifies the cutting insert as the binding wear part before the cycle
  becomes the limiting factor.

## 3. Significance

- The earlier residue experiment was partly masked by insert depletion; the two
  consume different margins.
- It also shows a concrete irreplaceable-consumable boundary in the current
  model: there is no insert refurbishment/replacement process.

## 4. Limits

- One consumable model; no insert wear distribution.
- Model-level units.
