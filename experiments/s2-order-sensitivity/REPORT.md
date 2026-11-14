# S2 Repair-Order Sensitivity — report

- Date: 2026-11-14
- Status: negative result; order does not close the Base-06 gap
- Runner: `cargo run --locked -q -p poie-viability --bin poie-order`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
max_failures=2 recoverable_cases=25 standard_joint=24 best_joint=24 order_sensitive=0
max_failures=3 recoverable_cases=44 standard_joint=40 best_joint=40 order_sensitive=0
```

## 2. Finding

Searching every repair order for two- and three-failure cases does not recover
the `pump-p1,pump-p2` case, nor any other blocked case. The gap is caused by shared material demand, not by
scheduling order.

This is a useful negative result: reordering is not a substitute for an extra
sleeve or local recycling.

## 3. Limits

- Up to three failures, one service, one model family.
- No concurrent execution or partial repair.
