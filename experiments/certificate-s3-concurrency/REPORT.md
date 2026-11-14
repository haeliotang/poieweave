# S3 Concurrency Certificate — report

- Date: 2026-11-14
- Status: certificate verified and bundled as the 11th layer
- Runner: `cargo run --locked -q -p poie-viability --bin poie-concurrency-certificate`
  then `python3 experiments/certificate-s3-concurrency/verify.py ...`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
bundle=ok
summary=ok
tamper=detected
```

## 2. Findings

- One robot: overlap true, sequential/parallel 5 h, 4 h deadline missed.
- Two robots: overlap false, parallel 3 h, deadline met.
- Python recomputes makespans and deadline flags without calling Rust.

## 3. Significance

- The scheduling observation is now a content-addressed bundle layer.
- It packages the distinction between local plan length and system-level
  contention.

## 4. Limits

- Static makespan model; no scheduler, preemption, or hardware.
