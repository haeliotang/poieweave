# Certificate Impact

v21 -> v22 because canonical resume now enforces an authorized full task
schedule from the trusted certificate payload.

```text
v20 frozen
v21 frozen (SHA-256 map)
v22 41 successors issued
legacy payloads without task_schedule retain previous behavior
```

Replayed all 41 claims; v21 historical, v22 verifier and tamper controls pass.
