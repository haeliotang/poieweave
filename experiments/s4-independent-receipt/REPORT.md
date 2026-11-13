# S4 Independent Receipt Verification — report

- Date: 2026-11-14
- Status: independent implementation check; not a physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-receipt-export`
  followed by `python3 experiments/s4-independent-receipt/verify.py`
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
receipts=2 verified=2
OK
```

## 2. Findings

- A separate Python implementation reproduces the receipt digest from the JSON
  body.
- It does not call the Rust verifier; only the canonical encoding and SHA-256
  domain are shared as a contract.
- This is the first S4 artifact: a different implementation checks the evidence
  produced by S3.

## 3. Limits

- Only the receipt encoding/hash is independently verified.
- The model, S1 closure and plan generation are still the Rust implementation.
- No separate process model, FMI co-simulation or isolated node yet.
- No physical or safety claim.
