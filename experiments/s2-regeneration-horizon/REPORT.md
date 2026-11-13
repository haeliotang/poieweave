# S2 Regeneration Horizon — report

- Date: 2026-11-14
- Status: model-level regeneration effect; not a physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-horizon`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
model	interval_days	survived_days	blocked_by
base-05	10	40	bronze-bar exhausted
base-05	15	60	bronze-bar exhausted
base-05	20	80	bronze-bar exhausted
base-05	30	90	none
base-08	10	60	cyclic process dependency at turn-sleeve-p1
base-08	15	90	none
base-08	20	90	none
base-08	30	90	none
```

## 2. Findings

- Local recycling extends the 10-day survival horizon from **40 to 60 days**.
- For 15/20/30-day pump intervals, Base-08 survives the whole 90-day window.
- The 10-day case still ends in a process cycle around `turn-sleeve-p1`; even
  with recycling, the material flow is not fully closed at this failure rate.
- This is the first S2 evidence that **Autonomous Regeneration changes the
  Survival Horizon**, not just a one-off repair coverage count.

## 3. Interpretation

Recycling is not free: it consumes lathe/gauge time, energy, and
machine-tending capability. The result is a model-level upper-bound benefit;
the next iteration must add recycling yield loss and contention with repair
work.

## 4. Limitations

- No recycling loss/contamination; 100% mass conversion.
- No operating demand or concurrent production.
- Synthetic initial swarf stock.
- Model-level only; no physical or safety claim.
