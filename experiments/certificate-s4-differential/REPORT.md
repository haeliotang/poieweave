# S4 Differential Certificate — report

- Date: 2026-11-14
- Status: content-addressed S4 evidence certificate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-s4-certificate`
  then `python3 experiments/certificate-s4-differential/verify.py`
- Certificate: [certificate.json](certificate.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. What is checked

- schema
- SHA-256 of the differential verdict expectation
- SHA-256 of the plan-replay expectation
- exact summaries (`112/112`, `26/26`)
- independent-implementation claim scope

## 3. Integration

The certificate is included as the S4 layer in the certificate bundle:

```text
S1 / S2 / S2-joint / S3 / S4
```

## 4. Limits

- Does not rerun the differential corpus.
- Unsigned, no attestation.
- Not an equivalence proof.
