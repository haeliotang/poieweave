# S4 Tool-Cycle Model — report

- Date: 2026-11-15
- Status: structural cycle/unblocker model matches 2/2
- Runner: `cargo run --locked -q -p poie-viability --bin poie-tool-cycle > /tmp/tcm.tsv`
  then `python3 experiments/s4-tool-cycle-model/verify.py /tmp/tcm.tsv`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
rows=2 matched=2
```

## 2. Findings

- The repair process restores `lathe-1` and needs `lathe-nut`.
- The nut producer occupies `lathe-1`, so with no stock the repair depends on
  the failed asset itself: a cycle.
- One stock unit yields the repair duration of 2 h.

## 3. Significance

- The S1 cycle is independently derivable from the process/resource graph.
- It complements the certificate with a model-driven structural check.

## 4. Limits

- Structural only; no alternative-restorer search.
