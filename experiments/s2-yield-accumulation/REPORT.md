# S2 Yield Accumulation — report

- Date: 2026-11-14
- Status: fixed; repeated partial-yield runs accumulate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-yield-accumulation`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
yield	survived_days	blocked_by
1.0	60	cyclic process dependency at turn-sleeve-p1
0.9	60	cyclic process dependency at turn-sleeve-p1
0.8	60	cyclic process dependency at turn-sleeve-p1
0.5	50	missing resource cutting-insert: need 1, have 0.5
```

## 2. Result before the fix

```text
yield	survived_days	blocked_by
1.0	60	cyclic process dependency at turn-sleeve-p1
0.9	40	missing resource cutting-insert: need 1, have 0.9
0.75	40	missing resource cutting-insert: need 1, have 0.75
0.5	40	missing resource cutting-insert: need 1, have 0.5
```

## 3. Finding

Before the fix, 0.9 yield blocked with "have 0.9": `ensure_resource` looked for a
single producer run whose output covered the whole need, and did not accumulate
multiple partial runs.

Fix: `ensure_resource` now repeats the producer while it makes progress (bounded
by recursion, horizon checks and a run cap). With the fix, 0.9/0.8 accumulate
enough inserts to reach 60 days; 0.5 reaches 50 days because worn-insert supply
becomes the limit.

## 4. Significance

- This removes a conservative false negative in S1: low-yield routes can be
  viable through repeated runs.
- It changes the S2 conclusion for 0.8/0.9 yield from 40 to 60 days.
- The 0.5 case shows that accumulation is not infinite: the input byproduct can
  become the next limit.

## 5. Limits

- One producer/policy; 2 inserts, 3 kg bronze.
- Model-level units.
