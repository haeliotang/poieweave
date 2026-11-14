# Metrics Certificate — report

- Date: 2026-11-14
- Status: typed S2 metrics certificate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-metrics-certificate`
  then `python3 experiments/certificate-s2-metrics/verify.py`
- Certificate: [certificate.json](certificate.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. Contents

- Base-05 SHA-256, 40-day Survival Horizon
- Base-08 SHA-256, 60-day Survival Horizon
- Recovery Coverage 5/10 = 0.5
- Regenerative Capacity +20 days
- Insert sharpening: 2 inserts 40 -> 60 days (+20)
- Yield sensitivity: 1.0/0.8/0.5 -> 60/60/50 days
- Recharge boundary: average 2.5 kW, switch 2.6 kW, margin 0.1 kW
- model-level metric claim scope

## 3. Integration

Added as `S2-metrics` in the certificate bundle (now seven layers).

## 4. Limits

- Model-level definitions only.
- Self-issued, unsigned.
- No uncertainty intervals or physical calibration.
