# S2 Canonical C9 Migration — report

- Date: 2026-11-16
- Status: EXPECTED_AGREEMENT on 6/6 rows after one implementation-defect fix
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c9`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode            load  legacy  canonical  legacy_energy  canonical_energy  termination
baseline        2.5   60      60         26.3           26.3              cycle
baseline        2.6   16      16         0.0            0.0               energy_depleted
baseline        3.0   3       3          0.0            0.0               energy_depleted
redundant-pump  2.5   90      90         39.7           39.7              completed
redundant-pump  2.6   17      17         0.0            0.0               energy_depleted
redundant-pump  3.0   3       3          0.0            0.0               energy_depleted
```

## 2. First divergence and fix

Initial run:

```text
redundant-pump 2.5: legacy 90, canonical 70
redundant-pump 2.6: legacy 17, canonical 16
```

Root cause: engine queued every `requires_any` failure for repair. With
`pump-p2` already serving `potable-water`, repairing `pump-p1` was unnecessary
and consumed energy.

Fix: a `requires_any` failure is repairable only when no other alternative in
that group is currently Available. Direct `requires` failures are always queued.
All failures still trace and interrupt running repairs if they are dependencies.

After the fix, 6/6 rows agree, including final energy.

## 3. Classification

- Initial divergence: `MIGRATION_IMPLEMENTATION_DEFECT`.
- Final: `EXPECTED_AGREEMENT`.
- Redundancy extension 60→90 at 2.5 kW preserved; 2.6 kW remains 17 days.

## 4. Limits

- One redundancy layout; model-level units.
