# Canonical Order — report

- Status: EXPECTED_AGREEMENT 2/2 rows
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-order`
- Output: [expected.txt](expected.txt)

```text
max  recoverable  standard  best  order_sensitive
2    25           24        24    0
3    44           40        40    0
```

Canonical semantics preserve the negative result: repair order does not close
the Base-06 shared-material gap. Both the standard order and the best order
recover 24/25 and 40/44 cases.
