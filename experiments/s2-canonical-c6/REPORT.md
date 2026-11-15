# S2 Canonical C6 Migration — report

- Date: 2026-11-16
- Status: EXPECTED_AGREEMENT after one implementation-defect fix
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c6`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
service	total	independent	legacy_joint	canonical_joint	first_divergence	classification
potable-water	56	25	24	24	none	EXPECTED_AGREEMENT
case	legacy_joint	canonical_joint	canonical_reason
```

## 2. First divergence and fix

The first canonical run produced joint 6/56. The first divergence was the
single failure `charger-1`:

```text
charger-1  legacy_joint=true  canonical_joint=false  NoRestorer(charger-1)
```

Root cause: the engine queued **every** failed asset for repair. Legacy survival
only repairs service requirements, so a failed asset that the service does not
require did not block it.

Fix: only queue a failure for repair when it is a direct `requires` or
`requires_any` member of the configured services. The asset is still marked
Failed and its energy effect still applies.

After the fix, all 56 cases match: independent 25, joint 24, no divergences.

## 3. Classification

- Initial divergence: `MIGRATION_IMPLEMENTATION_DEFECT`.
- Final result: `EXPECTED_AGREEMENT`.
- Non-service asset failures are traced but not repaired; PV/energy effects still
  apply through asset status.

## 4. Limits

- Coverage subset enumeration is an external adapter, not the engine itself.
- The test compares joint sets and counts, not every per-event state for all 56
  cases.
