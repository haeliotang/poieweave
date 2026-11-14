# S2 Joint Yield × Bronze — report

- Date: 2026-11-14
- Status: yield <1 overrides the bronze frontier
- Runner: `cargo run --locked -q -p poie-viability --bin poie-joint-yield`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
yield	bronze_kg	survived_days	blocked_by
1.0	3.0	60	cyclic process dependency at turn-sleeve-p1
1.0	6.0	90	none
0.8	3.0	60	cyclic process dependency at turn-sleeve-p1
0.8	6.0	80	missing resource cutting-insert: need 1, have 0.8
0.5	3.0	50	missing resource cutting-insert: need 1, have 0.5
0.5	6.0	50	missing resource cutting-insert: need 1, have 0.5
```

## 2. Findings

- At yield 1.0 the bronze frontier holds: 3 kg -> 60 days, 6 kg -> 90 days.
- At yield 0.8 the frontier persists but is lower: 3 kg -> 60, 6 kg -> 80 days
  (insert shortfall 0.8).
- At yield 0.5 both bronze stocks stop at 50 days on the same shortfall.
- Partial-yield accumulation (repeated sharpening runs) is what keeps 0.8/0.5
  alive this long; without it they stopped at 40.

## 3. Significance

- This shows a new precedence: yield failure can sit upstream of the consumable
  frontier.
- It also demonstrates that the newly applied yield semantics changes S2
  conclusions, not just a cosmetic field.

## 4. Limits

- One policy; model-level units.
