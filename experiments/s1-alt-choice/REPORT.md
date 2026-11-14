# S1 Alternative Choice — report

- Date: 2026-11-15
- Status: closure picks the 2 h spare path over the 4 h alternative
- Runner: `cargo run --locked -q -p poie-viability --bin poie-alt-choice`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
case	spare	alt_part	outcome	detail
cycle	0	0	blocked	cycle:asset:lathe-1
spare-only	1	0	restorable	7200s
alt-only	0	1	restorable	14400s
both	1	1	restorable	7200s
```

## 2. Findings

- No path: cycle.
- Spare only: 2 h.
- Alternative only: 4 h.
- Both: 2 h, so closure selects the cheaper restorer.

## 3. Significance

- This is cost-aware OR search, not just first-feasible or cycle detection.
- It complements the equal-duration cost tie-break with a cross-path duration
  comparison.

## 4. Limits

- One asset and one synthetic alternative; model-level units.
