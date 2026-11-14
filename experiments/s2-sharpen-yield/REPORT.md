# S2 Sharpening Yield — report

- Date: 2026-11-14
- Status: positive after fix; yield now affects S1/S2
- Runner: `cargo run --locked -q -p poie-viability --bin poie-sharpen-yield`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
yield	survived_days	blocked_by
1.0	60	cyclic process dependency at turn-sleeve-p1
0.8	40	missing resource cutting-insert: need 1, have 0.8
0.5	40	missing resource cutting-insert: need 1, have 0.5
```

## 2. Discovered gap and fix

The first run showed no effect: `process_yield` was applied by the ontology
graph/scheduler but not by closure/survival. Closure added the full output
quantity regardless of declared yield.

Fix: `apply_process_outputs` now applies the good fraction and emits the reject
share to the declared reject resource, with a unit test. After the fix, yield
0.8/0.5 produce only 0.8/0.5 inserts, so the plan blocks on missing
`cutting-insert`.

## 3. Findings

- Yield 1.0 still reaches 60 days.
- Yield 0.8 and 0.5 stop at 40 days with a shortfall of 0.2/0.5 inserts.
- Yield therefore is a real regeneration limit once it is consistently applied.

## 4. Significance

- This closes the third "declared field not enforced" gap found in this line of
  work (after initial-SOC capacity and stale producer indices).
- It makes yield-aware regeneration limits testable in S2.

## 5. Limits

- One process and one policy.
- Model-level units.
