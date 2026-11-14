# Repair-Order Certificate — report

- Date: 2026-11-14
- Status: negative-result certificate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-order-certificate`
  then `python3 experiments/certificate-s2-order/verify.py`
- Certificate: [certificate.json](certificate.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. Contents

- Base-06 SHA-256
- 2 failures: 24/25 fixed, 24/25 best order, 0 order-sensitive
- 3 failures: 40/44 fixed, 40/44 best order, 0 order-sensitive
- explicit model-level ordering claim scope

## 3. Integration

Added as `S2-order` to the certificate bundle (nine layers).

## 4. Limits

- Self-issued, unsigned.
- Model-level; not a scheduling optimality proof.
