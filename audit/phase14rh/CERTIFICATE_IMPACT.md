# Certificate Impact — v7 → v8

Change: `ExplicitRepairTask` gains `robot_alternatives`; explicit group scheduling
uses scheduler per-task `ResourceRequirement::alternatives`.

Classification: `EXECUTION_SEMANTIC_EXTENSION`.

- legacy `run_engine` / non-OR explicit groups: unchanged.
- v7 certificates: frozen bytes; historical verifier checks baseline `9a08498`
  engine hash and v7 index/certificate hashes.
- v8: 41 `CANONICAL_ISSUED_V8` successors with predecessor v7,
  `EXECUTION_SEMANTIC_EXTENSION_SUCCESSOR`.
- legacy replay/254 artifacts remain green.
