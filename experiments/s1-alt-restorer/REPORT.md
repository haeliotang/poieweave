# S1 Alternative Restorer — report

- Date: 2026-11-15
- Status: alternative restorer bypasses the cycle when stocked
- Runner: `cargo run --locked -q -p poie-viability --bin poie-alt-restorer`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
case	alt_part	outcome	detail
cycle-only	0	blocked	cycle:asset:lathe-1
alt-unblocked	1	restorable	14400s
```

## 2. Findings

- With no alternative part, the only path is the self-referential nut production
  and closure reports `cycle:asset:lathe-1`.
- With one external part, closure selects the alternative restorer and returns a
  4 h plan.

## 3. Significance

- This demonstrates OR search over restorers, not just cycle detection.
- The cycle is a property of the available process set, not of the asset alone.

## 4. Limits

- Synthetic alternative process; one asset; model-level units.
