# Viability Certificate — S2 Load — report

- Date: 2026-11-14
- Status: machine-checkable model-level certificate; not a safety certificate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-certificate`
  then `python3 experiments/certificate-s2-load/verify.py`
- Certificate: [certificate.json](certificate.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. What the certificate contains

- schema `poie.viability-certificate/1`
- Base-05 model path + SHA-256
- horizon and battery assumptions
- load rows with survived days and blockers
- conclusion and claim scope

## 3. What the verifier checks

- schema
- model file exists and hash matches
- horizon and battery assumptions
- the expected material-limited 0–2 kW rows and energy-limited 3–4 kW rows
- the explicit model-only claim scope

## 4. Limits

- Self-issued, unsigned; not third-party attested.
- Covers one result and one model.
- No physical evidence or safety claim.
