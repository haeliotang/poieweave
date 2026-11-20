# N9 Evidence

Frozen N9 resource-causal evidence:

```text
audit/phase9/contracts/
audit/phase9/canonical/
audit/phase9/external/
audit/phase9/comparisons/resource-trajectories.json
audit/phase9/negative-controls/resource-results.json
audit/phase9/negative-controls/integrity-results.json
audit/phase9/manifest.json
audit/phase9/manifest.sha256
```

N9-0 comparator integrity evidence remains in:

```text
audit/phase8h/comparisons/strict-trajectories.json
audit/phase8h/negative-controls/strict-results.json
audit/phase8h/negative-controls/integrity-results.json
audit/phase8h/manifest.json
```

Replay:

```bash
python3 audit/phase9/check.py --frozen
python3 audit/phase9/verify_evidence.py --frozen --claim
python3 audit/phase9/compare_resource.py
python3 audit/phase9/negative_resources.py
python3 audit/phase9/negative_integrity.py
```
