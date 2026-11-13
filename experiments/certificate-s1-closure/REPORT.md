# Closure Certificate — report

- Date: 2026-11-14
- Status: model-level closure certificate
- Runner: `cargo run --locked -q -p poie-viability --bin poie-closure-certificate`
  then `python3 experiments/certificate-s1-closure/verify.py`
- Certificate: [certificate.json](certificate.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. What is checked

- schema `poie.closure-certificate/1`
- Base-04 model SHA-256
- target `lathe-1`, 7200 s horizon, RESTORABLE
- plan replay: process, inputs, outputs, restore, duration
- model-level claim scope

## 3. Limits

- One case only.
- Self-issued, unsigned.
- No physical repair or hardware evidence.
