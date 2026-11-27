# Certificate Impact Matrix

N23 modifies canonical execution scheduling (`planned_start_s`) in a v13-bound
runtime. Classification: EXECUTION_SEMANTIC_EXTENSION.

- v13: 41 certificates frozen; historical bytes verified.
- v14: 41 successors issued with verifier + tamper controls.
- legacy default (`planned_start_s = 0`) preserved (m2 completion 19800).
- v14 successor certifies m3 committed 10000 -> 20800.
