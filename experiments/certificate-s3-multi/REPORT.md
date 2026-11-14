# Multi-Service Recovery Certificate — report

- Date: 2026-11-14
- Status: machine-checkable isolated multi-service recovery certificate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-multi-recovery-certificate`
  then `python3 experiments/certificate-s3-multi/verify.py`
- Certificate: [certificate.json](certificate.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. Contents

- Base-05 SHA-256
- services `potable-water` + `workshop-operability`
- day-10 pump and day-20 lathe events
- approved 8 h lease
- three receipts with a continuous inventory chain
- isolated-simulation claim scope

## 3. Integration

Added as `S3-multi` to the certificate bundle (eight layers).

## 4. Limits

- Self-issued, unsigned.
- Isolated simulation only.
- Two services and two events.
