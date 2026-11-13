# S2 Joint Coverage — report

- Date: 2026-11-14
- Status: model-level comparison; not a physical or safety claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-joint-coverage`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
service=potable-water horizon_s=43200 total=56 independent=25 joint=24 optimistic_gap=1
gap	failed=pump-p1,pump-p2	joint=BLOCKED
```

## 2. Finding

- Independent coverage claims 25/56 recoverable.
- Joint shared-inventory execution recovers 24/56.
- The only gap is the redundant pump pair: the independent checks each pump
  against the same sleeve, while the joint policy consumes the only sleeve on
  the first repair and cannot make another because bronze is insufficient.

This quantifies the S2 correction: **independent Recovery Coverage is an upper
bound, not a survival result**.

## 3. Limitations

- One fixed serial policy; repair order is model/requirements order.
- Immediate repair at t=0; no scheduling/lookahead.
- No resupply or local material regeneration.
- Synthetic Base-06; no physical safety implication.

## 4. Next

1. Add joint coverage after resupply/regeneration assumptions.
2. Add failure timing (not all at t=0) and lookahead repair order.
3. Compare against a survival-aware baseline over 30/90 days.
