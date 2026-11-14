# S2 Worn × Bronze at 0.5 Yield — report

- Date: 2026-11-14
- Status: both stocks gate the horizon
- Runner: `cargo run --locked -q -p poie-viability --bin poie-joint-worn`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
worn_stock	bronze_kg	survived_days	blocked_by
0	3.0	50	missing resource cutting-insert: need 1, have 0.5
0	6.0	50	missing resource cutting-insert: need 1, have 0.5
2	3.0	60	cyclic process dependency at turn-sleeve-p1
2	6.0	70	missing resource cutting-insert: need 1, have 0.5
4	3.0	60	cyclic process dependency at turn-sleeve-p1
4	6.0	90	none
```

## 2. Findings

- With no worn stock, bronze does not matter: both stop at 50 days on the 0.5
  insert shortfall.
- With 2 worn inserts, bronze starts to matter: 3 kg -> 60 days, 6 kg -> 70.
- With 4 worn inserts, 6 kg bronze reaches the full 90 days.
- Worn stock gates the insert-regeneration path; bronze gates the turn step.

## 3. Significance

- This is a two-stock regeneration frontier: neither stock alone explains the
  horizon.
- It also explains why raising bronze without worn stock had no effect at 0.5
  yield.

## 4. Limits

- Two stocks; one yield/policy; model-level units.
