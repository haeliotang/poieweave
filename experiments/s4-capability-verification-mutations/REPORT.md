# S4 Capability Verification Mutations — report

- Date: 2026-11-14
- Status: found and fixed a Rust capability-verification gap
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-capability-verification-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Initial result (before fix)

```text
MISMATCH capability=sleeve-replacement target=pump-p1 rust=RESTORABLE python=BLOCKED
MISMATCH capability=lathe-repair target=lathe-1 rust=RESTORABLE python=BLOCKED
mutated_models=5 cases=45 matched=43 mismatched=2
```

## 2. Root cause

`ensure_capability` only checked that a provider asset was usable; it did not
check the provider's per-capability verification level. Python's checker did.
Thus Rust accepted a `declared` capability provider while Python required
`modeled` or better.

## 3. Fix

Added `BlockReason::UnverifiedCapability { capability, required, actual }` and
rejected providers whose `capability_verification(capability)` is below the
minimum, before solving the provider asset. Added a regression test.

## 4. Result after fix

```text
mutated_models=5 cases=45 matched=45 mismatched=0
```

## 5. Significance

This is a real S4 outcome: independent verification found a semantic gap in
the Rust S1 closure analyzer, not merely a formatting difference.

## 6. Limits

- One capability at a time.
- Base-05 only.
- Model-level semantics; no physical claim.
