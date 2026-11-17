# N9 Evidence

N9-0 evidence remains in:

```text
audit/phase8h/comparisons/strict-trajectories.json
audit/phase8h/negative-controls/strict-results.json
audit/phase8h/negative-controls/integrity-results.json
audit/phase8h/manifest.json
audit/phase8h/manifest.sha256
```

N9 resource trajectory evidence is not yet produced.

```bash
python3 audit/phase8h/check.py --frozen
python3 audit/phase8h/verify_evidence.py --frozen --claim
python3 audit/phase8h/negative_controls.py
python3 audit/phase8h/negative_integrity.py
```
