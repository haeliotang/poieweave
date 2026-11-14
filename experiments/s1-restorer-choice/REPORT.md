# S1 Restorer Choice — report

- Date: 2026-11-14
- Status: fixed; the shortest successful restorer is chosen
- Runner: `cargo run --locked -q -p poie-viability --bin poie-restorer-choice`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
order=fast-first-h4	RESTORABLE 10800s
order=fast-first-h10	RESTORABLE 10800s
order=slow-first-h4	RESTORABLE 10800s
order=slow-first-h10	RESTORABLE 10800s
```

## 2. Before the fix

At the 10 h horizon, slow-first returned the 6 h plan while fast-first returned
the 3 h plan: S1 committed to the first feasible restorer, not the shortest.

## 3. Fix

`solve_asset` now evaluates every feasible restorer and keeps the state with the
smallest `total_s`, with a regression test
(`restorer_choice_minimizes_plan_duration`).

## 4. Findings

- At the 4 h horizon the slow restorer is infeasible, so both orders fall back
  to the 3 h plan.
- At the 10 h horizon both orders now return the 3 h plan.
- Duration is optimized; equal-duration input cost is still not part of the
  tie-break (see `s1-restorer-tie`).

## 5. Limits

- One pair of restorers; model-level units.
