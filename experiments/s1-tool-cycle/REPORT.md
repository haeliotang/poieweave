# S1 Tool Repair Cycle — report

- Date: 2026-11-15
- Status: cycle detected; one spare nut is the minimal unblocking condition
- Runner: `cargo run --locked -q -p poie-viability --bin poie-tool-cycle`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
case	nut_stock	outcome	detail
no-spare	0	blocked	cycle:asset:lathe-1
one-spare	1	restorable	7200s
```

## 2. Findings

- With no lathe-nut stock, repairing the lathe requires the lathe itself to make
  the nut: the closure search reports `cycle:asset:lathe-1`.
- One spare nut breaks the cycle; the repair is restorable in 2 hours.
- This is a direct S1 demonstration: the engine identifies a recursive repair
  dependency and the minimum inventory that resolves it.

## 3. Significance

- It connects the abstract capability graph to a concrete minimal blocker.
- It gives a template for "tool repairs itself" reverse dependencies.

## 4. Limits

- One asset and one stocked part; model-level units.
