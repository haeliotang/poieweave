# S4 Grease Model — report

- Date: 2026-11-14
- Status: independent repair-count model matches 6/6
- Runner: `cargo run --locked -q -p poie-viability --bin poie-grease-cost > /tmp/gc.tsv`
  then `python3 experiments/s4-grease-model/verify.py /tmp/gc.tsv`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
rows=6 matched=6
```

## 2. Findings

- The model deducts 0.05 kg (cheap) or 0.20 kg (expensive) at t=0, then at each
  10-day event, and reports the event day when the stock is insufficient.
- It reproduces 20/30/10-day cheap cases and 0/0/10-day expensive cases.
- Including the initial t=0 repair was necessary to match Rust.

## 3. Significance

- This is the first independent material/consumable model in S4, alongside the
  energy and PV models.
- It shares only the scenario TOML and the known expensive cost.

## 4. Limits

- One consumable; no closure, energy or occupancy.
