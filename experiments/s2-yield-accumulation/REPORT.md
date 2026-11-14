# S2 Yield Accumulation — report

- Date: 2026-11-14
- Status: negative; S1 does not combine partial-yield runs
- Runner: `cargo run --locked -q -p poie-viability --bin poie-yield-accumulation`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
yield	survived_days	blocked_by
1.0	60	cyclic process dependency at turn-sleeve-p1
0.9	40	missing resource cutting-insert: need 1, have 0.9
0.75	40	missing resource cutting-insert: need 1, have 0.75
0.5	40	missing resource cutting-insert: need 1, have 0.5
```

## 2. Finding

At 0.9 yield, two sharpening runs would produce 1.8 inserts, enough for one
insert. The closure nevertheless blocks with "have 0.9": `ensure_resource`
looks for a single producer run whose output covers the whole need, and does not
accumulate multiple partial runs.

## 3. Significance

- This is a conservative planning limitation (false negative), not an
  over-approximation.
- It matters for regeneration: low-yield routes can be viable through repeated
  runs, but the current S1 cannot see that.
- It gives a concrete next improvement: allow repeated producer runs when a
  single run's yield is below the need.

## 4. Limits

- One producer/policy; 2 inserts, 3 kg bronze.
- Model-level units.
