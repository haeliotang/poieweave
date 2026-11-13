# S4 Counterfactual Threshold — report

- Date: 2026-11-14
- Status: model-level threshold check; not a general minimal-repair proof
- Runner: `cargo run --locked -q -p poie-viability --bin poie-unblock`
  then `python3 experiments/s4-counterfactual/compare.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
bronze_kg	outcome
1.1	BLOCKED
1.2	RESTORABLE
1.5	RESTORABLE
```

Independent Python check:

```text
cases=3 matched=3 mismatched=0
```

## 2. Findings

- The `need 1.2` witness predicts the threshold exactly: 1.1 kg is insufficient
  and 1.2 kg unblocks the pump repair.
- This closes the loop from "why blocked" to "what minimal added stock would
  unblock it" for one fixture.
- The independent checker reproduces the same threshold.

## 3. Limits

- One resource and one target.
- No combined counterfactuals, resupply timing or cost.
- Not a general minimal-repair-set algorithm or formal certificate.
