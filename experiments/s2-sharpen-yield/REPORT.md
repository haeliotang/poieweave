# S2 Sharpening Yield — report

- Date: 2026-11-14
- Status: negative; yield is not applied by S1/S2
- Runner: `cargo run --locked -q -p poie-viability --bin poie-sharpen-yield`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
yield	survived_days	blocked_by
1.0	60	cyclic process dependency at turn-sleeve-p1
0.8	60	cyclic process dependency at turn-sleeve-p1
0.5	60	cyclic process dependency at turn-sleeve-p1
```

## 2. Findings

- Yield 0.5 does not change the horizon at all.
- A repository search shows `process_yield` is applied by the ontology graph and
  scheduling code, but **not** by the closure or survival code.
- S1/S2 therefore plan as if sharpening always returns one good insert.

## 3. Significance

- This is a cross-module consistency gap: the same model field means different
  things in different engines.
- It is also a negative result for the current S2 claim set: yield-based
  regeneration limits are not yet testable inside S2.

## 4. Next probes

- Apply `process_yield` in closure/survival output accounting.
- Add a differential test where ontology schedule and S2 agree on yield.
- Re-run this experiment to see the expected yield-dependent horizon.

## 5. Limits

- One process and one policy.
- Model-level units.
