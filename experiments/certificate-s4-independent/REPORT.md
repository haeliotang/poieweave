# S4 Independent Certificate — report

- Date: 2026-11-14
- Status: certificate verified and bundled as the 12th layer
- Runner: `cargo run --locked -q -p poie-viability --bin poie-independent-certificate`
  then `python3 experiments/certificate-s4-independent/verify.py ...`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
certificate=ok
bundle=ok
summary=ok
tamper=detected
```

## 2. Findings

- Six checks are aggregated: energy 4/4, PV 2/2, grease 6/6, concurrency 2/2,
time 2/2, approval-schedule 4/4.
- The verifier re-reads each expected file and checks the counts, so the layer
  cannot silently claim numbers the experiments do not publish.

## 3. Significance

- Independent-model evidence is now content-addressed with the rest of the
  certificates.
- The claim scope remains narrow: these are small independent checks, not a
  whole-system equivalence proof.

## 4. Limits

- Counts only; the verifier does not rerun the models.
