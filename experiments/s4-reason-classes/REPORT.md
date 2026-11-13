# S4 Reason Classes — report

- Date: 2026-11-14
- Status: coarse reason-class comparison; not full reason equivalence
- Runner: `cargo run --locked -q -p poie-viability --bin poie-closure-corpus-export`
  then `python3 experiments/s4-reason-classes/compare.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
blocked=86 matched=86 mismatched=0
```

## 2. Findings

- All 86 blocked cases agree on the coarse class:
  - no direct restorer, or
  - cyclic asset dependency.
- This extends the 112/112 verdict agreement to the reason-class layer.
- It also exposes a limitation: the corpus has no blocked cases from missing
  resources, horizon exceeded or unverified processes, so those classes are not
  independently checked yet.

## 3. Limits

- Only the first reason and two classes are compared.
- No minimal witness or exact reason text.
- Corpus lacks missing-resource/horizon blocked cases.
- Still not a formal equivalence proof.
