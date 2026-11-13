# Recovery Certificate — report

- Date: 2026-11-14
- Status: machine-checkable isolated recovery certificate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-recovery-certificate`
  then `python3 experiments/certificate-s3-recovery/verify.py`
- Certificate: [certificate.json](certificate.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. What is checked

- schema `poie.recovery-certificate/1`
- model SHA-256
- service/horizon
- approved 8 h lease
- day-10 pump failure event
- two receipt digests and the inventory chain
- isolated-simulation claim scope

## 3. Limits

- Self-issued, unsigned.
- No operator identity or hardware execution proof.
- One recovery trace and one model.
- Not a physical safety or authority certificate.
