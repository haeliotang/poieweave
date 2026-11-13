# S4 Differential Corpus — report

- Date: 2026-11-14
- Status: differential verdict corpus; not a formal equivalence proof
- Runner: `cargo run --locked -q -p poie-viability --bin poie-closure-corpus-export`
  then `python3 experiments/s4-differential-corpus/compare.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
cases=112 matched=112 mismatched=0
```

## 2. What happened during the run

The first comparison (27 cases) found three mismatches, all on `bus-1` in
Base-01/04/05:

- Rust: BLOCKED (power/robot cycle).
- Python: RESTORABLE, because it treated any available capability provider as
  usable and did not propagate the bus failure through charger/robot.

This was a bug in the independent checker, not in Rust. After adding:

- usable-asset dependency checking, and
- capability-provider usability checking,

the Python checker matched all 27 verdicts. The corpus was then expanded to
six models and two horizons: **112/112 verdicts agree**.

This is exactly the kind of S4 outcome the research plan asks for: the
independent implementation initially disagreed, the difference was investigated,
and the model of the checker was tightened.

## 3. Limits

- Only verdict classes are compared; blocking reasons are not.
- Two horizons and six models; the model family is still synthetic.
- The Python checker still lacks `requires_any`, shared-time accounting and
  full yield semantics.
- Not a formal equivalence proof.

## 4. Next

1. Expand the corpus across Base-06/07/08 and multiple horizons.
2. Compare blocking reasons and minimal witnesses.
3. Add automatically generated differential cases.
4. Introduce an independent process model/SMT for a stronger check.
