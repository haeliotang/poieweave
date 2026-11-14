# S2 Worn-Insert Boundary — report

- Date: 2026-11-14
- Status: worn stock drives the 0.5-yield horizon
- Runner: `cargo run --locked -q -p poie-viability --bin poie-worn-bound`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
worn_stock	survived_days	blocked_by
0	50	missing resource cutting-insert: need 1, have 0.5
2	70	missing resource cutting-insert: need 1, have 0.5
4	90	none
8	90	none
```

## 2. Findings

- With no initial worn inserts, 0.5 yield stops at 50 days on a 0.5 insert
  shortfall.
- 2 worn inserts extend it to 70 days; 4 reach the full 90 days.
- At 0.5 yield each repair produces one worn insert but sharpening needs two
  inputs per good insert, so the worn stock is the next binding constraint.

## 3. Significance

- This explains the 50-day result in `s2-sharpen-yield` and closes the local
  causal chain: yield -> worn stock -> horizon.
- It also shows partial-yield regeneration is viable when a worn buffer exists.

## 4. Limits

- One yield and policy; model-level units.
