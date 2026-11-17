# N6-E Evidence Integrity

## Static package

- `PREREG.md`, `SEMANTICS.md`, `RESOURCE_LIFETIME.md`, `REPORT.zh-CN.md`
- canonical schedules/audits/events and parameterized task contracts
- external C harness and fmpy raw CSVs/logs
- difference classifiers and negative controls
- restoration-state evidence
- SHA-256 `manifest.json`

## Replay commands

```bash
cargo run -q -p poie-viability --bin poie-s5v-n6-canonical
cargo run -q -p poie-viability --bin poie-s5v-n6-contract
cargo run -q -p poie-viability --bin poie-s5v-n6-boundaries
python3 audit/phase6/compare_lifetime.py
python3 audit/phase6/negative_lifetime.py
python3 audit/phase6/compare_scheduling.py
python3 audit/phase6/negative_scheduling.py
python3 audit/phase6/compare_dependencies.py
python3 audit/phase6/negative_dependencies.py
python3 audit/phase6/restoration_evidence.py
python3 audit/phase6/manifest.py
python3 audit/phase6/check.py
```

## Linux CI

Job `S5-V N6 resource lifetime Linux conformance` builds both external FMUs,
rebuilds/runs canonical evidence, executes C harness and fmpy for every accepted
contract, runs all comparators/negative controls, regenerates the manifest and
uploads `s5v-n6-linux-evidence`.

No repository CSV is used in place of the current-run outputs.

## Mutation coverage (key classes)

| class | control |
| --- | --- |
| RESOURCE_LIFETIME_MISMATCH | step release at 7201 |
| RESOURCE_CONTRACT_VIOLATION | overlapping robot, duration, pool |
| EXECUTION_DEPENDENCY_MISMATCH | tool failure ignored / completion interrupted |
| UPSTREAM_DEPENDENCY_MISMATCH | bus failure ignored |
| EVENT_TIME_DISPLACEMENT | late failure, horizon-accepted failure, makespan |
| EVENT_ORDER_MISMATCH | same-time order reversed |
| RESTORATION_COMMIT_MISMATCH | missing restoration |
| RESTORATION_PRECONDITION_VIOLATION | completed count/restoration flags |

## Certificate history

Only external FMUs, canonical runners and evidence were added. No v2-bound
runtime source (`engine.rs`, `survival.rs`, `clock.rs`, `closure.rs`,
`scheduler.rs`) changed. The 41 v2 certificates and frozen v1 bytes remain valid;
`audit/certificates/verify_v1.py` and historical verification remain green.

## Known limitations

- N6-A is group-level only; step-level release is not represented.
- N6-B does not represent an independent canonical priority input; B06 is a
  classified scheduling-policy difference.
- N6-D asset status is derived from the canonical completion trace; no native
  asset-status field exists in the current observatory.
