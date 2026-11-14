# S4 Path-Choice Model — report

- Date: 2026-11-15
- Status: independent path choice matches 4/4
- Runner: `cargo run --locked -q -p poie-viability --bin poie-alt-choice > /tmp/pc.tsv`
  then `python3 experiments/s4-path-choice-model/verify.py /tmp/pc.tsv`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
rows=4 matched=4
```

## 2. Findings

- The model derives 7200 s from the TOML repair duration.
- Alternative-only is 14400 s (declared).
- Both paths select 7200 s, the cheaper one.
- No paths report the cycle.

## 3. Significance

- The cross-path choice is independently derivable from durations.
- It complements the S1 certificate path-choice fields.

## 4. Limits

- Alternative duration declared, not parsed from a model file.
