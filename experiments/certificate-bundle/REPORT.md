# Certificate Bundle — report

- Date: 2026-11-14
- Status: content-addressed certificate index; unsigned
- Runner: `cargo run --locked -q -p poie-viability --bin poie-certificate-bundle`
  then `python3 experiments/certificate-bundle/verify.py`
- Bundle: [bundle.json](bundle.json)
- Expected verifier output: [expected.txt](expected.txt)

## 1. Result

```text
bundle=ok
```

## 2. What is checked

- bundle schema
- exact S1/S2/S2-joint/S2-regen/S2-metrics/S3/S4 layer set
- each certificate path and schema
- each certificate SHA-256
- explicit model/simulation claim scope

## 3. One-page summary

`SUMMARY.md` is rendered from the bundle. `verify_summary.py` checks that every
layer, hash prefix and disclaimer appears.

```text
summary=ok
```

## 4. Limits

- Unsigned bundle; no attestation.
- Detects content changes, not authorship or physical reality.
- Seven certificates only.
