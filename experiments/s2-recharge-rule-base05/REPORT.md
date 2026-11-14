# S2 Recharge Rule on Base-05 — report

- Date: 2026-11-14
- Status: rule reproduces on Base-05
- Runner: `cargo run --locked -q -p poie-viability --bin poie-recharge-rule-base05`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
recharge_kwh	average_kw	first_energy_limited_load_kw	margin_kw
10	1.25	1.3	0.05
20	2.50	2.6	0.10
40	5.00	5.1	0.10
```

## 2. Findings

- Base-05 produces the same switches as Base-08 for the same recharge changes.
- This suggests the rule is a property of the shared energy/recharge structure,
  not a Base-08-specific artifact.
- It strengthens the causal interpretation of the 2.6 kW boundary.

## 3. Limits

- Both models share the same recharge process; not an independent implementation.
- One schedule/horizon.
