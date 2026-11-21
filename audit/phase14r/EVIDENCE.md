# N14-R Evidence

- `REPRESENTABILITY_AUDIT.md` — R1/R2/R3 decision
- `SEMANTIC_EXTENSION_ADR.md` — explicit repair group ADR
- `GROUP_EXECUTION_CONTRACT.md`, `RESOURCE_LIFETIME_CONTRACT.md`
- `CERTIFICATE_IMPACT.md` — execution-semantic-extension matrix
- `contracts/N14R-0*.json|contract` — shared canonical/external inputs
- `canonical/N14R-*.jsonl` — canonical native events and task schedules
- `external/N14R-*.csv|journal.json` — N14 FMU execution
- `comparisons/group-cases.json` — 10 fixed two-task cases
- `generated-cases/results.json` — 12 generated two-task cases
- `negative-controls/group-results.json`
- `historical-replay/v6-freeze.json` — frozen v6 bytes and baseline engine hash
- `docs/research/canonical-claim-certificate-index-v7.json`
- `experiments/certificate-canonical-*-v7/`

Replay:

```bash
external/fmu-n14-scheduler/build.sh
cargo build -p poie-viability --bin poie-s5v-n14-canonical-adapter
python3 audit/phase14r/generate_group_cases.py
python3 audit/phase14r/generate_cross_model.py
python3 audit/phase14r/negative_group.py
python3 audit/phase14r/check.py
python3 scripts/verify-canonical-claim-certificates-v7.py
```
