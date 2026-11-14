# Certificate Clean-Room Verification — report

- Date: 2026-11-15
- Status: bundle verifies in an isolated temporary copy
- Runner: `python3 experiments/certificate-clean-room/clean_room.py`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
clean-room=ok
```

## 2. Findings

- The bundle verifier resolves every certificate and model digest relative to a
  supplied root.
- Copying only the bundle, the 13 certificate directories and `scenarios/` into
  a temporary directory is sufficient for verification.
- No other workspace file is needed.

## 3. Significance

- This is a file-level clean-room check for the evidence bundle.
- It catches accidental dependencies on generated or untracked workspace files.

## 4. Limits

- Same verifier code; no independent reimplementation of the verifier.
