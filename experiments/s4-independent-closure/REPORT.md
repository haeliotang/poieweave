# S4 Independent Direct Closure — report

- Date: 2026-11-14
- Status: independent direct-repair comparison; not full closure equivalence
- Runner: `python3 experiments/s4-independent-closure/solve.py`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
scenarios/base-04:lathe-1	RESTORABLE
scenarios/base-01:lathe-1	BLOCKED:no_feasible_restorer
scenarios/base-05:lathe-1	RESTORABLE
scenarios/base-05:filter-1	BLOCKED:no_restorer
scenarios/base-05:pump-p1	RESTORABLE
```

## 2. Comparison

| Case | Python verdict | Rust expectation | Agreement |
| --- | --- | --- | --- |
| Base-04 lathe | RESTORABLE | RESTORABLE | yes |
| Base-01 lathe | BLOCKED | BLOCKED (`cycle`) | class yes |
| Base-05 lathe | RESTORABLE | RESTORABLE | yes |
| Base-05 filter | BLOCKED | BLOCKED (no restorer) | yes |
| Base-05 pump | RESTORABLE | RESTORABLE | yes |

The Base-01 blocking reason differs in wording: Python reports no feasible
direct restorer; Rust reports a cycle. The verdict class agrees, and the reason
difference is a useful S4 observation: a shallow checker cannot distinguish a
cycle from a missing restorer.

## 3. Limits

- No production-chain search.
- No recursive repair of occupiers.
- No `requires_any` or shared inventory consumption.
- Agreement is only on five direct-repair cases, not full closure semantics.

## 4. Next

1. Extend the independent checker to production resources and cycles.
2. Compare full verdict + blocking reason.
3. Use a different language/runtime or an SMT solver for a stronger independent
   implementation.
