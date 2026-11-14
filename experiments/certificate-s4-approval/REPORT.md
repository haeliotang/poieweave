# S4 Approval Certificate — report

- Date: 2026-11-14
- Status: certificate verified and bundled as the 10th layer
- Runner: `cargo run --locked -q -p poie-viability --bin poie-approval-certificate`
  then `python3 experiments/certificate-s4-approval/verify.py ...`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
```

## 2. Findings

- The certificate carries the six approval cases and their observed outcomes.
- The verifier independently applies the gate rules and checks execution and
  blocker prefixes.
- It is included in `experiments/certificate-bundle/bundle.json` as layer
  `S4-approval`.

## 3. Limits

- Plan duration is Rust-emitted.
- One failure/policy matrix; model-level units.
