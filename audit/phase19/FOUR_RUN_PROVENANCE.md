# Four-Run Provenance

Each pair has four independent run directories and manifests:

```text
Canonical(B), Canonical(I(B)), External(B), External(I(B))
```

Each manifest binds run_id, model, contract hash, binary hash, timestamps,
exit status, raw output hash and event journal hash. A run_id is immutable;
deterministic replays use new run_ids under `replays/`.
