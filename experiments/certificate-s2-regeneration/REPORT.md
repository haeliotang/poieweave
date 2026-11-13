# Regeneration Certificate — report

- Date: 2026-11-14
- Status: model-level regeneration certificate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regeneration-certificate`
  then `python3 experiments/certificate-s2-regeneration/verify.py`
- Certificate: [certificate.json](certificate.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. Contents

- Base-05 SHA-256, 40-day survival
- Base-08 SHA-256, 60-day survival
- local-recycling conclusion
- model-only regeneration claim scope

## 3. Integration

Included as `S2-regen` in the certificate bundle.

## 4. Limits

- Model-level only; no physical yield.
- Self-issued, unsigned.
- One schedule and one model pair.
