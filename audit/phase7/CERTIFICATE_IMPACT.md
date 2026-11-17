# N7 Certificate Impact

Baseline v2 runtime: `117ba8169995c83a0d4225925e3faf6025913a9e`.

## Change classification

- `engine.rs`: OBSERVATION_ONLY_CHANGE — native AssetStatus snapshot plus RAII collector lifetime.
- No execution/scheduling/survival/closure rule changed.
- `check-experiment-replays.py`: 467 paths replayed, 254 artifacts regenerated; all expected agree.
- N4/N5/N6 evidence: unchanged; check-all regressions continue.

## Certificate migration

- 41 v2 certificates: bytes frozen in `audit/certificates/v2-frozen.json` (82 files + index).
- 41 v3 successors: `experiments/certificate-canonical-*-v3/`,
  predecessor relation `OBSERVATION_ONLY_SUCCESSOR`.
- v1: frozen and historically verified at `9f343b55`.
- v2: historical verifier at `117ba81`.
- v3: current runtime verifier with native evidence, actual experiment replay hash,
  source environment and predecessor hash.
