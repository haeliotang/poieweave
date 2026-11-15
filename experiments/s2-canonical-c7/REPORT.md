# S2 Canonical C7 Migration — report

- Date: 2026-11-16
- Status: EXPECTED_AGREEMENT on 6/6 rows
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c7`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
insert_count	mode	legacy_days	canonical_days	delta_days	legacy_energy_kwh	canonical_energy_kwh	termination	failure_times	classification
2	baseline	40	40	0.0	200.0	200.0	blocked:cutting-insert	match	EXPECTED_AGREEMENT
2	sharpen	60	60	0.0	200.0	200.0	blocked:cycle	match	EXPECTED_AGREEMENT
3	baseline	50	50	0.0	200.0	200.0	blocked:cutting-insert	match	EXPECTED_AGREEMENT
3	sharpen	60	60	0.0	200.0	200.0	blocked:cycle	match	EXPECTED_AGREEMENT
4	baseline	60	60	0.0	200.0	200.0	blocked:cycle	match	EXPECTED_AGREEMENT
4	sharpen	60	60	0.0	200.0	200.0	blocked:cycle	match	EXPECTED_AGREEMENT
```

## 2. Findings

- 2 inserts: 40 → 60 with sharpening.
- 3 inserts: 50 → 60 with sharpening.
- 4 inserts: already 60.
- Final energy and failure timestamps match.
- Classification: `EXPECTED_AGREEMENT`.

## 3. Limits

- Model-level result; no physical tool-wear claim.
- Event trace representation may differ by completion records; failure times match.
