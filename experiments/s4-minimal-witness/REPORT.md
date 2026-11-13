# S4 Minimal Witness Fixtures — report

- Date: 2026-11-14
- Status: exact fixture witness check; not a general minimizer
- Runner: `cargo run --locked -q -p poie-viability --bin poie-reason-fixtures`
  then `python3 experiments/s4-minimal-witness/compare.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
witnesses=2 matched=2 mismatched=0
```

Checked values:

- `missing resource bronze-bar: need 1.2, have 0`
- `unverified process replace-lathe-nut: Declared < Modeled`

## 2. Findings

- The Python checker independently locates the process that needs the missing
  bronze and reproduces the exact 1.2 kg amount.
- It independently reads the restorer verification and reproduces the
  `Declared < Modeled` witness.
- This goes beyond reason classes to exact blocking values for these fixtures.

## 3. Limits

- Only two hand-built fixtures.
- No automated minimal-witness search over failures or counterfactuals.
- Not a formal certificate.
