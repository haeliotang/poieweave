# S2 Joint Double-Failure — report

- Date: 2026-11-14
- Status: model-level joint-inventory counterexample; not a physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-joint`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
pair=failed=pump-p1,pump-p2
independent_coverage=true
joint_survived_days=10
joint_blocked_by=missing resource bronze-bar: need 1.2, have 0.6
```

## 2. Finding

This is the first explicit S2 counterexample to the independent coverage
envelope:

- The independent envelope says the pump-pair failure is recoverable.
- The joint policy consumes the only sleeve on the first pump and then cannot
  make another because the local bronze stock is 0.6 kg rather than 1.2 kg.
- `Recovery Coverage` must therefore be treated as an optimistic upper bound;
  `Survival Horizon` / joint execution is the binding measure.

## 3. Model design

Base-06 is synthetic and deliberately small: two pumps, one shared sleeve,
insufficient bronze for local production, otherwise the Base-05 repair loop.
The conflict is constructed to make the independent/joint distinction
observable; it is not evidence that any real plant behaves this way.

## 4. Next

1. Add a **joint coverage** metric: pairs that are recoverable under one shared
   inventory ledger and one serial repair order.
2. Enumerate all pairs in Base-06 and compare independent vs joint coverage.
3. Add resupply/regeneration assumptions and measure whether they restore the
   pair.
