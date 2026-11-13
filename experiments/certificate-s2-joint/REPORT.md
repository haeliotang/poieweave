# Joint Coverage Certificate — report

- Date: 2026-11-14
- Status: model-level coverage certificate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-joint-certificate`
  then `python3 experiments/certificate-s2-joint/verify.py`
- Certificate: [certificate.json](certificate.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. Contents

- Base-06 model SHA-256
- 56 cases, 12 h horizon, up to 2 failures
- independent 25, joint 24, optimistic gap 1
- gap pair `pump-p1,pump-p2`
- explicit model-level coverage claim scope

## 3. Limits

- Coverage certificate only; not a joint schedule or survival guarantee.
- Self-issued, unsigned.
- One model and one horizon.
