# N7 Evidence

## Frozen/current separation

- `audit/phase7/manifest.json`: FROZEN_TRACKED_EVIDENCE from tracked files.
- `execution-manifest.json` (CI): CURRENT_RUN_EXECUTION_EVIDENCE for the current run.

## Replay

```bash
cargo run -q -p poie-viability --bin poie-s5v-n7-observatory
python3 audit/phase7/check.py
python3 scripts/build-canonical-claim-certificates-v3.py --check
python3 scripts/verify-canonical-claim-certificates-v3.py
python3 scripts/check-canonical-claim-certificate-tamper-v3.py
python3 audit/certificates/verify_v2.py
python3 audit/certificates/verify_v1_historical.py
```

## Certificate lineage

```text
v1 frozen at 9f343b55
v2 frozen at 117ba81 -> audit/certificates/v2-frozen.json
v3 current -> 41 canonical claim certificates with v2 predecessors
```
