# S1 Restorer Choice — report

- Date: 2026-11-14
- Status: plan choice is order-independent and selects the 3 h restorer
- Runner: `cargo run --locked -q -p poie-viability --bin poie-restorer-choice`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
order=fast-first	RESTORABLE 10800s
order=slow-first	RESTORABLE 10800s
```

## 2. Findings

- Even when the 6 h restorer is placed first, closure returns the 3 h plan.
- The search compares alternative restorers and keeps the better feasible plan
  rather than committing to the first one.
- Restorer insertion order therefore does not change this plan choice.

## 3. Significance

- This is a positive control for S1: adding a slower alternative does not degrade
  the result, and index order alone cannot force a worse plan.
- It complements the producer-index bug: the restorer index is order-aware but
  the search still evaluates alternatives.

## 4. Limits

- One pair of restorers; no equal-duration ties.
