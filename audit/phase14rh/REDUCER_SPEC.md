# Reducer

The reducer re-executes canonical and external models for every candidate
contract. Failure signature:

```text
RESOURCE_ELIGIBILITY_VIOLATION
```

Original: 2 tasks, 2 robots, 4 failures, tool AND. Reducer removes failures,
task B and shortens duration while preserving the signature. Output:
`reducer/original.json`, `minimized.json`, `reduction-log.json`,
`failure-signature.json`, `replay.sh`.
