# S3 Recovery Loop — report

- Date: 2026-11-14
- Status: isolated simulation; not a hardware authority claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-recovery`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
approved=false	survived_days=0	events=1	blocked_by=approval_required:pump-p1
approved=true	survived_days=30	events=3	blocked_by=none
```

## 2. Findings

- Without approval, the loop refuses to apply even a valid S1 repair and stops
  at the initial degraded pump.
- With approval, the loop repairs the initial pump and the day-10/day-20
  failures, re-verifies `potable-water`, and reaches the 30-day horizon.
- Every approved entry is marked `verified=true`; no repair is silently applied
  outside the approval gate.
- This is the first S3 slice: approval gate + deterministic plan + post-repair
  verification in an isolated simulation.

## 3. Limits

- Approval is a boolean, not an operator workflow or signed authorization.
- No action lease, expiry, execution gate, or independent verifier.
- No hardware dispatch; no physical safety implication.
- The loop is still greedy and single-service.

## 4. Next S3 work

1. Replace the boolean approval with a proposal/approve/reject state machine.
2. Add lease/validity windows and re-evaluate on stale evidence.
3. Connect to the existing execution gate and receipt path in simulation.
4. Add an independent verification step before marking capability available.
