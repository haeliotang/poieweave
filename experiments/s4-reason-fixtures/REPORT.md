# S4 Reason Fixtures — report

- Date: 2026-11-14
- Status: two-fixture reason-class check; not full reason equivalence
- Runner: `cargo run --locked -q -p poie-viability --bin poie-reason-fixtures`
  then `python3 experiments/s4-reason-fixtures/compare.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
cases=2 matched=2 mismatched=0
```

Rust reasons:

```text
missing resource bronze-bar: need 1.2, have 0
unverified process replace-lathe-nut: Declared < Modeled
```

## 2. Findings

- Base-09 produces the `missing_resource` class.
- Base-10 produces the `unverified_process` class.
- Python classifies both correctly using stock and process verification.
- Together with the earlier corpora, S4 now covers reason classes:
  `no_restorer`, `cycle`, `horizon`, `missing_resource`,
  `unverified_process`.

## 3. Limits

- Two hand-built fixtures; not a corpus.
- No minimal witness, exact reason text or SMT proof.
- Base-09/10 are synthetic and not physically calibrated.
