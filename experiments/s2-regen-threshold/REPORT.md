# S2 Regeneration Energy Threshold — report

- Date: 2026-11-14
- Status: finite threshold found on the forced path
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-threshold`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
energy_x=1 survived_days=10
energy_x=10 survived_days=10
energy_x=20 survived_days=10
energy_x=30 survived_days=10
energy_x=40 survived_days=10
energy_x=50 survived_days=10
energy_x=60 survived_days=10
energy_x=70 survived_days=0
energy_x=80 survived_days=0
energy_x=90 survived_days=0
energy_x=100 survived_days=0
max_safe_x=60 first_blocked_x=70
```

## 2. Findings

- When recycling is forced onto the path, the energy multiplier has a sharp
  threshold: 60× still survives 10 days, 70× blocks immediately.
- The threshold is a discrete probe result, not a continuous formula.
- This turns the earlier “cost inert” result into a bounded, testable regime.

## 3. Limits

- Forced path; one model family.
- Only electricity; no mass/time coupling.
