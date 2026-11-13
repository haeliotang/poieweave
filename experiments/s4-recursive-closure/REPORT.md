# S4 Recursive Closure — report

- Date: 2026-11-14
- Status: independent recursive verdict check; not a formal equivalence proof
- Runner: `python3 experiments/s4-recursive-closure/solve.py`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
scenarios/base-01:pump-p1	RESTORABLE
scenarios/base-01:lathe-1	BLOCKED
scenarios/base-04:lathe-1	RESTORABLE
scenarios/base-05:pump-p1	RESTORABLE
scenarios/base-05:filter-1	BLOCKED
```

## 2. Comparison

All five verdict classes agree with Rust S1:

| Case | Python | Rust |
| --- | --- | --- |
| Base-01 pump (production chain) | RESTORABLE | RESTORABLE |
| Base-01 lathe (cycle) | BLOCKED | BLOCKED |
| Base-04 lathe (direct) | RESTORABLE | RESTORABLE |
| Base-05 pump (direct) | RESTORABLE | RESTORABLE |
| Base-05 filter (no restorer) | BLOCKED | BLOCKED |

The Base-01 pump case is the important step: the Python checker independently
found the production chain needed to make a sleeve before replacing the pump.

## 3. Limits

- No `requires_any`.
- No shared elapsed time across alternative branches.
- No parallel occupancy or yield modeling.
- Verdict classes agree; blocking reasons are not independently reproduced.
- Not a formal proof that the implementations are equivalent.

## 4. Next

1. Add `requires_any` and full duration accounting.
2. Compare blocking reasons and minimal counterexamples.
3. Generate a differential corpus automatically.
4. Introduce an independent process model or SMT solver.
