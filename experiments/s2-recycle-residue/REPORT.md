# S2 Mass-Conserving Recycling Residue — report

- Date: 2026-11-14
- Status: residue binds only at low recovery
- Runner: `cargo run --locked -q -p poie-viability --bin poie-recycle-residue`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
bar_recovery_kg	residue_kg	survived_days	blocked_by
1.2	0.0	60	cyclic process dependency at turn-sleeve-p1
1.0	0.2	60	cyclic process dependency at turn-sleeve-p1
0.9	0.3	60	cyclic process dependency at turn-sleeve-p1
0.6	0.6	50	missing resource bronze-bar: need 1.2, have 0.6
```

## 2. Findings

- At 1.2/1.0/0.9 kg recovery the horizon stays 60 days; the first blocker is the
  cyclic turn-sleeve dependency, not bronze mass.
- At 0.6 kg recovery the run drops to 50 days, blocked by missing `bronze-bar`
  (needs 1.2, has 0.6).
- So mass-conserving residue matters only after recovery falls below roughly
  0.9 kg in this schedule; above that, another constraint binds first.

## 3. Significance

- This adds residue to recycling without changing mass conservation.
- It bounds where residue is on the critical path: only at low recovery.
- It also shows the current long horizon is partly limited by other consumables
  (e.g. cutting inserts), which should be studied separately.

## 4. Limits

- One residue fraction sweep; no contamination/quality effects.
- Model-level units.
