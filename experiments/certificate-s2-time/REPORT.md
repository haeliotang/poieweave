# S2 Time Certificate — report

- Date: 2026-11-14
- Status: certificate verified and bundled as the 13th layer
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-certificate`
  then `python3 experiments/certificate-s2-time/verify.py ...`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
bundle=ok
summary=ok
tamper=detected
```

## 2. Findings

- Dense events shift 0/1 h to 3/6 h with 3 h repairs.
- Sparse horizons are identical under both semantics.
- The 3-day repair flips the 11-day deadline (10 meets, 13 misses).
- Grease and insert boundaries both shift and saturate higher under
  clock-advanced semantics.
- Regeneration horizons (Base-05/08) are identical under both semantics at the
  nominal repair duration, but with 5-day repairs and dense failures they shift
  (Base-05 8/12/20 -> 20/20/20; Base-08 12/18/30 -> 31/31/31).
- The adversarial minimum over 31 subsets rises from 0 to 5 days with
  clock-advanced repair.

## 3. Significance

- The time-semantics evidence is now content-addressed with the other
  certificates.
- The verifier checks the arithmetic invariants, so the certificate cannot
  silently change the deadline or saturation relationships.

## 4. Limits

- Certificate records results; it does not rerun the full simulator.
